use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use shared::{
    AppError, AttachmentId, AuditLogId, DocumentId, DocumentRevisionId, DocumentTemplateId,
    EvidenceId, PhaseDossierId, SpaceId, TaskDossierId, Timestamp, UserId,
};
use uuid::Uuid;

use crate::value_objects::{ArcStr, Email, RichText};

#[cfg(test)]
#[path = "wiki/tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlobalRole {
    Admin,
    User,
}

impl GlobalRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }
}

impl FromStr for GlobalRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Self::Admin),
            "user" => Ok(Self::User),
            other => Err(format!("unknown global role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceRole {
    Admin,
    Editor,
    Viewer,
}

impl SpaceRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Viewer => "viewer",
        }
    }

    pub fn can_write(self) -> bool {
        matches!(self, Self::Admin | Self::Editor)
    }
}

impl FromStr for SpaceRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Self::Admin),
            "editor" => Ok(Self::Editor),
            "viewer" => Ok(Self::Viewer),
            other => Err(format!("unknown space role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Page,
    Requirements,
    ResearchNote,
    ImplementationNote,
    TestPlan,
    ReleaseNote,
}

impl DocumentType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Requirements => "requirements",
            Self::ResearchNote => "research_note",
            Self::ImplementationNote => "implementation_note",
            Self::TestPlan => "test_plan",
            Self::ReleaseNote => "release_note",
        }
    }
}

impl FromStr for DocumentType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "page" => Ok(Self::Page),
            "requirements" => Ok(Self::Requirements),
            "research_note" => Ok(Self::ResearchNote),
            "implementation_note" => Ok(Self::ImplementationNote),
            "test_plan" => Ok(Self::TestPlan),
            "release_note" => Ok(Self::ReleaseNote),
            other => Err(format!("unknown document type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Published,
    Archived,
}

impl DocumentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }
}

impl FromStr for DocumentStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            "archived" => Ok(Self::Archived),
            other => Err(format!("unknown document status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    ExternalUrl,
    UploadedFile,
}

impl EvidenceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExternalUrl => "external_url",
            Self::UploadedFile => "uploaded_file",
        }
    }
}

impl FromStr for EvidenceType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "external_url" => Ok(Self::ExternalUrl),
            "uploaded_file" => Ok(Self::UploadedFile),
            other => Err(format!("unknown evidence type: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentOwnerType {
    Document,
    Revision,
    Evidence,
}

impl AttachmentOwnerType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Revision => "revision",
            Self::Evidence => "evidence",
        }
    }
}

