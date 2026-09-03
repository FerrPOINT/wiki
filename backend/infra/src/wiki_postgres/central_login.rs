//! Login proxy to the central fleet auth-server.
//!
//! When `WIKI_AUTH__CENTRAL_LOGIN_URL` is set, POST /api/v1/auth/login first
//! tries the central auth-server with the same email/password. On success the
//! central access/refresh tokens are returned to the caller verbatim — wiki
//! already validates central ES256 tokens on every protected route, so the
//! rest of the flow (shadow user, roles) is unchanged. When central auth is
//! unreachable or rejects the credentials, the legacy local login path runs.

#[derive(serde::Deserialize)]
pub struct CentralAuthPair {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

pub struct CentralLoginConfig {
    pub login_url: String,
    pub timeout_secs: u64,
}

pub fn central_login_config() -> Option<CentralLoginConfig> {
    let url = std::env::var("WIKI_AUTH__CENTRAL_LOGIN_URL").ok()?;
    if url.trim().is_empty() {
        return None;
    }
    Some(CentralLoginConfig {
        login_url: url,
        timeout_secs: std::env::var("WIKI_AUTH__CENTRAL_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5),
    })
}

/// Tries the central login. `Ok(None)` — central auth not configured.
/// `Err(None)` — central rejected the credentials (fall back to legacy).
/// `Err(Some(err))` — request-shape error (treated as fallback too, logged).
pub async fn try_central_login(
    config: &CentralLoginConfig,
    email: &str,
    password: &str,
) -> Result<Option<CentralAuthPair>, Option<String>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()
        .map_err(|e| Some(e.to_string()))?;
    let response = client
        .post(&config.login_url)
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, url = %config.login_url, "central login unreachable");
            Some(e.to_string())
        })?;
    if !response.status().is_success() {
        // Central rejected these credentials — not an infrastructure error.
        return Err(None);
    }
    let pair = response
        .json::<CentralAuthPair>()
        .await
        .map_err(|e| Some(e.to_string()))?;
    Ok(Some(pair))
}

/// Builds the legacy-shaped WikiAuthResponse carrying central tokens.
pub fn central_response(pair: CentralAuthPair, email: &str) -> shared::WikiAuthResponse {
    shared::WikiAuthResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        token_type: pair.token_type.unwrap_or_else(|| "Bearer".into()),
        // The frontend only reads the email for greeting; user details come
        // from /users/me, which resolves the shadow user by the token.
        user_id: String::new(), // resolved by /users/me from the token
        email: email.to_string(),
        username: email.split('@').next().unwrap_or("central").to_string(),
        display_name: email.split('@').next().unwrap_or("central").to_string(),
        expires_in: pair.expires_in.unwrap_or(900) as u64,
    }
}
