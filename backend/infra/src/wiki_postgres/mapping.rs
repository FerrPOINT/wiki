use app::wiki::snippet;
use chrono::{DateTime, Utc};
use shared::wiki_contract::*;
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

pub(super) fn user_response_from_row(row: &PgRow) -> WikiUserResponse {
    let role: String = row.get("global_role");
    WikiUserResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        email: row.get("email"),
        username: row.get("username"),
        display_name: row.get("display_name"),
        is_system_admin: role == "admin",
        role,
        active: row.get("is_active"),
    }
}

pub(super) fn space_response_from_row(row: &PgRow) -> SpaceResponse {
    let description: String = row.get("description");
    SpaceResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        key: row.get("key"),
        name: row.get("name"),
        description: if description.trim().is_empty() {
            None
        } else {
            Some(description)
        },
        owner_id: row.get::<Uuid, _>("owner_id").to_string(),
        status: row.get("status"),
        document_count: count_to_usize(row.get("document_count")),
        member_count: count_to_usize(row.get("member_count")),
        created_at: to_iso(row.get("created_at")),
        updated_at: to_iso(row.get("updated_at")),
    }
}

pub(super) fn space_member_response_from_row(row: &PgRow) -> SpaceMemberResponse {
    SpaceMemberResponse {
        user_id: row.get::<Uuid, _>("user_id").to_string(),
        email: row.get("email"),
        display_name: row.get("display_name"),
        role: row.get("role"),
        joined_at: to_iso(row.get("joined_at")),
    }
}

pub(super) fn revision_response_from_row(row: &PgRow) -> DocumentRevisionResponse {
    DocumentRevisionResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        document_id: row.get::<Uuid, _>("document_id").to_string(),
        version: row.get::<i32, _>("version") as u32,
        title: row.get("title"),
        body_markdown: row.get("content_markdown"),
        summary: row.get("summary"),
        author_id: row.get::<Uuid, _>("author_id").to_string(),
        published_at: to_iso(row.get("published_at")),
    }
}

pub(super) fn document_summary_from_row(row: &PgRow) -> DocumentSummaryResponse {
    DocumentSummaryResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        slug: row.get("slug"),
        title: row.get("title"),
        document_type: row.get("document_type"),
        status: row.get("status"),
        updated_at: to_iso(row.get("updated_at")),
    }
}

pub(super) fn evidence_response_from_row(row: &PgRow) -> EvidenceResponse {
    EvidenceResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        space_key: row.get("space_key"),
        document_id: row
            .get::<Option<Uuid>, _>("document_id")
            .map(|id| id.to_string()),
        task_key: row.get("task_key"),
        phase_key: row.get("phase_key"),
        title: row.get("title"),
        evidence_type: row.get("evidence_type"),
        url: row.get("url"),
        attachment_id: row
            .get::<Option<Uuid>, _>("attachment_id")
            .map(|id| id.to_string()),
        checksum: row.get("checksum"),
        created_by: row.get::<Uuid, _>("created_by").to_string(),
        created_at: to_iso(row.get("created_at")),
    }
}

pub(super) fn attachment_response_from_row(row: &PgRow) -> AttachmentResponse {
    AttachmentResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        file_name: row.get("file_name"),
        content_type: row.get("content_type"),
        size_bytes: count_to_usize(row.get("size_bytes")),
        checksum: row.get("checksum"),
        uploaded_by: row.get::<Uuid, _>("uploaded_by").to_string(),
        uploaded_at: to_iso(row.get("uploaded_at")),
    }
}

pub(super) fn template_response_from_row(row: &PgRow) -> TemplateResponse {
    TemplateResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        document_type: row.get("document_type"),
        body_markdown: row.get("content_markdown"),
    }
}

pub(super) fn audit_entry_from_row(row: &PgRow) -> AuditEntryResponse {
    AuditEntryResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        actor_id: row
            .get::<Option<Uuid>, _>("actor_id")
            .map(|id| id.to_string())
            .unwrap_or_default(),
        action: row.get("action"),
        entity_type: row.get("entity_type"),
        entity_id: row.get::<Uuid, _>("entity_id").to_string(),
        created_at: to_iso(row.get("created_at")),
    }
}

pub(super) fn search_result_from_row(row: &PgRow) -> SearchResultResponse {
    SearchResultResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        result_type: row.get("result_type"),
        title: row.get("title"),
        space_key: row.get("space_key"),
        url: row.get("url"),
        snippet: snippet(&row.get::<String, _>("snippet")),
        updated_at: to_iso(row.get("updated_at")),
    }
}

pub(super) fn build_db_tree(rows: &[PgRow], parent_id: Option<Uuid>) -> Vec<SpaceTreeNodeResponse> {
    rows.iter()
        .filter(|row| row.get::<Option<Uuid>, _>("parent_id") == parent_id)
        .map(|row| {
            let id: Uuid = row.get("id");
            SpaceTreeNodeResponse {
                id: id.to_string(),
                slug: row.get("slug"),
                title: row.get("title"),
                document_type: row.get("document_type"),
                status: row.get("status"),
                children: build_db_tree(rows, Some(id)),
            }
        })
        .collect()
}

pub(super) fn to_iso(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(super) fn count_to_usize(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

pub(super) fn parse_uuid(value: &str, entity: &str) -> Result<Uuid, shared::AppError> {
    Uuid::parse_str(value).map_err(|_| shared::AppError::not_found(entity, value))
}
