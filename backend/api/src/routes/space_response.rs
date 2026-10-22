//! Trusted in-process owner deadlines; response headers are never authority.
use axum::{body::Body, http::header, response::Response};
use futures_util::StreamExt;
use shared::{AppError, wiki_space_access::WikiSpaceAdmission};
use std::{sync::Arc, time::Instant};

#[derive(Clone)]
struct OwnerDeadline {
    until: i64,
    deadline: Instant,
}

pub(super) fn restrict(
    response: &mut Response,
    scope: &WikiSpaceAdmission,
    until: i64,
) -> Result<(), AppError> {
    let (until, deadline) = scope.response_deadline(until)?;
    let previous = response.extensions_mut().remove::<OwnerDeadline>();
    response.extensions_mut().insert(OwnerDeadline {
        until: previous.as_ref().map_or(until, |old| old.until.min(until)),
        deadline: previous.map_or(deadline, |old| old.deadline.min(deadline)),
    });
    Ok(())
}

pub(super) fn guard(
    mut response: Response,
    scope: Arc<WikiSpaceAdmission>,
) -> Result<Response, AppError> {
    let (mut until, mut deadline) = scope.response_deadline(i64::MAX)?;
    if let Some(owner) = response.extensions_mut().remove::<OwnerDeadline>() {
        until = until.min(owner.until);
        deadline = deadline.min(owner.deadline);
    }
    guard_until(response, until, deadline, move || scope.is_current())
}

pub(super) fn guard_identity(
    response: Response,
    lifetime: pdlc_auth_core::service_bridge::VerifiedTokenLifetime,
) -> Result<Response, AppError> {
    let until = lifetime.local_valid_until();
    let deadline = lifetime.monotonic_deadline();
    guard_until(response, until, deadline, move || lifetime.is_current())
}

fn guard_until(
    mut response: Response,
    until: i64,
    deadline: Instant,
    source_current: impl Fn() -> bool + Send + Sync + 'static,
) -> Result<Response, AppError> {
    let current = move || {
        source_current() && chrono::Utc::now().timestamp() < until && Instant::now() < deadline
    };
    if !current() {
        return Err(AppError::Forbidden);
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("private, no-store"),
    );
    let (parts, body) = response.into_parts();
    let stream = futures_util::stream::try_unfold(
        (body.into_data_stream(), current),
        move |(mut stream, current)| async move {
            let expired = || {
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "space_admission_expired",
                )
            };
            if !current() {
                return Err(expired());
            }
            let chunk = tokio::select! {
                biased;
                _=tokio::time::sleep_until(deadline.into())=>return Err(expired()),
                chunk=stream.next()=>chunk,
            };
            if !current() {
                return Err(expired());
            }
            match chunk {
                Some(Ok(bytes)) => Ok(Some((bytes, (stream, current)))),
                Some(Err(error)) => Err(std::io::Error::other(error)),
                None => Ok(None),
            }
        },
    );
    Ok(Response::from_parts(parts, Body::from_stream(stream)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Bytes, to_bytes},
        response::IntoResponse,
    };
    use pdlc_shared::space_access::*;
    use shared::wiki_space_access::WikiSpaceSubject;
    use uuid::Uuid;

    fn scope() -> Arc<WikiSpaceAdmission> {
        let identity = CentralIdentity {
            issuer: "https://auth.example.test".into(),
            subject: Uuid::new_v4().to_string(),
        };
        let now = chrono::Utc::now().timestamp();
        Arc::new(
            WikiSpaceAdmission::confirmed(
                WikiSpaceSubject {
                    user_id: Uuid::new_v4(),
                    identity: identity.clone(),
                    token_expiry: now + 60,
                    global_role: "user".into(),
                    may_read: true,
                    may_write: false,
                },
                SpaceAdmission {
                    inheritance: None,
                    schema_version: 1,
                    identity,
                    identity_active: true,
                    policy_revision: 1,
                    policy: SpaceAccessPolicy::Selected { spaces: vec![] },
                    validity: AdmissionValidity::confirmed(now, now + 60, i64::MAX).unwrap(),
                    spaces: vec![],
                },
                vec![],
            )
            .unwrap(),
        )
    }

    #[tokio::test]
    async fn current_response_keeps_bytes_and_is_private_even_with_untrusted_headers() {
        let scope = scope();
        let response = (
            [
                (header::CACHE_CONTROL, "public, max-age=3600"),
                ("x-valid-until".parse().unwrap(), "0"),
            ],
            "visible",
        )
            .into_response();
        let response = guard(response, scope).unwrap();
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(
            to_bytes(response.into_body(), 100).await.unwrap(),
            "visible"
        );
    }

    #[tokio::test]
    async fn shorter_owner_deadline_cancels_an_idle_body_without_a_later_chunk() {
        let scope = scope();
        let (sender, receiver) = tokio::sync::oneshot::channel::<Bytes>();
        let body = Body::from_stream(futures_util::stream::once(async move {
            Ok::<_, std::io::Error>(receiver.await.expect("sender remains alive"))
        }));
        let mut response = Response::new(body);
        let until = chrono::Utc::now().timestamp() + 1;
        restrict(&mut response, &scope, until).unwrap();
        restrict(&mut response, &scope, i64::MAX).unwrap();
        let response = guard(response, scope).unwrap();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            to_bytes(response.into_body(), 100),
        )
        .await
        .expect("idle body must finish at the owner deadline");
        assert!(result.is_err());
        assert!(
            sender
                .send(Bytes::from_static(b"retired private data"))
                .is_err()
        );
    }

    #[test]
    fn expired_owner_cannot_publish_headers() {
        let mut response = ().into_response();
        assert!(restrict(&mut response, &scope(), chrono::Utc::now().timestamp()).is_err());
    }
}
