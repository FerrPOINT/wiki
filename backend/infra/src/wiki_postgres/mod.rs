mod audit;
mod connection;
mod documents;
mod dossiers;
mod evidence;
mod identity;
mod mapping;
mod queries;
mod search;
mod spaces;
mod templates;
pub use connection::connect_postgres_wiki_backend;

use app::wiki::{WikiSpaceAccess as SpaceAccess, normalize_space_key, space_role_allows};
use mapping::parse_uuid;
use shared::wiki_contract::*;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
struct PostgresWikiBackend {
    pool: PgPool,
    auth: shared::AuthConfig,
    storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
    max_upload_bytes: usize,
    settings: WikiSettingsSnapshot,
}

impl PostgresWikiBackend {
    async fn ensure_admin(&self, claims: &WikiClaims) -> Result<Uuid, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let role = self.active_global_role(user_id).await?;
        if role == "admin" {
            Ok(user_id)
        } else {
            Err(shared::AppError::Forbidden)
        }
    }

    async fn active_global_role(&self, user_id: Uuid) -> Result<String, shared::AppError> {
        sqlx::query_scalar("SELECT global_role FROM users WHERE id = $1 AND is_active = true")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or(shared::AppError::Unauthorized)
    }

    async fn restricted_user_id(
        &self,
        claims: &WikiClaims,
    ) -> Result<Option<Uuid>, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let role = self.active_global_role(user_id).await?;
        if role == "admin" {
            Ok(None)
        } else {
            Ok(Some(user_id))
        }
    }

    async fn ensure_space_access(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        self.ensure_space_id_access(claims, space_id, required)
            .await?;
        Ok(space_id)
    }

    async fn ensure_space_id_access(
        &self,
        claims: &WikiClaims,
        space_id: Uuid,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let row = sqlx::query(
            r#"
            SELECT u.global_role, sm.role AS space_role
            FROM users u
            LEFT JOIN space_members sm ON sm.user_id = u.id AND sm.space_id = $2
            WHERE u.id = $1 AND u.is_active = true
            "#,
        )
        .bind(user_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or(shared::AppError::Unauthorized)?;

        let global_role: String = row.get("global_role");
        let space_role: Option<String> = row.get("space_role");
        if global_role == "admin" || space_role_allows(space_role.as_deref(), required) {
            Ok(user_id)
        } else {
            Err(shared::AppError::Forbidden)
        }
    }

    async fn ensure_space_accepts_writes(&self, space_id: Uuid) -> Result<(), shared::AppError> {
        let accepts_writes: bool =
            sqlx::query_scalar("SELECT archived_at IS NULL FROM spaces WHERE id = $1")
                .bind(space_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?
                .ok_or_else(|| shared::AppError::not_found("space", space_id))?;
        if accepts_writes {
            Ok(())
        } else {
            Err(shared::AppError::invalid_input(
                "archived space does not accept new documents or evidence",
            ))
        }
    }

    async fn ensure_document_access(
        &self,
        claims: &WikiClaims,
        document_id: Uuid,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let space_id = self.document_space_id(document_id).await?;
        self.ensure_space_id_access(claims, space_id, required)
            .await?;
        Ok(space_id)
    }

    async fn space_id(&self, space_key: &str) -> Result<Uuid, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        sqlx::query_scalar("SELECT id FROM spaces WHERE key = $1")
            .bind(&key)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("space", space_key))
    }
}

#[async_trait::async_trait]
impl WikiBackendPort for PostgresWikiBackend {
    async fn readiness_check(&self) -> Result<(), shared::AppError> {
        let _: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(())
    }

    async fn authenticate_access_token(&self, token: &str) -> Result<WikiClaims, shared::AppError> {
        PostgresWikiBackend::authenticate_access_token(self, token).await
    }

    async fn register(
        &self,
        body: WikiRegisterRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        PostgresWikiBackend::register(self, body).await
    }

    async fn login(&self, body: WikiLoginRequest) -> Result<WikiAuthResponse, shared::AppError> {
        PostgresWikiBackend::login(self, body).await
    }

    async fn refresh(
        &self,
        body: WikiRefreshRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        PostgresWikiBackend::refresh(self, body).await
    }

    async fn logout(&self, claims: &WikiClaims) -> Result<(), shared::AppError> {
        PostgresWikiBackend::logout(self, claims).await
    }

    async fn get_current_user(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserResponse, shared::AppError> {
        PostgresWikiBackend::get_current_user(self, claims).await
    }

    async fn list_users(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserListResponse, shared::AppError> {
        PostgresWikiBackend::list_users(self, claims).await
    }

    async fn create_user(
        &self,
        claims: &WikiClaims,
        body: WikiCreateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        PostgresWikiBackend::create_user(self, claims, body).await
    }

    async fn update_user(
        &self,
        claims: &WikiClaims,
        user_id: &str,
        body: WikiUpdateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        PostgresWikiBackend::update_user(self, claims, user_id, body).await
    }

    async fn get_settings(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiSettingsSnapshot, shared::AppError> {
        PostgresWikiBackend::get_settings(self, claims).await
    }

    async fn list_spaces(
        &self,
        claims: &WikiClaims,
    ) -> Result<SpaceListResponse, shared::AppError> {
        PostgresWikiBackend::list_spaces(self, claims).await
    }

    async fn create_space(
        &self,
        claims: &WikiClaims,
        body: CreateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::create_space(self, claims, body).await
    }

    async fn get_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::get_space(self, claims, space_key).await
    }

    async fn update_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: UpdateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::update_space(self, claims, space_key, body).await
    }

    async fn archive_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::archive_space(self, claims, space_key).await
    }

