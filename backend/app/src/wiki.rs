use domain::wiki::{DocumentSlug, DocumentType, EvidenceType, GlobalRole, PhaseKey, SpaceKey};
use domain::wiki::{SpaceRole, TaskKey};
use sha2::{Digest, Sha256};
use shared::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiSpaceAccess {
    View,
    Edit,
    Admin,
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
}
