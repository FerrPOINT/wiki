use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use domain::wiki::{DocumentSlug, DocumentType, EvidenceType, GlobalRole, PhaseKey, SpaceKey};
use domain::wiki::{SpaceRole, TaskKey};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::{AppConfig, AppError, AuthConfig};
use uuid::Uuid;

#[derive(Clone)]
pub struct WikiAppContext {
    pub config: Arc<AppConfig>,
}

impl WikiAppContext {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    pub fn server_addr(&self) -> String {
        self.config.server_addr()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiSpaceAccess {
    View,
    Edit,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiTokenClaims {
    pub sub: String,
    pub exp: usize,
    pub jti: String,
    pub typ: String,
}

#[derive(Debug, Clone)]
pub struct WikiTokenPair {
    pub session_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub expires_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiSearchCriteria {
    pub needle: String,
    pub evidence_like_pattern: String,
    pub space_key: Option<String>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub document_type: Option<&'static str>,
    pub include_archived: bool,
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiSettingsSnapshot {
    pub instance_name: String,
    pub api_base_path: String,
    pub default_space_key: String,
    pub default_language: String,
    pub timezone: String,
    pub registration_enabled: bool,
    pub public_links_enabled: bool,
    pub search_backend: String,
    pub storage_backend: String,
    pub max_upload_bytes: usize,
    pub markdown_renderer: String,
    pub html_sanitizer: String,
}

impl WikiSettingsSnapshot {
    pub fn from_config(config: &AppConfig) -> Self {
        Self::from_values(
            config.auth.registration_enabled,
            config.storage.max_upload_bytes,
        )
    }

    pub fn from_values(registration_enabled: bool, max_upload_bytes: usize) -> Self {
        Self {
            instance_name: "Wiki".to_string(),
            api_base_path: "/api/v1".to_string(),
            default_space_key: "SDLC".to_string(),
            default_language: "ru".to_string(),
            timezone: "Europe/Moscow".to_string(),
            registration_enabled,
            public_links_enabled: false,
            search_backend: "PostgreSQL FTS".to_string(),
            storage_backend: "local".to_string(),
            max_upload_bytes,
            markdown_renderer: "comrak".to_string(),
            html_sanitizer: "ammonia".to_string(),
        }
    }
}

pub fn clamp_limit(limit: Option<usize>, max: i64) -> i64 {
    limit.unwrap_or(max as usize).clamp(1, max as usize) as i64
}

pub fn normalize_required(value: &str, field: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::invalid_input(format!("{field} is required")));
    }
    Ok(value.to_string())
}

pub fn normalize_space_key(value: &str) -> Result<String, AppError> {
    Ok(SpaceKey::parse(value)?.to_string())
}

pub fn normalize_slug(value: &str) -> Result<String, AppError> {
    Ok(DocumentSlug::parse(value)?.to_string())
}

pub fn normalize_task_key(value: &str) -> Result<String, AppError> {
    Ok(TaskKey::parse(value)?.to_string())
}

pub fn normalize_phase_key(value: &str) -> Result<String, AppError> {
    Ok(PhaseKey::parse(value)?.to_string())
}

pub fn normalize_document_type(value: &str, allow_page: bool) -> Result<&'static str, AppError> {
    let document_type = value
        .trim()
        .parse::<DocumentType>()
        .map_err(|_| AppError::invalid_input("unsupported document type"))?;
    if !allow_page && document_type == DocumentType::Page {
        return Err(AppError::invalid_input("unsupported document type"));
    }
    Ok(document_type.as_str())
}

pub fn build_wiki_search_criteria(
    q: Option<&str>,
    space: Option<&str>,
    task_key: Option<&str>,
    phase_key: Option<&str>,
    document_type: Option<&str>,
    include_archived: Option<bool>,
    limit: Option<usize>,
) -> Result<WikiSearchCriteria, AppError> {
    let needle = q.unwrap_or_default().trim().to_string();
    Ok(WikiSearchCriteria {
        evidence_like_pattern: evidence_like_pattern(&needle),
        needle,
        space_key: space.map(normalize_space_key).transpose()?,
        task_key: task_key.map(normalize_task_key).transpose()?,
        phase_key: phase_key.map(normalize_phase_key).transpose()?,
        document_type: document_type
            .map(|value| normalize_document_type(value, true))
            .transpose()?,
        include_archived: include_archived.unwrap_or(false),
        limit: clamp_limit(limit, 50),
    })
}

fn evidence_like_pattern(value: &str) -> String {
    if value.is_empty() {
        return "%%".to_string();
    }

    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('%');
    for ch in value.chars().flat_map(char::to_lowercase) {
        if matches!(ch, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped.push('%');
    escaped
}

pub fn normalize_evidence_type(value: &str) -> Result<&'static str, AppError> {
    value
        .trim()
        .parse::<EvidenceType>()
        .map(|kind| kind.as_str())
        .map_err(|_| AppError::invalid_input("evidence_type must be external_url or uploaded_file"))
}

pub fn normalize_space_role(value: &str) -> Result<&'static str, AppError> {
    value
        .trim()
        .parse::<SpaceRole>()
        .map(|role| role.as_str())
        .map_err(|_| AppError::invalid_input("space member role must be admin, editor or viewer"))
}

pub fn space_role_allows(role: Option<&str>, required: WikiSpaceAccess) -> bool {
    matches!(
        (
            role.and_then(|role| role.parse::<SpaceRole>().ok()),
            required
        ),
        (Some(SpaceRole::Admin), _)
            | (
                Some(SpaceRole::Editor),
                WikiSpaceAccess::View | WikiSpaceAccess::Edit
            )
            | (Some(SpaceRole::Viewer), WikiSpaceAccess::View)
    )
}

pub fn global_role_from_request(value: &str) -> Result<&'static str, AppError> {
    match value.trim() {
        "editor" | "viewer" => Ok(GlobalRole::User.as_str()),
        other => other
            .parse::<GlobalRole>()
            .map(|role| role.as_str())
            .map_err(|_| {
                AppError::invalid_input("user role must be admin, user, editor or viewer")
            }),
    }
}

pub fn default_username(email: &str) -> String {
    let local = email.split('@').next().unwrap_or("admin");
    let username = slugify(local);
    if username.is_empty() {
        "admin".to_string()
    } else {
        username
    }
}

pub fn markdown_to_text(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| {
            line.trim()
                .trim_start_matches('#')
                .trim_start_matches(['-', '*', '>', ' '])
                .trim()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn slugify(value: &str) -> String {
    let slug: String = value
        .chars()
        .flat_map(|ch| ch.to_lowercase())
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect();
    slug.split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn snippet(markdown: &str) -> String {
    let normalized = markdown
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    normalized.chars().take(180).collect()
}

pub fn checksum(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(AppError::internal)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHash, PasswordVerifier},
    };
    let parsed = PasswordHash::new(hash).map_err(AppError::internal)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn create_token(
    config: &AuthConfig,
    user_id: Uuid,
    session_id: Uuid,
    token_type: &str,
    ttl: Duration,
) -> Result<String, AppError> {
    let exp = Utc::now() + ttl;
    let claims = WikiTokenClaims {
        sub: user_id.to_string(),
        exp: exp.timestamp() as usize,
        jti: session_id.to_string(),
        typ: token_type.to_string(),
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(AppError::internal)
}

pub fn create_wiki_session_token_pair(
    config: &AuthConfig,
    user_id: Uuid,
) -> Result<WikiTokenPair, AppError> {
    create_wiki_token_pair(config, user_id, Uuid::now_v7())
}

pub fn create_wiki_token_pair(
    config: &AuthConfig,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<WikiTokenPair, AppError> {
    let now = Utc::now();
    let access_ttl = Duration::minutes(config.access_token_ttl_minutes as i64);
    let refresh_ttl = Duration::days(config.refresh_token_ttl_days as i64);
    let access_expires_at = now + access_ttl;
    let refresh_expires_at = now + refresh_ttl;
    if refresh_expires_at <= access_expires_at {
        return Err(AppError::invalid_input(
            "refresh token lifetime must be longer than access token lifetime",
        ));
    }

    Ok(WikiTokenPair {
        session_id,
        access_token: create_token(config, user_id, session_id, "access", access_ttl)?,
        refresh_token: create_token(config, user_id, session_id, "refresh", refresh_ttl)?,
        access_expires_at,
        refresh_expires_at,
        expires_in: config.access_token_ttl_minutes * 60,
    })
}

pub fn decode_token(
    config: &AuthConfig,
    token: &str,
    expected_type: &str,
) -> Result<WikiTokenClaims, AppError> {
    let claims = jsonwebtoken::decode::<WikiTokenClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;
    if claims.typ == expected_type {
        Ok(claims)
    } else {
        Err(AppError::Unauthorized)
    }
}

pub fn safe_download_filename(file_name: &str) -> String {
    let sanitized: String = file_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "attachment.bin".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_auth_config() -> AuthConfig {
        AuthConfig {
            jwt_secret: "test-secret-32-chars-long!!!!!".to_string(),
            access_token_ttl_minutes: 15,
            refresh_token_ttl_days: 7,
            registration_enabled: true,
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: true,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        }
    }

    fn test_app_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.database.url = "postgres://wiki:secret-password@db.internal:5432/wiki".to_string();
        config.auth = test_auth_config();
        config.auth.jwt_secret = "super-secret-jwt".to_string();
        config.auth.registration_enabled = false;
        config.storage.dir = "/srv/wiki/private/uploads".to_string();
        config.storage.max_upload_bytes = 42 * 1024 * 1024;
        config
    }

    #[test]
    fn wiki_helpers_normalize_domain_values() {
        assert_eq!(normalize_space_key(" sdlc ").unwrap(), "SDLC");
        assert_eq!(
            normalize_slug(" Product-Requirements ").unwrap(),
            "product-requirements"
        );
        assert_eq!(normalize_task_key("SDLC-42").unwrap(), "SDLC-42");
        assert_eq!(
            normalize_phase_key(" Implementation ").unwrap(),
            "implementation"
        );
        assert_eq!(normalize_document_type("page", true).unwrap(), "page");
        assert!(normalize_document_type("page", false).is_err());
        assert_eq!(
            normalize_evidence_type("uploaded_file").unwrap(),
            "uploaded_file"
        );
        assert_eq!(normalize_space_role("editor").unwrap(), "editor");
    }

    #[test]
    fn wiki_helpers_keep_role_compatibility_without_expanding_global_roles() {
        assert_eq!(global_role_from_request("admin").unwrap(), "admin");
        assert_eq!(global_role_from_request("user").unwrap(), "user");
        assert_eq!(global_role_from_request("editor").unwrap(), "user");
        assert_eq!(global_role_from_request("viewer").unwrap(), "user");
        assert!(global_role_from_request("owner").is_err());
    }

    #[test]
    fn wiki_space_access_matches_mvp_roles() {
        assert!(space_role_allows(Some("admin"), WikiSpaceAccess::Admin));
        assert!(space_role_allows(Some("editor"), WikiSpaceAccess::Edit));
        assert!(space_role_allows(Some("viewer"), WikiSpaceAccess::View));
        assert!(!space_role_allows(Some("viewer"), WikiSpaceAccess::Edit));
        assert!(!space_role_allows(None, WikiSpaceAccess::View));
    }

    #[test]
    fn wiki_helpers_prepare_content_and_storage_names() {
        assert_eq!(normalize_required("  title  ", "title").unwrap(), "title");
        assert_eq!(clamp_limit(Some(500), 100), 100);
        assert_eq!(markdown_to_text("# Title\n\n- Item"), "Title Item");
        assert_eq!(slugify("Wiki MVP!"), "wiki-mvp");
        assert_eq!(snippet("a\n\nb"), "a b");
        assert_eq!(
            checksum(b"wiki"),
            "sha256:12a435ec8454c6d1c90a1d92812b09af11bee711fbe524d56a8f26ea7c5ccee8"
        );
        assert_eq!(safe_download_filename("report 1.md"), "report_1.md");
        assert_eq!(safe_download_filename(".."), "attachment.bin");
    }

    #[test]
    fn wiki_search_criteria_normalizes_filters_and_limits() {
        let criteria = build_wiki_search_criteria(
            Some("  Wiki MVP  "),
            Some("sdlc"),
            Some("SDLC-42"),
            Some("Implementation"),
            Some("requirements"),
            Some(true),
            Some(500),
        )
        .unwrap();

        assert_eq!(criteria.needle, "Wiki MVP");
        assert_eq!(criteria.evidence_like_pattern, "%wiki mvp%");
        assert_eq!(criteria.space_key.as_deref(), Some("SDLC"));
        assert_eq!(criteria.task_key.as_deref(), Some("SDLC-42"));
        assert_eq!(criteria.phase_key.as_deref(), Some("implementation"));
        assert_eq!(criteria.document_type, Some("requirements"));
        assert!(criteria.include_archived);
        assert_eq!(criteria.limit, 50);
    }

    #[test]
    fn wiki_search_criteria_escapes_evidence_like_wildcards() {
        let criteria = build_wiki_search_criteria(
            Some(r"  100%_Done\Release  "),
            None,
            None,
            None,
            None,
            None,
            Some(0),
        )
        .unwrap();

        assert_eq!(criteria.needle, r"100%_Done\Release");
        assert_eq!(criteria.evidence_like_pattern, r"%100\%\_done\\release%");
        assert_eq!(criteria.limit, 1);
    }

    #[test]
    fn wiki_search_criteria_treats_blank_query_as_unfiltered() {
        let criteria =
            build_wiki_search_criteria(Some("   "), None, None, None, None, None, None).unwrap();

        assert_eq!(criteria.needle, "");
        assert_eq!(criteria.evidence_like_pattern, "%%");
        assert_eq!(criteria.limit, 50);
    }

    #[test]
    fn wiki_settings_snapshot_exposes_only_safe_runtime_values() {
        let snapshot = WikiSettingsSnapshot::from_config(&test_app_config());

        assert_eq!(snapshot.instance_name, "Wiki");
        assert_eq!(snapshot.api_base_path, "/api/v1");
        assert_eq!(snapshot.default_space_key, "SDLC");
        assert_eq!(snapshot.default_language, "ru");
        assert_eq!(snapshot.timezone, "Europe/Moscow");
        assert!(!snapshot.registration_enabled);
        assert!(!snapshot.public_links_enabled);
        assert_eq!(snapshot.search_backend, "PostgreSQL FTS");
        assert_eq!(snapshot.storage_backend, "local");
        assert_eq!(snapshot.max_upload_bytes, 42 * 1024 * 1024);
        assert_eq!(snapshot.markdown_renderer, "comrak");
        assert_eq!(snapshot.html_sanitizer, "ammonia");

        let rendered = format!("{snapshot:?}");
        assert!(!rendered.contains("secret"));
        assert!(!rendered.contains("postgres://"));
        assert!(!rendered.contains("/srv/wiki"));
    }

    #[test]
    fn wiki_settings_snapshot_can_be_built_for_explicit_test_modes() {
        let snapshot = WikiSettingsSnapshot::from_values(true, 25 * 1024 * 1024);

        assert!(snapshot.registration_enabled);
        assert_eq!(snapshot.max_upload_bytes, 25 * 1024 * 1024);
    }

    #[test]
    fn wiki_auth_hashes_tokens_without_storing_plaintext() {
        let hash = hash_token("wiki-token");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, hash_token("wiki-token"));
        assert_ne!(hash, hash_token("other-token"));
    }

    #[test]
    fn wiki_auth_password_hash_verifies_and_rejects_wrong_password() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hash).unwrap());
        assert!(!verify_password("wrong password", &hash).unwrap());
        assert!(verify_password("password", "not-a-valid-hash").is_err());
    }

