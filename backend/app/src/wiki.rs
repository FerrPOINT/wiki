use chrono::{Duration, Utc};
use domain::wiki::{DocumentSlug, DocumentType, EvidenceType, GlobalRole, PhaseKey, SpaceKey};
use domain::wiki::{SpaceRole, TaskKey};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::{AppError, AuthConfig};
use uuid::Uuid;

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
            refresh_cookie_name: "refresh_token".to_string(),
            refresh_cookie_secure: true,
            refresh_cookie_same_site: "Lax".to_string(),
            refresh_cookie_domain: None,
            refresh_cookie_path: "/api/v1/auth".to_string(),
        }
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
}
