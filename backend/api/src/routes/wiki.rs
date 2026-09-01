use app::wiki::{
    WikiSpaceAccess, checksum, normalize_document_type, normalize_evidence_type,
    normalize_phase_key, normalize_required, normalize_space_key, normalize_space_role,
    normalize_task_key, safe_download_filename, slugify, snippet, space_role_allows,
};
use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, OnceLock},
};
use uuid::Uuid;

pub use shared::wiki_contract::*;

#[derive(Clone)]
pub struct WikiBackend {
    persistent: Option<Arc<dyn WikiBackendPort>>,
    settings: WikiSettingsSnapshot,
}

impl WikiBackend {
    pub fn memory() -> Self {
        Self::memory_with_registration(true)
    }

    pub fn memory_from_config(config: &shared::AppConfig) -> Self {
        Self {
            persistent: None,
            settings: WikiSettingsSnapshot::from_config(config),
        }
    }

    fn memory_with_registration(registration_enabled: bool) -> Self {
        Self {
            persistent: None,
            settings: WikiSettingsSnapshot::from_values(
                registration_enabled,
                shared::StorageConfig::default().max_upload_bytes,
            ),
        }
    }

    pub fn persistent(backend: Arc<dyn WikiBackendPort>, settings: WikiSettingsSnapshot) -> Self {
        Self {
            persistent: Some(backend),
            settings,
        }
    }

    fn persistent_backend(&self) -> Option<&dyn WikiBackendPort> {
        self.persistent.as_deref()
    }

    fn registration_enabled(&self) -> bool {
        self.settings.registration_enabled
    }

