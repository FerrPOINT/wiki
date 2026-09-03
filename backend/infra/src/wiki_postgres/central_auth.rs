//! Bridge between wiki auth and the central fleet auth-server (services-base).
//!
//! When `WIKI_AUTH__CENTRAL_JWKS_URI` is configured, access tokens issued by
//! the central auth-server (ES256, audience `sdlc`) are validated against its
//! JWKS and mapped onto wiki users by verified email. Local HS256 sessions
//! keep working as a fallback, so the migration is zero-downtime:
//!
//!   central token -> JWKS validate -> find-or-link local user -> WikiClaims
//!   legacy token  -> existing per-repo session check (unchanged)

use sdlc_auth_core::{AuthContext, JwksCache, Validator};
use shared::{AppError, AuthConfig, WikiClaims};
use tokio::sync::OnceCell;

static CENTRAL: OnceCell<Option<CentralAuth>> = OnceCell::const_new();

pub struct CentralAuth {
    validator: Validator,
    #[allow(dead_code)] // kept for future direct JWKS access (rotation checks)
    jwks: std::sync::Arc<JwksCache>,
}

/// Reads `WIKI_AUTH__CENTRAL_JWKS_URI` / `WIKI_AUTH__CENTRAL_ISSUER` once.
/// Returns None when central auth is not configured (legacy-only mode).
pub async fn central() -> Option<&'static CentralAuth> {
    CENTRAL
        .get_or_init(|| async {
            let uri = std::env::var("WIKI_AUTH__CENTRAL_JWKS_URI").ok()?;
            let issuer = std::env::var("WIKI_AUTH__CENTRAL_ISSUER")
                .unwrap_or_else(|_| "http://127.0.0.1:7701".into());
            match JwksCache::connect(&uri).await {
                Ok(jwks) => {
                    let jwks = std::sync::Arc::new(jwks);
                    let validator = Validator::Jwks {
                        jwks: jwks.clone(),
                        issuer: issuer.into(),
                    };
                    jwks.clone().spawn_refresh(std::time::Duration::from_secs(3600));
                    tracing::info!(jwks_uri = %uri, "central auth enabled");
                    Some(CentralAuth { validator, jwks })
                }
                Err(error) => {
                    tracing::warn!(%error, jwks_uri = %uri, "central auth unavailable; falling back to legacy sessions");
                    None
                }
            }
        })
        .await
        .as_ref()
}

/// Attempts central-token validation. Ok(None) = not a central token (caller
/// falls back to the legacy path).
pub async fn try_central(token: &str) -> Result<Option<AuthContext>, AppError> {
    let Some(central) = central().await else {
        return Ok(None);
    };
    // Central tokens are ES256 with a kid header; legacy HS256 tokens fail
    // kid resolution and are reported as Jwks errors -> treat as "not ours".
    match central.validator.validate(token) {
        Ok(ctx) => Ok(Some(ctx)),
        // kid resolution failure = legacy token, not ours
        Err(sdlc_auth_core::AuthError::Jwks(_)) => Ok(None),
        Err(sdlc_auth_core::AuthError::Expired) => Err(AppError::Unauthorized),
        Err(other) => {
            tracing::warn!(error = %other, "central token validation failed");
            Ok(None)
        }
    }
}

/// Keeps the signature honest for future use (session sync).
#[allow(dead_code)]
pub fn legacy_config_ok(_config: &AuthConfig) -> bool {
    true
}

/// Maps a central identity onto wiki claims; `user_id` is resolved by the
/// caller from the verified email (find-or-link).
pub fn claims_for(ctx: &AuthContext, wiki_user_id: String) -> WikiClaims {
    WikiClaims {
        user_id: wiki_user_id,
        session_id: ctx.session_id.clone(),
        request_id: None,
    }
}
