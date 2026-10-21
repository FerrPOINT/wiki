//! Wiki wiring of the shared central-auth bridge.
//!
//! JWKS mechanics live in `pdlc_auth_core::service_bridge`; this file only
//! maps bridge outcomes to wiki types.

use pdlc_auth_core::service_bridge::{BridgeOutcome, ServiceBridge};
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
    let (outcome, name, policy) = BRIDGE.try_token_with_policy(token).await;
    match outcome {
        BridgeOutcome::Validated(ctx) => Ok(Some((ctx, name, policy))),
        BridgeOutcome::NotOurs | BridgeOutcome::NotConfigured => Ok(None),
        BridgeOutcome::Forbidden => Err(AppError::Forbidden),
        BridgeOutcome::Expired => Err(AppError::Unauthorized),
        BridgeOutcome::Invalid(reason) => {
            tracing::debug!(reason, "central token rejected");
            Err(AppError::Unauthorized)
        }
        BridgeOutcome::Unavailable => Err(AppError::Unavailable(
            "Central Auth is temporarily unavailable".into(),
        )),
    }
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
    }
}