    fn settings_snapshot(&self) -> WikiSettingsSnapshot {
        self.settings.clone()
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

impl WikiBackend {
    async fn authenticate_access_token(&self, token: &str) -> Result<WikiClaims, shared::AppError> {
        if let Some(persistent) = self.persistent_backend() {
            return persistent.authenticate_access_token(token).await;
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

fn is_system_admin(store: &WikiStore, user_id: &str) -> Result<bool, shared::AppError> {
    let user = store
        .users
        .get(user_id)
        .filter(|user| user.active)
        .ok_or(shared::AppError::Unauthorized)?;
    Ok(user.is_system_admin)
}

fn ensure_space_access(
    store: &WikiStore,
    space_key: &str,
    user_id: &str,
    required: WikiSpaceAccess,
) -> Result<(), shared::AppError> {
    if !store.spaces.contains_key(space_key) {
        return Err(shared::AppError::not_found("space", space_key));
    }
    if is_system_admin(store, user_id)? {
        return Ok(());
    }
    let role = store
        .members
        .get(space_key)
        .and_then(|members| members.get(user_id))
        .map(String::as_str);
    if space_role_allows(role, required) {
        Ok(())
    } else {
        Err(shared::AppError::Forbidden)
    }
}

fn ensure_space_accepts_writes(store: &WikiStore, space_key: &str) -> Result<(), shared::AppError> {
    let space = store
        .spaces
        .get(space_key)
        .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
    if space.status == "archived" {
        Err(shared::AppError::invalid_input(
            "archived space does not accept new documents or evidence",
        ))
    } else {
        Ok(())
    }
}

fn can_view_space(store: &WikiStore, space_key: &str, user_id: &str) -> bool {
    ensure_space_access(store, space_key, user_id, WikiSpaceAccess::View).is_ok()
}

fn ensure_document_access(
    store: &WikiStore,
    document_id: &str,
    user_id: &str,
    required: WikiSpaceAccess,
) -> Result<String, shared::AppError> {
    let document = store
        .documents
        .get(document_id)
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;
    ensure_space_access(store, &document.space_key, user_id, required)?;
    Ok(document.space_key.clone())
}

fn ensure_attachment_access(
    store: &WikiStore,
    attachment_id: &str,
    user_id: &str,
) -> Result<(), shared::AppError> {
    let attachment = store
        .attachments
        .get(attachment_id)
        .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
    if let Some(evidence) = store
        .evidence
        .values()
        .find(|item| item.attachment_id.as_deref() == Some(attachment_id))
    {
        return ensure_space_access(store, &evidence.space_key, user_id, WikiSpaceAccess::View);
    }
    if is_system_admin(store, user_id)? || attachment.metadata.uploaded_by == user_id {
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
    responses((status = 201, body = WikiAuthResponse), (status = 403))
)]
pub async fn register(
    Extension(backend): Extension<WikiBackend>,
    Json(body): Json<WikiRegisterRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if !backend.registration_enabled() {
        return Err(shared::AppError::Forbidden);
    }

    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent.register(body).await?;
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.login(body).await?));
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.refresh(body).await?));
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
    if let Some(persistent) = backend.persistent_backend() {
        persistent.logout(&claims).await?;
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.get_current_user(&claims).await?));
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
    path = "/api/v1/settings",
    tag = "settings",
    responses((status = 200, body = WikiSettingsResponse), (status = 403)),
    security(("bearer" = []))
)]
pub async fn get_settings(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<WikiSettingsResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(WikiSettingsResponse::from_snapshot(
            persistent.get_settings(&claims).await?,
        )));
    }

    let store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    Ok(Json(WikiSettingsResponse::from_snapshot(
        backend.settings_snapshot(),
    )))
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_users(&claims).await?));
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
    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent.create_user(&claims, body).await?;
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.update_user(&claims, &user_id, body).await?));
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<SpaceListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_spaces(&claims).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let spaces = if is_system_admin(&store, &claims.user_id)? {
        store.spaces.values().cloned().collect()
    } else {
        store
            .spaces
            .values()
            .filter(|space| can_view_space(&store, &space.key, &claims.user_id))
            .cloned()
            .collect()
    };
    Ok(Json(SpaceListResponse { spaces }))
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
    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent.create_space(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let key = normalize_space_key(&body.key)?;
    let name = normalize_required(&body.name, "space name")?;
    let mut store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    if store.spaces.contains_key(&key) {
        return Err(shared::AppError::conflict("space already exists"));
    }
    let now = now_iso();
    let space = SpaceResponse {
        id: new_id(),
        key: key.clone(),
        name,
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<SpaceResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.get_space(&claims, &space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
    let space = store
        .spaces
        .get(&key)
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent.update_space(&claims, &space_key, body).await?,
        ));
    }

    let key = normalize_space_key(&space_key)?;
    let mut store = store().lock().expect("wiki store lock");
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Admin)?;
    let space = store
        .spaces
        .get_mut(&key)
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    if let Some(name) = body.name {
        let name = name.trim().to_string();
        if !name.is_empty() {
            space.name = name;
        }
    }
    if body.description.is_some() {
        space.description = body.description.map(|value| value.trim().to_string());
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.archive_space(&claims, &space_key).await?));
    }

    let key = normalize_space_key(&space_key)?;
    let mut store = store().lock().expect("wiki store lock");
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Admin)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<SpaceMemberListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent.list_space_members(&claims, &space_key).await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Admin)?;
    let members = store
        .members
        .get(&key)
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .upsert_space_member(&claims, &space_key, &user_id, body)
                .await?,
        ));
    }

    let key = normalize_space_key(&space_key)?;
    let mut store = store().lock().expect("wiki store lock");
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Admin)?;
    let user = store
        .users
        .get(&user_id)
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("user", &user_id))?;
    let role = normalize_space_role(&body.role)?.to_string();
    let members = store
        .members
        .get_mut(&key)
        .ok_or_else(|| shared::AppError::not_found("space", &space_key))?;
    members.insert(user_id.clone(), role.clone());
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
        role,
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
    if let Some(persistent) = backend.persistent_backend() {
        persistent
            .delete_space_member(&claims, &space_key, &user_id)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let key = normalize_space_key(&space_key)?;
    let mut store = store().lock().expect("wiki store lock");
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Admin)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<SpaceTreeResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.get_space_tree(&claims, &space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
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
    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent
            .create_document(&claims, &space_key, body)
            .await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let key = normalize_space_key(&space_key)?;
    let mut store = store().lock().expect("wiki store lock");
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Edit)?;
    ensure_space_accepts_writes(&store, &key)?;
    let title = normalize_required(&body.title, "document title")?;
    let document_type = normalize_document_type(&body.document_type, true)?.to_string();
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
    let mut slug = body.slug.unwrap_or_else(|| slugify(&title));
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
        task_keys.insert(normalize_task_key(&task_key)?);
    }
    let mut phase_keys = BTreeSet::new();
    if let Some(phase_key) = body.phase_key {
        phase_keys.insert(normalize_phase_key(&phase_key)?);
    }
    let document = DocumentRecord {
        id: id.clone(),
        space_key: key.clone(),
        parent_id,
        slug,
        title,
        document_type,
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.get_document(&claims, &document_id).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::View)?;
    Ok(Json(document_response(&store, &id)?))
}

