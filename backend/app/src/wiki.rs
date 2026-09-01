use std::{future::Future, pin::Pin, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use domain::wiki::{DocumentSlug, DocumentType, EvidenceType, GlobalRole, PhaseKey, SpaceKey};
use domain::wiki::{SpaceRole, TaskKey};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub use shared::WikiSettingsSnapshot;
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

pub type WikiAuthRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiAuthUserRecord {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub global_role: String,
    pub is_active: bool,
}

impl WikiAuthUserRecord {
    pub fn user_response(&self) -> shared::WikiUserResponse {
        shared::WikiUserResponse {
            id: self.id.to_string(),
            email: self.email.clone(),
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            role: self.global_role.clone(),
            is_system_admin: self.global_role == GlobalRole::Admin.as_str(),
            active: self.is_active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiSessionCommand {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: String,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiRegisterAuthCommand {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub session: WikiSessionCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiAccessSessionCommand {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub access_token_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiRefreshSessionCommand {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub refresh_token_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLogoutCommand {
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
}

pub trait WikiAuthRepository {
    fn authenticate_access_session<'a>(
        &'a self,
        command: WikiAccessSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, bool>;

    fn register_user<'a>(
        &'a self,
        command: WikiRegisterAuthCommand,
    ) -> WikiAuthRepositoryFuture<'a, WikiAuthUserRecord>;

    fn find_user_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> WikiAuthRepositoryFuture<'a, Option<WikiAuthUserRecord>>;

    fn create_login_session<'a>(
        &'a self,
        session: WikiSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, ()>;

    fn find_refresh_session<'a>(
        &'a self,
        command: WikiRefreshSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, Option<WikiAuthUserRecord>>;

    fn rotate_session<'a>(
        &'a self,
        session: WikiSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, ()>;

    fn revoke_sessions<'a>(
        &'a self,
        command: WikiLogoutCommand,
    ) -> WikiAuthRepositoryFuture<'a, ()>;

    fn get_current_user<'a>(
        &'a self,
        user_id: Uuid,
    ) -> WikiAuthRepositoryFuture<'a, WikiAuthUserRecord>;
}

pub struct WikiAuthUseCase<'a, R: WikiAuthRepository + ?Sized> {
    repository: &'a R,
    config: &'a AuthConfig,
}

impl<'a, R: WikiAuthRepository + ?Sized> WikiAuthUseCase<'a, R> {
    pub fn new(repository: &'a R, config: &'a AuthConfig) -> Self {
        Self { repository, config }
    }

    pub async fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<shared::WikiClaims, AppError> {
        let claims = decode_token(self.config, token, "access")?;
        let user_id = parse_token_uuid(&claims.sub)?;
        let session_id = parse_token_uuid(&claims.jti)?;
        let authenticated = self
            .repository
            .authenticate_access_session(WikiAccessSessionCommand {
                user_id,
                session_id,
                access_token_hash: hash_token(token),
            })
            .await?;

        if !authenticated {
            return Err(AppError::Unauthorized);
        }

        Ok(shared::WikiClaims {
            user_id: user_id.to_string(),
            session_id: Some(session_id.to_string()),
        })
    }

    pub async fn register(
        &self,
        body: shared::WikiRegisterRequest,
    ) -> Result<shared::WikiAuthResponse, AppError> {
        if !self.config.registration_enabled {
            return Err(AppError::Forbidden);
        }

        let email = normalize_required(&body.email, "email")?;
        let username = normalize_required(&body.username, "username")?;
        let password = normalize_required(&body.password, "password")?;
        let display_name = body
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(username.as_str())
            .to_string();
        let user_id = Uuid::now_v7();
        let token_pair = create_wiki_session_token_pair(self.config, user_id)?;
        let command = WikiRegisterAuthCommand {
            user_id,
            email,
            username,
            display_name,
            password_hash: hash_password(&password)?,
            session: session_command(user_id, &token_pair),
        };
        let user = self.repository.register_user(command).await?;
        Ok(auth_response(&user, token_pair))
    }

    pub async fn login(
        &self,
        body: shared::WikiLoginRequest,
    ) -> Result<shared::WikiAuthResponse, AppError> {
        let email = normalize_required(&body.email, "email")?;
        let password = normalize_required(&body.password, "password")?;
        let user = self
            .repository
            .find_user_by_email(&email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !user.is_active || !verify_password(&password, &user.password_hash)? {
            return Err(AppError::Unauthorized);
        }

        let token_pair = create_wiki_session_token_pair(self.config, user.id)?;
        self.repository
            .create_login_session(session_command(user.id, &token_pair))
            .await?;
        Ok(auth_response(&user, token_pair))
    }

    pub async fn refresh(
        &self,
        body: shared::WikiRefreshRequest,
    ) -> Result<shared::WikiAuthResponse, AppError> {
        let refresh_token = normalize_required(&body.refresh_token, "refresh_token")?;
        let claims = decode_token(self.config, &refresh_token, "refresh")?;
        let user_id = parse_token_uuid(&claims.sub)?;
        let session_id = parse_token_uuid(&claims.jti)?;
        let user = self
            .repository
            .find_refresh_session(WikiRefreshSessionCommand {
                user_id,
                session_id,
                refresh_token_hash: hash_token(&refresh_token),
            })
            .await?
            .ok_or(AppError::Unauthorized)?;

        let token_pair = create_wiki_token_pair(self.config, user_id, session_id)?;
        self.repository
            .rotate_session(session_command(user_id, &token_pair))
            .await?;
        Ok(auth_response(&user, token_pair))
    }

    pub async fn logout(&self, claims: &shared::WikiClaims) -> Result<(), AppError> {
        let command = WikiLogoutCommand {
            user_id: parse_claim_uuid(&claims.user_id, "user")?,
            session_id: claims
                .session_id
                .as_deref()
                .map(|value| parse_claim_uuid(value, "session"))
                .transpose()?,
        };
        self.repository.revoke_sessions(command).await
    }

    pub async fn current_user(
        &self,
        claims: &shared::WikiClaims,
    ) -> Result<shared::WikiUserResponse, AppError> {
        let user_id = parse_claim_uuid(&claims.user_id, "user")?;
        Ok(self
            .repository
            .get_current_user(user_id)
            .await?
            .user_response())
    }
}

pub type WikiUserRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiCreateUserCommand {
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub global_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiUpdateUserCommand {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub global_role: Option<String>,
    pub active: Option<bool>,
}

pub trait WikiUserRepository {
    fn list_users<'a>(&'a self) -> WikiUserRepositoryFuture<'a, Vec<shared::WikiUserResponse>>;

    fn create_user<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateUserCommand,
    ) -> WikiUserRepositoryFuture<'a, shared::WikiUserResponse>;

    fn update_user<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpdateUserCommand,
    ) -> WikiUserRepositoryFuture<'a, shared::WikiUserResponse>;
}

pub struct WikiUserUseCase<'a, R: WikiUserRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiUserRepository + ?Sized> WikiUserUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn list(&self) -> Result<shared::WikiUserListResponse, AppError> {
        Ok(shared::WikiUserListResponse {
            users: self.repository.list_users().await?,
        })
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        body: shared::WikiCreateUserRequest,
    ) -> Result<shared::WikiUserResponse, AppError> {
        let password = normalize_required(&body.password, "password")?;
        let command = WikiCreateUserCommand {
            email: normalize_required(&body.email, "email")?,
            username: normalize_required(&body.username, "username")?,
            display_name: normalize_required(&body.display_name, "display_name")?,
            password_hash: hash_password(&password)?,
            global_role: global_role_from_request(&body.role)?.to_string(),
        };
        self.repository.create_user(actor_id, command).await
    }

    pub async fn update(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
        body: shared::WikiUpdateUserRequest,
    ) -> Result<shared::WikiUserResponse, AppError> {
        let role = match body.role.as_deref() {
            Some(role) => Some(global_role_from_request(role)?.to_string()),
            None => None,
        };
        let global_role = if body.is_system_admin == Some(true) {
            Some(GlobalRole::Admin.as_str().to_string())
        } else if body.is_system_admin == Some(false) {
            Some(GlobalRole::User.as_str().to_string())
        } else {
            role
        };

        let command = WikiUpdateUserCommand {
            user_id,
            email: normalize_optional_update_value(body.email.as_deref()),
            username: normalize_optional_update_value(body.username.as_deref()),
            display_name: normalize_optional_update_value(body.display_name.as_deref()),
            global_role,
            active: body.active,
        };
        self.repository.update_user(actor_id, command).await
    }
}

pub type WikiSettingsRepositoryFuture<'a> =
    Pin<Box<dyn Future<Output = Result<WikiSettingsSnapshot, AppError>> + Send + 'a>>;

pub trait WikiSettingsRepository {
    fn get_settings<'a>(&'a self) -> WikiSettingsRepositoryFuture<'a>;
}

