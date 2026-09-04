//! Login proxy to the central fleet auth-server via the shared
//! `sdlc_auth_core::service_bridge` (env: WIKI_AUTH__CENTRAL_LOGIN_URL).

use super::central_auth::BRIDGE;
use sdlc_auth_core::service_bridge::CentralTokenPair;

/// Central login proxy; `None` = not configured / rejected / unreachable
/// (transport errors are logged, local login stays the fallback).
pub async fn try_central_login(email: &str, password: &str) -> Option<CentralTokenPair> {
    match BRIDGE.try_login(email, password).await {
        Ok(pair) => pair,
        Err(transport) => {
            tracing::warn!(%transport, "central login failed; local fallback");
            None
        }
    }
}

/// Builds the legacy-shaped WikiAuthResponse carrying central tokens.
pub fn central_response(pair: CentralTokenPair, email: &str) -> shared::WikiAuthResponse {
    shared::WikiAuthResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token.unwrap_or_default(),
        token_type: pair.token_type.unwrap_or_else(|| "Bearer".into()),
        // The frontend only reads the email for greeting; user details come
        // from /users/me, which resolves the shadow user by the token.
        user_id: String::new(), // resolved by /users/me from the token
        email: email.to_string(),
        username: email.split('@').next().unwrap_or("central").to_string(),
        display_name: email.split('@').next().unwrap_or("central").to_string(),
        expires_in: pair.expires_in.unwrap_or(900),
    }
}