#[utoipa::path(
    put,
    path = "/api/v1/documents/{document_id}/draft",
    tag = "documents",
    params(("document_id" = String, Path)),
    request_body = UpdateDocumentDraftRequest,
    responses(
        (status = 200, body = DocumentResponse),
        (status = 400, description = "Validation error"),
        (status = 404)
    ),
    security(("bearer" = []))
)]
pub async fn update_document_draft(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<UpdateDocumentDraftRequest>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .update_document_draft(&claims, &document_id, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::Edit)?;
    let document = store
        .documents
        .get_mut(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    if document.status == "archived" {
        return Err(shared::AppError::invalid_input(
            "archived document does not accept writes",
        ));
    }
    if let Some(title) = body.title {
        let title = title.trim().to_string();
        if title.is_empty() {
            return Err(shared::AppError::invalid_input(
                "document title is required",
            ));
        }
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
    responses(
        (status = 200, body = DocumentRevisionResponse),
        (status = 400, description = "Validation error"),
        (status = 404)
    ),
    security(("bearer" = []))
)]
pub async fn publish_document(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<PublishDocumentRequest>,
) -> Result<Json<DocumentRevisionResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .publish_document(&claims, &document_id, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::Edit)?;
    let version = store
        .revisions
        .get(&id)
        .map_or(1, |items| items.len() as u32 + 1);
    let revision_id = new_id();
    let document = store
        .documents
        .get_mut(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    if document.status == "archived" {
        return Err(shared::AppError::invalid_input(
            "archived document does not accept writes",
        ));
    }
    if document.draft_markdown.trim().is_empty() {
        return Err(shared::AppError::invalid_input(
            "published content is required",
        ));
    }
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent.archive_document(&claims, &document_id).await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::Edit)?;
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
    responses(
        (status = 200, body = DocumentResponse),
        (status = 400, description = "Validation error"),
        (status = 404)
    ),
    security(("bearer" = []))
)]
pub async fn move_document(
    Path(document_id): Path<String>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<MoveDocumentRequest>,
) -> Result<Json<DocumentResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .move_document(&claims, &document_id, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    let document_space =
        ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::Edit)?;
    let document = store
        .documents
        .get(&id)
        .ok_or_else(|| shared::AppError::not_found("document", &document_id))?;
    if document.status == "archived" {
        return Err(shared::AppError::invalid_input(
            "archived document does not accept writes",
        ));
    }
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
            if document_parent_chain_contains(&store, Some(resolved_parent_id.as_str()), &id)? {
                return Err(shared::AppError::invalid_input(
                    "document cannot be moved under its descendant",
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<DocumentRevisionListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .list_document_revisions(&claims, &document_id)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::View)?;
    let mut revisions = store.revisions.get(&id).cloned().unwrap_or_default();
    revisions.sort_by_key(|revision| std::cmp::Reverse(revision.version));
    Ok(Json(DocumentRevisionListResponse { revisions }))
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<DocumentRevisionResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .get_document_revision(&claims, &document_id, &revision_id)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let id = resolve_document_id(&store, &document_id)?;
    ensure_document_access(&store, &id, &claims.user_id, WikiSpaceAccess::View)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<TaskPageListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_tasks(&claims, &space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<TaskPageResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent.get_task(&claims, &space_key, &task_key).await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let task_key = normalize_task_key(&task_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
    Ok(Json(task_page(&store, &key, &task_key)))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_key}/tasks/{task_key}/links/documents",
    tag = "tasks",
    params(("space_key" = String, Path), ("task_key" = String, Path)),
    request_body = LinkDocumentRequest,
    responses(
        (status = 200, body = TaskPageResponse),
        (status = 400, description = "Validation error"),
        (status = 404)
    ),
    security(("bearer" = []))
)]
pub async fn link_task_document(
    Path((space_key, task_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<LinkDocumentRequest>,
) -> Result<Json<TaskPageResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .link_task_document(&claims, &space_key, &task_key, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let task_key = normalize_task_key(&task_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Edit)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<DocumentListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .list_task_documents(&claims, &space_key, &task_key)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let task_key = normalize_task_key(&task_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<EvidenceListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .list_task_evidence(&claims, &space_key, &task_key)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let task_key = normalize_task_key(&task_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<PhasePageListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_phases(&claims, &space_key).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<PhasePageResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .get_phase(&claims, &space_key, &phase_key)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let phase_key = normalize_phase_key(&phase_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
    Ok(Json(phase_page(&store, &key, &phase_key)))
}

#[utoipa::path(
    post,
    path = "/api/v1/spaces/{space_key}/phases/{phase_key}/links/documents",
    tag = "phases",
    params(("space_key" = String, Path), ("phase_key" = String, Path)),
    request_body = LinkDocumentRequest,
    responses(
        (status = 200, body = PhasePageResponse),
        (status = 400, description = "Validation error"),
        (status = 404)
    ),
    security(("bearer" = []))
)]
pub async fn link_phase_document(
    Path((space_key, phase_key)): Path<(String, String)>,
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<LinkDocumentRequest>,
) -> Result<Json<PhasePageResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .link_phase_document(&claims, &space_key, &phase_key, body)
                .await?,
        ));
    }

    let mut store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let phase_key = normalize_phase_key(&phase_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::Edit)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<DocumentListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .list_phase_documents(&claims, &space_key, &phase_key)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let phase_key = normalize_phase_key(&phase_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<EvidenceListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent
                .list_phase_evidence(&claims, &space_key, &phase_key)
                .await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    let key = normalize_space_key(&space_key)?;
    let phase_key = normalize_phase_key(&phase_key)?;
    ensure_space_access(&store, &key, &claims.user_id, WikiSpaceAccess::View)?;
    Ok(Json(EvidenceListResponse {
        evidence: evidence_for_phase(&store, &key, &phase_key),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/evidence",
    tag = "evidence",
    request_body = CreateEvidenceRequest,
    responses(
        (status = 201, body = EvidenceResponse),
        (status = 400, description = "Validation error"),
        (status = 404)
    ),
    security(("bearer" = []))
)]
pub async fn create_evidence(
    Extension(backend): Extension<WikiBackend>,
    Extension(claims): Extension<WikiClaims>,
    Json(body): Json<CreateEvidenceRequest>,
) -> Result<impl IntoResponse, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent.create_evidence(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
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
    let title = normalize_required(&title, "evidence title")?;
    let evidence_type = normalize_evidence_type(&evidence_type)?;
    let url_supplied = url.is_some();
    let checksum_supplied = checksum.is_some();
    let url = url
        .map(|value| normalize_required(&value, "evidence url"))
        .transpose()?;
    let checksum = checksum
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    match evidence_type {
        "external_url" if url.is_none() || attachment_id.is_some() || checksum_supplied => {
            return Err(shared::AppError::invalid_input(
                "external_url evidence requires url only",
            ));
        }
        "uploaded_file" if attachment_id.is_none() || url_supplied || checksum_supplied => {
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
        .map(|value| normalize_space_key(&value))
        .transpose()?
        .unwrap_or_else(|| "SDLC".to_string());
    if !store.spaces.contains_key(&space_key) {
        return Err(shared::AppError::not_found("space", &space_key));
    }
    ensure_space_access(&store, &space_key, &claims.user_id, WikiSpaceAccess::Edit)?;
    ensure_space_accepts_writes(&store, &space_key)?;
    if document_space.is_some_and(|document_space| document_space != space_key) {
        return Err(shared::AppError::invalid_input(
            "document belongs to another space",
        ));
    }
    if document_id.is_none() && task_key.is_none() && phase_key.is_none() {
        return Err(shared::AppError::invalid_input(
            "evidence must target a document, task or phase",
        ));
    }
    let stored_checksum = if let Some(attachment_id) = &attachment_id {
        let attachment = store
            .attachments
            .get(attachment_id)
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        if attachment.metadata.uploaded_by != claims.user_id
            || store
                .evidence
                .values()
                .any(|item| item.attachment_id.as_deref() == Some(attachment_id))
        {
            return Err(shared::AppError::not_found("attachment", attachment_id));
        }
        Some(attachment.metadata.checksum.clone())
    } else {
        checksum
    };
    let id = new_id();
    let evidence = EvidenceResponse {
        id: id.clone(),
        space_key,
        document_id,
        task_key: task_key
            .map(|value| normalize_task_key(&value))
            .transpose()?,
        phase_key: phase_key
            .map(|value| normalize_phase_key(&value))
            .transpose()?,
        title,
        evidence_type: evidence_type.to_string(),
        url,
        attachment_id,
        checksum: stored_checksum,
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
    Extension(claims): Extension<WikiClaims>,
    Query(query): Query<EvidenceQuery>,
) -> Result<Json<EvidenceListResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_evidence(Some(&claims), query).await?));
    }

    let requested_space = query
        .space
        .as_deref()
        .map(normalize_space_key)
        .transpose()?;
    let requested_task_key = query
        .task_key
        .as_deref()
        .map(normalize_task_key)
        .transpose()?;
    let requested_phase_key = query
        .phase_key
        .as_deref()
        .map(normalize_phase_key)
        .transpose()?;
    let store = store().lock().expect("wiki store lock");
    if let Some(key) = &requested_space {
        ensure_space_access(&store, key, &claims.user_id, WikiSpaceAccess::View)?;
    }
    let mut items: Vec<_> = store
        .evidence
        .values()
        .filter(|item| can_view_space(&store, &item.space_key, &claims.user_id))
        .filter(|item| {
            requested_space
                .as_ref()
                .is_none_or(|space| item.space_key == space.as_str())
        })
        .filter(|item| {
            query
                .document_id
                .as_ref()
                .is_none_or(|id| item.document_id.as_ref() == Some(id))
        })
        .filter(|item| {
            requested_task_key
                .as_ref()
                .is_none_or(|key| item.task_key.as_ref() == Some(key))
        })
        .filter(|item| {
            requested_phase_key
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<EvidenceResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.get_evidence(&claims, &evidence_id).await?));
    }

    let store = store().lock().expect("wiki store lock");
    let evidence = store
        .evidence
        .get(&evidence_id)
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("evidence", &evidence_id))?;
    ensure_space_access(
        &store,
        &evidence.space_key,
        &claims.user_id,
        WikiSpaceAccess::View,
    )?;
    Ok(Json(evidence))
}