    #[test]
    fn wiki_auth_tokens_round_trip_session_and_type() {
        let config = test_auth_config();
        let user_id = Uuid::now_v7();
        let session_id = Uuid::now_v7();
        let token = create_token(&config, user_id, session_id, "access", Duration::minutes(5))
            .expect("token should be created");

        let claims = decode_token(&config, &token, "access").expect("token should decode");
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.jti, session_id.to_string());
        assert_eq!(claims.typ, "access");
        assert!(matches!(
            decode_token(&config, &token, "refresh"),
            Err(AppError::Unauthorized)
        ));
    }

    #[test]
    fn wiki_auth_token_pair_builds_access_refresh_for_same_session() {
        let config = test_auth_config();
        let user_id = Uuid::now_v7();
        let session_id = Uuid::now_v7();
        let pair = create_wiki_token_pair(&config, user_id, session_id)
            .expect("token pair should be created");

        assert_eq!(pair.session_id, session_id);
        assert_eq!(pair.expires_in, 900);
        assert!(pair.refresh_expires_at > pair.access_expires_at);

        let access = decode_token(&config, &pair.access_token, "access")
            .expect("access token should decode");
        let refresh = decode_token(&config, &pair.refresh_token, "refresh")
            .expect("refresh token should decode");
        assert_eq!(access.sub, user_id.to_string());
        assert_eq!(refresh.sub, user_id.to_string());
        assert_eq!(access.jti, session_id.to_string());
        assert_eq!(refresh.jti, session_id.to_string());
    }

    #[test]
    fn wiki_auth_token_pair_rejects_invalid_ttl_order() {
        let mut config = test_auth_config();
        config.access_token_ttl_minutes = 15;
        config.refresh_token_ttl_days = 0;

        let result = create_wiki_token_pair(&config, Uuid::now_v7(), Uuid::now_v7());

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}