pub struct WikiSettingsUseCase<'a, R: WikiSettingsRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiSettingsRepository + ?Sized> WikiSettingsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn get(&self) -> Result<WikiSettingsSnapshot, AppError> {
        self.repository.get_settings().await
    }
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

pub type WikiSearchRepositoryFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<shared::SearchResultResponse>, AppError>> + Send + 'a>>;

pub trait WikiSearchRepository {
    fn search_documents<'a>(
        &'a self,
        criteria: &'a WikiSearchCriteria,
        restricted_user_id: Option<Uuid>,
    ) -> WikiSearchRepositoryFuture<'a>;

    fn search_evidence<'a>(
        &'a self,
        criteria: &'a WikiSearchCriteria,
        restricted_user_id: Option<Uuid>,
    ) -> WikiSearchRepositoryFuture<'a>;
}

pub struct WikiSearchUseCase<'a, R: WikiSearchRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiSearchRepository + ?Sized> WikiSearchUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        criteria: WikiSearchCriteria,
        restricted_user_id: Option<Uuid>,
    ) -> Result<shared::SearchResponse, AppError> {
        let mut results = self
            .repository
            .search_documents(&criteria, restricted_user_id)
            .await?;
        results.extend(
            self.repository
                .search_evidence(&criteria, restricted_user_id)
                .await?,
        );
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(criteria.limit as usize);
        Ok(shared::SearchResponse { results })
    }
}