#[utoipa::path(
    post,
    path = "/api/v1/attachments",
    tag = "attachments",
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 201, body = AttachmentResponse),
        (status = 400, description = "Validation error")
    ),
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

    if let Some(field) = multipart
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
    }

    if bytes.is_empty() {
        return Err(shared::AppError::invalid_input("file is required"));
    }

    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Json<AttachmentResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(
            persistent.get_attachment(&claims, &attachment_id).await?,
        ));
    }

    let store = store().lock().expect("wiki store lock");
    ensure_attachment_access(&store, &attachment_id, &claims.user_id)?;
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
    Extension(claims): Extension<WikiClaims>,
) -> Result<Response, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        let download = persistent
            .download_attachment(&claims, &attachment_id)
            .await?;
        return attachment_download_response(download);
    }

    let store = store().lock().expect("wiki store lock");
    ensure_attachment_access(&store, &attachment_id, &claims.user_id)?;
    let attachment = store
        .attachments
        .get(&attachment_id)
        .cloned()
        .ok_or_else(|| shared::AppError::not_found("attachment", &attachment_id))?;
    attachment_download_response(AttachmentDownloadResponse {
        file_name: attachment.metadata.file_name,
        content_type: attachment.metadata.content_type,
        bytes: attachment.bytes,
    })
}

