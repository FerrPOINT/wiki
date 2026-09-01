use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{AppConfig, AppError};

#[derive(Clone, Debug)]
pub struct WikiClaims {
    pub user_id: String,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
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
#[async_trait::async_trait]
pub trait WikiBackendPort: Send + Sync {
    async fn readiness_check(&self) -> Result<(), AppError>;
    async fn authenticate_access_token(&self, token: &str) -> Result<WikiClaims, AppError>;
    async fn register(
        &self,
        request_id: Option<String>,
        body: WikiRegisterRequest,
    ) -> Result<WikiAuthResponse, AppError>;
    async fn login(
        &self,
        request_id: Option<String>,
        body: WikiLoginRequest,
    ) -> Result<WikiAuthResponse, AppError>;
    async fn refresh(&self, body: WikiRefreshRequest) -> Result<WikiAuthResponse, AppError>;
    async fn logout(&self, claims: &WikiClaims) -> Result<(), AppError>;
    async fn get_current_user(&self, claims: &WikiClaims) -> Result<WikiUserResponse, AppError>;
    async fn list_users(&self, claims: &WikiClaims) -> Result<WikiUserListResponse, AppError>;
    async fn create_user(
        &self,
        claims: &WikiClaims,
        body: WikiCreateUserRequest,
    ) -> Result<WikiUserResponse, AppError>;
    async fn update_user(
        &self,
        claims: &WikiClaims,
        user_id: &str,
        body: WikiUpdateUserRequest,
    ) -> Result<WikiUserResponse, AppError>;
    async fn get_settings(&self, claims: &WikiClaims) -> Result<WikiSettingsSnapshot, AppError>;
    async fn list_spaces(&self, claims: &WikiClaims) -> Result<SpaceListResponse, AppError>;
    async fn create_space(
        &self,
        claims: &WikiClaims,
        body: CreateSpaceRequest,
    ) -> Result<SpaceResponse, AppError>;
    async fn get_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, AppError>;
    async fn update_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: UpdateSpaceRequest,
    ) -> Result<SpaceResponse, AppError>;
    async fn archive_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, AppError>;
    async fn list_space_members(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, AppError>;
    async fn upsert_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
        body: UpsertSpaceMemberRequest,
    ) -> Result<SpaceMemberResponse, AppError>;
    async fn delete_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
    ) -> Result<(), AppError>;
    async fn get_space_tree(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceTreeResponse, AppError>;
    async fn create_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: CreateDocumentRequest,
    ) -> Result<DocumentResponse, AppError>;
    async fn get_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, AppError>;
    async fn update_document_draft(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: UpdateDocumentDraftRequest,
    ) -> Result<DocumentResponse, AppError>;
    async fn publish_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: PublishDocumentRequest,
    ) -> Result<DocumentRevisionResponse, AppError>;
    async fn archive_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, AppError>;
    async fn move_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: MoveDocumentRequest,
    ) -> Result<DocumentResponse, AppError>;
    async fn list_document_revisions(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        query: DocumentRevisionQuery,
    ) -> Result<DocumentRevisionListResponse, AppError>;
    async fn get_document_revision(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        revision_id: &str,
    ) -> Result<DocumentRevisionResponse, AppError>;
    async fn list_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<TaskPageListResponse, AppError>;
    async fn get_task(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, AppError>;
    async fn link_task_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<TaskPageResponse, AppError>;
    async fn list_task_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, AppError>;
    async fn list_task_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, AppError>;
    async fn list_phases(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<PhasePageListResponse, AppError>;
    async fn get_phase(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, AppError>;
    async fn link_phase_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<PhasePageResponse, AppError>;
    async fn list_phase_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, AppError>;
    async fn list_phase_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, AppError>;
    async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, AppError>;
    async fn list_evidence(
        &self,
        claims: Option<&WikiClaims>,
        query: EvidenceQuery,
    ) -> Result<EvidenceListResponse, AppError>;
    async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, AppError>;
    async fn upload_attachment(
        &self,
        claims: &WikiClaims,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentResponse, AppError>;
    async fn get_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, AppError>;
    async fn download_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentDownloadResponse, AppError>;
    async fn list_templates(&self) -> Result<TemplateListResponse, AppError>;
    async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, AppError>;
    async fn list_audit_log(
        &self,
        claims: &WikiClaims,
        query: AuditLogQuery,
    ) -> Result<AuditLogResponse, AppError>;
    async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, AppError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiRegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiRefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiUserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub is_system_admin: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiUserListResponse {
    pub users: Vec<WikiUserResponse>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct WikiCreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
    #[serde(default = "default_user_role")]
    pub role: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct WikiUpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub is_system_admin: Option<bool>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WikiSettingsResponse {
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

impl WikiSettingsResponse {
    pub fn from_snapshot(snapshot: WikiSettingsSnapshot) -> Self {
        Self {
            instance_name: snapshot.instance_name,
            api_base_path: snapshot.api_base_path,
            default_space_key: snapshot.default_space_key,
            default_language: snapshot.default_language,
            timezone: snapshot.timezone,
            registration_enabled: snapshot.registration_enabled,
            public_links_enabled: snapshot.public_links_enabled,
            search_backend: snapshot.search_backend,
            storage_backend: snapshot.storage_backend,
            max_upload_bytes: snapshot.max_upload_bytes,
            markdown_renderer: snapshot.markdown_renderer,
            html_sanitizer: snapshot.html_sanitizer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpaceResponse {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub status: String,
    pub document_count: usize,
    pub member_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpaceListResponse {
    pub spaces: Vec<SpaceResponse>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateSpaceRequest {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateSpaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpaceMemberResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpaceMemberListResponse {
    pub members: Vec<SpaceMemberResponse>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpsertSpaceMemberRequest {
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpaceTreeNodeResponse {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub document_type: String,
    pub status: String,
    #[schema(no_recursion)]
    pub children: Vec<SpaceTreeNodeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpaceTreeResponse {
    pub space_key: String,
    pub documents: Vec<SpaceTreeNodeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentRevisionResponse {
    pub id: String,
    pub document_id: String,
    pub version: u32,
    pub title: String,
    pub body_markdown: String,
    pub body_html: String,
    pub summary: Option<String>,
    pub author_id: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentRevisionListResponse {
    pub revisions: Vec<DocumentRevisionResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct DocumentRevisionQuery {
    #[param(minimum = 1, maximum = 100)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentResponse {
    pub id: String,
    pub space_key: String,
    pub parent_id: Option<String>,
    pub slug: String,
    pub title: String,
    pub document_type: String,
    pub status: String,
    pub body_markdown: String,
    pub body_html: String,
    pub draft_markdown: String,
    pub current_revision: Option<DocumentRevisionResponse>,
    pub task_keys: Vec<String>,
    pub phase_keys: Vec<String>,
    pub evidence: Vec<EvidenceResponse>,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentResponse>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub slug: Option<String>,
    #[serde(default = "default_document_type")]
    pub document_type: String,
    pub parent_id: Option<String>,
    #[serde(default)]
    pub content_markdown: String,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateDocumentDraftRequest {
    pub title: Option<String>,
    pub content_markdown: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PublishDocumentRequest {
    pub base_revision_id: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct MoveDocumentRequest {
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LinkDocumentRequest {
    pub document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskPageResponse {
    pub space_key: String,
    pub task_key: String,
    pub title: Option<String>,
    pub document_count: usize,
    pub evidence_count: usize,
    pub documents: Vec<DocumentSummaryResponse>,
    pub evidence: Vec<EvidenceResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskPageListResponse {
    pub tasks: Vec<TaskPageResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhasePageResponse {
    pub space_key: String,
    pub phase_key: String,
    pub title: Option<String>,
    pub document_count: usize,
    pub evidence_count: usize,
    pub documents: Vec<DocumentSummaryResponse>,
    pub evidence: Vec<EvidenceResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PhasePageListResponse {
    pub phases: Vec<PhasePageResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentSummaryResponse {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub document_type: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvidenceResponse {
    pub id: String,
    pub space_key: String,
    pub document_id: Option<String>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub title: String,
    pub evidence_type: String,
    pub url: Option<String>,
    pub attachment_id: Option<String>,
    pub checksum: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvidenceListResponse {
    pub evidence: Vec<EvidenceResponse>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateEvidenceRequest {
    pub space: Option<String>,
    pub document_id: Option<String>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub title: String,
    #[serde(default = "default_evidence_type")]
    pub evidence_type: String,
    pub url: Option<String>,
    pub attachment_id: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct EvidenceQuery {
    pub space: Option<String>,
    pub document_id: Option<String>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    #[param(minimum = 1, maximum = 100)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttachmentResponse {
    pub id: String,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: usize,
    pub checksum: String,
    pub uploaded_by: String,
    pub uploaded_at: String,
}

#[derive(Debug, Clone)]
pub struct AttachmentDownloadResponse {
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub document_type: String,
    pub body_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateResponse>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub document_type: String,
    pub body_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditEntryResponse {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub request_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditEntryResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct AuditLogQuery {
    #[param(minimum = 1, maximum = 200)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub space: Option<String>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub document_type: Option<String>,
    pub include_archived: Option<bool>,
    #[param(minimum = 1, maximum = 100)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResultResponse {
    pub id: String,
    pub result_type: String,
    pub title: String,
    pub space_key: String,
    pub url: String,
    pub snippet: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchResultResponse>,
}

fn default_user_role() -> String {
    "viewer".to_string()
}

fn default_document_type() -> String {
    "page".to_string()
}

fn default_evidence_type() -> String {
    "external_url".to_string()
}
