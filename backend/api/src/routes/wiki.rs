use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgPoolOptions, postgres::PgRow};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct WikiClaims {
    pub user_id: String,
    pub session_id: Option<String>,
}

#[derive(Clone)]
pub struct WikiBackend {
    postgres: Option<Arc<PostgresWikiBackend>>,
}

#[derive(Clone)]
struct PostgresWikiBackend {
    pool: PgPool,
    auth: shared::AuthConfig,
    storage_dir: PathBuf,
    max_upload_bytes: usize,
}

impl WikiBackend {
    pub fn memory() -> Self {
        Self { postgres: None }
    }

    pub async fn from_config(config: &shared::AppConfig) -> Result<Self, shared::AppError> {
        if config.database.url.trim().is_empty() {
            return Ok(Self::memory());
        }

        let pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                config.database.connect_timeout_seconds,
            ))
            .idle_timeout(std::time::Duration::from_secs(
                config.database.idle_timeout_seconds,
            ))
            .connect(&config.database.url)
            .await
            .map_err(shared::AppError::database)?;

        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .map_err(shared::AppError::database)?;

        let backend = PostgresWikiBackend {
            pool,
            auth: config.auth.clone(),
            storage_dir: PathBuf::from(&config.storage.dir),
            max_upload_bytes: config.storage.max_upload_bytes,
        };
        backend.bootstrap(&config.bootstrap).await?;
        Ok(Self {
            postgres: Some(Arc::new(backend)),
        })
    }

    fn postgres(&self) -> Option<&PostgresWikiBackend> {
        self.postgres.as_deref()
    }
}