impl FromStr for AttachmentOwnerType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "document" => Ok(Self::Document),
            "revision" => Ok(Self::Revision),
            "evidence" => Ok(Self::Evidence),
            other => Err(format!("unknown attachment owner type: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpaceKey(ArcStr);

impl SpaceKey {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        let normalized = value.trim().to_ascii_uppercase();
        let valid = (2..=32).contains(&normalized.len())
            && normalized
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-')
            && !normalized.starts_with('-')
            && !normalized.ends_with('-');
        if !valid {
            return Err(AppError::invalid_input(
                "space key must be 2-32 uppercase letters, digits or hyphens",
            ));
        }
        Ok(Self(normalized.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for SpaceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for SpaceKey {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentSlug(ArcStr);

impl DocumentSlug {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        let slug = value.trim().to_ascii_lowercase();
        let valid = (1..=96).contains(&slug.len())
            && slug
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
            && !slug.contains("--");
        if !valid {
            return Err(AppError::invalid_input(
                "document slug must be 1-96 lowercase letters, digits or single hyphens",
            ));
        }
        Ok(Self(slug.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for DocumentSlug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DocumentSlug {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskKey(ArcStr);

impl TaskKey {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        let trimmed = value.trim();
        if trimmed.is_empty()
            || trimmed.len() > 96
            || trimmed
                .chars()
                .any(|ch| ch.is_control() || ch.is_whitespace())
        {
            return Err(AppError::invalid_input(
                "task key must be non-empty and contain no whitespace",
            ));
        }
        Ok(Self(trimmed.to_string().into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for TaskKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskKey {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhaseKey(ArcStr);

impl PhaseKey {
    pub fn parse(value: &str) -> Result<Self, AppError> {
        let key = value.trim().to_ascii_lowercase();
        let valid = (1..=64).contains(&key.len())
            && key
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
            && !(key.starts_with('-') || key.starts_with('_'))
            && !(key.ends_with('-') || key.ends_with('_'));
        if !valid {
            return Err(AppError::invalid_input(
                "phase key must be 1-64 lowercase letters, digits, hyphens or underscores",
            ));
        }
        Ok(Self(key.into()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for PhaseKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PhaseKey {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiUser {
    pub id: UserId,
    pub email: Email,
    pub display_name: ArcStr,
    pub password_hash: ArcStr,
    pub global_role: GlobalRole,
    pub is_active: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: SpaceId,
    pub key: SpaceKey,
    pub name: ArcStr,
    pub description: ArcStr,
    pub owner_id: UserId,
    pub archived_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Space {
    pub fn create(
        key: SpaceKey,
        name: impl Into<ArcStr>,
        description: impl Into<ArcStr>,
        owner_id: UserId,
    ) -> Result<Self, AppError> {
        let name = name.into();
        if name.as_str().trim().is_empty() {
            return Err(AppError::invalid_input("space name is required"));
        }
        let now = shared::now();
        Ok(Self {
            id: SpaceId::new(),
            key,
            name,
            description: description.into(),
            owner_id,
            archived_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn archive(&mut self) {
        let now = shared::now();
        self.archived_at = Some(now);
        self.updated_at = now;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceMember {
    pub space_id: SpaceId,
    pub user_id: UserId,
    pub role: SpaceRole,
    pub joined_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub space_id: SpaceId,
    pub parent_id: Option<DocumentId>,
    pub slug: DocumentSlug,
    pub title: ArcStr,
    pub document_type: DocumentType,
    pub status: DocumentStatus,
    pub current_revision_id: Option<DocumentRevisionId>,
    pub owner_id: UserId,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub archived_at: Option<Timestamp>,
}

impl Document {
    pub fn create(
        space_id: SpaceId,
        parent_id: Option<DocumentId>,
        slug: DocumentSlug,
        title: impl Into<ArcStr>,
        document_type: DocumentType,
        owner_id: UserId,
    ) -> Result<Self, AppError> {
        let title = title.into();
        if title.as_str().trim().is_empty() {
            return Err(AppError::invalid_input("document title is required"));
        }
        let now = shared::now();
        Ok(Self {
            id: DocumentId::new(),
            space_id,
            parent_id,
            slug,
            title,
            document_type,
            status: DocumentStatus::Draft,
            current_revision_id: None,
            owner_id,
            created_at: now,
            updated_at: now,
            archived_at: None,
        })
    }

    pub fn mark_published(&mut self, revision_id: DocumentRevisionId) {
        self.status = DocumentStatus::Published;
        self.current_revision_id = Some(revision_id);
        self.updated_at = shared::now();
    }

    pub fn archive(&mut self) {
        let now = shared::now();
        self.status = DocumentStatus::Archived;
        self.archived_at = Some(now);
        self.updated_at = now;
    }

    pub fn move_to(&mut self, parent_id: Option<DocumentId>) -> Result<(), AppError> {
        if parent_id == Some(self.id) {
            return Err(AppError::invalid_input("document cannot be its own parent"));
        }
        self.parent_id = parent_id;
        self.updated_at = shared::now();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDraft {
    pub document_id: DocumentId,
    pub author_id: UserId,
    pub content_markdown: RichText,
    pub base_revision_id: Option<DocumentRevisionId>,
    pub updated_at: Timestamp,
}

impl DocumentDraft {
    pub fn new(
        document_id: DocumentId,
        author_id: UserId,
        content_markdown: impl Into<RichText>,
        base_revision_id: Option<DocumentRevisionId>,
    ) -> Self {
        Self {
            document_id,
            author_id,
            content_markdown: content_markdown.into(),
            base_revision_id,
            updated_at: shared::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRevision {
    pub id: DocumentRevisionId,
    pub document_id: DocumentId,
    pub version: i32,
    pub title: ArcStr,
    pub content_markdown: RichText,
    pub content_html: RichText,
    pub content_text: RichText,
    pub content_checksum: ArcStr,
    pub summary: Option<ArcStr>,
    pub author_id: UserId,
    pub published_at: Timestamp,
}

impl DocumentRevision {
    #[allow(clippy::too_many_arguments)]
    pub fn publish(
        document: &Document,
        version: i32,
        content_markdown: impl Into<RichText>,
        content_html: impl Into<RichText>,
        content_text: impl Into<RichText>,
        content_checksum: impl Into<ArcStr>,
        summary: Option<ArcStr>,
        author_id: UserId,
    ) -> Result<Self, AppError> {
        if version < 1 {
            return Err(AppError::invalid_input("revision version must be positive"));
        }
        let content_markdown = content_markdown.into();
        if content_markdown.as_str().trim().is_empty() {
            return Err(AppError::invalid_input("published content is required"));
        }
        Ok(Self {
            id: DocumentRevisionId::new(),
            document_id: document.id,
            version,
            title: document.title.clone(),
            content_markdown,
            content_html: content_html.into(),
            content_text: content_text.into(),
            content_checksum: content_checksum.into(),
            summary,
            author_id,
            published_at: shared::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDossier {
    pub id: TaskDossierId,
    pub space_id: SpaceId,
    pub task_key: TaskKey,
    pub title_snapshot: Option<ArcStr>,
    pub external_url: Option<ArcStr>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDossier {
    pub id: PhaseDossierId,
    pub space_id: SpaceId,
    pub phase_key: PhaseKey,
    pub phase_name: Option<ArcStr>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceTarget {
    pub document_id: Option<DocumentId>,
    pub task_dossier_id: Option<TaskDossierId>,
    pub phase_dossier_id: Option<PhaseDossierId>,
}

impl EvidenceTarget {
    pub fn document(document_id: DocumentId) -> Self {
        Self {
            document_id: Some(document_id),
            ..Self::default()
        }
    }

    pub fn task(task_dossier_id: TaskDossierId) -> Self {
        Self {
            task_dossier_id: Some(task_dossier_id),
            ..Self::default()
        }
    }

    pub fn phase(phase_dossier_id: PhaseDossierId) -> Self {
        Self {
            phase_dossier_id: Some(phase_dossier_id),
            ..Self::default()
        }
    }

    pub fn is_empty(self) -> bool {
        self.document_id.is_none()
            && self.task_dossier_id.is_none()
            && self.phase_dossier_id.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: EvidenceId,
    pub space_id: SpaceId,
    pub target: EvidenceTarget,
    pub evidence_type: EvidenceType,
    pub title: ArcStr,
    pub url: Option<ArcStr>,
    pub attachment_id: Option<AttachmentId>,
    pub checksum: Option<ArcStr>,
    pub metadata: serde_json::Value,
    pub created_by: UserId,
    pub created_at: Timestamp,
}

impl EvidenceItem {
    pub fn external_url(
        space_id: SpaceId,
        target: EvidenceTarget,
        title: impl Into<ArcStr>,
        url: impl Into<ArcStr>,
        created_by: UserId,
    ) -> Result<Self, AppError> {
        Self::build(
            space_id,
            target,
            EvidenceType::ExternalUrl,
            title,
            Some(url.into()),
            None,
            None,
            created_by,
        )
    }

    pub fn uploaded_file(
        space_id: SpaceId,
        target: EvidenceTarget,
        title: impl Into<ArcStr>,
        attachment_id: AttachmentId,
        checksum: impl Into<ArcStr>,
        created_by: UserId,
    ) -> Result<Self, AppError> {
        Self::build(
            space_id,
            target,
            EvidenceType::UploadedFile,
            title,
            None,
            Some(attachment_id),
            Some(checksum.into()),
            created_by,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        space_id: SpaceId,
        target: EvidenceTarget,
        evidence_type: EvidenceType,
        title: impl Into<ArcStr>,
        url: Option<ArcStr>,
        attachment_id: Option<AttachmentId>,
        checksum: Option<ArcStr>,
        created_by: UserId,
    ) -> Result<Self, AppError> {
        if target.is_empty() {
            return Err(AppError::invalid_input(
                "evidence must target a document, task or phase",
            ));
        }
        let title = title.into();
        if title.as_str().trim().is_empty() {
            return Err(AppError::invalid_input("evidence title is required"));
        }
        match evidence_type {
            EvidenceType::ExternalUrl if url.is_none() || attachment_id.is_some() => {
                return Err(AppError::invalid_input(
                    "external_url evidence requires url only",
                ));
            }
            EvidenceType::UploadedFile if attachment_id.is_none() || url.is_some() => {
                return Err(AppError::invalid_input(
                    "uploaded_file evidence requires attachment_id only",
                ));
            }
            EvidenceType::ExternalUrl | EvidenceType::UploadedFile => {}
        }
        Ok(Self {
            id: EvidenceId::new(),
            space_id,
            target,
            evidence_type,
            title,
            url,
            attachment_id,
            checksum,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_by,
            created_at: shared::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    pub id: AttachmentId,
    pub space_id: Option<SpaceId>,
    pub owner_entity_type: Option<AttachmentOwnerType>,
    pub owner_entity_id: Option<Uuid>,
    pub file_name: ArcStr,
    pub content_type: ArcStr,
    pub size_bytes: i64,
    pub storage_key: ArcStr,
    pub checksum: ArcStr,
    pub uploaded_by: UserId,
    pub uploaded_at: Timestamp,
}

impl AttachmentMetadata {
    pub fn staged(
        file_name: impl Into<ArcStr>,
        content_type: impl Into<ArcStr>,
        size_bytes: i64,
        storage_key: impl Into<ArcStr>,
        checksum: impl Into<ArcStr>,
        uploaded_by: UserId,
    ) -> Result<Self, AppError> {
        let file_name = file_name.into();
        if invalid_file_name(file_name.as_str()) {
            return Err(AppError::invalid_input("attachment file name is invalid"));
        }
        let content_type = content_type.into();
        if content_type.as_str().trim().is_empty() {
            return Err(AppError::invalid_input(
                "attachment content type is required",
            ));
        }
        let storage_key = storage_key.into();
        if storage_key.as_str().trim().is_empty() {
            return Err(AppError::invalid_input(
                "attachment storage key is required",
            ));
        }
        let checksum = checksum.into();
        if checksum.as_str().trim().is_empty() {
            return Err(AppError::invalid_input("attachment checksum is required"));
        }
        if size_bytes <= 0 {
            return Err(AppError::invalid_input("attachment must not be empty"));
        }
        Ok(Self {
            id: AttachmentId::new(),
            space_id: None,
            owner_entity_type: None,
            owner_entity_id: None,
            file_name,
            content_type,
            size_bytes,
            storage_key,
            checksum,
            uploaded_by,
            uploaded_at: shared::now(),
        })
    }

    pub fn claim_for_evidence(&mut self, space_id: SpaceId, evidence_id: EvidenceId) {
        self.space_id = Some(space_id);
        self.owner_entity_type = Some(AttachmentOwnerType::Evidence);
        self.owner_entity_id = Some(evidence_id.as_uuid());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    pub id: DocumentTemplateId,
    pub space_id: Option<SpaceId>,
    pub name: ArcStr,
    pub document_type: DocumentType,
    pub content_markdown: RichText,
    pub is_active: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: AuditLogId,
    pub actor_id: Option<UserId>,
    pub action: ArcStr,
    pub entity_type: ArcStr,
    pub entity_id: Uuid,
    pub diff: Option<serde_json::Value>,
    pub request_id: ArcStr,
    pub created_at: Timestamp,
}

fn invalid_file_name(file_name: &str) -> bool {
    let trimmed = file_name.trim();
    trimmed.is_empty()
        || trimmed != file_name
        || trimmed
            .chars()
            .any(|ch| matches!(ch, '/' | '\\' | '\0') || ch.is_control())
}
