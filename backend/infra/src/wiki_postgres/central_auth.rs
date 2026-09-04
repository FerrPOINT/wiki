//! Wiki wiring of the shared central-auth bridge.
//!
//! JWKS mechanics live in `sdlc_auth_core::service_bridge`; this file only
//! maps bridge outcomes to wiki types.

use sdlc_auth_core::service_bridge::{BridgeOutcome, ServiceBridge};
use shared::{AppError, AuthConfig, WikiClaims};

/// Env prefix: WIKI_AUTH__CENTRAL_{JWKS_URI,ISSUER}.
pub static BRIDGE: ServiceBridge = ServiceBridge::new("WIKI_AUTH__CENTRAL");

/// Central-first bearer validation. `Ok(None)` = legacy path.
pub async fn try_central(token: &str) -> Result<Option<sdlc_auth_core::AuthContext>, AppError> {
    match BRIDGE.try_token(token).await {
        BridgeOutcome::Validated(ctx) => Ok(Some(ctx)),
        BridgeOutcome::NotOurs | BridgeOutcome::NotConfigured => Ok(None),
        BridgeOutcome::Expired => Err(AppError::Unauthorized),
        BridgeOutcome::Invalid(reason) => {
            tracing::debug!(reason, "bearer is not a valid central token; legacy path");
            Ok(None)
        }
    }
}

pub fn legacy_config_ok(_config: &AuthConfig) -> bool {
    true
}

/// Maps a central identity onto wiki claims; `user_id` is resolved by the
/// caller from the verified email (find-or-link).
pub fn claims_for(ctx: &sdlc_auth_core::AuthContext, wiki_user_id: String) -> WikiClaims {
    WikiClaims {
        user_id: wiki_user_id,
        session_id: ctx.session_id.clone(),
        request_id: None,
    }
}
