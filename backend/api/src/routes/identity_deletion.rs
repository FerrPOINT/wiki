//! Private coordinator boundary; no user JIT, role elevation, or admission lookup.
use super::wiki::WikiBackend;
use axum::{
    Extension, Json, Router,
    extract::Path,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
    routing::post,
};
use shared::{AppError, identity_deletion::*};
use uuid::Uuid;

struct RegisteredCoordinator {
    window: IdentityDeletionWindow,
}
impl RegisteredCoordinator {
    async fn verify(headers: &HeaderMap, request: &DeletionRequest) -> Result<Self, AppError> {
        let token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;
        let verified = infra::wiki_postgres::central_auth::BRIDGE
            .try_verified_token(token)
            .await
            .map_err(|error| match error {
                pdlc_auth_core::service_bridge::VerifiedBridgeError::Unavailable
                | pdlc_auth_core::service_bridge::VerifiedBridgeError::NotConfigured => {
                    AppError::Unavailable("identity_owner_confirmation_unavailable".into())
                }
                _ => AppError::Unauthorized,
            })?;
        let configured = std::env::var("WIKI_AUTH__CENTRAL_ISSUER")
            .map_err(|_| AppError::Unavailable("identity_owner_issuer_unconfigured".into()))?;
        let registered = std::env::var("WIKI_IDENTITY_DELETION_COORDINATOR_SUBJECTS")
            .unwrap_or_default()
            .split(',')
            .any(|s| !s.trim().is_empty() && s.trim() == verified.context.user_id);
        if !may_coordinate(
            &verified.context,
            verified.subject_role.as_deref(),
            registered,
        ) || verified.issuer != configured
            || request.identity.issuer != configured
        {
            return Err(AppError::Forbidden);
        }
        let validity = pdlc_shared::space_access::AdmissionValidity::confirmed(
            verified.lifetime.source_observed_at,
            verified.lifetime.local_valid_until(),
            i64::MAX,
        )
        .ok_or(AppError::Unauthorized)?;
        let remaining = chrono::DateTime::from_timestamp(validity.valid_until, 0)
            .ok_or(AppError::Unauthorized)?
            .signed_duration_since(chrono::Utc::now())
            .to_std()
            .map_err(|_| AppError::Unauthorized)?;
        let deadline = std::time::Instant::now()
            .checked_add(remaining)
            .ok_or(AppError::Unauthorized)?
            .min(verified.lifetime.monotonic_deadline());
        let window = IdentityDeletionWindow::restricted(deadline, validity.valid_until);
        window.require_current()?;
        Ok(Self { window })
    }
}
fn may_coordinate(ctx: &pdlc_auth_core::AuthContext, role: Option<&str>, registered: bool) -> bool {
    role == Some("service_account")
        && pdlc_auth_core::is_personal_token(&ctx.token)
        && ctx.session_id.is_none()
        && registered
        && ctx.has_scope("wiki:identity-lifecycle")
}
pub fn router() -> Router<std::sync::Arc<app::WikiAppContext>> {
    Router::new()
        .route(
            "/api/internal/v1/user-deletions/{operation_id}/check",
            post(check),
        )
        .route(
            "/api/internal/v1/user-deletions/{operation_id}/freeze",
            post(freeze),
        )
        .route(
            "/api/internal/v1/user-deletions/{operation_id}/release",
            post(release),
        )
        .layer(axum::extract::DefaultBodyLimit::max(16_384))
        .layer(axum::middleware::from_fn(
            |request: axum::extract::Request, next: axum::middleware::Next| async move {
                match tokio::time::timeout(std::time::Duration::from_secs(9), next.run(request))
                    .await
                {
                    Ok(response) => response,
                    Err(_) => {
                        AppError::Unavailable("identity_owner_timeout".into()).into_response()
                    }
                }
            },
        ))
        .layer(axum::middleware::map_response(
            |mut response: Response| async move {
                response.headers_mut().insert(
                    header::CACHE_CONTROL,
                    "no-store".parse().expect("static header"),
                );
                response
            },
        ))
}
macro_rules! owner_route {
 ($name:ident,$path:literal,$output:ty,$method:ident)=>{
  #[utoipa::path(post,path=$path,tag="identity-lifecycle",params(("operation_id"=Uuid,Path)),request_body=DeletionRequest,responses((status=200,body=$output),(status=401,description="Expired or invalid coordinator"),(status=403,description="Registered machine capability required"),(status=409,description="Operation conflict"),(status=503,description="Unknown binding or unavailable owner")))]
  pub async fn $name(Extension(backend):Extension<WikiBackend>,Path(operation):Path<Uuid>,headers:HeaderMap,Json(request):Json<DeletionRequest>)->Result<Json<$output>,AppError>{
   if !request.valid() || operation!=request.operation_id{return Err(AppError::invalid_input("invalid_identity_deletion_request"));}
   let coordinator=RegisteredCoordinator::verify(&headers,&request).await?;
   let result=backend.namespace_port()?.$method(&request,&coordinator.window).await?;
   coordinator.window.require_current()?;
   Ok(Json(result))
  }
 }
}
owner_route!(
    check,
    "/api/internal/v1/user-deletions/{operation_id}/check",
    OwnerReferenceSummary,
    identity_deletion_check
);
owner_route!(
    freeze,
    "/api/internal/v1/user-deletions/{operation_id}/freeze",
    OwnerFreezeReceipt,
    identity_deletion_freeze
);
owner_route!(
    release,
    "/api/internal/v1/user-deletions/{operation_id}/release",
    OwnerReleaseReceipt,
    identity_deletion_release
);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn human_and_unregistered_or_ordinary_machine_cannot_coordinate() {
        let mut ctx = pdlc_auth_core::AuthContext {
            user_id: Uuid::new_v4().to_string(),
            role: Some("admin".into()),
            scopes: ["wiki:identity-lifecycle".into()].into(),
            session_id: Some(Uuid::new_v4().to_string()),
            email: None,
            token: "sdlc_pat_synthetic".into(),
        };
        assert!(!may_coordinate(&ctx, Some("admin"), true));
        assert!(!may_coordinate(&ctx, Some("service_account"), true));
        ctx.session_id = None;
        assert!(!may_coordinate(&ctx, Some("service_account"), false));
        ctx.scopes = ["wiki:write".into()].into();
        assert!(!may_coordinate(&ctx, Some("service_account"), true));
        ctx.scopes = ["wiki:identity-lifecycle".into()].into();
        assert!(may_coordinate(&ctx, Some("service_account"), true));
        ctx.token = "synthetic-jwt".into();
        assert!(!may_coordinate(&ctx, Some("service_account"), true));
    }
    #[tokio::test]
    async fn private_http_routes_require_machine_auth_and_never_cache_rejections() {
        let ctx = std::sync::Arc::new(app::WikiAppContext::new(std::sync::Arc::new(
            shared::AppConfig::default(),
        )));
        let router = router()
            .layer(Extension(WikiBackend::memory()))
            .with_state(ctx);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        struct Server(tokio::task::JoinHandle<()>);
        impl Drop for Server {
            fn drop(&mut self) {
                self.0.abort();
            }
        }
        let _server = Server(tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap()
        }));
        let client = reqwest::Client::new();
        let command = DeletionRequest {
            schema_version: 1,
            operation_id: Uuid::new_v4(),
            identity: pdlc_shared::space_access::CentralIdentity {
                issuer: "https://auth.example.test".into(),
                subject: Uuid::new_v4().to_string(),
            },
            inventory_revision: Uuid::new_v4(),
        };
        for action in ["check", "freeze", "release"] {
            let response = client
                .post(format!(
                    "{base}/api/internal/v1/user-deletions/{}/{action}",
                    command.operation_id
                ))
                .json(&command)
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
            assert_eq!(response.headers()["cache-control"], "no-store");
        }
        let response = client
            .post(format!(
                "{base}/api/internal/v1/user-deletions/{}/freeze",
                Uuid::new_v4()
            ))
            .json(&command)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let response = client
            .post(format!(
                "{base}/api/internal/v1/user-deletions/{}/freeze",
                command.operation_id
            ))
            .header("content-type", "application/json")
            .body("x".repeat(17000))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(response.headers()["cache-control"], "no-store");
    }
}