    async fn list_space_members(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, shared::AppError> {
        PostgresWikiBackend::list_space_members(self, claims, space_key).await
    }

    async fn upsert_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
        body: UpsertSpaceMemberRequest,
    ) -> Result<SpaceMemberResponse, shared::AppError> {
        PostgresWikiBackend::upsert_space_member(self, claims, space_key, user_id, body).await
    }

    async fn delete_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
    ) -> Result<(), shared::AppError> {
        PostgresWikiBackend::delete_space_member(self, claims, space_key, user_id).await
    }

    async fn get_space_tree(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceTreeResponse, shared::AppError> {
        PostgresWikiBackend::get_space_tree(self, claims, space_key).await
    }

    async fn create_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: CreateDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::create_document(self, claims, space_key, body).await
    }

    async fn get_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::get_document(self, claims, document_id).await
    }

    async fn update_document_draft(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: UpdateDocumentDraftRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::update_document_draft(self, claims, document_id, body).await
    }

    async fn publish_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: PublishDocumentRequest,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        PostgresWikiBackend::publish_document(self, claims, document_id, body).await
    }

    async fn archive_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::archive_document(self, claims, document_id).await
    }

    async fn move_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: MoveDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::move_document(self, claims, document_id, body).await
    }

    async fn list_document_revisions(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentRevisionListResponse, shared::AppError> {
        PostgresWikiBackend::list_document_revisions(self, claims, document_id).await
    }

    async fn get_document_revision(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        revision_id: &str,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        PostgresWikiBackend::get_document_revision(self, claims, document_id, revision_id).await
    }

    async fn list_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<TaskPageListResponse, shared::AppError> {
        PostgresWikiBackend::list_tasks(self, claims, space_key).await
    }

    async fn get_task(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        PostgresWikiBackend::get_task(self, claims, space_key, task_key).await
    }

    async fn link_task_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<TaskPageResponse, shared::AppError> {
        PostgresWikiBackend::link_task_document(self, claims, space_key, task_key, body).await
    }

    async fn list_task_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        PostgresWikiBackend::list_task_documents(self, claims, space_key, task_key).await
    }

    async fn list_task_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        PostgresWikiBackend::list_task_evidence(self, claims, space_key, task_key).await
    }

    async fn list_phases(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<PhasePageListResponse, shared::AppError> {
        PostgresWikiBackend::list_phases(self, claims, space_key).await
    }

    async fn get_phase(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        PostgresWikiBackend::get_phase(self, claims, space_key, phase_key).await
    }

    async fn link_phase_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<PhasePageResponse, shared::AppError> {
        PostgresWikiBackend::link_phase_document(self, claims, space_key, phase_key, body).await
    }

    async fn list_phase_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        PostgresWikiBackend::list_phase_documents(self, claims, space_key, phase_key).await
    }

    async fn list_phase_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        PostgresWikiBackend::list_phase_evidence(self, claims, space_key, phase_key).await
    }

    async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, shared::AppError> {
        PostgresWikiBackend::create_evidence(self, claims, body).await
    }

    async fn list_evidence(
        &self,
        claims: Option<&WikiClaims>,
        query: EvidenceQuery,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        PostgresWikiBackend::list_evidence(self, claims, query).await
    }

    async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, shared::AppError> {
        PostgresWikiBackend::get_evidence(self, claims, evidence_id).await
    }

    async fn upload_attachment(
        &self,
        claims: &WikiClaims,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentResponse, shared::AppError> {
        PostgresWikiBackend::upload_attachment(self, claims, file_name, content_type, bytes).await
    }

    async fn get_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, shared::AppError> {
        PostgresWikiBackend::get_attachment(self, claims, attachment_id).await
    }

    async fn download_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentDownloadResponse, shared::AppError> {
        PostgresWikiBackend::download_attachment(self, claims, attachment_id).await
    }

    async fn list_templates(&self) -> Result<TemplateListResponse, shared::AppError> {
        PostgresWikiBackend::list_templates(self).await
    }

    async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, shared::AppError> {
        PostgresWikiBackend::create_template(self, claims, body).await
    }

    async fn list_audit_log(
        &self,
        claims: &WikiClaims,
    ) -> Result<AuditLogResponse, shared::AppError> {
        PostgresWikiBackend::list_audit_log(self, claims).await
    }

    async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, shared::AppError> {
        PostgresWikiBackend::search(self, claims, query).await
    }
}
