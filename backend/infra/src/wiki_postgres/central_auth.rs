//! Wiki wiring of the shared central-auth bridge.
//!
//! JWKS mechanics live in `pdlc_auth_core::service_bridge`; this file only
//! maps bridge outcomes to wiki types.

use pdlc_auth_core::service_bridge::ServiceBridge;
use shared::{AppError, AuthConfig, WikiClaims};

/// Env prefix: WIKI_AUTH__CENTRAL_{JWKS_URI,ISSUER}.
pub static BRIDGE: ServiceBridge = ServiceBridge::for_application("WIKI_AUTH__CENTRAL", "wiki");

/// Central-first bearer validation. `Ok(None)` = legacy path.
pub async fn try_central(token: &str) -> Result<Option<pdlc_auth_core::AuthContext>, AppError> {
    Ok(try_central_with_name(token)
        .await?
        .map(|(context, _)| context))
}

pub async fn try_central_with_name(
    token: &str,
) -> Result<Option<(pdlc_auth_core::AuthContext, Option<String>)>, AppError> {
    Ok(try_central_with_policy(token)
        .await?
        .map(|(context, name, _)| (context, name)))
}

pub async fn try_central_with_policy(
    token: &str,
) -> Result<
    Option<(
        pdlc_auth_core::AuthContext,
        Option<String>,
        Option<pdlc_auth_core::application_access::VerifiedApplicationAccess>,
    )>,
    AppError,
> {
    match verified_for_request(token).await {
        Ok(mut actor) => {
            if actor.context.role.is_none() {
                actor.context.role = actor.subject_role.clone();
            }
            Ok(Some((
                actor.context,
                actor.display_name,
                actor.application_access,
            )))
        }
        Err(
            pdlc_auth_core::service_bridge::VerifiedBridgeError::NotOurs
            | pdlc_auth_core::service_bridge::VerifiedBridgeError::NotConfigured,
        ) => Ok(None),
        Err(error) => Err(super::admission::auth_error(error)),
    }
}

// The HTTP adapter and persistent port share one already verified request. This
// is process-local, is not deserialized, and is not inherited by spawned jobs.
tokio::task_local! {
    static VERIFIED_REQUEST: pdlc_auth_core::service_bridge::VerifiedCentralToken;
}
pub async fn with_verified_request<F: std::future::Future>(
    actor: pdlc_auth_core::service_bridge::VerifiedCentralToken,
    future: F,
) -> F::Output {
    VERIFIED_REQUEST.scope(actor, future).await
}
pub async fn verified_for_request(
    token: &str,
) -> Result<
    pdlc_auth_core::service_bridge::VerifiedCentralToken,
    pdlc_auth_core::service_bridge::VerifiedBridgeError,
> {
    if let Ok(actor) = VERIFIED_REQUEST.try_with(Clone::clone) {
        if actor.context.token != token || !actor.lifetime.is_current() {
            return Err(pdlc_auth_core::service_bridge::VerifiedBridgeError::Forbidden);
        }
        return Ok(actor);
    }
    BRIDGE.try_verified_token(token).await
}

pub fn legacy_config_ok(_config: &AuthConfig) -> bool {
    true
}

/// Maps a central identity onto wiki claims; `user_id` is resolved by the
/// caller from the immutable central subject, never from an email match.
pub fn claims_for(ctx: &pdlc_auth_core::AuthContext, wiki_user_id: String) -> WikiClaims {
    WikiClaims {
        user_id: wiki_user_id,
        session_id: ctx.session_id.clone(),
        request_id: None,
        admission: None,
    }
}

#[cfg(test)]
pub(super) async fn verified_test_identity(
    prefix: &'static str,
    context: &pdlc_auth_core::AuthContext,
    access: Option<pdlc_auth_core::application_access::VerifiedApplicationAccess>,
) -> pdlc_auth_core::service_bridge::VerifiedCentralToken {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    struct Fixture {
        server: tokio::task::JoinHandle<()>,
        environment: Vec<(String, Option<std::ffi::OsString>)>,
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            self.server.abort();
            for (key, previous) in &self.environment {
                // Each caller owns its test-only prefix; production settings are never read.
                unsafe { match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                } }
            }
        }
    }
    let issuer = access.as_ref().map_or("http://wiki-owner.example.test", |proof| proof.decision.issuer.as_str()).to_owned();
    let now = chrono::Utc::now();
    let mut body = serde_json::json!({"sub":context.user_id,
        "email":context.email.as_deref().unwrap_or("fixture@example.test"),
        "subject_role":context.role.as_deref().unwrap_or("user"), "scopes":context.scopes,
        "observed_at":now,"expires_at":now + chrono::Duration::minutes(2)});
    if let Some(access) = access { body["application_access"] = serde_json::to_value(access.decision).unwrap(); }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.windows(4).any(|part| part == b"\r\n\r\n") {
                let mut buffer = [0; 1024];
                let size = stream.read(&mut buffer).await.unwrap();
                assert_ne!(size, 0);
                request.extend_from_slice(&buffer[..size]);
            }
            let request = String::from_utf8(request).unwrap();
            let value = if request.starts_with("GET /oidc/jwks ") { serde_json::json!({"keys":[]}) } else {
                assert!(request.starts_with("GET /auth/tokens/introspect?include_lifetime=true "));
                assert!(request.to_lowercase().contains("authorization: bearer sdlc_pat_wiki-fixture"));
                body.clone()
            };
            let body = value.to_string();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let environment = [(format!("{prefix}_JWKS_URI"), format!("http://{address}/oidc/jwks")),
        (format!("{prefix}_ISSUER"), issuer), (format!("{prefix}_APPLICATION_ACCESS_REQUIRED"), "false".into())]
        .into_iter().map(|(key, value)| {
            let previous = std::env::var_os(&key);
            unsafe { std::env::set_var(&key, value) };
            (key, previous)
        }).collect();
    let _fixture = Fixture { server, environment };
    ServiceBridge::for_application(prefix, "wiki").try_verified_token("sdlc_pat_wiki-fixture").await.unwrap()
}