pub type WikiTemplateRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiCreateTemplateCommand {
    pub name: String,
    pub document_type: String,
    pub body_markdown: String,
}

pub trait WikiTemplateRepository {
    fn list_active_templates<'a>(
        &'a self,
    ) -> WikiTemplateRepositoryFuture<'a, Vec<shared::TemplateResponse>>;

    fn create_template<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateTemplateCommand,
    ) -> WikiTemplateRepositoryFuture<'a, shared::TemplateResponse>;
}

pub struct WikiTemplateUseCase<'a, R: WikiTemplateRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiTemplateRepository + ?Sized> WikiTemplateUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn list(&self) -> Result<shared::TemplateListResponse, AppError> {
        Ok(shared::TemplateListResponse {
            templates: self.repository.list_active_templates().await?,
        })
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        body: shared::CreateTemplateRequest,
    ) -> Result<shared::TemplateResponse, AppError> {
        let command = WikiCreateTemplateCommand {
            name: normalize_required(&body.name, "template name")?,
            document_type: normalize_document_type(&body.document_type, false)?.to_string(),
            body_markdown: normalize_required(&body.body_markdown, "template body_markdown")?,
        };
        self.repository.create_template(actor_id, command).await
    }
}

pub type WikiAuditRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiAuditCommand {
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
}

pub trait WikiAuditRepository {
    fn list_recent_entries<'a>(
        &'a self,
        limit: usize,
    ) -> WikiAuditRepositoryFuture<'a, Vec<shared::AuditEntryResponse>>;

    fn record_entry<'a>(&'a self, command: WikiAuditCommand) -> WikiAuditRepositoryFuture<'a, ()>;
}

pub struct WikiAuditUseCase<'a, R: WikiAuditRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiAuditRepository + ?Sized> WikiAuditUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn list_recent(&self) -> Result<shared::AuditLogResponse, AppError> {
        Ok(shared::AuditLogResponse {
            entries: self.repository.list_recent_entries(200).await?,
        })
    }

    pub async fn record(
        &self,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), AppError> {
        let command = WikiAuditCommand {
            actor_id,
            action: normalize_required(action, "audit action")?,
            entity_type: normalize_required(entity_type, "audit entity_type")?,
            entity_id,
        };
        self.repository.record_entry(command).await
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

fn normalize_optional_update_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_token_uuid(value: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| AppError::Unauthorized)
}

fn parse_claim_uuid(value: &str, entity: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| AppError::not_found(entity, value))
}

fn session_command(user_id: Uuid, token_pair: &WikiTokenPair) -> WikiSessionCommand {
    WikiSessionCommand {
        session_id: token_pair.session_id,
        user_id,
        access_token_hash: hash_token(&token_pair.access_token),
        refresh_token_hash: hash_token(&token_pair.refresh_token),
        access_expires_at: token_pair.access_expires_at,
        refresh_expires_at: token_pair.refresh_expires_at,
    }
}

fn auth_response(user: &WikiAuthUserRecord, token_pair: WikiTokenPair) -> shared::WikiAuthResponse {
    shared::WikiAuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: "Bearer".to_string(),
        user_id: user.id.to_string(),
        email: user.email.clone(),
        username: user.username.clone(),
        display_name: user.display_name.clone(),
        expires_in: token_pair.expires_in,
    }
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