pub async fn require_wiki_auth(
    State(backend): State<WikiBackend>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix("Bearer ")
                .or_else(|| value.strip_prefix("bearer "))
        })
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = match backend.authenticate_access_token(token).await {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
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
    pub summary: Option<String>,
    pub author_id: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentRevisionListResponse {
    pub revisions: Vec<DocumentRevisionResponse>,
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
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuditLogResponse {
    pub entries: Vec<AuditEntryResponse>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub space: Option<String>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub document_type: Option<String>,
    pub include_archived: Option<bool>,
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

#[derive(Debug, Clone)]
struct DocumentRecord {
    id: String,
    space_key: String,
    parent_id: Option<String>,
    slug: String,
    title: String,
    document_type: String,
    status: String,
    draft_markdown: String,
    current_revision_id: Option<String>,
    task_keys: BTreeSet<String>,
    phase_keys: BTreeSet<String>,
    created_by: String,
    updated_by: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct AttachmentRecord {
    metadata: AttachmentResponse,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct WikiStore {
    users: BTreeMap<String, WikiUserResponse>,
    passwords: BTreeMap<String, String>,
    tokens: BTreeMap<String, String>,
    refresh_tokens: BTreeMap<String, String>,
    spaces: BTreeMap<String, SpaceResponse>,
    members: BTreeMap<String, BTreeMap<String, String>>,
    documents: BTreeMap<String, DocumentRecord>,
    revisions: BTreeMap<String, Vec<DocumentRevisionResponse>>,
    evidence: BTreeMap<String, EvidenceResponse>,
    attachments: BTreeMap<String, AttachmentRecord>,
    templates: BTreeMap<String, TemplateResponse>,
    audit: Vec<AuditEntryResponse>,
}

impl WikiStore {
    fn seeded() -> Self {
        let now = now_iso();
        let user_id = "00000000-0000-0000-0000-000000000001".to_string();
        let user = WikiUserResponse {
            id: user_id.clone(),
            email: "demo@example.com".to_string(),
            username: "demo".to_string(),
            display_name: "Демо пользователь".to_string(),
            role: "admin".to_string(),
            is_system_admin: true,
            active: true,
        };
        let space = SpaceResponse {
            id: "space-sdlc".to_string(),
            key: "SDLC".to_string(),
            name: "База знаний SDLC".to_string(),
            description: Some("Основное пространство Wiki для документов SDLC".to_string()),
            owner_id: user_id.clone(),
            status: "active".to_string(),
            document_count: 1,
            member_count: 1,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let revision = DocumentRevisionResponse {
            id: "revision-product-requirements-1".to_string(),
            document_id: "product-requirements".to_string(),
            version: 1,
            title: "Требования к Wiki MVP".to_string(),
            body_markdown: "# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.".to_string(),
            summary: Some("Исходные требования MVP".to_string()),
            author_id: user_id.clone(),
            published_at: now.clone(),
        };
        let mut task_keys = BTreeSet::new();
        task_keys.insert("SDLC-42".to_string());
        let mut phase_keys = BTreeSet::new();
        phase_keys.insert("implementation".to_string());
        let document = DocumentRecord {
            id: "product-requirements".to_string(),
            space_key: "SDLC".to_string(),
            parent_id: None,
            slug: "product-requirements".to_string(),
            title: "Требования к Wiki MVP".to_string(),
            document_type: "requirements".to_string(),
            status: "published".to_string(),
            draft_markdown: revision.body_markdown.clone(),
            current_revision_id: Some(revision.id.clone()),
            task_keys,
            phase_keys,
            created_by: user_id.clone(),
            updated_by: user_id.clone(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let evidence = EvidenceResponse {
            id: "evidence-smoke".to_string(),
            space_key: "SDLC".to_string(),
            document_id: Some("product-requirements".to_string()),
            task_key: Some("SDLC-42".to_string()),
            phase_key: Some("implementation".to_string()),
            title: "Материал smoke-проверки фронта".to_string(),
            evidence_type: "external_url".to_string(),
            url: Some("https://ci.local/jobs/wiki-smoke".to_string()),
            attachment_id: None,
            checksum: None,
            created_by: user_id.clone(),
            created_at: now.clone(),
        };
        let templates = [
            ("requirements", "requirements", "Требования"),
            ("research-note", "research_note", "Исследование"),
            ("implementation-note", "implementation_note", "Реализация"),
            ("test-plan", "test_plan", "План проверки"),
            ("release-note", "release_note", "Релизная заметка"),
        ]
        .into_iter()
        .map(|(id, document_type, name)| {
            (
                id.to_string(),
                TemplateResponse {
                    id: id.to_string(),
                    name: name.to_string(),
                    document_type: document_type.to_string(),
                    body_markdown: format!(
                        "# {name}\n\n## Контекст\n\n## Решения\n\n## Проверки\n"
                    ),
                },
            )
        })
        .collect();

        Self {
            users: BTreeMap::from([(user_id.clone(), user)]),
            passwords: BTreeMap::from([(user_id.clone(), "demo".to_string())]),
            tokens: BTreeMap::from([("wiki-dev-token".to_string(), user_id.clone())]),
            refresh_tokens: BTreeMap::from([("wiki-dev-refresh".to_string(), user_id.clone())]),
            spaces: BTreeMap::from([("SDLC".to_string(), space)]),
            members: BTreeMap::from([(
                "SDLC".to_string(),
                BTreeMap::from([(user_id.clone(), "admin".to_string())]),
            )]),
            documents: BTreeMap::from([("product-requirements".to_string(), document)]),
            revisions: BTreeMap::from([("product-requirements".to_string(), vec![revision])]),
            evidence: BTreeMap::from([("evidence-smoke".to_string(), evidence)]),
            attachments: BTreeMap::new(),
            templates,
            audit: vec![AuditEntryResponse {
                id: "audit-initial".to_string(),
                actor_id: user_id,
                action: "wiki.seeded".to_string(),
                entity_type: "space".to_string(),
                entity_id: "SDLC".to_string(),
                created_at: now,
            }],
        }
    }

    fn audit(&mut self, actor_id: &str, action: &str, entity_type: &str, entity_id: &str) {
        self.audit.push(AuditEntryResponse {
            id: new_id(),
            actor_id: actor_id.to_string(),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            created_at: now_iso(),
        });
    }
}

static STORE: OnceLock<Mutex<WikiStore>> = OnceLock::new();

fn store() -> &'static Mutex<WikiStore> {
    STORE.get_or_init(|| Mutex::new(WikiStore::seeded()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    exp: usize,
    jti: String,
    typ: String,
}

impl PostgresWikiBackend {
    async fn bootstrap(&self, config: &shared::BootstrapConfig) -> Result<(), shared::AppError> {
        tokio::fs::create_dir_all(&self.storage_dir)
            .await
            .map_err(shared::AppError::internal)?;
        self.seed_templates().await?;

        let admin_email = config
            .admin_email
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let admin_password = config
            .admin_password
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        match (admin_email, admin_password) {
            (Some(email), Some(password)) => {
                let username = config
                    .admin_username
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| default_username(email));
                let display_name = config
                    .admin_display_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("Wiki Admin");
                let password_hash = hash_password(password)?;
                let user_id = Uuid::now_v7();

                let row = sqlx::query(
                    r#"
                    INSERT INTO users (
                        id, email, username, display_name, password_hash,
                        global_role, is_active, created_at, updated_at
                    )
                    VALUES ($1, $2, $3, $4, $5, 'admin', true, now(), now())
                    ON CONFLICT (lower(email))
                    DO UPDATE SET
                        username = EXCLUDED.username,
                        display_name = EXCLUDED.display_name,
                        password_hash = EXCLUDED.password_hash,
                        global_role = 'admin',
                        is_active = true,
                        updated_at = now()
                    RETURNING id
                    "#,
                )
                .bind(user_id)
                .bind(email)
                .bind(&username)
                .bind(display_name)
                .bind(password_hash)
                .fetch_one(&self.pool)
                .await
                .map_err(shared::AppError::database)?;
                let admin_id: Uuid = row.get("id");

                let space_row = sqlx::query(
                    r#"
                    INSERT INTO spaces (id, key, name, description, owner_id, created_at, updated_at)
                    VALUES ($1, 'SDLC', 'База знаний SDLC',
                            'Основное пространство Wiki для документов SDLC', $2, now(), now())
                    ON CONFLICT (key)
                    DO UPDATE SET owner_id = EXCLUDED.owner_id, updated_at = now()
                    RETURNING id
                    "#,
                )
                .bind(Uuid::now_v7())
                .bind(admin_id)
                .fetch_one(&self.pool)
                .await
                .map_err(shared::AppError::database)?;
                let space_id: Uuid = space_row.get("id");

                sqlx::query(
                    r#"
                    INSERT INTO space_members (space_id, user_id, role, joined_at)
                    VALUES ($1, $2, 'admin', now())
                    ON CONFLICT (space_id, user_id)
                    DO UPDATE SET role = 'admin'
                    "#,
                )
                .bind(space_id)
                .bind(admin_id)
                .execute(&self.pool)
                .await
                .map_err(shared::AppError::database)?;

                self.audit(Some(admin_id), "wiki.bootstrap", "space", space_id)
                    .await?;
            }
            (None, None) => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM users")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(shared::AppError::database)?;
                if count == 0 {
                    tracing::warn!(
                        "Wiki database has no users; set WIKI_BOOTSTRAP__ADMIN_EMAIL and WIKI_BOOTSTRAP__ADMIN_PASSWORD or register the first user"
                    );
                }
            }
            _ => {
                return Err(shared::AppError::invalid_input(
                    "bootstrap admin email and password must be set together",
                ));
            }
        }

        Ok(())
    }

    async fn seed_templates(&self) -> Result<(), shared::AppError> {
        for (name, document_type, body_markdown) in [
            (
                "Требования",
                "requirements",
                "# Требования\n\n## Контекст\n\n## Цели\n\n## Функциональные требования\n\n## Проверки\n",
            ),
            (
                "Исследование",
                "research_note",
                "# Исследование\n\n## Вопрос\n\n## Наблюдения\n\n## Вывод\n",
            ),
            (
                "Реализация",
                "implementation_note",
                "# Реализация\n\n## Решение\n\n## Изменения\n\n## Риски\n",
            ),
            (
                "План проверки",
                "test_plan",
                "# План проверки\n\n## Сценарии\n\n## Данные\n\n## Критерии готовности\n",
            ),
            (
                "Релизная заметка",
                "release_note",
                "# Релизная заметка\n\n## Изменения\n\n## Миграции\n\n## Проверки\n",
            ),
        ] {
            sqlx::query(
                r#"
                INSERT INTO document_templates (
                    id, space_id, name, document_type, content_markdown, is_active,
                    created_at, updated_at
                )
                VALUES ($1, NULL, $2, $3, $4, true, now(), now())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(name)
            .bind(document_type)
            .bind(body_markdown)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        }
        Ok(())
    }

    async fn authenticate_access_token(&self, token: &str) -> Result<WikiClaims, shared::AppError> {
        let claims = decode_token(&self.auth, token, "access")?;
        let user_id = parse_uuid(&claims.sub, "user")?;
        let session_id = parse_uuid(&claims.jti, "session")?;
        let access_token_hash = hash_token(token);

        let found: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT u.id
            FROM auth_sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.id = $1
              AND s.user_id = $2
              AND s.access_token_hash = $3
              AND s.revoked_at IS NULL
              AND s.expires_at > now()
              AND u.is_active = true
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(access_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        if found.is_none() {
            return Err(shared::AppError::Unauthorized);
        }

        sqlx::query("UPDATE auth_sessions SET last_used_at = now() WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;

        Ok(WikiClaims {
            user_id: user_id.to_string(),
            session_id: Some(session_id.to_string()),
        })
    }

    async fn register(
        &self,
        body: WikiRegisterRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let email = normalize_required(&body.email, "email")?;
        let username = normalize_required(&body.username, "username")?;
        let password = normalize_required(&body.password, "password")?;
        let display_name = body
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&username)
            .to_string();
        let user_id = Uuid::now_v7();
        let password_hash = hash_password(&password)?;

        let row = sqlx::query(
            r#"
            INSERT INTO users (
                id, email, username, display_name, password_hash,
                global_role, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 'user', true, now(), now())
            RETURNING id, email, username, display_name, global_role, is_active
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(username)
        .bind(display_name)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        let user_id: Uuid = row.get("id");
        self.audit(Some(user_id), "user.register", "user", user_id)
            .await?;
        self.issue_tokens(user_id, &row).await
    }

    async fn login(&self, body: WikiLoginRequest) -> Result<WikiAuthResponse, shared::AppError> {
        let email = normalize_required(&body.email, "email")?;
        let row = sqlx::query(
            r#"
            SELECT id, email, username, display_name, password_hash, global_role, is_active
            FROM users
            WHERE lower(email) = lower($1)
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or(shared::AppError::Unauthorized)?;

        let active: bool = row.get("is_active");
        let password_hash: String = row.get("password_hash");
        if !active || !verify_password(&body.password, &password_hash)? {
            return Err(shared::AppError::Unauthorized);
        }

        let user_id: Uuid = row.get("id");
        self.issue_tokens(user_id, &row).await
    }

    async fn refresh(
        &self,
        body: WikiRefreshRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let refresh_token = normalize_required(&body.refresh_token, "refresh_token")?;
        let claims = decode_token(&self.auth, &refresh_token, "refresh")?;
        let user_id = parse_uuid(&claims.sub, "user")?;
        let session_id = parse_uuid(&claims.jti, "session")?;
        let refresh_token_hash = hash_token(&refresh_token);

        let row = sqlx::query(
            r#"
            SELECT u.id, u.email, u.username, u.display_name, u.global_role, u.is_active
            FROM auth_sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.id = $1
              AND s.user_id = $2
              AND s.refresh_token_hash = $3
              AND s.revoked_at IS NULL
              AND s.refresh_expires_at > now()
              AND u.is_active = true
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(refresh_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or(shared::AppError::Unauthorized)?;

        let access = create_token(
            &self.auth,
            user_id,
            session_id,
            "access",
            Duration::minutes(self.auth.access_token_ttl_minutes as i64),
        )?;
        let refresh = create_token(
            &self.auth,
            user_id,
            session_id,
            "refresh",
            Duration::days(self.auth.refresh_token_ttl_days as i64),
        )?;
        let access_expires_at =
            Utc::now() + Duration::minutes(self.auth.access_token_ttl_minutes as i64);
        let refresh_expires_at =
            Utc::now() + Duration::days(self.auth.refresh_token_ttl_days as i64);

        sqlx::query(
            r#"
            UPDATE auth_sessions
            SET access_token_hash = $1,
                refresh_token_hash = $2,
                expires_at = $3,
                refresh_expires_at = $4,
                last_used_at = now()
            WHERE id = $5
            "#,
        )
        .bind(hash_token(&access))
        .bind(hash_token(&refresh))
        .bind(access_expires_at)
        .bind(refresh_expires_at)
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        Ok(WikiAuthResponse {
            access_token: access,
            refresh_token: refresh,
            token_type: "Bearer".to_string(),
            user_id: user_id.to_string(),
            email: row.get("email"),
            username: row.get("username"),
            display_name: row.get("display_name"),
            expires_in: self.auth.access_token_ttl_minutes * 60,
        })
    }

    async fn logout(&self, claims: &WikiClaims) -> Result<(), shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        if let Some(session_id) = claims.session_id.as_deref() {
            let session_id = parse_uuid(session_id, "session")?;
            sqlx::query(
                "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2",
            )
            .bind(session_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        } else {
            sqlx::query(
                "UPDATE auth_sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
            )
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        }
        Ok(())
    }

    async fn get_current_user(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        self.user_response(user_id).await
    }

    async fn list_users(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserListResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let rows = sqlx::query(
            r#"
            SELECT id, email, username, display_name, global_role, is_active
            FROM users
            ORDER BY lower(email)
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        Ok(WikiUserListResponse {
            users: rows.iter().map(user_response_from_row).collect(),
        })
    }

    async fn create_user(
        &self,
        claims: &WikiClaims,
        body: WikiCreateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let email = normalize_required(&body.email, "email")?;
        let username = normalize_required(&body.username, "username")?;
        let display_name = normalize_required(&body.display_name, "display_name")?;
        let password = normalize_required(&body.password, "password")?;
        let role = global_role_from_request(&body.role)?;
        let user_id = Uuid::now_v7();
        let password_hash = hash_password(&password)?;

        let row = sqlx::query(
            r#"
            INSERT INTO users (
                id, email, username, display_name, password_hash,
                global_role, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, true, now(), now())
            RETURNING id, email, username, display_name, global_role, is_active
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind(username)
        .bind(display_name)
        .bind(password_hash)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        self.audit(Some(actor_id), "user.create", "user", user_id)
            .await?;
        Ok(user_response_from_row(&row))
    }

    async fn update_user(
        &self,
        claims: &WikiClaims,
        user_id: &str,
        body: WikiUpdateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let user_id = parse_uuid(user_id, "user")?;
        let role = match body.role.as_deref() {
            Some(role) => Some(global_role_from_request(role)?),
            None => None,
        };
        let global_role = if body.is_system_admin == Some(true) {
            Some("admin")
        } else if body.is_system_admin == Some(false) {
            Some("user")
        } else {
            role
        };

        let row = sqlx::query(
            r#"
            UPDATE users
            SET email = COALESCE($2, email),
                username = COALESCE($3, username),
                display_name = COALESCE($4, display_name),
                global_role = COALESCE($5, global_role),
                is_active = COALESCE($6, is_active),
                updated_at = now()
            WHERE id = $1
            RETURNING id, email, username, display_name, global_role, is_active
            "#,
        )
        .bind(user_id)
        .bind(
            body.email
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(
            body.username
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(
            body.display_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(global_role)
        .bind(body.active)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("user", user_id))?;

        self.audit(Some(actor_id), "user.update", "user", user_id)
            .await?;
        Ok(user_response_from_row(&row))
    }

    async fn list_spaces(&self) -> Result<SpaceListResponse, shared::AppError> {
        let rows = sqlx::query(SPACE_LIST_SQL)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(SpaceListResponse {
            spaces: rows.iter().map(space_response_from_row).collect(),
        })
    }

    async fn create_space(
        &self,
        claims: &WikiClaims,
        body: CreateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(&body.key)?;
        let name = normalize_required(&body.name, "space name")?;
        let description = body.description.unwrap_or_default();
        let space_id = Uuid::now_v7();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let row = sqlx::query(
            r#"
            INSERT INTO spaces (id, key, name, description, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, now(), now())
            RETURNING id
            "#,
        )
        .bind(space_id)
        .bind(&key)
        .bind(name)
        .bind(description)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        let space_id: Uuid = row.get("id");

        sqlx::query(
            r#"
            INSERT INTO space_members (space_id, user_id, role, joined_at)
            VALUES ($1, $2, 'admin', now())
            "#,
        )
        .bind(space_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        self.insert_audit(&mut tx, Some(actor_id), "space.create", "space", space_id)
            .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.get_space_by_key(&key).await
    }

    async fn get_space_by_key(&self, space_key: &str) -> Result<SpaceResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let row = sqlx::query(SPACE_ONE_SQL)
            .bind(&key)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
        Ok(space_response_from_row(&row))
    }

    async fn update_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: UpdateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let row = sqlx::query(
            r#"
            UPDATE spaces
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = now()
            WHERE key = $1
            RETURNING id
            "#,
        )
        .bind(&key)
        .bind(
            body.name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(body.description.as_deref().map(str::trim))
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
        let space_id: Uuid = row.get("id");
        self.audit(Some(actor_id), "space.update", "space", space_id)
            .await?;
        self.get_space_by_key(&key).await
    }

    async fn archive_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let row = sqlx::query(
            "UPDATE spaces SET archived_at = now(), updated_at = now() WHERE key = $1 RETURNING id",
        )
        .bind(&key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
        let space_id: Uuid = row.get("id");
        self.audit(Some(actor_id), "space.archive", "space", space_id)
            .await?;
        self.get_space_by_key(&key).await
    }

    async fn list_space_members(
        &self,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let rows = sqlx::query(
            r#"
            SELECT sm.user_id, u.email, u.display_name, sm.role, sm.joined_at
            FROM space_members sm
            JOIN users u ON u.id = sm.user_id
            WHERE sm.space_id = $1
            ORDER BY lower(u.email)
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(SpaceMemberListResponse {
            members: rows.iter().map(space_member_response_from_row).collect(),
        })
    }

    async fn upsert_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
        body: UpsertSpaceMemberRequest,
    ) -> Result<SpaceMemberResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let space_id = self.space_id(space_key).await?;
        let user_id = parse_uuid(user_id, "user")?;
        let role = normalize_space_role(&body.role)?;

        let row = sqlx::query(
            r#"
            INSERT INTO space_members (space_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (space_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            RETURNING user_id, role, joined_at
            "#,
        )
        .bind(space_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        self.audit(Some(actor_id), "space.member_upsert", "space", space_id)
            .await?;
        let user = self.user_response(user_id).await?;
        Ok(SpaceMemberResponse {
            user_id: row.get::<Uuid, _>("user_id").to_string(),
            email: user.email,
            display_name: user.display_name,
            role: row.get("role"),
            joined_at: to_iso(row.get("joined_at")),
        })
    }

    async fn delete_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
    ) -> Result<(), shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let space_id = self.space_id(space_key).await?;
        let user_id = parse_uuid(user_id, "user")?;
        sqlx::query("DELETE FROM space_members WHERE space_id = $1 AND user_id = $2")
            .bind(space_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "space.member_delete", "space", space_id)
            .await?;
        Ok(())
    }

    async fn get_space_tree(&self, space_key: &str) -> Result<SpaceTreeResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self.space_id(&key).await?;
        let rows = sqlx::query(
            r#"
            SELECT id, parent_id, slug, title, document_type, status, position, updated_at
            FROM documents
            WHERE space_id = $1 AND archived_at IS NULL
            ORDER BY parent_id NULLS FIRST, position, title
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents = build_db_tree(&rows, None);
        Ok(SpaceTreeResponse {
            space_key: key,
            documents,
        })
    }

    async fn create_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: CreateDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let space_id = self.space_id(&key).await?;
        let title = normalize_required(&body.title, "document title")?;
        let document_type = normalize_document_type(&body.document_type, true)?;
        let document_id = Uuid::now_v7();
        let mut slug = body.slug.unwrap_or_else(|| slugify(&title));
        slug = slugify(&slug);
        if slug.is_empty() {
            slug = format!("document-{}", document_id.simple());
            slug.truncate(17);
        }
        let slug = normalize_slug(&slug)?;

        let parent_id = match body.parent_id {
            Some(parent_id) => {
                let resolved = self.resolve_document_id(&parent_id).await?;
                let parent_space_id = self.document_space_id(resolved).await?;
                if parent_space_id != space_id {
                    return Err(shared::AppError::invalid_input(
                        "parent document belongs to another space",
                    ));
                }
                Some(resolved)
            }
            None => None,
        };

        let task_key = match body.task_key {
            Some(value) => Some(normalize_task_key(&value)?),
            None => None,
        };
        let phase_key = match body.phase_key {
            Some(value) => Some(normalize_phase_key(&value)?),
            None => None,
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let position: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM documents
            WHERE space_id = $1
              AND (($2::uuid IS NULL AND parent_id IS NULL) OR parent_id = $2)
            "#,
        )
        .bind(space_id)
        .bind(parent_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        sqlx::query(
            r#"
            INSERT INTO documents (
                id, space_id, parent_id, slug, title, document_type, status,
                owner_id, position, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7, $8, now(), now())
            "#,
        )
        .bind(document_id)
        .bind(space_id)
        .bind(parent_id)
        .bind(&slug)
        .bind(title)
        .bind(document_type)
        .bind(actor_id)
        .bind(position as i32)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        sqlx::query(
            r#"
            INSERT INTO document_drafts (document_id, author_id, content_markdown, updated_at)
            VALUES ($1, $2, $3, now())
            "#,
        )
        .bind(document_id)
        .bind(actor_id)
        .bind(body.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        if let Some(task_key) = &task_key {
            let task_id = self
                .upsert_task_dossier_tx(&mut tx, space_id, task_key)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO document_task_links (space_id, document_id, task_dossier_id, created_by, created_at)
                VALUES ($1, $2, $3, $4, now())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(space_id)
            .bind(document_id)
            .bind(task_id)
            .bind(actor_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        }
        if let Some(phase_key) = &phase_key {
            let phase_id = self
                .upsert_phase_dossier_tx(&mut tx, space_id, phase_key)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO document_phase_links (space_id, document_id, phase_dossier_id, created_by, created_at)
                VALUES ($1, $2, $3, $4, now())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(space_id)
            .bind(document_id)
            .bind(phase_id)
            .bind(actor_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        }

        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "document.create",
            "document",
            document_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.document_response(document_id).await
    }

    async fn get_document(&self, document_id: &str) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.document_response(document_id).await
    }

    async fn update_document_draft(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: UpdateDocumentDraftRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let document_id = self.resolve_document_id(document_id).await?;
        let title = body
            .title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE id = $1")
            .bind(document_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        if exists.is_none() {
            return Err(shared::AppError::not_found("document", document_id));
        }
        sqlx::query(
            r#"
            UPDATE documents
            SET title = COALESCE($2, title),
                status = 'draft',
                archived_at = NULL,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(document_id)
        .bind(title)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        sqlx::query(
            r#"
            INSERT INTO document_drafts (document_id, author_id, content_markdown, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (document_id)
            DO UPDATE SET author_id = EXCLUDED.author_id,
                          content_markdown = EXCLUDED.content_markdown,
                          updated_at = now()
            "#,
        )
        .bind(document_id)
        .bind(actor_id)
        .bind(body.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "document.draft_update",
            "document",
            document_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.document_response(document_id).await
    }

    async fn publish_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: PublishDocumentRequest,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let document_id = self.resolve_document_id(document_id).await?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let row = sqlx::query(
            r#"
            SELECT d.title, COALESCE(dd.content_markdown, '') AS content_markdown
            FROM documents d
            LEFT JOIN document_drafts dd ON dd.document_id = d.id
            WHERE d.id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;
        let title: String = row.get("title");
        let content_markdown: String = row.get("content_markdown");
        if content_markdown.trim().is_empty() {
            return Err(shared::AppError::invalid_input(
                "published content is required",
            ));
        }
        let version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM document_revisions WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        let revision_id = Uuid::now_v7();
        let content_text = markdown_to_text(&content_markdown);
        let content_checksum = checksum(content_markdown.as_bytes());

        let revision_row = sqlx::query(
            r#"
            INSERT INTO document_revisions (
                id, document_id, version, title, content_markdown, content_html,
                content_text, content_checksum, summary, author_id, published_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8, $9, now())
            RETURNING id, document_id, version, title, content_markdown, summary, author_id, published_at
            "#,
        )
        .bind(revision_id)
        .bind(document_id)
        .bind(version)
        .bind(title)
        .bind(&content_markdown)
        .bind(content_text)
        .bind(content_checksum)
        .bind(body.summary)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        sqlx::query(
            r#"
            UPDATE documents
            SET current_revision_id = $2, status = 'published', archived_at = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(document_id)
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        sqlx::query(
            "UPDATE document_drafts SET base_revision_id = $2, updated_at = now() WHERE document_id = $1",
        )
        .bind(document_id)
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "document.publish",
            "document",
            document_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        Ok(revision_response_from_row(&revision_row))
    }

    async fn archive_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let document_id = self.resolve_document_id(document_id).await?;
        let row = sqlx::query(
            r#"
            UPDATE documents
            SET status = 'archived', archived_at = now(), updated_at = now()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;
        let document_id: Uuid = row.get("id");
        self.audit(Some(actor_id), "document.archive", "document", document_id)
            .await?;
        self.document_response(document_id).await
    }

    async fn move_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: MoveDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let document_id = self.resolve_document_id(document_id).await?;
        let document_space_id = self.document_space_id(document_id).await?;
        let parent_id = match body.parent_id {
            Some(parent_id) => {
                let parent_id = self.resolve_document_id(&parent_id).await?;
                if parent_id == document_id {
                    return Err(shared::AppError::invalid_input(
                        "document cannot be moved under itself",
                    ));
                }
                let parent_space_id = self.document_space_id(parent_id).await?;
                if parent_space_id != document_space_id {
                    return Err(shared::AppError::invalid_input(
                        "parent document belongs to another space",
                    ));
                }
                Some(parent_id)
            }
            None => None,
        };
        sqlx::query("UPDATE documents SET parent_id = $2, updated_at = now() WHERE id = $1")
            .bind(document_id)
            .bind(parent_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "document.move", "document", document_id)
            .await?;
        self.document_response(document_id).await
    }

    async fn list_document_revisions(
        &self,
        document_id: &str,
    ) -> Result<DocumentRevisionListResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        let rows = sqlx::query(
            r#"
            SELECT id, document_id, version, title, content_markdown, summary, author_id, published_at
            FROM document_revisions
            WHERE document_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(DocumentRevisionListResponse {
            revisions: rows.iter().map(revision_response_from_row).collect(),
        })
    }

    async fn get_document_revision(
        &self,
        document_id: &str,
        revision_id: &str,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        let revision_id = parse_uuid(revision_id, "revision")?;
        let row = sqlx::query(
            r#"
            SELECT id, document_id, version, title, content_markdown, summary, author_id, published_at
            FROM document_revisions
            WHERE document_id = $1 AND id = $2
            "#,
        )
        .bind(document_id)
        .bind(revision_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("revision", revision_id))?;
        Ok(revision_response_from_row(&row))
    }

    async fn list_tasks(&self, space_key: &str) -> Result<TaskPageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self.space_id(&key).await?;
        let rows = sqlx::query(
            r#"
            SELECT task_key
            FROM task_dossiers
            WHERE space_id = $1
            ORDER BY task_key
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let task_key: String = row.get("task_key");
            tasks.push(self.task_page(&key, &task_key).await?);
        }
        Ok(TaskPageListResponse { tasks })
    }

    async fn get_task(
        &self,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.space_id(&key).await?;
        let task_key = normalize_task_key(task_key)?;
        self.task_page(&key, &task_key).await
    }

    async fn link_task_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let space_id = self.space_id(&key).await?;
        let task_key = normalize_task_key(task_key)?;
        let document_id = self.resolve_document_id(&body.document_id).await?;
        if self.document_space_id(document_id).await? != space_id {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let task_id = self
            .upsert_task_dossier_tx(&mut tx, space_id, &task_key)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO document_task_links (space_id, document_id, task_dossier_id, created_by, created_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(space_id)
        .bind(document_id)
        .bind(task_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "task.link_document",
            "task",
            task_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.task_page(&key, &task_key).await
    }

    async fn list_task_documents(
        &self,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let task = self.get_task(space_key, task_key).await?;
        let mut documents = Vec::with_capacity(task.documents.len());
        for summary in task.documents {
            documents.push(
                self.document_response(parse_uuid(&summary.id, "document")?)
                    .await?,
            );
        }
        Ok(DocumentListResponse { documents })
    }

    async fn list_task_evidence(
        &self,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_task(space_key, task_key).await?.evidence,
        })
    }

    async fn list_phases(
        &self,
        space_key: &str,
    ) -> Result<PhasePageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self.space_id(&key).await?;
        let rows = sqlx::query(
            r#"
            SELECT phase_key
            FROM phase_dossiers
            WHERE space_id = $1
            ORDER BY phase_key
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let mut phases = Vec::with_capacity(rows.len());
        for row in rows {
            let phase_key: String = row.get("phase_key");
            phases.push(self.phase_page(&key, &phase_key).await?);
        }
        Ok(PhasePageListResponse { phases })
    }

    async fn get_phase(
        &self,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.space_id(&key).await?;
        let phase_key = normalize_phase_key(phase_key)?;
        self.phase_page(&key, &phase_key).await
    }

    async fn link_phase_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let space_id = self.space_id(&key).await?;
        let phase_key = normalize_phase_key(phase_key)?;
        let document_id = self.resolve_document_id(&body.document_id).await?;
        if self.document_space_id(document_id).await? != space_id {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let phase_id = self
            .upsert_phase_dossier_tx(&mut tx, space_id, &phase_key)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO document_phase_links (space_id, document_id, phase_dossier_id, created_by, created_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(space_id)
        .bind(document_id)
        .bind(phase_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "phase.link_document",
            "phase",
            phase_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.phase_page(&key, &phase_key).await
    }

    async fn list_phase_documents(
        &self,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let phase = self.get_phase(space_key, phase_key).await?;
        let mut documents = Vec::with_capacity(phase.documents.len());
        for summary in phase.documents {
            documents.push(
                self.document_response(parse_uuid(&summary.id, "document")?)
                    .await?,
            );
        }
        Ok(DocumentListResponse { documents })
    }

    async fn list_phase_evidence(
        &self,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_phase(space_key, phase_key).await?.evidence,
        })
    }

    async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let evidence_type = normalize_evidence_type(&body.evidence_type)?;
        match evidence_type {
            "external_url" if body.url.is_none() || body.attachment_id.is_some() => {
                return Err(shared::AppError::invalid_input(
                    "external_url evidence requires url only",
                ));
            }
            "uploaded_file" if body.attachment_id.is_none() || body.url.is_some() => {
                return Err(shared::AppError::invalid_input(
                    "uploaded_file evidence requires attachment_id only",
                ));
            }
            "external_url" | "uploaded_file" => {}
            _ => unreachable!("validated evidence type"),
        }

        let document_id = match body.document_id.as_deref() {
            Some(value) => Some(self.resolve_document_id(value).await?),
            None => None,
        };
        let document_space_id = match document_id {
            Some(id) => Some(self.document_space_id(id).await?),
            None => None,
        };
        let space_key = body
            .space
            .as_deref()
            .map(normalize_space_key)
            .transpose()?
            .unwrap_or_else(|| "SDLC".to_string());
        let space_id = if let Some(document_space_id) = document_space_id {
            let requested_space_id = self.space_id(&space_key).await?;
            if requested_space_id != document_space_id {
                return Err(shared::AppError::invalid_input(
                    "document belongs to another space",
                ));
            }
            requested_space_id
        } else {
            self.space_id(&space_key).await?
        };
        let task_key = body
            .task_key
            .as_deref()
            .map(normalize_task_key)
            .transpose()?;
        let phase_key = body
            .phase_key
            .as_deref()
            .map(normalize_phase_key)
            .transpose()?;
        if document_id.is_none() && task_key.is_none() && phase_key.is_none() {
            return Err(shared::AppError::invalid_input(
                "evidence must target a document, task or phase",
            ));
        }
        let title = normalize_required(&body.title, "evidence title")?;
        let evidence_id = Uuid::now_v7();
        let attachment_id = body
            .attachment_id
            .as_deref()
            .map(|value| parse_uuid(value, "attachment"))
            .transpose()?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let task_dossier_id = match &task_key {
            Some(task_key) => Some(
                self.upsert_task_dossier_tx(&mut tx, space_id, task_key)
                    .await?,
            ),
            None => None,
        };
        let phase_dossier_id = match &phase_key {
            Some(phase_key) => Some(
                self.upsert_phase_dossier_tx(&mut tx, space_id, phase_key)
                    .await?,
            ),
            None => None,
        };
        let mut stored_checksum = body.checksum;
        if let Some(attachment_id) = attachment_id {
            let attachment_row = sqlx::query(
                "SELECT checksum FROM attachments WHERE id = $1 AND owner_entity_id IS NULL",
            )
            .bind(attachment_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
            stored_checksum = Some(attachment_row.get("checksum"));
            sqlx::query(
                r#"
                UPDATE attachments
                SET space_id = $2, owner_entity_type = 'evidence', owner_entity_id = $3
                WHERE id = $1
                "#,
            )
            .bind(attachment_id)
            .bind(space_id)
            .bind(evidence_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        }

        let row = sqlx::query(
            r#"
            INSERT INTO evidence_items (
                id, space_id, document_id, task_dossier_id, phase_dossier_id,
                evidence_type, title, url, attachment_id, checksum, metadata,
                created_by, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, '{}'::jsonb, $11, now())
            RETURNING id
            "#,
        )
        .bind(evidence_id)
        .bind(space_id)
        .bind(document_id)
        .bind(task_dossier_id)
        .bind(phase_dossier_id)
        .bind(evidence_type)
        .bind(title)
        .bind(body.url)
        .bind(attachment_id)
        .bind(stored_checksum)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        let evidence_id: Uuid = row.get("id");
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "evidence.create",
            "evidence",
            evidence_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.get_evidence_by_id(evidence_id).await
    }

    async fn list_evidence(
        &self,
        query: EvidenceQuery,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        let space_key = query
            .space
            .as_deref()
            .map(normalize_space_key)
            .transpose()?;
        let document_id = match query.document_id.as_deref() {
            Some(value) => Some(self.resolve_document_id(value).await?),
            None => None,
        };
        let task_key = query
            .task_key
            .as_deref()
            .map(normalize_task_key)
            .transpose()?;
        let phase_key = query
            .phase_key
            .as_deref()
            .map(normalize_phase_key)
            .transpose()?;
        let limit = clamp_limit(query.limit, 100);
        let rows = sqlx::query(EVIDENCE_LIST_SQL)
            .bind(space_key)
            .bind(document_id)
            .bind(task_key)
            .bind(phase_key)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(EvidenceListResponse {
            evidence: rows.iter().map(evidence_response_from_row).collect(),
        })
    }

    async fn get_evidence_by_id(
        &self,
        evidence_id: Uuid,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let row = sqlx::query(EVIDENCE_ONE_SQL)
            .bind(evidence_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("evidence", evidence_id))?;
        Ok(evidence_response_from_row(&row))
    }

    async fn get_evidence(&self, evidence_id: &str) -> Result<EvidenceResponse, shared::AppError> {
        self.get_evidence_by_id(parse_uuid(evidence_id, "evidence")?)
            .await
    }

    async fn upload_attachment(
        &self,
        claims: &WikiClaims,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        if bytes.is_empty() {
            return Err(shared::AppError::invalid_input("file is required"));
        }
        if bytes.len() > self.max_upload_bytes {
            return Err(shared::AppError::invalid_input("file is too large"));
        }
        let id = Uuid::now_v7();
        let safe_name = safe_download_filename(&file_name);
        let storage_key = format!("attachments/{id}/{safe_name}");
        let path = self.storage_dir.join(&storage_key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(shared::AppError::internal)?;
        }
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(shared::AppError::internal)?;

        let checksum = checksum(&bytes);
        let size_bytes = bytes.len() as i64;
        let row = match sqlx::query(
            r#"
            INSERT INTO attachments (
                id, file_name, content_type, size_bytes, storage_key,
                checksum, uploaded_by, uploaded_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, now())
            RETURNING id, file_name, content_type, size_bytes, checksum, uploaded_by, uploaded_at
            "#,
        )
        .bind(id)
        .bind(file_name)
        .bind(content_type)
        .bind(size_bytes)
        .bind(&storage_key)
        .bind(checksum)
        .bind(actor_id)
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => row,
            Err(err) => {
                let _ = tokio::fs::remove_file(&path).await;
                return Err(shared::AppError::database(err));
            }
        };

        self.audit(Some(actor_id), "attachment.upload", "attachment", id)
            .await?;
        Ok(attachment_response_from_row(&row))
    }

    async fn get_attachment(
        &self,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        let row = sqlx::query(ATTACHMENT_ONE_SQL)
            .bind(attachment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        Ok(attachment_response_from_row(&row))
    }

    async fn download_attachment(&self, attachment_id: &str) -> Result<Response, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        let row = sqlx::query(
            r#"
            SELECT file_name, content_type, storage_key
            FROM attachments
            WHERE id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        let file_name: String = row.get("file_name");
        let content_type: String = row.get("content_type");
        let storage_key: String = row.get("storage_key");
        let bytes = tokio::fs::read(self.storage_dir.join(storage_key))
            .await
            .map_err(shared::AppError::internal)?;
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&content_type).map_err(shared::AppError::internal)?,
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachment; filename=\"{}\"",
                safe_download_filename(&file_name)
            ))
            .map_err(shared::AppError::internal)?,
        );
        Ok((headers, bytes).into_response())
    }

    async fn list_templates(&self) -> Result<TemplateListResponse, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, document_type, content_markdown
            FROM document_templates
            WHERE is_active = true
            ORDER BY lower(name)
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(TemplateListResponse {
            templates: rows.iter().map(template_response_from_row).collect(),
        })
    }

    async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let name = normalize_required(&body.name, "template name")?;
        let document_type = normalize_document_type(&body.document_type, false)?;
        let body_markdown = normalize_required(&body.body_markdown, "template body_markdown")?;
        let id = Uuid::now_v7();
        let row = sqlx::query(
            r#"
            INSERT INTO document_templates (
                id, name, document_type, content_markdown, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, true, now(), now())
            RETURNING id, name, document_type, content_markdown
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(document_type)
        .bind(body_markdown)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "template.create", "template", id)
            .await?;
        Ok(template_response_from_row(&row))
    }

    async fn list_audit_log(
        &self,
        claims: &WikiClaims,
    ) -> Result<AuditLogResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let rows = sqlx::query(
            r#"
            SELECT id, actor_id, action, entity_type, entity_id, created_at
            FROM audit_log
            ORDER BY created_at DESC
            LIMIT 200
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(AuditLogResponse {
            entries: rows.iter().map(audit_entry_from_row).collect(),
        })
    }

    async fn search(&self, query: SearchQuery) -> Result<SearchResponse, shared::AppError> {
        let needle = query.q.unwrap_or_default();
        let pattern = format!("%{}%", needle.to_lowercase());
        let space_key = query
            .space
            .as_deref()
            .map(normalize_space_key)
            .transpose()?;
        let task_key = query
            .task_key
            .as_deref()
            .map(normalize_task_key)
            .transpose()?;
        let phase_key = query
            .phase_key
            .as_deref()
            .map(normalize_phase_key)
            .transpose()?;
        let document_type = match query.document_type.as_deref() {
            Some(value) => Some(normalize_document_type(value, true)?),
            None => None,
        };
        let include_archived = query.include_archived.unwrap_or(false);
        let limit = clamp_limit(query.limit, 50);

        let document_rows = sqlx::query(SEARCH_DOCUMENTS_SQL)
            .bind(&pattern)
            .bind(space_key.as_deref())
            .bind(task_key.as_deref())
            .bind(phase_key.as_deref())
            .bind(document_type.as_deref())
            .bind(include_archived)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        let evidence_rows = sqlx::query(SEARCH_EVIDENCE_SQL)
            .bind(&pattern)
            .bind(space_key.as_deref())
            .bind(task_key.as_deref())
            .bind(phase_key.as_deref())
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;

        let mut results = document_rows
            .iter()
            .map(search_result_from_row)
            .chain(evidence_rows.iter().map(search_result_from_row))
            .collect::<Vec<_>>();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(limit as usize);
        Ok(SearchResponse { results })
    }

    async fn issue_tokens(
        &self,
        user_id: Uuid,
        user: &PgRow,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let session_id = Uuid::now_v7();
        let access_expires_at =
            Utc::now() + Duration::minutes(self.auth.access_token_ttl_minutes as i64);
        let refresh_expires_at =
            Utc::now() + Duration::days(self.auth.refresh_token_ttl_days as i64);
        if refresh_expires_at <= access_expires_at {
            return Err(shared::AppError::invalid_input(
                "refresh token lifetime must be longer than access token lifetime",
            ));
        }
        let access = create_token(
            &self.auth,
            user_id,
            session_id,
            "access",
            Duration::minutes(self.auth.access_token_ttl_minutes as i64),
        )?;
        let refresh = create_token(
            &self.auth,
            user_id,
            session_id,
            "refresh",
            Duration::days(self.auth.refresh_token_ttl_days as i64),
        )?;

        sqlx::query(
            r#"
            INSERT INTO auth_sessions (
                id, user_id, access_token_hash, refresh_token_hash,
                expires_at, refresh_expires_at, created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now(), now())
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(hash_token(&access))
        .bind(hash_token(&refresh))
        .bind(access_expires_at)
        .bind(refresh_expires_at)
        .execute(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        Ok(WikiAuthResponse {
            access_token: access,
            refresh_token: refresh,
            token_type: "Bearer".to_string(),
            user_id: user_id.to_string(),
            email: user.get("email"),
            username: user.get("username"),
            display_name: user.get("display_name"),
            expires_in: self.auth.access_token_ttl_minutes * 60,
        })
    }

    async fn ensure_admin(&self, claims: &WikiClaims) -> Result<Uuid, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let role: Option<String> =
            sqlx::query_scalar("SELECT global_role FROM users WHERE id = $1 AND is_active = true")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?;
        match role.as_deref() {
            Some("admin") => Ok(user_id),
            Some(_) => Err(shared::AppError::Forbidden),
            None => Err(shared::AppError::Unauthorized),
        }
    }

    async fn user_response(&self, user_id: Uuid) -> Result<WikiUserResponse, shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, username, display_name, global_role, is_active
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("user", user_id))?;
        Ok(user_response_from_row(&row))
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

    async fn document_space_id(&self, document_id: Uuid) -> Result<Uuid, shared::AppError> {
        sqlx::query_scalar("SELECT space_id FROM documents WHERE id = $1")
            .bind(document_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("document", document_id))
    }

    async fn resolve_document_id(&self, value: &str) -> Result<Uuid, shared::AppError> {
        if let Ok(id) = Uuid::parse_str(value) {
            let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?;
            return exists.ok_or_else(|| shared::AppError::not_found("document", value));
        }

        let rows = sqlx::query(
            r#"
            SELECT id
            FROM documents
            WHERE slug = $1 AND archived_at IS NULL
            ORDER BY updated_at DESC
            LIMIT 2
            "#,
        )
        .bind(value)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        match rows.as_slice() {
            [row] => Ok(row.get("id")),
            [] => Err(shared::AppError::not_found("document", value)),
            _ => Err(shared::AppError::conflict(
                "document slug is ambiguous across spaces",
            )),
        }
    }

    async fn document_response(
        &self,
        document_id: Uuid,
    ) -> Result<DocumentResponse, shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT d.id, s.key AS space_key, d.parent_id, d.slug, d.title,
                   d.document_type, d.status, d.current_revision_id, d.owner_id,
                   d.created_at, d.updated_at,
                   COALESCE(dd.content_markdown, '') AS draft_markdown
            FROM documents d
            JOIN spaces s ON s.id = d.space_id
            LEFT JOIN document_drafts dd ON dd.document_id = d.id
            WHERE d.id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;

        let current_revision_id: Option<Uuid> = row.get("current_revision_id");
        let current_revision = match current_revision_id {
            Some(revision_id) => Some(self.revision_response(document_id, revision_id).await?),
            None => None,
        };
        let task_keys = self.document_task_keys(document_id).await?;
        let phase_keys = self.document_phase_keys(document_id).await?;
        let evidence = self
            .list_evidence(EvidenceQuery {
                space: None,
                document_id: Some(document_id.to_string()),
                task_key: None,
                phase_key: None,
                limit: Some(100),
            })
            .await?
            .evidence;
        let owner_id: Uuid = row.get("owner_id");

        Ok(DocumentResponse {
            id: row.get::<Uuid, _>("id").to_string(),
            space_key: row.get("space_key"),
            parent_id: row
                .get::<Option<Uuid>, _>("parent_id")
                .map(|id| id.to_string()),
            slug: row.get("slug"),
            title: row.get("title"),
            document_type: row.get("document_type"),
            status: row.get("status"),
            body_markdown: current_revision
                .as_ref()
                .map(|revision| revision.body_markdown.clone())
                .unwrap_or_default(),
            draft_markdown: row.get("draft_markdown"),
            current_revision,
            task_keys,
            phase_keys,
            evidence,
            created_by: owner_id.to_string(),
            updated_by: owner_id.to_string(),
            created_at: to_iso(row.get("created_at")),
            updated_at: to_iso(row.get("updated_at")),
        })
    }

    async fn revision_response(
        &self,
        document_id: Uuid,
        revision_id: Uuid,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, document_id, version, title, content_markdown, summary, author_id, published_at
            FROM document_revisions
            WHERE document_id = $1 AND id = $2
            "#,
        )
        .bind(document_id)
        .bind(revision_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("revision", revision_id))?;
        Ok(revision_response_from_row(&row))
    }

    async fn document_task_keys(&self, document_id: Uuid) -> Result<Vec<String>, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT td.task_key
            FROM document_task_links dtl
            JOIN task_dossiers td ON td.id = dtl.task_dossier_id
            WHERE dtl.document_id = $1
            ORDER BY td.task_key
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(|row| row.get("task_key")).collect())
    }

    async fn document_phase_keys(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<String>, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT pd.phase_key
            FROM document_phase_links dpl
            JOIN phase_dossiers pd ON pd.id = dpl.phase_dossier_id
            WHERE dpl.document_id = $1
            ORDER BY pd.phase_key
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(|row| row.get("phase_key")).collect())
    }

    async fn task_page(
        &self,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let task_row = sqlx::query(
            "SELECT id, title_snapshot FROM task_dossiers WHERE space_id = $1 AND task_key = $2",
        )
        .bind(space_id)
        .bind(task_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let Some(task_row) = task_row else {
            return Ok(TaskPageResponse {
                space_key: space_key.to_string(),
                task_key: task_key.to_string(),
                title: None,
                document_count: 0,
                evidence_count: 0,
                documents: Vec::new(),
                evidence: Vec::new(),
            });
        };
        let task_id: Uuid = task_row.get("id");
        let document_rows = sqlx::query(
            r#"
            SELECT d.id, d.slug, d.title, d.document_type, d.status, d.updated_at
            FROM document_task_links dtl
            JOIN documents d ON d.id = dtl.document_id
            WHERE dtl.task_dossier_id = $1 AND d.archived_at IS NULL
            ORDER BY d.updated_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents: Vec<_> = document_rows
            .iter()
            .map(document_summary_from_row)
            .collect();
        let evidence = self.evidence_for_target(Some(task_id), None).await?;
        let title_snapshot: Option<String> = task_row.get("title_snapshot");
        let title =
            title_snapshot.or_else(|| documents.first().map(|document| document.title.clone()));
        Ok(TaskPageResponse {
            space_key: space_key.to_string(),
            task_key: task_key.to_string(),
            title,
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        })
    }

    async fn phase_page(
        &self,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let phase_row = sqlx::query(
            "SELECT id, phase_name FROM phase_dossiers WHERE space_id = $1 AND phase_key = $2",
        )
        .bind(space_id)
        .bind(phase_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let Some(phase_row) = phase_row else {
            return Ok(PhasePageResponse {
                space_key: space_key.to_string(),
                phase_key: phase_key.to_string(),
                title: Some(phase_key.to_string()),
                document_count: 0,
                evidence_count: 0,
                documents: Vec::new(),
                evidence: Vec::new(),
            });
        };
        let phase_id: Uuid = phase_row.get("id");
        let document_rows = sqlx::query(
            r#"
            SELECT d.id, d.slug, d.title, d.document_type, d.status, d.updated_at
            FROM document_phase_links dpl
            JOIN documents d ON d.id = dpl.document_id
            WHERE dpl.phase_dossier_id = $1 AND d.archived_at IS NULL
            ORDER BY d.updated_at DESC
            "#,
        )
        .bind(phase_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents: Vec<_> = document_rows
            .iter()
            .map(document_summary_from_row)
            .collect();
        let evidence = self.evidence_for_target(None, Some(phase_id)).await?;
        let phase_name: Option<String> = phase_row.get("phase_name");
        Ok(PhasePageResponse {
            space_key: space_key.to_string(),
            phase_key: phase_key.to_string(),
            title: phase_name.or_else(|| Some(phase_key.to_string())),
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        })
    }

    async fn evidence_for_target(
        &self,
        task_dossier_id: Option<Uuid>,
        phase_dossier_id: Option<Uuid>,
    ) -> Result<Vec<EvidenceResponse>, shared::AppError> {
        let rows = sqlx::query(EVIDENCE_TARGET_SQL)
            .bind(task_dossier_id)
            .bind(phase_dossier_id)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(evidence_response_from_row).collect())
    }

    async fn upsert_task_dossier_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        space_id: Uuid,
        task_key: &str,
    ) -> Result<Uuid, shared::AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_dossiers (id, space_id, task_key, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            ON CONFLICT (space_id, task_key)
            DO UPDATE SET updated_at = now()
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(space_id)
        .bind(task_key)
        .fetch_one(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(row.get("id"))
    }

    async fn upsert_phase_dossier_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        space_id: Uuid,
        phase_key: &str,
    ) -> Result<Uuid, shared::AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO phase_dossiers (id, space_id, phase_key, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            ON CONFLICT (space_id, phase_key)
            DO UPDATE SET updated_at = now()
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(space_id)
        .bind(phase_key)
        .fetch_one(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(row.get("id"))
    }

    async fn audit(
        &self,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), shared::AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, actor_id, action, entity_type, entity_id, request_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(format!("api-{}", Uuid::now_v7()))
        .execute(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(())
    }

    async fn insert_audit(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), shared::AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, actor_id, action, entity_type, entity_id, request_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(format!("api-{}", Uuid::now_v7()))
        .execute(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(())
    }
}

impl WikiBackend {
    async fn authenticate_access_token(&self, token: &str) -> Result<WikiClaims, shared::AppError> {
        if let Some(postgres) = self.postgres() {
            return postgres.authenticate_access_token(token).await;
        }

        let user_id = {
            let store = store().lock().expect("wiki store lock");
            store
                .tokens
                .get(token)
                .and_then(|user_id| store.users.get(user_id).filter(|user| user.active))
                .map(|user| user.id.clone())
        }
        .ok_or(shared::AppError::Unauthorized)?;

        Ok(WikiClaims {
            user_id,
            session_id: None,
        })
    }
}

fn ensure_system_admin(store: &WikiStore, user_id: &str) -> Result<(), shared::AppError> {
    let user = store
        .users
        .get(user_id)
        .filter(|user| user.active)
        .ok_or(shared::AppError::Unauthorized)?;
    if user.is_system_admin {
        Ok(())
    } else {
        Err(shared::AppError::Forbidden)
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = WikiRegisterRequest,
    responses((status = 201, body = WikiAuthResponse))
)]
pub async fn register(
    Extension(backend): Extension<WikiBackend>,
    Json(body): Json<WikiRegisterRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        let response = postgres.register(body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    if body.email.trim().is_empty()
        || body.username.trim().is_empty()
        || body.password.trim().is_empty()
    {
        return Err(shared::AppError::invalid_input(
            "email, username and password are required",
        ));
    }
    let mut store = store().lock().expect("wiki store lock");
    if store.users.values().any(|user| user.email == body.email) {
        return Err(shared::AppError::conflict("email already exists"));
    }
    let user_id = new_id();
    let user = WikiUserResponse {
        id: user_id.clone(),
        email: body.email,
        username: body.username.clone(),
        display_name: body.name.unwrap_or(body.username),
        role: default_user_role(),
        is_system_admin: false,
        active: true,
    };
    store.passwords.insert(user_id.clone(), body.password);
    store.users.insert(user_id.clone(), user.clone());
    store.audit(&user_id, "auth.register", "user", &user_id);
    Ok((StatusCode::CREATED, Json(auth_response(&mut store, &user))))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = WikiLoginRequest,
    responses((status = 200, body = WikiAuthResponse), (status = 401))
)]
pub async fn login(
    Extension(backend): Extension<WikiBackend>,
    Json(body): Json<WikiLoginRequest>,
) -> Result<Json<WikiAuthResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.login(body).await?));
    }

    let mut store = store().lock().expect("wiki store lock");
    let user = store
        .users
        .values()
        .find(|user| user.email == body.email && user.active)
        .cloned()
        .ok_or(shared::AppError::Unauthorized)?;
    if store.passwords.get(&user.id) != Some(&body.password) {
        return Err(shared::AppError::Unauthorized);
    }
    store.audit(&user.id, "auth.login", "user", &user.id);
    Ok(Json(auth_response(&mut store, &user)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "auth",
    request_body = WikiRefreshRequest,
    responses((status = 200, body = WikiAuthResponse), (status = 401))
)]
pub async fn refresh(
    Extension(backend): Extension<WikiBackend>,
    Json(body): Json<WikiRefreshRequest>,
) -> Result<Json<WikiAuthResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.refresh(body).await?));
    }

    let mut store = store().lock().expect("wiki store lock");
    let user_id = store
        .refresh_tokens
        .get(&body.refresh_token)
        .cloned()
        .ok_or(shared::AppError::Unauthorized)?;
    let user = store
        .users
        .get(&user_id)
        .filter(|user| user.active)
        .cloned()
        .ok_or(shared::AppError::Unauthorized)?;
    Ok(Json(auth_response(&mut store, &user)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses((status = 204), (status = 401)),
    security(("bearer" = []))
)]
pub async fn logout(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<StatusCode, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        postgres.logout(&claims).await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let mut store = store().lock().expect("wiki store lock");
    store.tokens.retain(|_, user_id| user_id != &claims.user_id);
    store.audit(&claims.user_id, "auth.logout", "user", &claims.user_id);
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "users",
    responses((status = 200, body = WikiUserResponse), (status = 401)),
    security(("bearer" = []))
)]
pub async fn get_current_user(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<WikiUserResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_current_user(&claims).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let user = store
        .users
        .get(&claims.user_id)
        .cloned()
        .ok_or(shared::AppError::Unauthorized)?;
    Ok(Json(user))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses((status = 200, body = WikiUserListResponse), (status = 403)),
    security(("bearer" = []))
)]
pub async fn list_users(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<WikiUserListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_users(&claims).await?));
    }

    let store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    Ok(Json(WikiUserListResponse {
        users: store.users.values().cloned().collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = WikiCreateUserRequest,
    responses((status = 201, body = WikiUserResponse), (status = 403)),
    security(("bearer" = []))
)]
pub async fn create_user(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<WikiCreateUserRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        let response = postgres.create_user(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let mut store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    if body.email.trim().is_empty()
        || body.username.trim().is_empty()
        || body.password.trim().is_empty()
        || body.display_name.trim().is_empty()
    {
        return Err(shared::AppError::invalid_input(
            "email, username, password and display_name are required",
        ));
    }
    if store.users.values().any(|user| user.email == body.email) {
        return Err(shared::AppError::conflict("email already exists"));
    }
    let user_id = new_id();
    let user = WikiUserResponse {
        id: user_id.clone(),
        email: body.email,
        username: body.username,
        display_name: body.display_name,
        role: body.role.clone(),
        is_system_admin: body.role == "admin",
        active: true,
    };
    store.passwords.insert(user_id.clone(), body.password);
    store.users.insert(user_id.clone(), user.clone());
    store.audit(&claims.user_id, "user.create", "user", &user_id);
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}",
    tag = "users",
    params(("user_id" = String, Path)),
    request_body = WikiUpdateUserRequest,
    responses((status = 200, body = WikiUserResponse), (status = 403), (status = 404)),
    security(("bearer" = []))
)]
pub async fn update_user(
    Path(user_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<WikiUpdateUserRequest>,
) -> Result<Json<WikiUserResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.update_user(&claims, &user_id, body).await?));
    }

    let mut store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    if let Some(email) = &body.email {
        if store
            .users
            .values()
            .any(|user| user.id != user_id && user.email == *email)
        {
            return Err(shared::AppError::conflict("email already exists"));
        }
    }
    let user = store
        .users
        .get_mut(&user_id)
        .ok_or_else(|| shared::AppError::not_found("user", &user_id))?;
    if let Some(email) = body.email {
        user.email = email;
    }
    if let Some(username) = body.username {
        user.username = username;
    }
    if let Some(display_name) = body.display_name {
        user.display_name = display_name;
    }
    if let Some(role) = body.role {
        user.role = role;
    }
    if let Some(is_system_admin) = body.is_system_admin {
        user.is_system_admin = is_system_admin;
    }
    if let Some(active) = body.active {
        user.active = active;
    }
    let response = user.clone();
    store.audit(&claims.user_id, "user.update", "user", &user_id);
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces",
    tag = "spaces",
    responses((status = 200, body = SpaceListResponse)),
    security(("bearer" = []))
)]
pub async fn list_spaces(
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<SpaceListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_spaces().await?));
    }

    let store = store().lock().expect("wiki store lock");
    Ok(Json(SpaceListResponse {
        spaces: store.spaces.values().cloned().collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces",
    tag = "spaces",
    request_body = CreateSpaceRequest,
    responses((status = 201, body = SpaceResponse)),
    security(("bearer" = []))
)]
pub async fn create_space(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<CreateSpaceRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        let response = postgres.create_space(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let key = body.key.trim().to_ascii_uppercase();
    if key.is_empty() {
        return Err(shared::AppError::invalid_input("space key is required"));
    }
    let mut store = store().lock().expect("wiki store lock");
    if store.spaces.contains_key(&key) {
        return Err(shared::AppError::conflict("space already exists"));
    }
    let now = now_iso();
    let space = SpaceResponse {
        id: new_id(),
        key: key.clone(),
        name: body.name,
        description: body.description,
        owner_id: claims.user_id.clone(),
        status: "active".to_string(),
        document_count: 0,
        member_count: 1,
        created_at: now.clone(),
        updated_at: now,
    };
    store.spaces.insert(key.clone(), space.clone());
    store.members.insert(
        key.clone(),
        BTreeMap::from([(claims.user_id.clone(), "admin".to_string())]),
    );
    store.audit(&claims.user_id, "space.create", "space", &key);
    Ok((StatusCode::CREATED, Json(space)))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}",
    tag = "spaces",
    params(("space_key" = String, Path)),
    responses((status = 200, body = SpaceResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn get_space(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<SpaceResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_space_by_key(&space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let space = store
        .spaces
        .get(&space_key.to_ascii_uppercase())
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    Ok(Json(space))
}

#[utoipa::path(
    put,
    path = "/api/v1/spaces/{space_key}",
    tag = "spaces",
    params(("space_key" = String, Path)),
    request_body = UpdateSpaceRequest,
    responses((status = 200, body = SpaceResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn update_space(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<UpdateSpaceRequest>,
) -> Result<Json<SpaceResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres.update_space(&claims, &space_key, body).await?,
        ));
    }

    let key = space_key.to_ascii_uppercase();
    let mut store = store().lock().expect("wiki store lock");
    let space = store
        .spaces
        .get_mut(&key)
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    if let Some(name) = body.name {
        space.name = name;
    }
    if body.description.is_some() {
        space.description = body.description;
    }
    space.updated_at = now_iso();
    let response = space.clone();
    store.audit(&claims.user_id, "space.update", "space", &key);
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_key}/archive",
    tag = "spaces",
    params(("space_key" = String, Path)),
    responses((status = 200, body = SpaceResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn archive_space(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<SpaceResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.archive_space(&claims, &space_key).await?));
    }

    let key = space_key.to_ascii_uppercase();
    let mut store = store().lock().expect("wiki store lock");
    let space = store
        .spaces
        .get_mut(&key)
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    space.status = "archived".to_string();
    space.updated_at = now_iso();
    let response = space.clone();
    store.audit(&claims.user_id, "space.archive", "space", &key);
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/members",
    tag = "spaces",
    params(("space_key" = String, Path)),
    responses((status = 200, body = SpaceMemberListResponse)),
    security(("bearer" = []))
)]
pub async fn list_space_members(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<SpaceMemberListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_space_members(&space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let members = store
        .members
        .get(&space_key.to_ascii_uppercase())
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?
        .iter()
        .filter_map(|(user_id, role)| {
            store.users.get(user_id).map(|user| SpaceMemberResponse {
                user_id: user_id.clone(),
                email: user.email.clone(),
                display_name: user.display_name.clone(),
                role: role.clone(),
                joined_at: now_iso(),
            })
        })
        .collect();
    Ok(Json(SpaceMemberListResponse { members }))
}

#[utoipa::path(
    put,
    path = "/api/v1/spaces/{space_key}/members/{user_id}",
    tag = "spaces",
    params(("space_key" = String, Path), ("user_id" = String, Path)),
    request_body = UpsertSpaceMemberRequest,
    responses((status = 200, body = SpaceMemberResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn upsert_space_member(
    Path((space_key, user_id)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<UpsertSpaceMemberRequest>,
) -> Result<Json<SpaceMemberResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .upsert_space_member(&claims, &space_key, &user_id, body)
                .await?,
        ));
    }

    let key = space_key.to_ascii_uppercase();
    let mut store = store().lock().expect("wiki store lock");
    let user = store
        .users
        .get(&user_id)
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("user", &user_id))?;
    let members = store
        .members
        .get_mut(&key)
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    members.insert(user_id.clone(), body.role.clone());
    let member_count = members.len();
    if let Some(space) = store.spaces.get_mut(&key) {
        space.member_count = member_count;
        space.updated_at = now_iso();
    }
    store.audit(&claims.user_id, "space.member_upsert", "space", &key);
    Ok(Json(SpaceMemberResponse {
        user_id,
        email: user.email,
        display_name: user.display_name,
        role: body.role,
        joined_at: now_iso(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/spaces/{space_key}/members/{user_id}",
    tag = "spaces",
    params(("space_key" = String, Path), ("user_id" = String, Path)),
    responses((status = 204), (status = 404)),
    security(("bearer" = []))
)]
pub async fn delete_space_member(
    Path((space_key, user_id)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<StatusCode, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        postgres
            .delete_space_member(&claims, &space_key, &user_id)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let key = space_key.to_ascii_uppercase();
    let mut store = store().lock().expect("wiki store lock");
    let members = store
        .members
        .get_mut(&key)
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    members.remove(&user_id);
    let member_count = members.len();
    if let Some(space) = store.spaces.get_mut(&key) {
        space.member_count = member_count;
        space.updated_at = now_iso();
    }
    store.audit(&claims.user_id, "space.member_delete", "space", &key);
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/tree",
    tag = "documents",
    params(("space_key" = String, Path)),
    responses((status = 200, body = SpaceTreeResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn get_space_tree(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<SpaceTreeResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_space_tree(&space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let documents = build_tree(&store, &key, None);
    Ok(Json(SpaceTreeResponse {
        space_key: key,
        documents,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_key}/documents",
    tag = "documents",
    params(("space_key" = String, Path)),
    request_body = CreateDocumentRequest,
    responses((status = 201, body = DocumentResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn create_document(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<CreateDocumentRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        let response = postgres.create_document(&claims, &space_key, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let key = space_key.to_ascii_uppercase();
    let mut store = store().lock().expect("wiki store lock");
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let parent_id = match body.parent_id {
        Some(parent_id) => {
            let resolved_parent_id = resolve_document_id(&store, &parent_id)?;
            let parent = store
                .documents
                .get(&resolved_parent_id)
                .ok_or_else(|| shared::AppError::not_found("document", &parent_id))?;
            if parent.space_key != key {
                return Err(shared::AppError::invalid_input(
                    "parent document belongs to another space",
                ));
            }
            Some(resolved_parent_id)
        }
        None => None,
    };
    let id = new_id();
    let mut slug = body.slug.unwrap_or_else(|| slugify(&body.title));
    slug = slugify(&slug);
    if slug.is_empty() {
        slug = format!("document-{}", &id[..8]);
    }
    let parent_for_unique = parent_id.clone();
    if store.documents.values().any(|document| {
        document.space_key == key
            && document.parent_id == parent_for_unique
            && document.slug == slug
    }) {
        return Err(shared::AppError::conflict("document slug already exists"));
    }
    let now = now_iso();
    let mut task_keys = BTreeSet::new();
    if let Some(task_key) = body.task_key {
        task_keys.insert(task_key);
    }
    let mut phase_keys = BTreeSet::new();
    if let Some(phase_key) = body.phase_key {
        phase_keys.insert(phase_key);
    }
    let document = DocumentRecord {
        id: id.clone(),
        space_key: key.clone(),
        parent_id,
        slug,
        title: body.title,
        document_type: body.document_type,
        status: "draft".to_string(),
        draft_markdown: body.content_markdown,
        current_revision_id: None,
        task_keys,
        phase_keys,
        created_by: claims.user_id.clone(),
        updated_by: claims.user_id.clone(),
        created_at: now.clone(),
        updated_at: now,
    };
    store.documents.insert(id.clone(), document);
    let document_count = store
        .documents
        .values()
        .filter(|document| document.space_key == key)
        .count();
    if let Some(space) = store.spaces.get_mut(&key) {
        space.document_count = document_count;
        space.updated_at = now_iso();
    }
    store.audit(&claims.user_id, "document.create", "document", &id);
    let response = document_response(&store, &id)?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}",
    tag = "documents",
    params(("document_id" = String, Path)),
    responses((status = 200, body = DocumentResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn get_document(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_document(&document_id).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    Ok(Json(document_response(&store, &id)?))
}

#[utoipa::path(
    put,
    path = "/api/v1/documents/{document_id}/draft",
    tag = "documents",
    params(("document_id" = String, Path)),
    request_body = UpdateDocumentDraftRequest,
    responses((status = 200, body = DocumentResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn update_document_draft(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<UpdateDocumentDraftRequest>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .update_document_draft(&claims, &document_id, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    let document = store
        .documents
        .get_mut(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    if let Some(title) = body.title {
        document.title = title;
    }
    document.draft_markdown = body.content_markdown;
    document.status = "draft".to_string();
    document.updated_by = claims.user_id.clone();
    document.updated_at = now_iso();
    store.audit(&claims.user_id, "document.draft_update", "document", &id);
    Ok(Json(document_response(&store, &id)?))
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/publish",
    tag = "documents",
    params(("document_id" = String, Path)),
    request_body = PublishDocumentRequest,
    responses((status = 200, body = DocumentRevisionResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn publish_document(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<PublishDocumentRequest>,
) -> Result<Json<DocumentRevisionResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .publish_document(&claims, &document_id, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    let version = store
        .revisions
        .get(&id)
        .map_or(1, |items| items.len() as u32 + 1);
    let revision_id = new_id();
    let document = store
        .documents
        .get_mut(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    let revision = DocumentRevisionResponse {
        id: revision_id.clone(),
        document_id: id.clone(),
        version,
        title: document.title.clone(),
        body_markdown: document.draft_markdown.clone(),
        summary: body.summary,
        author_id: claims.user_id.clone(),
        published_at: now_iso(),
    };
    document.current_revision_id = Some(revision_id);
    document.status = "published".to_string();
    document.updated_by = claims.user_id.clone();
    document.updated_at = now_iso();
    store
        .revisions
        .entry(id.clone())
        .or_default()
        .push(revision.clone());
    store.audit(&claims.user_id, "document.publish", "document", &id);
    Ok(Json(revision))
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/archive",
    tag = "documents",
    params(("document_id" = String, Path)),
    responses((status = 200, body = DocumentResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn archive_document(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres.archive_document(&claims, &document_id).await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    let document = store
        .documents
        .get_mut(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    document.status = "archived".to_string();
    document.updated_by = claims.user_id.clone();
    document.updated_at = now_iso();
    store.audit(&claims.user_id, "document.archive", "document", &id);
    Ok(Json(document_response(&store, &id)?))
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{document_id}/move",
    tag = "documents",
    params(("document_id" = String, Path)),
    request_body = MoveDocumentRequest,
    responses((status = 200, body = DocumentResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn move_document(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<MoveDocumentRequest>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres.move_document(&claims, &document_id, body).await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    let document_space = store
        .documents
        .get(&id)
        .map(|document| document.space_key.clone())
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    let parent_id = match body.parent_id {
        Some(parent_id) => {
            let resolved_parent_id = resolve_document_id(&store, &parent_id)?;
            if resolved_parent_id == id {
                return Err(shared::AppError::invalid_input(
                    "document cannot be moved under itself",
                ));
            }
            let parent = store
                .documents
                .get(&resolved_parent_id)
                .ok_or_else(|| shared::AppError::not_found("document", &parent_id))?;
            if parent.space_key != document_space {
                return Err(shared::AppError::invalid_input(
                    "parent document belongs to another space",
                ));
            }
            Some(resolved_parent_id)
        }
        None => None,
    };
    let document = store
        .documents
        .get_mut(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    document.parent_id = parent_id;
    document.updated_by = claims.user_id.clone();
    document.updated_at = now_iso();
    store.audit(&claims.user_id, "document.move", "document", &id);
    Ok(Json(document_response(&store, &id)?))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/revisions",
    tag = "documents",
    params(("document_id" = String, Path)),
    responses((status = 200, body = DocumentRevisionListResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn list_document_revisions(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<DocumentRevisionListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_document_revisions(&document_id).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    Ok(Json(DocumentRevisionListResponse {
        revisions: store.revisions.get(&id).cloned().unwrap_or_default(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/revisions/{revision_id}",
    tag = "documents",
    params(("document_id" = String, Path), ("revision_id" = String, Path)),
    responses((status = 200, body = DocumentRevisionResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn get_document_revision(
    Path((document_id, revision_id)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<DocumentRevisionResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .get_document_revision(&document_id, &revision_id)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    let revision = store
        .revisions
        .get(&id)
        .and_then(|items| items.iter().find(|revision| revision.id == revision_id))
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("revision", &revision_id))?;
    Ok(Json(revision))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/tasks",
    tag = "tasks",
    params(("space_key" = String, Path)),
    responses((status = 200, body = TaskPageListResponse)),
    security(("bearer" = []))
)]
pub async fn list_tasks(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<TaskPageListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_tasks(&space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let mut task_keys = BTreeSet::new();
    for document in store
        .documents
        .values()
        .filter(|document| document.space_key == key)
    {
        task_keys.extend(document.task_keys.iter().cloned());
    }
    for item in store.evidence.values().filter(|item| item.space_key == key) {
        if let Some(task_key) = &item.task_key {
            task_keys.insert(task_key.clone());
        }
    }
    let tasks = task_keys
        .into_iter()
        .map(|task_key| task_page(&store, &key, &task_key))
        .collect();
    Ok(Json(TaskPageListResponse { tasks }))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/tasks/{task_key}",
    tag = "tasks",
    params(("space_key" = String, Path), ("task_key" = String, Path)),
    responses((status = 200, body = TaskPageResponse)),
    security(("bearer" = []))
)]
pub async fn get_task(
    Path((space_key, task_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<TaskPageResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_task(&space_key, &task_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    Ok(Json(task_page(&store, &key, &task_key)))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_key}/tasks/{task_key}/links/documents",
    tag = "tasks",
    params(("space_key" = String, Path), ("task_key" = String, Path)),
    request_body = LinkDocumentRequest,
    responses((status = 200, body = TaskPageResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn link_task_document(
    Path((space_key, task_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<LinkDocumentRequest>,
) -> Result<Json<TaskPageResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .link_task_document(&claims, &space_key, &task_key, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let document_id = resolve_document_id(&store, &body.document_id)?;
    let document = store
        .documents
        .get_mut(&document_id)
        .ok_or_else(|| shared::AppError::not_found("document", &body.document_id))?;
    if document.space_key != key {
        return Err(shared::AppError::invalid_input(
            "document belongs to another space",
        ));
    }
    document.task_keys.insert(task_key.clone());
    document.updated_at = now_iso();
    store.audit(&claims.user_id, "task.link_document", "task", &task_key);
    Ok(Json(task_page(&store, &key, &task_key)))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/tasks/{task_key}/documents",
    tag = "tasks",
    params(("space_key" = String, Path), ("task_key" = String, Path)),
    responses((status = 200, body = DocumentListResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn list_task_documents(
    Path((space_key, task_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<DocumentListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres.list_task_documents(&space_key, &task_key).await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let documents = task_page(&store, &key, &task_key)
        .documents
        .into_iter()
        .filter_map(|summary| document_response(&store, &summary.id).ok())
        .collect();
    Ok(Json(DocumentListResponse { documents }))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/tasks/{task_key}/evidence",
    tag = "tasks",
    params(("space_key" = String, Path), ("task_key" = String, Path)),
    responses((status = 200, body = EvidenceListResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn list_task_evidence(
    Path((space_key, task_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<EvidenceListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres.list_task_evidence(&space_key, &task_key).await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    Ok(Json(EvidenceListResponse {
        evidence: evidence_for_task(&store, &key, &task_key),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/phases",
    tag = "phases",
    params(("space_key" = String, Path)),
    responses((status = 200, body = PhasePageListResponse)),
    security(("bearer" = []))
)]
pub async fn list_phases(
    Path(space_key): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<PhasePageListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_phases(&space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let mut phase_keys = BTreeSet::new();
    for document in store
        .documents
        .values()
        .filter(|document| document.space_key == key)
    {
        phase_keys.extend(document.phase_keys.iter().cloned());
    }
    for item in store.evidence.values().filter(|item| item.space_key == key) {
        if let Some(phase_key) = &item.phase_key {
            phase_keys.insert(phase_key.clone());
        }
    }
    let phases = phase_keys
        .into_iter()
        .map(|phase_key| phase_page(&store, &key, &phase_key))
        .collect();
    Ok(Json(PhasePageListResponse { phases }))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/phases/{phase_key}",
    tag = "phases",
    params(("space_key" = String, Path), ("phase_key" = String, Path)),
    responses((status = 200, body = PhasePageResponse)),
    security(("bearer" = []))
)]
pub async fn get_phase(
    Path((space_key, phase_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<PhasePageResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_phase(&space_key, &phase_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    Ok(Json(phase_page(&store, &key, &phase_key)))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_key}/phases/{phase_key}/links/documents",
    tag = "phases",
    params(("space_key" = String, Path), ("phase_key" = String, Path)),
    request_body = LinkDocumentRequest,
    responses((status = 200, body = PhasePageResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn link_phase_document(
    Path((space_key, phase_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<LinkDocumentRequest>,
) -> Result<Json<PhasePageResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .link_phase_document(&claims, &space_key, &phase_key, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let document_id = resolve_document_id(&store, &body.document_id)?;
    let document = store
        .documents
        .get_mut(&document_id)
        .ok_or_else(|| shared::AppError::not_found("document", &body.document_id))?;
    if document.space_key != key {
        return Err(shared::AppError::invalid_input(
            "document belongs to another space",
        ));
    }
    document.phase_keys.insert(phase_key.clone());
    document.updated_at = now_iso();
    store.audit(&claims.user_id, "phase.link_document", "phase", &phase_key);
    Ok(Json(phase_page(&store, &key, &phase_key)))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/phases/{phase_key}/documents",
    tag = "phases",
    params(("space_key" = String, Path), ("phase_key" = String, Path)),
    responses((status = 200, body = DocumentListResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn list_phase_documents(
    Path((space_key, phase_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<DocumentListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres
                .list_phase_documents(&space_key, &phase_key)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    let documents = phase_page(&store, &key, &phase_key)
        .documents
        .into_iter()
        .filter_map(|summary| document_response(&store, &summary.id).ok())
        .collect();
    Ok(Json(DocumentListResponse { documents }))
}

#[utoipa::path(
    get,
    path = "/api/v1/spaces/{space_key}/phases/{phase_key}/evidence",
    tag = "phases",
    params(("space_key" = String, Path), ("phase_key" = String, Path)),
    responses((status = 200, body = EvidenceListResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn list_phase_evidence(
    Path((space_key, phase_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<EvidenceListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(
            postgres.list_phase_evidence(&space_key, &phase_key).await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = space_key.to_ascii_uppercase();
    if !store.spaces.contains_key(&key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    Ok(Json(EvidenceListResponse {
        evidence: evidence_for_phase(&store, &key, &phase_key),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/evidence",
    tag = "evidence",
    request_body = CreateEvidenceRequest,
    responses((status = 201, body = EvidenceResponse)),
    security(("bearer" = []))
)]
pub async fn create_evidence(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<CreateEvidenceRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        let response = postgres.create_evidence(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    if body.url.is_none() && body.attachment_id.is_none() {
        return Err(shared::AppError::invalid_input(
            "url or attachment_id is required",
        ));
    }
    let mut store = store().lock().expect("wiki store lock");
    let CreateEvidenceRequest {
        space,
        document_id,
        task_key,
        phase_key,
        title,
        evidence_type,
        url,
        attachment_id,
        checksum,
    } = body;
    match evidence_type.as_str() {
        "external_url" if url.is_none() || attachment_id.is_some() => {
            return Err(shared::AppError::invalid_input(
                "external_url evidence requires url only",
            ));
        }
        "uploaded_file" if attachment_id.is_none() || url.is_some() => {
            return Err(shared::AppError::invalid_input(
                "uploaded_file evidence requires attachment_id only",
            ));
        }
        "external_url" | "uploaded_file" => {}
        _ => {
            return Err(shared::AppError::invalid_input(
                "evidence_type must be external_url or uploaded_file",
            ));
        }
    }
    if let Some(attachment_id) = &attachment_id {
        if !store.attachments.contains_key(attachment_id) {
            return Err(shared::AppError::not_found("attachment", attachment_id));
        }
    }
    let document_id = match document_id {
        Some(document_id) => Some(resolve_document_id(&store, &document_id)?),
        None => None,
    };
    let document_space = document_id
        .as_ref()
        .and_then(|id| store.documents.get(id))
        .map(|document| document.space_key.clone());
    let space_key = space
        .or(document_space.clone())
        .unwrap_or_else(|| "SDLC".to_string())
        .to_ascii_uppercase();
    if !store.spaces.contains_key(&space_key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    if document_space.is_some_and(|document_space| document_space != space_key) {
        return Err(shared::AppError::invalid_input(
            "document belongs to another space",
        ));
    }
    let id = new_id();
    let evidence = EvidenceResponse {
        id: id.clone(),
        space_key,
        document_id,
        task_key,
        phase_key,
        title,
        evidence_type,
        url,
        attachment_id,
        checksum,
        created_by: claims.user_id.clone(),
        created_at: now_iso(),
    };
    store.evidence.insert(id.clone(), evidence.clone());
    store.audit(&claims.user_id, "evidence.create", "evidence", &id);
    Ok((StatusCode::CREATED, Json(evidence)))
}

#[utoipa::path(
    get,
    path = "/api/v1/evidence",
    tag = "evidence",
    params(EvidenceQuery),
    responses((status = 200, body = EvidenceListResponse)),
    security(("bearer" = []))
)]
pub async fn list_evidence(
    Extension(backend): Extension<WikiBackend>,
    Query(query): Query<EvidenceQuery>,
) -> Result<Json<EvidenceListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_evidence(query).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let mut items: Vec<_> = store
        .evidence
        .values()
        .filter(|item| {
            query
                .space
                .as_ref()
                .is_none_or(|space| item.space_key == space.to_ascii_uppercase())
        })
        .filter(|item| {
            query
                .document_id
                .as_ref()
                .is_none_or(|id| item.document_id.as_ref() == Some(id))
        })
        .filter(|item| {
            query
                .task_key
                .as_ref()
                .is_none_or(|key| item.task_key.as_ref() == Some(key))
        })
        .filter(|item| {
            query
                .phase_key
                .as_ref()
                .is_none_or(|key| item.phase_key.as_ref() == Some(key))
        })
        .cloned()
        .collect();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if let Some(limit) = query.limit {
        items.truncate(limit);
    }
    Ok(Json(EvidenceListResponse { evidence: items }))
}

#[utoipa::path(
    get,
    path = "/api/v1/evidence/{evidence_id}",
    tag = "evidence",
    params(("evidence_id" = String, Path)),
    responses((status = 200, body = EvidenceResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn get_evidence(
    Path(evidence_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<EvidenceResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_evidence(&evidence_id).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let evidence = store
        .evidence
        .get(&evidence_id)
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("evidence", &evidence_id))?;
    Ok(Json(evidence))
}

#[utoipa::path(
    post,
    path = "/api/v1/attachments",
    tag = "attachments",
    request_body(content = String, content_type = "multipart/form-data"),
    responses((status = 201, body = AttachmentResponse)),
    security(("bearer" = []))
)]
pub async fn upload_attachment(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, shared::AppError> {
    let mut file_name = "attachment.bin".to_string();
    let mut content_type = "application/octet-stream".to_string();
    let mut bytes = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| shared::AppError::invalid_input(err.to_string()))?
    {
        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }
        if let Some(kind) = field.content_type() {
            content_type = kind.to_string();
        }
        bytes = field
            .bytes()
            .await
            .map_err(|err| shared::AppError::invalid_input(err.to_string()))?
            .to_vec();
        break;
    }

    if bytes.is_empty() {
        return Err(shared::AppError::invalid_input("file is required"));
    }

    if let Some(postgres) = backend.postgres() {
        let response = postgres
            .upload_attachment(&claims, file_name, content_type, bytes)
            .await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let id = new_id();
    let metadata = AttachmentResponse {
        id: id.clone(),
        file_name,
        content_type,
        size_bytes: bytes.len(),
        checksum: checksum(&bytes),
        uploaded_by: claims.user_id.clone(),
        uploaded_at: now_iso(),
    };
    let mut store = store().lock().expect("wiki store lock");
    store.attachments.insert(
        id.clone(),
        AttachmentRecord {
            metadata: metadata.clone(),
            bytes,
        },
    );
    store.audit(&claims.user_id, "attachment.upload", "attachment", &id);
    Ok((StatusCode::CREATED, Json(metadata)))
}

#[utoipa::path(
    get,
    path = "/api/v1/attachments/{attachment_id}",
    tag = "attachments",
    params(("attachment_id" = String, Path)),
    responses((status = 200, body = AttachmentResponse), (status = 404)),
    security(("bearer" = []))
)]
pub async fn get_attachment(
    Path(attachment_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<AttachmentResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.get_attachment(&attachment_id).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let attachment = store
        .attachments
        .get(&attachment_id)
        .map(|record| record.metadata.clone())
        .ok_or_else(|| shared::AppError::not_found("attachment", &attachment_id))?;
    Ok(Json(attachment))
}

#[utoipa::path(
    get,
    path = "/api/v1/attachments/{attachment_id}/download",
    tag = "attachments",
    params(("attachment_id" = String, Path)),
    responses((status = 200, description = "Attachment bytes"), (status = 404)),
    security(("bearer" = []))
)]
pub async fn download_attachment(
    Path(attachment_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
) -> Result<Response, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return postgres.download_attachment(&attachment_id).await;
    }

    let store = store().lock().expect("wiki store lock");
    let attachment = store
        .attachments
        .get(&attachment_id)
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("attachment", &attachment_id))?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&attachment.metadata.content_type)
            .map_err(|err| shared::AppError::internal(err.to_string()))?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            safe_download_filename(&attachment.metadata.file_name)
        ))
        .map_err(|err| shared::AppError::internal(err.to_string()))?,
    );
    Ok((headers, attachment.bytes).into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/templates",
    tag = "templates",
    responses((status = 200, body = TemplateListResponse)),
    security(("bearer" = []))
)]
pub async fn list_templates(
    Extension(backend): Extension<WikiBackend>,
) -> Result<Json<TemplateListResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_templates().await?));
    }

    let store = store().lock().expect("wiki store lock");
    Ok(Json(TemplateListResponse {
        templates: store.templates.values().cloned().collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/templates",
    tag = "templates",
    request_body = CreateTemplateRequest,
    responses((status = 201, body = TemplateResponse)),
    security(("bearer" = []))
)]
pub async fn create_template(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<CreateTemplateRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        let response = postgres.create_template(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = slugify(&body.name);
    let template = TemplateResponse {
        id: id.clone(),
        name: body.name,
        document_type: body.document_type,
        body_markdown: body.body_markdown,
    };
    store.templates.insert(id.clone(), template.clone());
    store.audit(&claims.user_id, "template.create", "template", &id);
    Ok((StatusCode::CREATED, Json(template)))
}

#[utoipa::path(
    get,
    path = "/api/v1/audit-log",
    tag = "audit",
    responses((status = 200, body = AuditLogResponse), (status = 403)),
    security(("bearer" = []))
)]
pub async fn list_audit_log(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<AuditLogResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.list_audit_log(&claims).await?));
    }

    let store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    Ok(Json(AuditLogResponse {
        entries: store.audit.iter().rev().cloned().collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    tag = "search",
    params(SearchQuery),
    responses((status = 200, body = SearchResponse)),
    security(("bearer" = []))
)]
pub async fn search(
    Extension(backend): Extension<WikiBackend>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, shared::AppError> {
    if let Some(postgres) = backend.postgres() {
        return Ok(Json(postgres.search(query).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let needle = query.q.unwrap_or_default().to_lowercase();
    let include_archived = query.include_archived.unwrap_or(false);
    let mut results = Vec::new();

    for document in store.documents.values() {
        if !include_archived && document.status == "archived" {
            continue;
        }
        if query
            .space
            .as_ref()
            .is_some_and(|space| document.space_key != space.to_ascii_uppercase())
        {
            continue;
        }
        if query
            .document_type
            .as_ref()
            .is_some_and(|document_type| document.document_type != *document_type)
        {
            continue;
        }
        if query
            .task_key
            .as_ref()
            .is_some_and(|task_key| !document.task_keys.contains(task_key))
        {
            continue;
        }
        if query
            .phase_key
            .as_ref()
            .is_some_and(|phase_key| !document.phase_keys.contains(phase_key))
        {
            continue;
        }
        let haystack = format!("{} {}", document.title, document.draft_markdown).to_lowercase();
        if needle.is_empty() || haystack.contains(&needle) {
            results.push(SearchResultResponse {
                id: document.id.clone(),
                result_type: "document".to_string(),
                title: document.title.clone(),
                space_key: document.space_key.clone(),
                url: format!("/documents/{}", document.slug),
                snippet: snippet(&document.draft_markdown),
                updated_at: document.updated_at.clone(),
            });
        }
    }

    for item in store.evidence.values() {
        if query
            .space
            .as_ref()
            .is_some_and(|space| item.space_key != space.to_ascii_uppercase())
        {
            continue;
        }
        if query
            .task_key
            .as_ref()
            .is_some_and(|task_key| item.task_key.as_ref() != Some(task_key))
        {
            continue;
        }
        if query
            .phase_key
            .as_ref()
            .is_some_and(|phase_key| item.phase_key.as_ref() != Some(phase_key))
        {
            continue;
        }
        let haystack =
            format!("{} {}", item.title, item.url.clone().unwrap_or_default()).to_lowercase();
        if needle.is_empty() || haystack.contains(&needle) {
            results.push(SearchResultResponse {
                id: item.id.clone(),
                result_type: "evidence".to_string(),
                title: item.title.clone(),
                space_key: item.space_key.clone(),
                url: format!("/evidence?id={}", item.id),
                snippet: item
                    .url
                    .clone()
                    .unwrap_or_else(|| item.evidence_type.clone()),
                updated_at: item.created_at.clone(),
            });
        }
    }

    results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    if let Some(limit) = query.limit {
        results.truncate(limit);
    }
    Ok(Json(SearchResponse { results }))
}

fn auth_response(store: &mut WikiStore, user: &WikiUserResponse) -> WikiAuthResponse {
    let token = format!("wiki-token-{}", new_id());
    let refresh_token = format!("wiki-refresh-{}", new_id());
    store.tokens.insert(token.clone(), user.id.clone());
    store
        .refresh_tokens
        .insert(refresh_token.clone(), user.id.clone());
    WikiAuthResponse {
        access_token: token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user_id: user.id.clone(),
        email: user.email.clone(),
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        expires_in: 900,
    }
}

const SPACE_LIST_SQL: &str = r#"
    SELECT s.id, s.key, s.name, s.description, s.owner_id,
           CASE WHEN s.archived_at IS NULL THEN 'active' ELSE 'archived' END AS status,
           (
               SELECT COUNT(*)::bigint
               FROM documents d
               WHERE d.space_id = s.id AND d.archived_at IS NULL
           ) AS document_count,
           (
               SELECT COUNT(*)::bigint
               FROM space_members sm
               WHERE sm.space_id = s.id
           ) AS member_count,
           s.created_at, s.updated_at
    FROM spaces s
    ORDER BY s.key
"#;

const SPACE_ONE_SQL: &str = r#"
    SELECT s.id, s.key, s.name, s.description, s.owner_id,
           CASE WHEN s.archived_at IS NULL THEN 'active' ELSE 'archived' END AS status,
           (
               SELECT COUNT(*)::bigint
               FROM documents d
               WHERE d.space_id = s.id AND d.archived_at IS NULL
           ) AS document_count,
           (
               SELECT COUNT(*)::bigint
               FROM space_members sm
               WHERE sm.space_id = s.id
           ) AS member_count,
           s.created_at, s.updated_at
    FROM spaces s
    WHERE s.key = $1
"#;

const EVIDENCE_ONE_SQL: &str = r#"
    SELECT e.id, s.key AS space_key, e.document_id, td.task_key, pd.phase_key,
           e.title, e.evidence_type, e.url, e.attachment_id, e.checksum,
           e.created_by, e.created_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE e.id = $1
"#;

const EVIDENCE_LIST_SQL: &str = r#"
    SELECT e.id, s.key AS space_key, e.document_id, td.task_key, pd.phase_key,
           e.title, e.evidence_type, e.url, e.attachment_id, e.checksum,
           e.created_by, e.created_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE ($1::text IS NULL OR s.key = $1)
      AND ($2::uuid IS NULL OR e.document_id = $2)
      AND ($3::text IS NULL OR td.task_key = $3)
      AND ($4::text IS NULL OR pd.phase_key = $4)
    ORDER BY e.created_at DESC
    LIMIT $5
"#;

const EVIDENCE_TARGET_SQL: &str = r#"
    SELECT e.id, s.key AS space_key, e.document_id, td.task_key, pd.phase_key,
           e.title, e.evidence_type, e.url, e.attachment_id, e.checksum,
           e.created_by, e.created_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE ($1::uuid IS NULL OR e.task_dossier_id = $1)
      AND ($2::uuid IS NULL OR e.phase_dossier_id = $2)
    ORDER BY e.created_at DESC
"#;

const ATTACHMENT_ONE_SQL: &str = r#"
    SELECT id, file_name, content_type, size_bytes, checksum, uploaded_by, uploaded_at
    FROM attachments
    WHERE id = $1
"#;

const SEARCH_DOCUMENTS_SQL: &str = r#"
    WITH latest_revision AS (
        SELECT DISTINCT ON (document_id)
               document_id, content_markdown, content_text, published_at
        FROM document_revisions
        ORDER BY document_id, published_at DESC
    )
    SELECT d.id,
           'document' AS result_type,
           d.title,
           s.key AS space_key,
           '/documents/' || d.slug AS url,
           COALESCE(NULLIF(lr.content_text, ''), NULLIF(dd.content_markdown, ''), d.title) AS snippet,
           d.updated_at
    FROM documents d
    JOIN spaces s ON s.id = d.space_id
    LEFT JOIN document_drafts dd ON dd.document_id = d.id
    LEFT JOIN latest_revision lr ON lr.document_id = d.id
    WHERE (
        $1 = '%%'
        OR lower(d.title) LIKE $1
        OR lower(COALESCE(dd.content_markdown, '')) LIKE $1
        OR lower(COALESCE(lr.content_text, '')) LIKE $1
    )
      AND ($2::text IS NULL OR s.key = $2)
      AND ($3::text IS NULL OR EXISTS (
          SELECT 1
          FROM document_task_links dtl
          JOIN task_dossiers td ON td.id = dtl.task_dossier_id
          WHERE dtl.document_id = d.id AND td.task_key = $3
      ))
      AND ($4::text IS NULL OR EXISTS (
          SELECT 1
          FROM document_phase_links dpl
          JOIN phase_dossiers pd ON pd.id = dpl.phase_dossier_id
          WHERE dpl.document_id = d.id AND pd.phase_key = $4
      ))
      AND ($5::text IS NULL OR d.document_type = $5)
      AND ($6::boolean OR d.archived_at IS NULL)
    ORDER BY d.updated_at DESC
    LIMIT $7
"#;

const SEARCH_EVIDENCE_SQL: &str = r#"
    SELECT e.id,
           'evidence' AS result_type,
           e.title,
           s.key AS space_key,
           '/evidence?id=' || e.id::text AS url,
           COALESCE(e.url, e.evidence_type) AS snippet,
           e.created_at AS updated_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE (
        $1 = '%%'
        OR lower(e.title) LIKE $1
        OR lower(COALESCE(e.url, '')) LIKE $1
    )
      AND ($2::text IS NULL OR s.key = $2)
      AND ($3::text IS NULL OR td.task_key = $3)
      AND ($4::text IS NULL OR pd.phase_key = $4)
    ORDER BY e.created_at DESC
    LIMIT $5
"#;

fn user_response_from_row(row: &PgRow) -> WikiUserResponse {
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

fn space_response_from_row(row: &PgRow) -> SpaceResponse {
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

fn space_member_response_from_row(row: &PgRow) -> SpaceMemberResponse {
    SpaceMemberResponse {
        user_id: row.get::<Uuid, _>("user_id").to_string(),
        email: row.get("email"),
        display_name: row.get("display_name"),
        role: row.get("role"),
        joined_at: to_iso(row.get("joined_at")),
    }
}

fn revision_response_from_row(row: &PgRow) -> DocumentRevisionResponse {
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

fn document_summary_from_row(row: &PgRow) -> DocumentSummaryResponse {
    DocumentSummaryResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        slug: row.get("slug"),
        title: row.get("title"),
        document_type: row.get("document_type"),
        status: row.get("status"),
        updated_at: to_iso(row.get("updated_at")),
    }
}

fn evidence_response_from_row(row: &PgRow) -> EvidenceResponse {
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

fn attachment_response_from_row(row: &PgRow) -> AttachmentResponse {
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

fn template_response_from_row(row: &PgRow) -> TemplateResponse {
    TemplateResponse {
        id: row.get::<Uuid, _>("id").to_string(),
        name: row.get("name"),
        document_type: row.get("document_type"),
        body_markdown: row.get("content_markdown"),
    }
}

fn audit_entry_from_row(row: &PgRow) -> AuditEntryResponse {
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

fn search_result_from_row(row: &PgRow) -> SearchResultResponse {
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

fn build_db_tree(rows: &[PgRow], parent_id: Option<Uuid>) -> Vec<SpaceTreeNodeResponse> {
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

fn to_iso(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn count_to_usize(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

fn clamp_limit(limit: Option<usize>, max: i64) -> i64 {
    limit.unwrap_or(max as usize).clamp(1, max as usize) as i64
}

fn normalize_required(value: &str, field: &str) -> Result<String, shared::AppError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(shared::AppError::invalid_input(format!(
            "{field} is required"
        )));
    }
    Ok(value.to_string())
}

fn normalize_space_key(value: &str) -> Result<String, shared::AppError> {
    Ok(domain::wiki::SpaceKey::parse(value)?.to_string())
}

fn normalize_slug(value: &str) -> Result<String, shared::AppError> {
    Ok(domain::wiki::DocumentSlug::parse(value)?.to_string())
}

fn normalize_task_key(value: &str) -> Result<String, shared::AppError> {
    Ok(domain::wiki::TaskKey::parse(value)?.to_string())
}

fn normalize_phase_key(value: &str) -> Result<String, shared::AppError> {
    Ok(domain::wiki::PhaseKey::parse(value)?.to_string())
}

fn normalize_document_type(
    value: &str,
    allow_page: bool,
) -> Result<&'static str, shared::AppError> {
    match value.trim() {
        "page" if allow_page => Ok("page"),
        "requirements" => Ok("requirements"),
        "research_note" => Ok("research_note"),
        "implementation_note" => Ok("implementation_note"),
        "test_plan" => Ok("test_plan"),
        "release_note" => Ok("release_note"),
        _ => Err(shared::AppError::invalid_input("unsupported document type")),
    }
}

fn normalize_evidence_type(value: &str) -> Result<&'static str, shared::AppError> {
    match value.trim() {
        "external_url" => Ok("external_url"),
        "uploaded_file" => Ok("uploaded_file"),
        _ => Err(shared::AppError::invalid_input(
            "evidence_type must be external_url or uploaded_file",
        )),
    }
}

fn normalize_space_role(value: &str) -> Result<&'static str, shared::AppError> {
    match value.trim() {
        "admin" => Ok("admin"),
        "editor" => Ok("editor"),
        "viewer" => Ok("viewer"),
        _ => Err(shared::AppError::invalid_input(
            "space member role must be admin, editor or viewer",
        )),
    }
}

fn global_role_from_request(value: &str) -> Result<&'static str, shared::AppError> {
    match value.trim() {
        "admin" => Ok("admin"),
        "user" | "editor" | "viewer" => Ok("user"),
        _ => Err(shared::AppError::invalid_input(
            "user role must be admin, user, editor or viewer",
        )),
    }
}

fn parse_uuid(value: &str, entity: &str) -> Result<Uuid, shared::AppError> {
    Uuid::parse_str(value).map_err(|_| shared::AppError::not_found(entity, value))
}

fn default_username(email: &str) -> String {
    let local = email.split('@').next().unwrap_or("admin");
    let username = slugify(local);
    if username.is_empty() {
        "admin".to_string()
    } else {
        username
    }
}

fn markdown_to_text(markdown: &str) -> String {
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

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn hash_password(password: &str) -> Result<String, shared::AppError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(shared::AppError::internal)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, shared::AppError> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHash, PasswordVerifier},
    };
    let parsed = PasswordHash::new(hash).map_err(shared::AppError::internal)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn create_token(
    config: &shared::AuthConfig,
    user_id: Uuid,
    session_id: Uuid,
    token_type: &str,
    ttl: Duration,
) -> Result<String, shared::AppError> {
    let exp = Utc::now() + ttl;
    let claims = TokenClaims {
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
    .map_err(shared::AppError::internal)
}

fn decode_token(
    config: &shared::AuthConfig,
    token: &str,
    expected_type: &str,
) -> Result<TokenClaims, shared::AppError> {
    let claims = jsonwebtoken::decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| shared::AppError::Unauthorized)?
    .claims;
    if claims.typ == expected_type {
        Ok(claims)
    } else {
        Err(shared::AppError::Unauthorized)
    }
}

fn document_response(store: &WikiStore, id: &str) -> Result<DocumentResponse, shared::AppError> {
    let document = store
        .documents
        .get(id)
        .ok_or_else(|| shared::AppError::not_found("document", id))?;
    let current_revision = document
        .current_revision_id
        .as_ref()
        .and_then(|revision_id| {
            store
                .revisions
                .get(id)
                .and_then(|items| items.iter().find(|revision| &revision.id == revision_id))
                .cloned()
        });
    Ok(DocumentResponse {
        id: document.id.clone(),
        space_key: document.space_key.clone(),
        parent_id: document.parent_id.clone(),
        slug: document.slug.clone(),
        title: document.title.clone(),
        document_type: document.document_type.clone(),
        status: document.status.clone(),
        body_markdown: current_revision
            .as_ref()
            .map(|revision| revision.body_markdown.clone())
            .unwrap_or_default(),
        draft_markdown: document.draft_markdown.clone(),
        current_revision,
        task_keys: document.task_keys.iter().cloned().collect(),
        phase_keys: document.phase_keys.iter().cloned().collect(),
        evidence: store
            .evidence
            .values()
            .filter(|item| item.document_id.as_ref() == Some(&document.id))
            .cloned()
            .collect(),
        created_by: document.created_by.clone(),
        updated_by: document.updated_by.clone(),
        created_at: document.created_at.clone(),
        updated_at: document.updated_at.clone(),
    })
}

fn document_summary(document: &DocumentRecord) -> DocumentSummaryResponse {
    DocumentSummaryResponse {
        id: document.id.clone(),
        slug: document.slug.clone(),
        title: document.title.clone(),
        document_type: document.document_type.clone(),
        status: document.status.clone(),
        updated_at: document.updated_at.clone(),
    }
}

fn build_tree(
    store: &WikiStore,
    space_key: &str,
    parent_id: Option<&str>,
) -> Vec<SpaceTreeNodeResponse> {
    store
        .documents
        .values()
        .filter(|document| {
            document.space_key == space_key
                && document.status != "archived"
                && document.parent_id.as_deref() == parent_id
        })
        .map(|document| SpaceTreeNodeResponse {
            id: document.id.clone(),
            slug: document.slug.clone(),
            title: document.title.clone(),
            document_type: document.document_type.clone(),
            status: document.status.clone(),
            children: build_tree(store, space_key, Some(&document.id)),
        })
        .collect()
}

fn task_page(store: &WikiStore, space_key: &str, task_key: &str) -> TaskPageResponse {
    let documents: Vec<_> = store
        .documents
        .values()
        .filter(|document| document.space_key == space_key && document.task_keys.contains(task_key))
        .map(document_summary)
        .collect();
    let evidence = evidence_for_task(store, space_key, task_key);
    TaskPageResponse {
        space_key: space_key.to_string(),
        task_key: task_key.to_string(),
        title: documents.first().map(|document| document.title.clone()),
        document_count: documents.len(),
        evidence_count: evidence.len(),
        documents,
        evidence,
    }
}

fn phase_page(store: &WikiStore, space_key: &str, phase_key: &str) -> PhasePageResponse {
    let documents: Vec<_> = store
        .documents
        .values()
        .filter(|document| {
            document.space_key == space_key && document.phase_keys.contains(phase_key)
        })
        .map(document_summary)
        .collect();
    let evidence = evidence_for_phase(store, space_key, phase_key);
    PhasePageResponse {
        space_key: space_key.to_string(),
        phase_key: phase_key.to_string(),
        title: Some(phase_key.to_string()),
        document_count: documents.len(),
        evidence_count: evidence.len(),
        documents,
        evidence,
    }
}

fn evidence_for_task(store: &WikiStore, space_key: &str, task_key: &str) -> Vec<EvidenceResponse> {
    store
        .evidence
        .values()
        .filter(|item| item.space_key == space_key && item.task_key.as_deref() == Some(task_key))
        .cloned()
        .collect()
}

fn evidence_for_phase(
    store: &WikiStore,
    space_key: &str,
    phase_key: &str,
) -> Vec<EvidenceResponse> {
    store
        .evidence
        .values()
        .filter(|item| item.space_key == space_key && item.phase_key.as_deref() == Some(phase_key))
        .cloned()
        .collect()
}

fn resolve_document_id(store: &WikiStore, document_id: &str) -> Result<String, shared::AppError> {
    if store.documents.contains_key(document_id) {
        return Ok(document_id.to_string());
    }
    store
        .documents
        .values()
        .find(|document| document.slug == document_id)
        .map(|document| document.id.clone())
        .ok_or_else(|| shared::AppError::not_found("document", document_id))
}

fn new_id() -> String {
    Uuid::now_v7().to_string()
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn slugify(value: &str) -> String {
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

fn snippet(markdown: &str) -> String {
    let normalized = markdown
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    normalized.chars().take(180).collect()
}

fn checksum(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn safe_download_filename(file_name: &str) -> String {
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
    if sanitized.is_empty() {
        "attachment.bin".to_string()
    } else {
        sanitized
    }
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
