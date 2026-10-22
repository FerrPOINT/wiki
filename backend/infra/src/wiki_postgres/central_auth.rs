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