pub fn build_wiki_search_criteria_from_query(
    query: &shared::SearchQuery,
) -> Result<WikiSearchCriteria, AppError> {
    build_wiki_search_criteria(
        query.q.as_deref(),
        query.space.as_deref(),
        query.task_key.as_deref(),
        query.phase_key.as_deref(),
        query.document_type.as_deref(),
        query.include_archived,
        query.limit,
    )
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

    struct StaticSearchRepository {
        documents: Vec<shared::SearchResultResponse>,
        evidence: Vec<shared::SearchResultResponse>,
    }

    struct RecordingUserRepository {
        users: Vec<shared::WikiUserResponse>,
        created: std::sync::Mutex<Vec<(Uuid, WikiCreateUserCommand)>>,
        updated: std::sync::Mutex<Vec<(Uuid, WikiUpdateUserCommand)>>,
    }

    struct StaticSettingsRepository {
        snapshot: WikiSettingsSnapshot,
    }

    struct RecordingTemplateRepository {
        created: std::sync::Mutex<Vec<(Uuid, WikiCreateTemplateCommand)>>,
    }

    struct RecordingAuditRepository {
        entries: Vec<shared::AuditEntryResponse>,
        recorded: std::sync::Mutex<Vec<WikiAuditCommand>>,
    }

    struct RecordingAuthRepository {
        user_by_email: Option<WikiAuthUserRecord>,
        refresh_user: Option<WikiAuthUserRecord>,
        current_user: Option<WikiAuthUserRecord>,
        access_authenticated: bool,
        registered: std::sync::Mutex<Vec<WikiRegisterAuthCommand>>,
        email_lookups: std::sync::Mutex<Vec<String>>,
        login_sessions: std::sync::Mutex<Vec<WikiSessionCommand>>,
        refresh_lookups: std::sync::Mutex<Vec<WikiRefreshSessionCommand>>,
        rotated_sessions: std::sync::Mutex<Vec<WikiSessionCommand>>,
        revoked_sessions: std::sync::Mutex<Vec<WikiLogoutCommand>>,
        current_user_lookups: std::sync::Mutex<Vec<Uuid>>,
        access_lookups: std::sync::Mutex<Vec<WikiAccessSessionCommand>>,
    }

    impl WikiSearchRepository for StaticSearchRepository {
        fn search_documents<'a>(
            &'a self,
            _criteria: &'a WikiSearchCriteria,
            _restricted_user_id: Option<Uuid>,
        ) -> WikiSearchRepositoryFuture<'a> {
            let documents = self.documents.clone();
            Box::pin(async move { Ok(documents) })
        }

        fn search_evidence<'a>(
            &'a self,
            _criteria: &'a WikiSearchCriteria,
            _restricted_user_id: Option<Uuid>,
        ) -> WikiSearchRepositoryFuture<'a> {
            let evidence = self.evidence.clone();
            Box::pin(async move { Ok(evidence) })
        }
    }

    impl WikiUserRepository for RecordingUserRepository {
        fn list_users<'a>(&'a self) -> WikiUserRepositoryFuture<'a, Vec<shared::WikiUserResponse>> {
            Box::pin(async move { Ok(self.users.clone()) })
        }

        fn create_user<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiCreateUserCommand,
        ) -> WikiUserRepositoryFuture<'a, shared::WikiUserResponse> {
            Box::pin(async move {
                self.created
                    .lock()
                    .expect("user create commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(shared::WikiUserResponse {
                    id: Uuid::now_v7().to_string(),
                    email: command.email,
                    username: command.username,
                    display_name: command.display_name,
                    role: command.global_role.clone(),
                    is_system_admin: command.global_role == GlobalRole::Admin.as_str(),
                    active: true,
                })
            })
        }

        fn update_user<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiUpdateUserCommand,
        ) -> WikiUserRepositoryFuture<'a, shared::WikiUserResponse> {
            Box::pin(async move {
                self.updated
                    .lock()
                    .expect("user update commands should be lockable")
                    .push((actor_id, command.clone()));
                let role = command
                    .global_role
                    .clone()
                    .unwrap_or_else(|| GlobalRole::User.as_str().to_string());
                Ok(shared::WikiUserResponse {
                    id: command.user_id.to_string(),
                    email: command
                        .email
                        .clone()
                        .unwrap_or_else(|| "user@example.test".to_string()),
                    username: command
                        .username
                        .clone()
                        .unwrap_or_else(|| "user".to_string()),
                    display_name: command
                        .display_name
                        .clone()
                        .unwrap_or_else(|| "User".to_string()),
                    role: role.clone(),
                    is_system_admin: role == GlobalRole::Admin.as_str(),
                    active: command.active.unwrap_or(true),
                })
            })
        }
    }

    impl WikiSettingsRepository for StaticSettingsRepository {
        fn get_settings<'a>(&'a self) -> WikiSettingsRepositoryFuture<'a> {
            Box::pin(async move { Ok(self.snapshot.clone()) })
        }
    }

    impl WikiTemplateRepository for RecordingTemplateRepository {
        fn list_active_templates<'a>(
            &'a self,
        ) -> WikiTemplateRepositoryFuture<'a, Vec<shared::TemplateResponse>> {
            Box::pin(async move {
                Ok(vec![shared::TemplateResponse {
                    id: Uuid::nil().to_string(),
                    name: "Requirements".to_string(),
                    document_type: "requirements".to_string(),
                    body_markdown: "# Requirements".to_string(),
                }])
            })
        }

        fn create_template<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiCreateTemplateCommand,
        ) -> WikiTemplateRepositoryFuture<'a, shared::TemplateResponse> {
            Box::pin(async move {
                self.created
                    .lock()
                    .expect("template commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(shared::TemplateResponse {
                    id: Uuid::now_v7().to_string(),
                    name: command.name,
                    document_type: command.document_type,
                    body_markdown: command.body_markdown,
                })
            })
        }
    }

    impl WikiAuditRepository for RecordingAuditRepository {
        fn list_recent_entries<'a>(
            &'a self,
            limit: usize,
        ) -> WikiAuditRepositoryFuture<'a, Vec<shared::AuditEntryResponse>> {
            Box::pin(async move { Ok(self.entries.iter().take(limit).cloned().collect()) })
        }

        fn record_entry<'a>(
            &'a self,
            command: WikiAuditCommand,
        ) -> WikiAuditRepositoryFuture<'a, ()> {
            Box::pin(async move {
                self.recorded
                    .lock()
                    .expect("audit commands should be lockable")
                    .push(command);
                Ok(())
            })
        }
    }

    impl WikiAuthRepository for RecordingAuthRepository {
        fn authenticate_access_session<'a>(
            &'a self,
            command: WikiAccessSessionCommand,
        ) -> WikiAuthRepositoryFuture<'a, bool> {
            Box::pin(async move {
                self.access_lookups
                    .lock()
                    .expect("access lookups should be lockable")
                    .push(command);
                Ok(self.access_authenticated)
            })
        }

        fn register_user<'a>(
            &'a self,
            command: WikiRegisterAuthCommand,
        ) -> WikiAuthRepositoryFuture<'a, WikiAuthUserRecord> {
            Box::pin(async move {
                self.registered
                    .lock()
                    .expect("register commands should be lockable")
                    .push(command.clone());
                Ok(WikiAuthUserRecord {
                    id: command.user_id,
                    email: command.email,
                    username: command.username,
                    display_name: command.display_name,
                    password_hash: command.password_hash,
                    global_role: GlobalRole::User.as_str().to_string(),
                    is_active: true,
                })
            })
        }

        fn find_user_by_email<'a>(
            &'a self,
            email: &'a str,
        ) -> WikiAuthRepositoryFuture<'a, Option<WikiAuthUserRecord>> {
            Box::pin(async move {
                self.email_lookups
                    .lock()
                    .expect("email lookups should be lockable")
                    .push(email.to_string());
                Ok(self.user_by_email.clone())
            })
        }

        fn create_login_session<'a>(
            &'a self,
            session: WikiSessionCommand,
        ) -> WikiAuthRepositoryFuture<'a, ()> {
            Box::pin(async move {
                self.login_sessions
                    .lock()
                    .expect("login sessions should be lockable")
                    .push(session);
                Ok(())
            })
        }

        fn find_refresh_session<'a>(
            &'a self,
            command: WikiRefreshSessionCommand,
        ) -> WikiAuthRepositoryFuture<'a, Option<WikiAuthUserRecord>> {
            Box::pin(async move {
                self.refresh_lookups
                    .lock()
                    .expect("refresh lookups should be lockable")
                    .push(command);
                Ok(self.refresh_user.clone())
            })
        }

        fn rotate_session<'a>(
            &'a self,
            session: WikiSessionCommand,
        ) -> WikiAuthRepositoryFuture<'a, ()> {
            Box::pin(async move {
                self.rotated_sessions
                    .lock()
                    .expect("rotated sessions should be lockable")
                    .push(session);
                Ok(())
            })
        }

        fn revoke_sessions<'a>(
            &'a self,
            command: WikiLogoutCommand,
        ) -> WikiAuthRepositoryFuture<'a, ()> {
            Box::pin(async move {
                self.revoked_sessions
                    .lock()
                    .expect("revoked sessions should be lockable")
                    .push(command);
                Ok(())
            })
        }

        fn get_current_user<'a>(
            &'a self,
            user_id: Uuid,
        ) -> WikiAuthRepositoryFuture<'a, WikiAuthUserRecord> {
            Box::pin(async move {
                self.current_user_lookups
                    .lock()
                    .expect("current user lookups should be lockable")
                    .push(user_id);
                self.current_user
                    .clone()
                    .ok_or_else(|| AppError::not_found("user", user_id))
            })
        }
    }

    fn audit_entry(action: &str) -> shared::AuditEntryResponse {
        shared::AuditEntryResponse {
            id: Uuid::now_v7().to_string(),
            actor_id: Uuid::nil().to_string(),
            action: action.to_string(),
            entity_type: "document".to_string(),
            entity_id: Uuid::now_v7().to_string(),
            created_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn wiki_user(email: &str, role: &str) -> shared::WikiUserResponse {
        shared::WikiUserResponse {
            id: Uuid::now_v7().to_string(),
            email: email.to_string(),
            username: default_username(email),
            display_name: email.to_string(),
            role: role.to_string(),
            is_system_admin: role == GlobalRole::Admin.as_str(),
            active: true,
        }
    }

    fn auth_user(
        email: &str,
        password: &str,
        global_role: &str,
        is_active: bool,
    ) -> WikiAuthUserRecord {
        WikiAuthUserRecord {
            id: Uuid::now_v7(),
            email: email.to_string(),
            username: default_username(email),
            display_name: email.to_string(),
            password_hash: hash_password(password).expect("test password should hash"),
            global_role: global_role.to_string(),
            is_active,
        }
    }

    fn recording_auth_repository(user: Option<WikiAuthUserRecord>) -> RecordingAuthRepository {
        RecordingAuthRepository {
            user_by_email: user.clone(),
            refresh_user: user.clone(),
            current_user: user,
            access_authenticated: true,
            registered: std::sync::Mutex::new(Vec::new()),
            email_lookups: std::sync::Mutex::new(Vec::new()),
            login_sessions: std::sync::Mutex::new(Vec::new()),
            refresh_lookups: std::sync::Mutex::new(Vec::new()),
            rotated_sessions: std::sync::Mutex::new(Vec::new()),
            revoked_sessions: std::sync::Mutex::new(Vec::new()),
            current_user_lookups: std::sync::Mutex::new(Vec::new()),
            access_lookups: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn search_result(
        id: &str,
        result_type: &str,
        title: &str,
        updated_at: &str,
    ) -> shared::SearchResultResponse {
        shared::SearchResultResponse {
            id: id.to_string(),
            result_type: result_type.to_string(),
            title: title.to_string(),
            space_key: "SDLC".to_string(),
            url: format!("/{result_type}/{id}"),
            snippet: title.to_string(),
            updated_at: updated_at.to_string(),
        }
    }

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

    #[tokio::test]
    async fn wiki_user_use_case_lists_and_validates_admin_commands() {
        let repository = RecordingUserRepository {
            users: vec![wiki_user("admin@example.test", "admin")],
            created: std::sync::Mutex::new(Vec::new()),
            updated: std::sync::Mutex::new(Vec::new()),
        };
        let actor_id = Uuid::now_v7();

        let list = WikiUserUseCase::new(&repository).list().await.unwrap();
        assert_eq!(list.users.len(), 1);
        assert_eq!(list.users[0].email, "admin@example.test");

        let created = WikiUserUseCase::new(&repository)
            .create(
                actor_id,
                shared::WikiCreateUserRequest {
                    email: "  editor@example.test  ".to_string(),
                    username: "  editor  ".to_string(),
                    password: "  secret-password  ".to_string(),
                    display_name: "  Editor User  ".to_string(),
                    role: "editor".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(created.email, "editor@example.test");
        assert_eq!(created.username, "editor");
        assert_eq!(created.display_name, "Editor User");
        assert_eq!(created.role, "user");

        {
            let created_commands = repository
                .created
                .lock()
                .expect("user create commands should be lockable");
            assert_eq!(created_commands.len(), 1);
            assert_eq!(created_commands[0].0, actor_id);
            assert_eq!(created_commands[0].1.email, "editor@example.test");
            assert_eq!(created_commands[0].1.username, "editor");
            assert_eq!(created_commands[0].1.display_name, "Editor User");
            assert_eq!(created_commands[0].1.global_role, "user");
            assert_ne!(created_commands[0].1.password_hash, "secret-password");
            assert!(
                verify_password("secret-password", &created_commands[0].1.password_hash).unwrap()
            );
        }

        assert!(
            WikiUserUseCase::new(&repository)
                .create(
                    actor_id,
                    shared::WikiCreateUserRequest {
                        email: "admin@example.test".to_string(),
                        username: "admin".to_string(),
                        password: " ".to_string(),
                        display_name: "Admin".to_string(),
                        role: "admin".to_string(),
                    },
                )
                .await
                .is_err()
        );

        let user_id = Uuid::now_v7();
        let updated = WikiUserUseCase::new(&repository)
            .update(
                actor_id,
                user_id,
                shared::WikiUpdateUserRequest {
                    email: Some("   ".to_string()),
                    username: Some("  viewer  ".to_string()),
                    display_name: Some("  Viewer User  ".to_string()),
                    role: Some("admin".to_string()),
                    is_system_admin: Some(false),
                    active: Some(false),
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.id, user_id.to_string());
        assert_eq!(updated.email, "user@example.test");
        assert_eq!(updated.username, "viewer");
        assert_eq!(updated.display_name, "Viewer User");
        assert_eq!(updated.role, "user");
        assert!(!updated.active);

        assert_eq!(
            repository
                .updated
                .lock()
                .expect("user update commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiUpdateUserCommand {
                    user_id,
                    email: None,
                    username: Some("viewer".to_string()),
                    display_name: Some("Viewer User".to_string()),
                    global_role: Some("user".to_string()),
                    active: Some(false),
                }
            )]
        );
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

    #[tokio::test]
    async fn wiki_search_use_case_merges_sorts_and_limits_repository_results() {
        let repository = StaticSearchRepository {
            documents: vec![
                search_result(
                    "doc-old",
                    "document",
                    "Old document",
                    "2026-08-30T10:00:00Z",
                ),
                search_result(
                    "doc-new",
                    "document",
                    "New document",
                    "2026-09-01T10:00:00Z",
                ),
            ],
            evidence: vec![search_result(
                "evidence-mid",
                "evidence",
                "Middle evidence",
                "2026-08-31T10:00:00Z",
            )],
        };
        let criteria =
            build_wiki_search_criteria(Some("wiki"), None, None, None, None, None, Some(2))
                .unwrap();

        let response = WikiSearchUseCase::new(&repository)
            .execute(criteria, Some(Uuid::nil()))
            .await
            .unwrap();

        assert_eq!(
            response
                .results
                .iter()
                .map(|result| result.id.as_str())
                .collect::<Vec<_>>(),
            vec!["doc-new", "evidence-mid"]
        );
    }

    #[tokio::test]
    async fn wiki_template_use_case_validates_and_normalizes_create_command() {
        let repository = RecordingTemplateRepository {
            created: std::sync::Mutex::new(Vec::new()),
        };
        let actor_id = Uuid::now_v7();

        let response = WikiTemplateUseCase::new(&repository)
            .create(
                actor_id,
                shared::CreateTemplateRequest {
                    name: "  Test plan  ".to_string(),
                    document_type: "test_plan".to_string(),
                    body_markdown: "\n# Test plan\n".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(response.name, "Test plan");
        assert_eq!(response.document_type, "test_plan");
        assert_eq!(response.body_markdown, "# Test plan");
        assert_eq!(
            repository
                .created
                .lock()
                .expect("template commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiCreateTemplateCommand {
                    name: "Test plan".to_string(),
                    document_type: "test_plan".to_string(),
                    body_markdown: "# Test plan".to_string(),
                }
            )]
        );

        assert!(
            WikiTemplateUseCase::new(&repository)
                .create(
                    actor_id,
                    shared::CreateTemplateRequest {
                        name: "Page".to_string(),
                        document_type: "page".to_string(),
                        body_markdown: "# Page".to_string(),
                    },
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn wiki_audit_use_case_lists_and_validates_record_commands() {
        let repository = RecordingAuditRepository {
            entries: vec![audit_entry("document.publish")],
            recorded: std::sync::Mutex::new(Vec::new()),
        };
        let entity_id = Uuid::now_v7();

        let response = WikiAuditUseCase::new(&repository)
            .list_recent()
            .await
            .unwrap();
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].action, "document.publish");

        WikiAuditUseCase::new(&repository)
            .record(
                Some(Uuid::nil()),
                " document.archive ",
                " document ",
                entity_id,
            )
            .await
            .unwrap();

        assert_eq!(
            repository
                .recorded
                .lock()
                .expect("audit commands should be lockable")
                .as_slice(),
            [WikiAuditCommand {
                actor_id: Some(Uuid::nil()),
                action: "document.archive".to_string(),
                entity_type: "document".to_string(),
                entity_id,
            }]
        );
        assert!(
            WikiAuditUseCase::new(&repository)
                .record(None, " ", "document", entity_id)
                .await
                .is_err()
        );
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

    #[tokio::test]
    async fn wiki_settings_use_case_returns_repository_snapshot() {
        let snapshot = WikiSettingsSnapshot::from_values(true, 25 * 1024 * 1024);
        let repository = StaticSettingsRepository {
            snapshot: snapshot.clone(),
        };

        assert_eq!(
            WikiSettingsUseCase::new(&repository).get().await.unwrap(),
            snapshot
        );
    }

    #[tokio::test]
    async fn wiki_auth_use_case_registers_user_with_session_command() {
        let config = test_auth_config();
        let repository = recording_auth_repository(None);

        let response = WikiAuthUseCase::new(&repository, &config)
            .register(shared::WikiRegisterRequest {
                email: "  new@example.test  ".to_string(),
                username: "  new-user  ".to_string(),
                password: "  secret-password  ".to_string(),
                name: Some("  New User  ".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(response.email, "new@example.test");
        assert_eq!(response.username, "new-user");
        assert_eq!(response.display_name, "New User");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 900);

        let registered = repository
            .registered
            .lock()
            .expect("register commands should be lockable");
        assert_eq!(registered.len(), 1);
        let command = &registered[0];
        assert_eq!(response.user_id, command.user_id.to_string());
        assert_eq!(command.email, "new@example.test");
        assert_eq!(command.username, "new-user");
        assert_eq!(command.display_name, "New User");
        assert_ne!(command.password_hash, "secret-password");
        assert!(verify_password("secret-password", &command.password_hash).unwrap());

        let access = decode_token(&config, &response.access_token, "access").unwrap();
        let refresh = decode_token(&config, &response.refresh_token, "refresh").unwrap();
        assert_eq!(access.sub, command.user_id.to_string());
        assert_eq!(refresh.sub, command.user_id.to_string());
        assert_eq!(access.jti, command.session.session_id.to_string());
        assert_eq!(refresh.jti, command.session.session_id.to_string());
        assert_eq!(
            command.session.access_token_hash,
            hash_token(&response.access_token)
        );
        assert_eq!(
            command.session.refresh_token_hash,
            hash_token(&response.refresh_token)
        );
    }

    #[tokio::test]
    async fn wiki_auth_use_case_rejects_disabled_registration_and_bad_login() {
        let mut disabled_config = test_auth_config();
        disabled_config.registration_enabled = false;
        let repository = recording_auth_repository(None);

        assert!(matches!(
            WikiAuthUseCase::new(&repository, &disabled_config)
                .register(shared::WikiRegisterRequest {
                    email: "new@example.test".to_string(),
                    username: "new-user".to_string(),
                    password: "secret-password".to_string(),
                    name: None,
                })
                .await,
            Err(AppError::Forbidden)
        ));
        assert!(
            repository
                .registered
                .lock()
                .expect("register commands should be lockable")
                .is_empty()
        );

        let config = test_auth_config();
        let inactive_repository = recording_auth_repository(Some(auth_user(
            "editor@example.test",
            "secret-password",
            GlobalRole::User.as_str(),
            false,
        )));
        assert!(matches!(
            WikiAuthUseCase::new(&inactive_repository, &config)
                .login(shared::WikiLoginRequest {
                    email: " editor@example.test ".to_string(),
                    password: "secret-password".to_string(),
                })
                .await,
            Err(AppError::Unauthorized)
        ));

        let active_repository = recording_auth_repository(Some(auth_user(
            "editor@example.test",
            "secret-password",
            GlobalRole::User.as_str(),
            true,
        )));
        assert!(matches!(
            WikiAuthUseCase::new(&active_repository, &config)
                .login(shared::WikiLoginRequest {
                    email: "editor@example.test".to_string(),
                    password: "wrong-password".to_string(),
                })
                .await,
            Err(AppError::Unauthorized)
        ));
        assert!(
            active_repository
                .login_sessions
                .lock()
                .expect("login sessions should be lockable")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn wiki_auth_use_case_handles_session_lifecycle_and_current_user() {
        let config = test_auth_config();
        let user = auth_user(
            "admin@example.test",
            "secret-password",
            GlobalRole::Admin.as_str(),
            true,
        );
        let repository = recording_auth_repository(Some(user.clone()));
        let use_case = WikiAuthUseCase::new(&repository, &config);

        let login = use_case
            .login(shared::WikiLoginRequest {
                email: "  admin@example.test  ".to_string(),
                password: "secret-password".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(login.email, "admin@example.test");
        assert_eq!(
            repository
                .email_lookups
                .lock()
                .expect("email lookups should be lockable")
                .as_slice(),
            ["admin@example.test"]
        );
        let session = {
            let login_sessions = repository
                .login_sessions
                .lock()
                .expect("login sessions should be lockable");
            assert_eq!(login_sessions.len(), 1);
            login_sessions[0].clone()
        };
        assert_eq!(session.user_id, user.id);
        assert_eq!(session.access_token_hash, hash_token(&login.access_token));
        assert_eq!(session.refresh_token_hash, hash_token(&login.refresh_token));

        let claims = use_case
            .authenticate_access_token(&login.access_token)
            .await
            .unwrap();
        assert_eq!(claims.user_id, user.id.to_string());
        let session_id = session.session_id.to_string();
        assert_eq!(claims.session_id.as_deref(), Some(session_id.as_str()));
        assert_eq!(
            repository
                .access_lookups
                .lock()
                .expect("access lookups should be lockable")
                .as_slice(),
            [WikiAccessSessionCommand {
                user_id: user.id,
                session_id: session.session_id,
                access_token_hash: hash_token(&login.access_token),
            }]
        );

        let refreshed = use_case
            .refresh(shared::WikiRefreshRequest {
                refresh_token: login.refresh_token.clone(),
            })
            .await
            .unwrap();
        assert_eq!(refreshed.user_id, user.id.to_string());
        assert_eq!(
            repository
                .refresh_lookups
                .lock()
                .expect("refresh lookups should be lockable")
                .as_slice(),
            [WikiRefreshSessionCommand {
                user_id: user.id,
                session_id: session.session_id,
                refresh_token_hash: hash_token(&login.refresh_token),
            }]
        );
        {
            let rotated_sessions = repository
                .rotated_sessions
                .lock()
                .expect("rotated sessions should be lockable");
            assert_eq!(rotated_sessions.len(), 1);
            assert_eq!(rotated_sessions[0].session_id, session.session_id);
            assert_eq!(rotated_sessions[0].user_id, user.id);
        }

        let me = use_case.current_user(&claims).await.unwrap();
        assert_eq!(me.id, user.id.to_string());
        assert_eq!(me.email, "admin@example.test");
        assert!(me.is_system_admin);
        assert_eq!(
            repository
                .current_user_lookups
                .lock()
                .expect("current user lookups should be lockable")
                .as_slice(),
            [user.id]
        );

        use_case.logout(&claims).await.unwrap();
        assert_eq!(
            repository
                .revoked_sessions
                .lock()
                .expect("revoked sessions should be lockable")
                .as_slice(),
            [WikiLogoutCommand {
                user_id: user.id,
                session_id: Some(session.session_id),
            }]
        );
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