fn attachment_download_response(
    download: AttachmentDownloadResponse,
) -> Result<Response, shared::AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&download.content_type)
            .map_err(|err| shared::AppError::internal(err.to_string()))?,
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            safe_download_filename(&download.file_name)
        ))
        .map_err(|err| shared::AppError::internal(err.to_string()))?,
    );
    Ok((headers, download.bytes).into_response())
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_templates().await?));
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
    if let Some(persistent) = backend.persistent_backend() {
        let response = persistent.create_template(&claims, body).await?;
        return Ok((StatusCode::CREATED, Json(response)));
    }

    let mut store = store().lock().expect("wiki store lock");
    ensure_system_admin(&store, &claims.user_id)?;
    let name = normalize_required(&body.name, "template name")?;
    let document_type = normalize_document_type(&body.document_type, false)?.to_string();
    let body_markdown = normalize_required(&body.body_markdown, "template body_markdown")?;
    let id = slugify(&name);
    let template = TemplateResponse {
        id: id.clone(),
        name,
        document_type,
        body_markdown,
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
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.list_audit_log(&claims).await?));
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
    Extension(claims): Extension<WikiClaims>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, shared::AppError> {
    if let Some(persistent) = backend.persistent_backend() {
        return Ok(Json(persistent.search(&claims, query).await?));
    }

    let requested_space = query
        .space
        .as_deref()
        .map(normalize_space_key)
        .transpose()?;
    let requested_task_key = query
        .task_key
        .as_deref()
        .map(normalize_task_key)
        .transpose()?;
    let requested_phase_key = query
        .phase_key
        .as_deref()
        .map(normalize_phase_key)
        .transpose()?;
    let requested_document_type = query
        .document_type
        .as_deref()
        .map(|value| normalize_document_type(value, true))
        .transpose()?;
    let store = store().lock().expect("wiki store lock");
    if let Some(key) = &requested_space {
        ensure_space_access(&store, key, &claims.user_id, WikiSpaceAccess::View)?;
    }
    let needle = query.q.unwrap_or_default().to_lowercase();
    let include_archived = query.include_archived.unwrap_or(false);
    let mut results = Vec::new();

    for document in store.documents.values() {
        if !can_view_space(&store, &document.space_key, &claims.user_id) {
            continue;
        }
        if !include_archived && document.status == "archived" {
            continue;
        }
        if requested_space
            .as_ref()
            .is_some_and(|space| document.space_key != space.as_str())
        {
            continue;
        }
        if requested_document_type
            .is_some_and(|document_type| document.document_type != document_type)
        {
            continue;
        }
        if requested_task_key
            .as_ref()
            .is_some_and(|task_key| !document.task_keys.contains(task_key))
        {
            continue;
        }
        if requested_phase_key
            .as_ref()
            .is_some_and(|phase_key| !document.phase_keys.contains(phase_key))
        {
            continue;
        }
        let current_revision = document
            .current_revision_id
            .as_ref()
            .and_then(|revision_id| {
                store
                    .revisions
                    .get(&document.id)
                    .and_then(|items| items.iter().find(|revision| &revision.id == revision_id))
            });
        let indexed_title =
            current_revision.map_or(document.title.as_str(), |revision| revision.title.as_str());
        let indexed_markdown = current_revision
            .map_or(document.draft_markdown.as_str(), |revision| {
                revision.body_markdown.as_str()
            });
        let haystack = format!("{indexed_title} {indexed_markdown}").to_lowercase();
        if needle.is_empty() || haystack.contains(&needle) {
            results.push(SearchResultResponse {
                id: document.id.clone(),
                result_type: "document".to_string(),
                title: indexed_title.to_string(),
                space_key: document.space_key.clone(),
                url: format!("/documents/{}", document.slug),
                snippet: snippet(indexed_markdown),
                updated_at: document.updated_at.clone(),
            });
        }
    }

    for item in store.evidence.values() {
        if !can_view_space(&store, &item.space_key, &claims.user_id) {
            continue;
        }
        if requested_space
            .as_ref()
            .is_some_and(|space| item.space_key != space.as_str())
        {
            continue;
        }
        if requested_task_key
            .as_ref()
            .is_some_and(|task_key| item.task_key.as_ref() != Some(task_key))
        {
            continue;
        }
        if requested_phase_key
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

fn document_parent_chain_contains(
    store: &WikiStore,
    parent_id: Option<&str>,
    document_id: &str,
) -> Result<bool, shared::AppError> {
    let mut visited = BTreeSet::new();
    let mut current_id = parent_id.map(str::to_string);
    while let Some(id) = current_id {
        if id == document_id {
            return Ok(true);
        }
        if !visited.insert(id.clone()) {
            return Err(shared::AppError::conflict("document tree contains a cycle"));
        }
        let parent = store
            .documents
            .get(&id)
            .ok_or_else(|| shared::AppError::not_found("document", &id))?;
        current_id = parent.parent_id.clone();
    }
    Ok(false)
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

fn default_user_role() -> String {
    "viewer".to_string()
}
