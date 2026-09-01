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
    #[serde(default)]
    pub token_id: String,
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
            request_id: None,
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

pub type WikiSpaceRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiCreateSpaceCommand {
    pub space_id: Uuid,
    pub key: String,
    pub name: String,
    pub description: String,
    pub owner_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiUpdateSpaceCommand {
    pub space_id: Uuid,
    pub key: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiArchiveSpaceCommand {
    pub space_id: Uuid,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiUpsertSpaceMemberCommand {
    pub space_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiDeleteSpaceMemberCommand {
    pub space_id: Uuid,
    pub user_id: Uuid,
}

pub trait WikiSpaceRepository {
    fn list_spaces<'a>(
        &'a self,
        user_id: Uuid,
    ) -> WikiSpaceRepositoryFuture<'a, Vec<shared::SpaceResponse>>;

    fn create_space<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateSpaceCommand,
    ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse>;

    fn get_space<'a>(
        &'a self,
        key: &'a str,
    ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse>;

    fn update_space<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpdateSpaceCommand,
    ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse>;

    fn archive_space<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiArchiveSpaceCommand,
    ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse>;

    fn list_members<'a>(
        &'a self,
        space_id: Uuid,
    ) -> WikiSpaceRepositoryFuture<'a, Vec<shared::SpaceMemberResponse>>;

    fn upsert_member<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpsertSpaceMemberCommand,
    ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceMemberResponse>;

    fn delete_member<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiDeleteSpaceMemberCommand,
    ) -> WikiSpaceRepositoryFuture<'a, ()>;

    fn get_tree<'a>(
        &'a self,
        space_id: Uuid,
    ) -> WikiSpaceRepositoryFuture<'a, Vec<shared::SpaceTreeNodeResponse>>;
}

pub struct WikiSpaceUseCase<'a, R: WikiSpaceRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiSpaceRepository + ?Sized> WikiSpaceUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn list(&self, user_id: Uuid) -> Result<shared::SpaceListResponse, AppError> {
        Ok(shared::SpaceListResponse {
            spaces: self.repository.list_spaces(user_id).await?,
        })
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        body: shared::CreateSpaceRequest,
    ) -> Result<shared::SpaceResponse, AppError> {
        let command = WikiCreateSpaceCommand {
            space_id: Uuid::now_v7(),
            key: normalize_space_key(&body.key)?,
            name: normalize_required(&body.name, "space name")?,
            description: normalize_space_description(body.description),
            owner_id: actor_id,
        };
        self.repository.create_space(actor_id, command).await
    }

    pub async fn get(&self, space_key: &str) -> Result<shared::SpaceResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        self.repository.get_space(&key).await
    }

    pub async fn update(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        space_key: &str,
        body: shared::UpdateSpaceRequest,
    ) -> Result<shared::SpaceResponse, AppError> {
        let command = WikiUpdateSpaceCommand {
            space_id,
            key: normalize_space_key(space_key)?,
            name: normalize_optional_update_value(body.name.as_deref()),
            description: body.description.map(|value| value.trim().to_string()),
        };
        self.repository.update_space(actor_id, command).await
    }

    pub async fn archive(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        space_key: &str,
    ) -> Result<shared::SpaceResponse, AppError> {
        let command = WikiArchiveSpaceCommand {
            space_id,
            key: normalize_space_key(space_key)?,
        };
        self.repository.archive_space(actor_id, command).await
    }

    pub async fn list_members(
        &self,
        space_id: Uuid,
    ) -> Result<shared::SpaceMemberListResponse, AppError> {
        Ok(shared::SpaceMemberListResponse {
            members: self.repository.list_members(space_id).await?,
        })
    }

    pub async fn upsert_member(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        user_id: Uuid,
        body: shared::UpsertSpaceMemberRequest,
    ) -> Result<shared::SpaceMemberResponse, AppError> {
        let command = WikiUpsertSpaceMemberCommand {
            space_id,
            user_id,
            role: normalize_space_role(&body.role)?.to_string(),
        };
        self.repository.upsert_member(actor_id, command).await
    }

    pub async fn delete_member(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repository
            .delete_member(actor_id, WikiDeleteSpaceMemberCommand { space_id, user_id })
            .await
    }

    pub async fn tree(
        &self,
        space_id: Uuid,
        space_key: &str,
    ) -> Result<shared::SpaceTreeResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        Ok(shared::SpaceTreeResponse {
            space_key: key,
            documents: self.repository.get_tree(space_id).await?,
        })
    }
}

pub type WikiDocumentRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiCreateDocumentCommand {
    pub document_id: Uuid,
    pub space_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub slug: String,
    pub title: String,
    pub document_type: String,
    pub content_markdown: String,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub owner_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiUpdateDocumentDraftCommand {
    pub document_id: Uuid,
    pub title: Option<String>,
    pub content_markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublishDocumentCommand {
    pub document_id: Uuid,
    pub revision_id: Uuid,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiArchiveDocumentCommand {
    pub document_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiMoveDocumentCommand {
    pub document_id: Uuid,
    pub parent_id: Option<Uuid>,
}

pub trait WikiDocumentRepository {
    fn create_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse>;

    fn get_document<'a>(
        &'a self,
        document_id: Uuid,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse>;

    fn update_document_draft<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpdateDocumentDraftCommand,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse>;

    fn publish_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiPublishDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentRevisionResponse>;

    fn archive_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiArchiveDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse>;

    fn move_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiMoveDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse>;

    fn list_revisions<'a>(
        &'a self,
        document_id: Uuid,
    ) -> WikiDocumentRepositoryFuture<'a, Vec<shared::DocumentRevisionResponse>>;

    fn get_revision<'a>(
        &'a self,
        document_id: Uuid,
        revision_id: Uuid,
    ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentRevisionResponse>;
}

pub struct WikiDocumentUseCase<'a, R: WikiDocumentRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiDocumentRepository + ?Sized> WikiDocumentUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        parent_id: Option<Uuid>,
        body: shared::CreateDocumentRequest,
    ) -> Result<shared::DocumentResponse, AppError> {
        let document_id = Uuid::now_v7();
        let title = normalize_required(&body.title, "document title")?;
        let document_type = normalize_document_type(&body.document_type, true)?.to_string();
        let mut slug = body.slug.unwrap_or_else(|| slugify(&title));
        slug = slugify(&slug);
        if slug.is_empty() {
            slug = format!("document-{}", document_id.simple());
            slug.truncate(17);
        }

        let command = WikiCreateDocumentCommand {
            document_id,
            space_id,
            parent_id,
            slug: normalize_slug(&slug)?,
            title,
            document_type,
            content_markdown: body.content_markdown,
            task_key: body
                .task_key
                .map(|value| normalize_task_key(&value))
                .transpose()?,
            phase_key: body
                .phase_key
                .map(|value| normalize_phase_key(&value))
                .transpose()?,
            owner_id: actor_id,
        };
        self.repository.create_document(actor_id, command).await
    }

    pub async fn get(&self, document_id: Uuid) -> Result<shared::DocumentResponse, AppError> {
        self.repository.get_document(document_id).await
    }

    pub async fn update_draft(
        &self,
        actor_id: Uuid,
        document_id: Uuid,
        body: shared::UpdateDocumentDraftRequest,
    ) -> Result<shared::DocumentResponse, AppError> {
        let command = WikiUpdateDocumentDraftCommand {
            document_id,
            title: normalize_optional_update_value(body.title.as_deref()),
            content_markdown: body.content_markdown,
        };
        self.repository
            .update_document_draft(actor_id, command)
            .await
    }

    pub async fn publish(
        &self,
        actor_id: Uuid,
        document_id: Uuid,
        body: shared::PublishDocumentRequest,
    ) -> Result<shared::DocumentRevisionResponse, AppError> {
        let command = WikiPublishDocumentCommand {
            document_id,
            revision_id: Uuid::now_v7(),
            summary: body.summary,
        };
        self.repository.publish_document(actor_id, command).await
    }

    pub async fn archive(
        &self,
        actor_id: Uuid,
        document_id: Uuid,
    ) -> Result<shared::DocumentResponse, AppError> {
        self.repository
            .archive_document(actor_id, WikiArchiveDocumentCommand { document_id })
            .await
    }

    pub async fn move_document(
        &self,
        actor_id: Uuid,
        document_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<shared::DocumentResponse, AppError> {
        if parent_id == Some(document_id) {
            return Err(AppError::invalid_input(
                "document cannot be moved under itself",
            ));
        }
        self.repository
            .move_document(
                actor_id,
                WikiMoveDocumentCommand {
                    document_id,
                    parent_id,
                },
            )
            .await
    }

    pub async fn list_revisions(
        &self,
        document_id: Uuid,
    ) -> Result<shared::DocumentRevisionListResponse, AppError> {
        Ok(shared::DocumentRevisionListResponse {
            revisions: self.repository.list_revisions(document_id).await?,
        })
    }

    pub async fn get_revision(
        &self,
        document_id: Uuid,
        revision_id: Uuid,
    ) -> Result<shared::DocumentRevisionResponse, AppError> {
        self.repository.get_revision(document_id, revision_id).await
    }
}

pub type WikiDossierRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLinkTaskDocumentCommand {
    pub space_id: Uuid,
    pub space_key: String,
    pub task_key: String,
    pub document_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLinkPhaseDocumentCommand {
    pub space_id: Uuid,
    pub space_key: String,
    pub phase_key: String,
    pub document_id: Uuid,
}

pub trait WikiDossierRepository {
    fn list_tasks<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, Vec<shared::TaskPageResponse>>;

    fn get_task<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
        task_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, shared::TaskPageResponse>;

    fn link_task_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiLinkTaskDocumentCommand,
    ) -> WikiDossierRepositoryFuture<'a, shared::TaskPageResponse>;

    fn list_task_documents<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
        task_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, Vec<shared::DocumentResponse>>;

    fn list_task_evidence<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
        task_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, Vec<shared::EvidenceResponse>>;

    fn list_phases<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, Vec<shared::PhasePageResponse>>;

    fn get_phase<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
        phase_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, shared::PhasePageResponse>;

    fn link_phase_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiLinkPhaseDocumentCommand,
    ) -> WikiDossierRepositoryFuture<'a, shared::PhasePageResponse>;

    fn list_phase_documents<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
        phase_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, Vec<shared::DocumentResponse>>;

    fn list_phase_evidence<'a>(
        &'a self,
        space_id: Uuid,
        space_key: &'a str,
        phase_key: &'a str,
    ) -> WikiDossierRepositoryFuture<'a, Vec<shared::EvidenceResponse>>;
}

pub struct WikiDossierUseCase<'a, R: WikiDossierRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiDossierRepository + ?Sized> WikiDossierUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn list_tasks(
        &self,
        space_id: Uuid,
        space_key: &str,
    ) -> Result<shared::TaskPageListResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        Ok(shared::TaskPageListResponse {
            tasks: self.repository.list_tasks(space_id, &key).await?,
        })
    }

    pub async fn get_task(
        &self,
        space_id: Uuid,
        space_key: &str,
        task_key: &str,
    ) -> Result<shared::TaskPageResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        let task_key = normalize_task_key(task_key)?;
        self.repository.get_task(space_id, &key, &task_key).await
    }

    pub async fn link_task_document(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        space_key: &str,
        task_key: &str,
        document_id: Uuid,
    ) -> Result<shared::TaskPageResponse, AppError> {
        let command = WikiLinkTaskDocumentCommand {
            space_id,
            space_key: normalize_space_key(space_key)?,
            task_key: normalize_task_key(task_key)?,
            document_id,
        };
        self.repository.link_task_document(actor_id, command).await
    }

    pub async fn list_task_documents(
        &self,
        space_id: Uuid,
        space_key: &str,
        task_key: &str,
    ) -> Result<shared::DocumentListResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        let task_key = normalize_task_key(task_key)?;
        Ok(shared::DocumentListResponse {
            documents: self
                .repository
                .list_task_documents(space_id, &key, &task_key)
                .await?,
        })
    }

    pub async fn list_task_evidence(
        &self,
        space_id: Uuid,
        space_key: &str,
        task_key: &str,
    ) -> Result<shared::EvidenceListResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        let task_key = normalize_task_key(task_key)?;
        Ok(shared::EvidenceListResponse {
            evidence: self
                .repository
                .list_task_evidence(space_id, &key, &task_key)
                .await?,
        })
    }

    pub async fn list_phases(
        &self,
        space_id: Uuid,
        space_key: &str,
    ) -> Result<shared::PhasePageListResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        Ok(shared::PhasePageListResponse {
            phases: self.repository.list_phases(space_id, &key).await?,
        })
    }

    pub async fn get_phase(
        &self,
        space_id: Uuid,
        space_key: &str,
        phase_key: &str,
    ) -> Result<shared::PhasePageResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        let phase_key = normalize_phase_key(phase_key)?;
        self.repository.get_phase(space_id, &key, &phase_key).await
    }

    pub async fn link_phase_document(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        space_key: &str,
        phase_key: &str,
        document_id: Uuid,
    ) -> Result<shared::PhasePageResponse, AppError> {
        let command = WikiLinkPhaseDocumentCommand {
            space_id,
            space_key: normalize_space_key(space_key)?,
            phase_key: normalize_phase_key(phase_key)?,
            document_id,
        };
        self.repository.link_phase_document(actor_id, command).await
    }

    pub async fn list_phase_documents(
        &self,
        space_id: Uuid,
        space_key: &str,
        phase_key: &str,
    ) -> Result<shared::DocumentListResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        let phase_key = normalize_phase_key(phase_key)?;
        Ok(shared::DocumentListResponse {
            documents: self
                .repository
                .list_phase_documents(space_id, &key, &phase_key)
                .await?,
        })
    }

    pub async fn list_phase_evidence(
        &self,
        space_id: Uuid,
        space_key: &str,
        phase_key: &str,
    ) -> Result<shared::EvidenceListResponse, AppError> {
        let key = normalize_space_key(space_key)?;
        let phase_key = normalize_phase_key(phase_key)?;
        Ok(shared::EvidenceListResponse {
            evidence: self
                .repository
                .list_phase_evidence(space_id, &key, &phase_key)
                .await?,
        })
    }
}

pub type WikiEvidenceRepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiCreateEvidenceCommand {
    pub evidence_id: Uuid,
    pub space_id: Uuid,
    pub document_id: Option<Uuid>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub title: String,
    pub evidence_type: String,
    pub url: Option<String>,
    pub attachment_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiEvidenceQueryCriteria {
    pub space_key: Option<String>,
    pub document_id: Option<Uuid>,
    pub task_key: Option<String>,
    pub phase_key: Option<String>,
    pub access_user_id: Option<Uuid>,
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiUploadAttachmentCommand {
    pub attachment_id: Uuid,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_key: String,
    pub checksum: String,
    pub bytes: Vec<u8>,
}

pub trait WikiEvidenceRepository {
    fn create_evidence<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateEvidenceCommand,
    ) -> WikiEvidenceRepositoryFuture<'a, shared::EvidenceResponse>;

    fn list_evidence<'a>(
        &'a self,
        criteria: &'a WikiEvidenceQueryCriteria,
    ) -> WikiEvidenceRepositoryFuture<'a, Vec<shared::EvidenceResponse>>;

    fn get_evidence<'a>(
        &'a self,
        evidence_id: Uuid,
    ) -> WikiEvidenceRepositoryFuture<'a, shared::EvidenceResponse>;

    fn upload_attachment<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUploadAttachmentCommand,
    ) -> WikiEvidenceRepositoryFuture<'a, shared::AttachmentResponse>;

    fn get_attachment<'a>(
        &'a self,
        attachment_id: Uuid,
    ) -> WikiEvidenceRepositoryFuture<'a, shared::AttachmentResponse>;

    fn download_attachment<'a>(
        &'a self,
        attachment_id: Uuid,
    ) -> WikiEvidenceRepositoryFuture<'a, shared::AttachmentDownloadResponse>;
}

pub struct WikiEvidenceUseCase<'a, R: WikiEvidenceRepository + ?Sized> {
    repository: &'a R,
}

impl<'a, R: WikiEvidenceRepository + ?Sized> WikiEvidenceUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        space_id: Uuid,
        document_id: Option<Uuid>,
        body: shared::CreateEvidenceRequest,
    ) -> Result<shared::EvidenceResponse, AppError> {
        let title = normalize_required(&body.title, "evidence title")?;
        let evidence_type = normalize_evidence_type(&body.evidence_type)?;
        let url_supplied = body.url.is_some();
        let attachment_id_supplied = body.attachment_id.is_some();
        let checksum_supplied = body.checksum.is_some();
        let url = body
            .url
            .as_deref()
            .map(|value| normalize_required(value, "evidence url"))
            .transpose()?;

        match evidence_type {
            "external_url" if url.is_none() || attachment_id_supplied || checksum_supplied => {
                return Err(AppError::invalid_input(
                    "external_url evidence requires url only",
                ));
            }
            "uploaded_file" if !attachment_id_supplied || url_supplied || checksum_supplied => {
                return Err(AppError::invalid_input(
                    "uploaded_file evidence requires attachment_id only",
                ));
            }
            "external_url" | "uploaded_file" => {}
            _ => unreachable!("validated evidence type"),
        }

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
            return Err(AppError::invalid_input(
                "evidence must target a document, task or phase",
            ));
        }

        let command = WikiCreateEvidenceCommand {
            evidence_id: Uuid::now_v7(),
            space_id,
            document_id,
            task_key,
            phase_key,
            title,
            evidence_type: evidence_type.to_string(),
            url,
            attachment_id: body
                .attachment_id
                .as_deref()
                .map(|value| parse_request_uuid(value, "attachment"))
                .transpose()?,
        };
        self.repository.create_evidence(actor_id, command).await
    }

    pub async fn list(
        &self,
        space_key: Option<&str>,
        document_id: Option<Uuid>,
        task_key: Option<&str>,
        phase_key: Option<&str>,
        access_user_id: Option<Uuid>,
        limit: Option<usize>,
    ) -> Result<shared::EvidenceListResponse, AppError> {
        let criteria = WikiEvidenceQueryCriteria {
            space_key: space_key.map(normalize_space_key).transpose()?,
            document_id,
            task_key: task_key.map(normalize_task_key).transpose()?,
            phase_key: phase_key.map(normalize_phase_key).transpose()?,
            access_user_id,
            limit: clamp_limit(limit, 100),
        };
        Ok(shared::EvidenceListResponse {
            evidence: self.repository.list_evidence(&criteria).await?,
        })
    }

    pub async fn get(&self, evidence_id: Uuid) -> Result<shared::EvidenceResponse, AppError> {
        self.repository.get_evidence(evidence_id).await
    }

    pub async fn upload_attachment(
        &self,
        actor_id: Uuid,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
        max_upload_bytes: usize,
    ) -> Result<shared::AttachmentResponse, AppError> {
        if bytes.is_empty() {
            return Err(AppError::invalid_input("file is required"));
        }
        if bytes.len() > max_upload_bytes {
            return Err(AppError::invalid_input("file is too large"));
        }

        let attachment_id = Uuid::now_v7();
        let file_name = normalize_attachment_file_name(&file_name)?;
        let content_type = normalize_required(&content_type, "attachment content type")?;
        let storage_key = format!(
            "attachments/{attachment_id}/{}",
            safe_download_filename(&file_name)
        );
        let command = WikiUploadAttachmentCommand {
            attachment_id,
            file_name,
            content_type,
            size_bytes: bytes.len() as i64,
            storage_key,
            checksum: checksum(&bytes),
            bytes,
        };
        self.repository.upload_attachment(actor_id, command).await
    }

    pub async fn get_attachment(
        &self,
        attachment_id: Uuid,
    ) -> Result<shared::AttachmentResponse, AppError> {
        self.repository.get_attachment(attachment_id).await
    }

    pub async fn download_attachment(
        &self,
        attachment_id: Uuid,
    ) -> Result<shared::AttachmentDownloadResponse, AppError> {
        self.repository.download_attachment(attachment_id).await
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

    pub async fn list_recent(
        &self,
        query: shared::AuditLogQuery,
    ) -> Result<shared::AuditLogResponse, AppError> {
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        Ok(shared::AuditLogResponse {
            entries: self.repository.list_recent_entries(limit).await?,
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

fn normalize_space_description(value: Option<String>) -> String {
    value
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_string()
}

fn parse_token_uuid(value: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| AppError::Unauthorized)
}

fn parse_claim_uuid(value: &str, entity: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| AppError::not_found(entity, value))
}

fn parse_request_uuid(value: &str, entity: &str) -> Result<Uuid, AppError> {
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

pub fn normalize_evidence_space_key(
    space_key: Option<&str>,
    document_space_key: Option<&str>,
) -> Result<String, AppError> {
    space_key
        .or(document_space_key)
        .map(normalize_space_key)
        .transpose()
        .map(|key| key.unwrap_or_else(|| "SDLC".to_string()))
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

pub fn markdown_to_html(markdown: &str) -> String {
    let html = comrak::markdown_to_html(markdown, &comrak::Options::default());
    ammonia::clean(&html)
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
        token_id: Uuid::now_v7().to_string(),
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

pub fn normalize_attachment_file_name(file_name: &str) -> Result<String, AppError> {
    let normalized = normalize_required(file_name, "attachment file name")?;
    if normalized != file_name
        || normalized
            .chars()
            .any(|ch| matches!(ch, '/' | '\\' | '\0') || ch.is_control())
    {
        return Err(AppError::invalid_input("attachment file name is invalid"));
    }
    Ok(normalized)
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

    struct RecordingSpaceRepository {
        spaces: Vec<shared::SpaceResponse>,
        space: shared::SpaceResponse,
        members: Vec<shared::SpaceMemberResponse>,
        tree: Vec<shared::SpaceTreeNodeResponse>,
        listed_users: std::sync::Mutex<Vec<Uuid>>,
        requested_spaces: std::sync::Mutex<Vec<String>>,
        created: std::sync::Mutex<Vec<(Uuid, WikiCreateSpaceCommand)>>,
        updated: std::sync::Mutex<Vec<(Uuid, WikiUpdateSpaceCommand)>>,
        archived: std::sync::Mutex<Vec<(Uuid, WikiArchiveSpaceCommand)>>,
        listed_members: std::sync::Mutex<Vec<Uuid>>,
        upserted_members: std::sync::Mutex<Vec<(Uuid, WikiUpsertSpaceMemberCommand)>>,
        deleted_members: std::sync::Mutex<Vec<(Uuid, WikiDeleteSpaceMemberCommand)>>,
        tree_requests: std::sync::Mutex<Vec<Uuid>>,
    }

    struct RecordingDocumentRepository {
        document: shared::DocumentResponse,
        revision: shared::DocumentRevisionResponse,
        revisions: Vec<shared::DocumentRevisionResponse>,
        created: std::sync::Mutex<Vec<(Uuid, WikiCreateDocumentCommand)>>,
        requested_documents: std::sync::Mutex<Vec<Uuid>>,
        updated_drafts: std::sync::Mutex<Vec<(Uuid, WikiUpdateDocumentDraftCommand)>>,
        published: std::sync::Mutex<Vec<(Uuid, WikiPublishDocumentCommand)>>,
        archived: std::sync::Mutex<Vec<(Uuid, WikiArchiveDocumentCommand)>>,
        moved: std::sync::Mutex<Vec<(Uuid, WikiMoveDocumentCommand)>>,
        listed_revisions: std::sync::Mutex<Vec<Uuid>>,
        requested_revisions: std::sync::Mutex<Vec<(Uuid, Uuid)>>,
    }

    struct RecordingDossierRepository {
        task: shared::TaskPageResponse,
        phase: shared::PhasePageResponse,
        document: shared::DocumentResponse,
        evidence: shared::EvidenceResponse,
        listed_tasks: std::sync::Mutex<Vec<(Uuid, String)>>,
        requested_tasks: std::sync::Mutex<Vec<(Uuid, String, String)>>,
        linked_tasks: std::sync::Mutex<Vec<(Uuid, WikiLinkTaskDocumentCommand)>>,
        listed_task_documents: std::sync::Mutex<Vec<(Uuid, String, String)>>,
        listed_task_evidence: std::sync::Mutex<Vec<(Uuid, String, String)>>,
        listed_phases: std::sync::Mutex<Vec<(Uuid, String)>>,
        requested_phases: std::sync::Mutex<Vec<(Uuid, String, String)>>,
        linked_phases: std::sync::Mutex<Vec<(Uuid, WikiLinkPhaseDocumentCommand)>>,
        listed_phase_documents: std::sync::Mutex<Vec<(Uuid, String, String)>>,
        listed_phase_evidence: std::sync::Mutex<Vec<(Uuid, String, String)>>,
    }

    struct RecordingEvidenceRepository {
        evidence: shared::EvidenceResponse,
        attachment: shared::AttachmentResponse,
        download: shared::AttachmentDownloadResponse,
        created: std::sync::Mutex<Vec<(Uuid, WikiCreateEvidenceCommand)>>,
        listed: std::sync::Mutex<Vec<WikiEvidenceQueryCriteria>>,
        requested_evidence: std::sync::Mutex<Vec<Uuid>>,
        uploaded: std::sync::Mutex<Vec<(Uuid, WikiUploadAttachmentCommand)>>,
        requested_attachments: std::sync::Mutex<Vec<Uuid>>,
        downloaded_attachments: std::sync::Mutex<Vec<Uuid>>,
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

    impl WikiSpaceRepository for RecordingSpaceRepository {
        fn list_spaces<'a>(
            &'a self,
            user_id: Uuid,
        ) -> WikiSpaceRepositoryFuture<'a, Vec<shared::SpaceResponse>> {
            Box::pin(async move {
                self.listed_users
                    .lock()
                    .expect("listed users should be lockable")
                    .push(user_id);
                Ok(self.spaces.clone())
            })
        }

        fn create_space<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiCreateSpaceCommand,
        ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse> {
            Box::pin(async move {
                self.created
                    .lock()
                    .expect("space create commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(space_response(&command.key, &command.name, Some(actor_id)))
            })
        }

        fn get_space<'a>(
            &'a self,
            key: &'a str,
        ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse> {
            Box::pin(async move {
                self.requested_spaces
                    .lock()
                    .expect("space requests should be lockable")
                    .push(key.to_string());
                Ok(self.space.clone())
            })
        }

        fn update_space<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiUpdateSpaceCommand,
        ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse> {
            Box::pin(async move {
                self.updated
                    .lock()
                    .expect("space update commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(space_response(
                    &command.key,
                    command.name.as_deref().unwrap_or("Space"),
                    Some(actor_id),
                ))
            })
        }

        fn archive_space<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiArchiveSpaceCommand,
        ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceResponse> {
            Box::pin(async move {
                self.archived
                    .lock()
                    .expect("space archive commands should be lockable")
                    .push((actor_id, command.clone()));
                let mut response = space_response(&command.key, "Space", Some(actor_id));
                response.status = "archived".to_string();
                Ok(response)
            })
        }

        fn list_members<'a>(
            &'a self,
            space_id: Uuid,
        ) -> WikiSpaceRepositoryFuture<'a, Vec<shared::SpaceMemberResponse>> {
            Box::pin(async move {
                self.listed_members
                    .lock()
                    .expect("listed member spaces should be lockable")
                    .push(space_id);
                Ok(self.members.clone())
            })
        }

        fn upsert_member<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiUpsertSpaceMemberCommand,
        ) -> WikiSpaceRepositoryFuture<'a, shared::SpaceMemberResponse> {
            Box::pin(async move {
                self.upserted_members
                    .lock()
                    .expect("space member upserts should be lockable")
                    .push((actor_id, command.clone()));
                Ok(space_member_response(command.user_id, &command.role))
            })
        }

        fn delete_member<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiDeleteSpaceMemberCommand,
        ) -> WikiSpaceRepositoryFuture<'a, ()> {
            Box::pin(async move {
                self.deleted_members
                    .lock()
                    .expect("space member deletes should be lockable")
                    .push((actor_id, command));
                Ok(())
            })
        }

        fn get_tree<'a>(
            &'a self,
            space_id: Uuid,
        ) -> WikiSpaceRepositoryFuture<'a, Vec<shared::SpaceTreeNodeResponse>> {
            Box::pin(async move {
                self.tree_requests
                    .lock()
                    .expect("tree requests should be lockable")
                    .push(space_id);
                Ok(self.tree.clone())
            })
        }
    }

    impl WikiDocumentRepository for RecordingDocumentRepository {
        fn create_document<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiCreateDocumentCommand,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse> {
            Box::pin(async move {
                self.created
                    .lock()
                    .expect("document create commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(document_response(
                    command.document_id,
                    &command.slug,
                    &command.title,
                    None,
                ))
            })
        }

        fn get_document<'a>(
            &'a self,
            document_id: Uuid,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse> {
            Box::pin(async move {
                self.requested_documents
                    .lock()
                    .expect("document requests should be lockable")
                    .push(document_id);
                Ok(self.document.clone())
            })
        }

        fn update_document_draft<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiUpdateDocumentDraftCommand,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse> {
            Box::pin(async move {
                self.updated_drafts
                    .lock()
                    .expect("document draft commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(document_response(
                    command.document_id,
                    "updated",
                    command.title.as_deref().unwrap_or("Updated"),
                    None,
                ))
            })
        }

        fn publish_document<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiPublishDocumentCommand,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentRevisionResponse> {
            Box::pin(async move {
                self.published
                    .lock()
                    .expect("document publish commands should be lockable")
                    .push((actor_id, command.clone()));
                Ok(document_revision_response(
                    command.document_id,
                    command.revision_id,
                    1,
                    "Published",
                ))
            })
        }

        fn archive_document<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiArchiveDocumentCommand,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse> {
            Box::pin(async move {
                self.archived
                    .lock()
                    .expect("document archive commands should be lockable")
                    .push((actor_id, command.clone()));
                let mut response =
                    document_response(command.document_id, "archived", "Archived", None);
                response.status = "archived".to_string();
                Ok(response)
            })
        }

        fn move_document<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiMoveDocumentCommand,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentResponse> {
            Box::pin(async move {
                self.moved
                    .lock()
                    .expect("document move commands should be lockable")
                    .push((actor_id, command.clone()));
                let mut response = document_response(command.document_id, "moved", "Moved", None);
                response.parent_id = command.parent_id.map(|id| id.to_string());
                Ok(response)
            })
        }

        fn list_revisions<'a>(
            &'a self,
            document_id: Uuid,
        ) -> WikiDocumentRepositoryFuture<'a, Vec<shared::DocumentRevisionResponse>> {
            Box::pin(async move {
                self.listed_revisions
                    .lock()
                    .expect("listed revision documents should be lockable")
                    .push(document_id);
                Ok(self.revisions.clone())
            })
        }

        fn get_revision<'a>(
            &'a self,
            document_id: Uuid,
            revision_id: Uuid,
        ) -> WikiDocumentRepositoryFuture<'a, shared::DocumentRevisionResponse> {
            Box::pin(async move {
                self.requested_revisions
                    .lock()
                    .expect("requested revisions should be lockable")
                    .push((document_id, revision_id));
                Ok(self.revision.clone())
            })
        }
    }

    impl WikiDossierRepository for RecordingDossierRepository {
        fn list_tasks<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, Vec<shared::TaskPageResponse>> {
            Box::pin(async move {
                self.listed_tasks
                    .lock()
                    .expect("listed tasks should be lockable")
                    .push((space_id, space_key.to_string()));
                Ok(vec![self.task.clone()])
            })
        }

        fn get_task<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
            task_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, shared::TaskPageResponse> {
            Box::pin(async move {
                self.requested_tasks
                    .lock()
                    .expect("task requests should be lockable")
                    .push((space_id, space_key.to_string(), task_key.to_string()));
                Ok(self.task.clone())
            })
        }

        fn link_task_document<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiLinkTaskDocumentCommand,
        ) -> WikiDossierRepositoryFuture<'a, shared::TaskPageResponse> {
            Box::pin(async move {
                self.linked_tasks
                    .lock()
                    .expect("task link commands should be lockable")
                    .push((actor_id, command));
                Ok(self.task.clone())
            })
        }

        fn list_task_documents<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
            task_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, Vec<shared::DocumentResponse>> {
            Box::pin(async move {
                self.listed_task_documents
                    .lock()
                    .expect("listed task documents should be lockable")
                    .push((space_id, space_key.to_string(), task_key.to_string()));
                Ok(vec![self.document.clone()])
            })
        }

        fn list_task_evidence<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
            task_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, Vec<shared::EvidenceResponse>> {
            Box::pin(async move {
                self.listed_task_evidence
                    .lock()
                    .expect("listed task evidence should be lockable")
                    .push((space_id, space_key.to_string(), task_key.to_string()));
                Ok(vec![self.evidence.clone()])
            })
        }

        fn list_phases<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, Vec<shared::PhasePageResponse>> {
            Box::pin(async move {
                self.listed_phases
                    .lock()
                    .expect("listed phases should be lockable")
                    .push((space_id, space_key.to_string()));
                Ok(vec![self.phase.clone()])
            })
        }

        fn get_phase<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
            phase_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, shared::PhasePageResponse> {
            Box::pin(async move {
                self.requested_phases
                    .lock()
                    .expect("phase requests should be lockable")
                    .push((space_id, space_key.to_string(), phase_key.to_string()));
                Ok(self.phase.clone())
            })
        }

        fn link_phase_document<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiLinkPhaseDocumentCommand,
        ) -> WikiDossierRepositoryFuture<'a, shared::PhasePageResponse> {
            Box::pin(async move {
                self.linked_phases
                    .lock()
                    .expect("phase link commands should be lockable")
                    .push((actor_id, command));
                Ok(self.phase.clone())
            })
        }

        fn list_phase_documents<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
            phase_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, Vec<shared::DocumentResponse>> {
            Box::pin(async move {
                self.listed_phase_documents
                    .lock()
                    .expect("listed phase documents should be lockable")
                    .push((space_id, space_key.to_string(), phase_key.to_string()));
                Ok(vec![self.document.clone()])
            })
        }

        fn list_phase_evidence<'a>(
            &'a self,
            space_id: Uuid,
            space_key: &'a str,
            phase_key: &'a str,
        ) -> WikiDossierRepositoryFuture<'a, Vec<shared::EvidenceResponse>> {
            Box::pin(async move {
                self.listed_phase_evidence
                    .lock()
                    .expect("listed phase evidence should be lockable")
                    .push((space_id, space_key.to_string(), phase_key.to_string()));
                Ok(vec![self.evidence.clone()])
            })
        }
    }

    impl WikiEvidenceRepository for RecordingEvidenceRepository {
        fn create_evidence<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiCreateEvidenceCommand,
        ) -> WikiEvidenceRepositoryFuture<'a, shared::EvidenceResponse> {
            Box::pin(async move {
                self.created
                    .lock()
                    .expect("evidence create commands should be lockable")
                    .push((actor_id, command));
                Ok(self.evidence.clone())
            })
        }

        fn list_evidence<'a>(
            &'a self,
            criteria: &'a WikiEvidenceQueryCriteria,
        ) -> WikiEvidenceRepositoryFuture<'a, Vec<shared::EvidenceResponse>> {
            Box::pin(async move {
                self.listed
                    .lock()
                    .expect("evidence list criteria should be lockable")
                    .push(criteria.clone());
                Ok(vec![self.evidence.clone()])
            })
        }

        fn get_evidence<'a>(
            &'a self,
            evidence_id: Uuid,
        ) -> WikiEvidenceRepositoryFuture<'a, shared::EvidenceResponse> {
            Box::pin(async move {
                self.requested_evidence
                    .lock()
                    .expect("evidence requests should be lockable")
                    .push(evidence_id);
                Ok(self.evidence.clone())
            })
        }

        fn upload_attachment<'a>(
            &'a self,
            actor_id: Uuid,
            command: WikiUploadAttachmentCommand,
        ) -> WikiEvidenceRepositoryFuture<'a, shared::AttachmentResponse> {
            Box::pin(async move {
                self.uploaded
                    .lock()
                    .expect("attachment upload commands should be lockable")
                    .push((actor_id, command));
                Ok(self.attachment.clone())
            })
        }

        fn get_attachment<'a>(
            &'a self,
            attachment_id: Uuid,
        ) -> WikiEvidenceRepositoryFuture<'a, shared::AttachmentResponse> {
            Box::pin(async move {
                self.requested_attachments
                    .lock()
                    .expect("attachment requests should be lockable")
                    .push(attachment_id);
                Ok(self.attachment.clone())
            })
        }

        fn download_attachment<'a>(
            &'a self,
            attachment_id: Uuid,
        ) -> WikiEvidenceRepositoryFuture<'a, shared::AttachmentDownloadResponse> {
            Box::pin(async move {
                self.downloaded_attachments
                    .lock()
                    .expect("attachment download requests should be lockable")
                    .push(attachment_id);
                Ok(self.download.clone())
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
            request_id: "test-request".to_string(),
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

    fn space_response(key: &str, name: &str, owner_id: Option<Uuid>) -> shared::SpaceResponse {
        shared::SpaceResponse {
            id: Uuid::now_v7().to_string(),
            key: key.to_string(),
            name: name.to_string(),
            description: Some("Base space".to_string()),
            owner_id: owner_id.unwrap_or_else(Uuid::now_v7).to_string(),
            status: "active".to_string(),
            document_count: 2,
            member_count: 3,
            created_at: "2026-09-01T10:00:00Z".to_string(),
            updated_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn space_member_response(user_id: Uuid, role: &str) -> shared::SpaceMemberResponse {
        shared::SpaceMemberResponse {
            user_id: user_id.to_string(),
            email: "member@example.test".to_string(),
            display_name: "Member".to_string(),
            role: role.to_string(),
            joined_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn space_tree_node(title: &str) -> shared::SpaceTreeNodeResponse {
        shared::SpaceTreeNodeResponse {
            id: Uuid::now_v7().to_string(),
            slug: slugify(title),
            title: title.to_string(),
            document_type: "page".to_string(),
            status: "published".to_string(),
            children: Vec::new(),
        }
    }

    fn document_revision_response(
        document_id: Uuid,
        revision_id: Uuid,
        version: u32,
        title: &str,
    ) -> shared::DocumentRevisionResponse {
        shared::DocumentRevisionResponse {
            id: revision_id.to_string(),
            document_id: document_id.to_string(),
            version,
            title: title.to_string(),
            body_markdown: "# Published".to_string(),
            body_html: "<h1>Published</h1>\n".to_string(),
            summary: Some("Initial publish".to_string()),
            author_id: Uuid::now_v7().to_string(),
            published_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn document_response(
        document_id: Uuid,
        slug: &str,
        title: &str,
        current_revision: Option<shared::DocumentRevisionResponse>,
    ) -> shared::DocumentResponse {
        shared::DocumentResponse {
            id: document_id.to_string(),
            space_key: "SDLC".to_string(),
            parent_id: None,
            slug: slug.to_string(),
            title: title.to_string(),
            document_type: "requirements".to_string(),
            status: if current_revision.is_some() {
                "published".to_string()
            } else {
                "draft".to_string()
            },
            body_markdown: current_revision
                .as_ref()
                .map(|revision| revision.body_markdown.clone())
                .unwrap_or_default(),
            body_html: current_revision
                .as_ref()
                .map(|revision| revision.body_html.clone())
                .unwrap_or_default(),
            draft_markdown: "# Draft".to_string(),
            current_revision,
            task_keys: vec!["SDLC-42".to_string()],
            phase_keys: vec!["implementation".to_string()],
            evidence: Vec::new(),
            created_by: Uuid::now_v7().to_string(),
            updated_by: Uuid::now_v7().to_string(),
            created_at: "2026-09-01T10:00:00Z".to_string(),
            updated_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn document_summary(
        document_id: Uuid,
        slug: &str,
        title: &str,
    ) -> shared::DocumentSummaryResponse {
        shared::DocumentSummaryResponse {
            id: document_id.to_string(),
            slug: slug.to_string(),
            title: title.to_string(),
            document_type: "requirements".to_string(),
            status: "published".to_string(),
            updated_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn evidence_response(title: &str) -> shared::EvidenceResponse {
        shared::EvidenceResponse {
            id: Uuid::now_v7().to_string(),
            space_key: "SDLC".to_string(),
            document_id: None,
            task_key: Some("SDLC-42".to_string()),
            phase_key: Some("implementation".to_string()),
            title: title.to_string(),
            evidence_type: "external_url".to_string(),
            url: Some("https://ci.local/jobs/42".to_string()),
            attachment_id: None,
            checksum: None,
            created_by: Uuid::now_v7().to_string(),
            created_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn attachment_response(attachment_id: Uuid, checksum: &str) -> shared::AttachmentResponse {
        shared::AttachmentResponse {
            id: attachment_id.to_string(),
            file_name: "build.log".to_string(),
            content_type: "text/plain".to_string(),
            size_bytes: 9,
            checksum: checksum.to_string(),
            uploaded_by: Uuid::now_v7().to_string(),
            uploaded_at: "2026-09-01T10:00:00Z".to_string(),
        }
    }

    fn attachment_download_response(bytes: Vec<u8>) -> shared::AttachmentDownloadResponse {
        shared::AttachmentDownloadResponse {
            file_name: "build.log".to_string(),
            content_type: "text/plain".to_string(),
            bytes,
        }
    }

    fn task_page_response(document_id: Uuid) -> shared::TaskPageResponse {
        let evidence = vec![evidence_response("Task evidence")];
        let documents = vec![document_summary(
            document_id,
            "requirements",
            "Requirements",
        )];
        shared::TaskPageResponse {
            space_key: "SDLC".to_string(),
            task_key: "SDLC-42".to_string(),
            title: Some("Requirements".to_string()),
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        }
    }

    fn phase_page_response(document_id: Uuid) -> shared::PhasePageResponse {
        let evidence = vec![evidence_response("Phase evidence")];
        let documents = vec![document_summary(
            document_id,
            "implementation",
            "Implementation",
        )];
        shared::PhasePageResponse {
            space_key: "SDLC".to_string(),
            phase_key: "implementation".to_string(),
            title: Some("implementation".to_string()),
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        }
    }

    fn recording_document_repository() -> RecordingDocumentRepository {
        let document_id = Uuid::now_v7();
        let revision_id = Uuid::now_v7();
        let revision = document_revision_response(document_id, revision_id, 1, "Requirements");
        RecordingDocumentRepository {
            document: document_response(
                document_id,
                "requirements",
                "Requirements",
                Some(revision.clone()),
            ),
            revision: revision.clone(),
            revisions: vec![revision],
            created: std::sync::Mutex::new(Vec::new()),
            requested_documents: std::sync::Mutex::new(Vec::new()),
            updated_drafts: std::sync::Mutex::new(Vec::new()),
            published: std::sync::Mutex::new(Vec::new()),
            archived: std::sync::Mutex::new(Vec::new()),
            moved: std::sync::Mutex::new(Vec::new()),
            listed_revisions: std::sync::Mutex::new(Vec::new()),
            requested_revisions: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn recording_dossier_repository() -> RecordingDossierRepository {
        let document_id = Uuid::now_v7();
        RecordingDossierRepository {
            task: task_page_response(document_id),
            phase: phase_page_response(document_id),
            document: document_response(document_id, "requirements", "Requirements", None),
            evidence: evidence_response("Evidence"),
            listed_tasks: std::sync::Mutex::new(Vec::new()),
            requested_tasks: std::sync::Mutex::new(Vec::new()),
            linked_tasks: std::sync::Mutex::new(Vec::new()),
            listed_task_documents: std::sync::Mutex::new(Vec::new()),
            listed_task_evidence: std::sync::Mutex::new(Vec::new()),
            listed_phases: std::sync::Mutex::new(Vec::new()),
            requested_phases: std::sync::Mutex::new(Vec::new()),
            linked_phases: std::sync::Mutex::new(Vec::new()),
            listed_phase_documents: std::sync::Mutex::new(Vec::new()),
            listed_phase_evidence: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn recording_evidence_repository() -> RecordingEvidenceRepository {
        let attachment_id = Uuid::now_v7();
        RecordingEvidenceRepository {
            evidence: evidence_response("Evidence"),
            attachment: attachment_response(attachment_id, "sha256-test"),
            download: attachment_download_response(b"build log".to_vec()),
            created: std::sync::Mutex::new(Vec::new()),
            listed: std::sync::Mutex::new(Vec::new()),
            requested_evidence: std::sync::Mutex::new(Vec::new()),
            uploaded: std::sync::Mutex::new(Vec::new()),
            requested_attachments: std::sync::Mutex::new(Vec::new()),
            downloaded_attachments: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn recording_space_repository() -> RecordingSpaceRepository {
        let member_id = Uuid::now_v7();
        RecordingSpaceRepository {
            spaces: vec![space_response("SDLC", "SDLC Wiki", None)],
            space: space_response("SDLC", "SDLC Wiki", None),
            members: vec![space_member_response(member_id, "editor")],
            tree: vec![space_tree_node("Requirements")],
            listed_users: std::sync::Mutex::new(Vec::new()),
            requested_spaces: std::sync::Mutex::new(Vec::new()),
            created: std::sync::Mutex::new(Vec::new()),
            updated: std::sync::Mutex::new(Vec::new()),
            archived: std::sync::Mutex::new(Vec::new()),
            listed_members: std::sync::Mutex::new(Vec::new()),
            upserted_members: std::sync::Mutex::new(Vec::new()),
            deleted_members: std::sync::Mutex::new(Vec::new()),
            tree_requests: std::sync::Mutex::new(Vec::new()),
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
    fn wiki_helpers_keep_supported_global_roles_closed() {
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

    #[tokio::test]
    async fn wiki_space_use_case_lists_gets_members_and_tree() {
        let repository = recording_space_repository();
        let use_case = WikiSpaceUseCase::new(&repository);
        let user_id = Uuid::now_v7();
        let space_id = Uuid::now_v7();

        let list = use_case.list(user_id).await.unwrap();
        assert_eq!(list.spaces.len(), 1);
        assert_eq!(list.spaces[0].key, "SDLC");
        assert_eq!(
            repository
                .listed_users
                .lock()
                .expect("listed users should be lockable")
                .as_slice(),
            [user_id]
        );

        let space = use_case.get(" sdlc ").await.unwrap();
        assert_eq!(space.key, "SDLC");
        assert_eq!(
            repository
                .requested_spaces
                .lock()
                .expect("space requests should be lockable")
                .as_slice(),
            ["SDLC"]
        );

        let members = use_case.list_members(space_id).await.unwrap();
        assert_eq!(members.members.len(), 1);
        assert_eq!(members.members[0].role, "editor");
        assert_eq!(
            repository
                .listed_members
                .lock()
                .expect("listed member spaces should be lockable")
                .as_slice(),
            [space_id]
        );

        let tree = use_case.tree(space_id, " sdlc ").await.unwrap();
        assert_eq!(tree.space_key, "SDLC");
        assert_eq!(tree.documents.len(), 1);
        assert_eq!(tree.documents[0].title, "Requirements");
        assert_eq!(
            repository
                .tree_requests
                .lock()
                .expect("tree requests should be lockable")
                .as_slice(),
            [space_id]
        );
    }

    #[tokio::test]
    async fn wiki_space_use_case_normalizes_write_commands() {
        let repository = recording_space_repository();
        let use_case = WikiSpaceUseCase::new(&repository);
        let actor_id = Uuid::now_v7();
        let space_id = Uuid::now_v7();
        let user_id = Uuid::now_v7();

        let created = use_case
            .create(
                actor_id,
                shared::CreateSpaceRequest {
                    key: " sdlc ".to_string(),
                    name: " SDLC Wiki ".to_string(),
                    description: Some(" Base docs ".to_string()),
                },
            )
            .await
            .unwrap();
        assert_eq!(created.key, "SDLC");
        assert_eq!(created.name, "SDLC Wiki");
        {
            let created_commands = repository
                .created
                .lock()
                .expect("space create commands should be lockable");
            assert_eq!(created_commands.len(), 1);
            let create_command = &created_commands[0].1;
            assert_eq!(created_commands[0].0, actor_id);
            assert_eq!(create_command.key, "SDLC");
            assert_eq!(create_command.name, "SDLC Wiki");
            assert_eq!(create_command.description, "Base docs");
            assert_eq!(create_command.owner_id, actor_id);
        }

        use_case
            .update(
                actor_id,
                space_id,
                " sdlc ",
                shared::UpdateSpaceRequest {
                    name: Some("   ".to_string()),
                    description: Some("  cleared  ".to_string()),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            repository
                .updated
                .lock()
                .expect("space update commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiUpdateSpaceCommand {
                    space_id,
                    key: "SDLC".to_string(),
                    name: None,
                    description: Some("cleared".to_string()),
                }
            )]
        );

        use_case.archive(actor_id, space_id, "sdlc").await.unwrap();
        assert_eq!(
            repository
                .archived
                .lock()
                .expect("space archive commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiArchiveSpaceCommand {
                    space_id,
                    key: "SDLC".to_string(),
                }
            )]
        );

        let member = use_case
            .upsert_member(
                actor_id,
                space_id,
                user_id,
                shared::UpsertSpaceMemberRequest {
                    role: " editor ".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(member.role, "editor");
        assert_eq!(
            repository
                .upserted_members
                .lock()
                .expect("space member upserts should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiUpsertSpaceMemberCommand {
                    space_id,
                    user_id,
                    role: "editor".to_string(),
                }
            )]
        );

        assert!(
            use_case
                .upsert_member(
                    actor_id,
                    space_id,
                    user_id,
                    shared::UpsertSpaceMemberRequest {
                        role: "owner".to_string(),
                    },
                )
                .await
                .is_err()
        );

        use_case
            .delete_member(actor_id, space_id, user_id)
            .await
            .unwrap();
        assert_eq!(
            repository
                .deleted_members
                .lock()
                .expect("space member deletes should be lockable")
                .as_slice(),
            [(actor_id, WikiDeleteSpaceMemberCommand { space_id, user_id })]
        );

        assert!(
            use_case
                .create(
                    actor_id,
                    shared::CreateSpaceRequest {
                        key: "bad key".to_string(),
                        name: "Space".to_string(),
                        description: None,
                    },
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn wiki_document_use_case_reads_document_and_revisions() {
        let repository = recording_document_repository();
        let use_case = WikiDocumentUseCase::new(&repository);
        let document_id = Uuid::now_v7();
        let revision_id = Uuid::now_v7();

        let document = use_case.get(document_id).await.unwrap();
        assert_eq!(document.title, "Requirements");
        assert_eq!(
            repository
                .requested_documents
                .lock()
                .expect("document requests should be lockable")
                .as_slice(),
            [document_id]
        );

        let revisions = use_case.list_revisions(document_id).await.unwrap();
        assert_eq!(revisions.revisions.len(), 1);
        assert_eq!(revisions.revisions[0].version, 1);
        assert_eq!(
            repository
                .listed_revisions
                .lock()
                .expect("listed revision documents should be lockable")
                .as_slice(),
            [document_id]
        );

        let revision = use_case
            .get_revision(document_id, revision_id)
            .await
            .unwrap();
        assert_eq!(revision.title, "Requirements");
        assert_eq!(
            repository
                .requested_revisions
                .lock()
                .expect("requested revisions should be lockable")
                .as_slice(),
            [(document_id, revision_id)]
        );
    }

    #[tokio::test]
    async fn wiki_document_use_case_normalizes_write_commands() {
        let repository = recording_document_repository();
        let use_case = WikiDocumentUseCase::new(&repository);
        let actor_id = Uuid::now_v7();
        let space_id = Uuid::now_v7();
        let parent_id = Uuid::now_v7();
        let document_id = Uuid::now_v7();
        let next_parent_id = Uuid::now_v7();

        let created = use_case
            .create(
                actor_id,
                space_id,
                Some(parent_id),
                shared::CreateDocumentRequest {
                    title: " Product Requirements ".to_string(),
                    slug: Some(" Product Requirements! ".to_string()),
                    document_type: "requirements".to_string(),
                    parent_id: None,
                    content_markdown: "# Draft".to_string(),
                    task_key: Some("SDLC-42".to_string()),
                    phase_key: Some("Implementation".to_string()),
                },
            )
            .await
            .unwrap();
        assert_eq!(created.slug, "product-requirements");
        assert_eq!(created.title, "Product Requirements");
        {
            let created_commands = repository
                .created
                .lock()
                .expect("document create commands should be lockable");
            assert_eq!(created_commands.len(), 1);
            let command = &created_commands[0].1;
            assert_eq!(created_commands[0].0, actor_id);
            assert_eq!(command.space_id, space_id);
            assert_eq!(command.parent_id, Some(parent_id));
            assert_eq!(command.slug, "product-requirements");
            assert_eq!(command.title, "Product Requirements");
            assert_eq!(command.document_type, "requirements");
            assert_eq!(command.content_markdown, "# Draft");
            assert_eq!(command.task_key.as_deref(), Some("SDLC-42"));
            assert_eq!(command.phase_key.as_deref(), Some("implementation"));
            assert_eq!(command.owner_id, actor_id);
        }

        let fallback = use_case
            .create(
                actor_id,
                space_id,
                None,
                shared::CreateDocumentRequest {
                    title: "!!!".to_string(),
                    slug: None,
                    document_type: "page".to_string(),
                    parent_id: None,
                    content_markdown: String::new(),
                    task_key: None,
                    phase_key: None,
                },
            )
            .await
            .unwrap();
        assert!(fallback.slug.starts_with("document-"));
        assert_eq!(fallback.slug.len(), 17);

        use_case
            .update_draft(
                actor_id,
                document_id,
                shared::UpdateDocumentDraftRequest {
                    title: Some("   ".to_string()),
                    content_markdown: "# Updated".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            repository
                .updated_drafts
                .lock()
                .expect("document draft commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiUpdateDocumentDraftCommand {
                    document_id,
                    title: None,
                    content_markdown: "# Updated".to_string(),
                }
            )]
        );

        let revision = use_case
            .publish(
                actor_id,
                document_id,
                shared::PublishDocumentRequest {
                    summary: Some(" Publish summary ".to_string()),
                },
            )
            .await
            .unwrap();
        assert_eq!(revision.document_id, document_id.to_string());
        {
            let published = repository
                .published
                .lock()
                .expect("document publish commands should be lockable");
            assert_eq!(published.len(), 1);
            let command = &published[0].1;
            assert_eq!(published[0].0, actor_id);
            assert_eq!(command.document_id, document_id);
            assert_ne!(command.revision_id, Uuid::nil());
            assert_eq!(command.summary.as_deref(), Some(" Publish summary "));
        }

        use_case.archive(actor_id, document_id).await.unwrap();
        assert_eq!(
            repository
                .archived
                .lock()
                .expect("document archive commands should be lockable")
                .as_slice(),
            [(actor_id, WikiArchiveDocumentCommand { document_id })]
        );

        use_case
            .move_document(actor_id, document_id, Some(next_parent_id))
            .await
            .unwrap();
        assert_eq!(
            repository
                .moved
                .lock()
                .expect("document move commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiMoveDocumentCommand {
                    document_id,
                    parent_id: Some(next_parent_id),
                }
            )]
        );

        assert!(
            use_case
                .move_document(actor_id, document_id, Some(document_id))
                .await
                .is_err()
        );
        assert!(
            use_case
                .create(
                    actor_id,
                    space_id,
                    None,
                    shared::CreateDocumentRequest {
                        title: "Doc".to_string(),
                        slug: None,
                        document_type: "unsupported".to_string(),
                        parent_id: None,
                        content_markdown: String::new(),
                        task_key: None,
                        phase_key: None,
                    },
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn wiki_dossier_use_case_normalizes_task_reads_and_links() {
        let repository = recording_dossier_repository();
        let use_case = WikiDossierUseCase::new(&repository);
        let actor_id = Uuid::now_v7();
        let space_id = Uuid::now_v7();
        let document_id = Uuid::now_v7();

        let tasks = use_case.list_tasks(space_id, " sdlc ").await.unwrap();
        assert_eq!(tasks.tasks.len(), 1);
        assert_eq!(
            repository
                .listed_tasks
                .lock()
                .expect("listed tasks should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string())]
        );

        let task = use_case
            .get_task(space_id, "sdlc", " SDLC-42 ")
            .await
            .unwrap();
        assert_eq!(task.task_key, "SDLC-42");
        assert_eq!(
            repository
                .requested_tasks
                .lock()
                .expect("task requests should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string(), "SDLC-42".to_string())]
        );

        let documents = use_case
            .list_task_documents(space_id, "sdlc", "SDLC-42")
            .await
            .unwrap();
        assert_eq!(documents.documents.len(), 1);
        assert_eq!(
            repository
                .listed_task_documents
                .lock()
                .expect("listed task documents should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string(), "SDLC-42".to_string())]
        );

        let evidence = use_case
            .list_task_evidence(space_id, "sdlc", "SDLC-42")
            .await
            .unwrap();
        assert_eq!(evidence.evidence.len(), 1);
        assert_eq!(
            repository
                .listed_task_evidence
                .lock()
                .expect("listed task evidence should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string(), "SDLC-42".to_string())]
        );

        use_case
            .link_task_document(actor_id, space_id, "sdlc", " SDLC-42 ", document_id)
            .await
            .unwrap();
        assert_eq!(
            repository
                .linked_tasks
                .lock()
                .expect("task link commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiLinkTaskDocumentCommand {
                    space_id,
                    space_key: "SDLC".to_string(),
                    task_key: "SDLC-42".to_string(),
                    document_id,
                }
            )]
        );

        assert!(
            use_case
                .get_task(space_id, "bad space", "SDLC-42")
                .await
                .is_err()
        );
        assert!(
            use_case
                .get_task(space_id, "SDLC", "SDLC 42")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn wiki_dossier_use_case_normalizes_phase_reads_and_links() {
        let repository = recording_dossier_repository();
        let use_case = WikiDossierUseCase::new(&repository);
        let actor_id = Uuid::now_v7();
        let space_id = Uuid::now_v7();
        let document_id = Uuid::now_v7();

        let phases = use_case.list_phases(space_id, " sdlc ").await.unwrap();
        assert_eq!(phases.phases.len(), 1);
        assert_eq!(
            repository
                .listed_phases
                .lock()
                .expect("listed phases should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string())]
        );

        let phase = use_case
            .get_phase(space_id, "sdlc", " Implementation ")
            .await
            .unwrap();
        assert_eq!(phase.phase_key, "implementation");
        assert_eq!(
            repository
                .requested_phases
                .lock()
                .expect("phase requests should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string(), "implementation".to_string())]
        );

        let documents = use_case
            .list_phase_documents(space_id, "sdlc", "Implementation")
            .await
            .unwrap();
        assert_eq!(documents.documents.len(), 1);
        assert_eq!(
            repository
                .listed_phase_documents
                .lock()
                .expect("listed phase documents should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string(), "implementation".to_string())]
        );

        let evidence = use_case
            .list_phase_evidence(space_id, "sdlc", "Implementation")
            .await
            .unwrap();
        assert_eq!(evidence.evidence.len(), 1);
        assert_eq!(
            repository
                .listed_phase_evidence
                .lock()
                .expect("listed phase evidence should be lockable")
                .as_slice(),
            [(space_id, "SDLC".to_string(), "implementation".to_string())]
        );

        use_case
            .link_phase_document(actor_id, space_id, "sdlc", " Implementation ", document_id)
            .await
            .unwrap();
        assert_eq!(
            repository
                .linked_phases
                .lock()
                .expect("phase link commands should be lockable")
                .as_slice(),
            [(
                actor_id,
                WikiLinkPhaseDocumentCommand {
                    space_id,
                    space_key: "SDLC".to_string(),
                    phase_key: "implementation".to_string(),
                    document_id,
                }
            )]
        );

        assert!(
            use_case
                .get_phase(space_id, "bad space", "implementation")
                .await
                .is_err()
        );
        assert!(
            use_case
                .get_phase(space_id, "SDLC", "_implementation")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn wiki_evidence_use_case_normalizes_create_and_list_requests() {
        let repository = recording_evidence_repository();
        let use_case = WikiEvidenceUseCase::new(&repository);
        let actor_id = Uuid::now_v7();
        let space_id = Uuid::now_v7();
        let document_id = Uuid::now_v7();
        let access_user_id = Uuid::now_v7();

        let response = use_case
            .create(
                actor_id,
                space_id,
                Some(document_id),
                shared::CreateEvidenceRequest {
                    space: Some("sdlc".to_string()),
                    document_id: Some(document_id.to_string()),
                    task_key: Some(" SDLC-42 ".to_string()),
                    phase_key: Some(" Implementation ".to_string()),
                    title: " Build log ".to_string(),
                    evidence_type: " external_url ".to_string(),
                    url: Some(" https://ci.local/jobs/42 ".to_string()),
                    attachment_id: None,
                    checksum: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(response.title, "Evidence");

        {
            let created = repository
                .created
                .lock()
                .expect("created evidence should be lockable");
            assert_eq!(created.len(), 1);
            assert_eq!(created[0].0, actor_id);
            let command = &created[0].1;
            assert_ne!(command.evidence_id, Uuid::nil());
            assert_eq!(command.space_id, space_id);
            assert_eq!(command.document_id, Some(document_id));
            assert_eq!(command.task_key.as_deref(), Some("SDLC-42"));
            assert_eq!(command.phase_key.as_deref(), Some("implementation"));
            assert_eq!(command.title, "Build log");
            assert_eq!(command.evidence_type, "external_url");
            assert_eq!(command.url.as_deref(), Some("https://ci.local/jobs/42"));
            assert_eq!(command.attachment_id, None);
        }

        let list = use_case
            .list(
                Some("sdlc"),
                Some(document_id),
                Some(" SDLC-42 "),
                Some(" Implementation "),
                Some(access_user_id),
                Some(500),
            )
            .await
            .unwrap();
        assert_eq!(list.evidence.len(), 1);
        assert_eq!(
            repository
                .listed
                .lock()
                .expect("listed evidence should be lockable")
                .as_slice(),
            [WikiEvidenceQueryCriteria {
                space_key: Some("SDLC".to_string()),
                document_id: Some(document_id),
                task_key: Some("SDLC-42".to_string()),
                phase_key: Some("implementation".to_string()),
                access_user_id: Some(access_user_id),
                limit: 100,
            }]
        );

        assert_eq!(
            normalize_evidence_space_key(None, Some("docs")).unwrap(),
            "DOCS"
        );
        assert_eq!(normalize_evidence_space_key(None, None).unwrap(), "SDLC");
        assert!(normalize_evidence_space_key(Some("bad space"), None).is_err());
    }

    #[tokio::test]
    async fn wiki_evidence_use_case_handles_attachment_commands_and_validation() {
        let repository = recording_evidence_repository();
        let use_case = WikiEvidenceUseCase::new(&repository);
        let actor_id = Uuid::now_v7();
        let evidence_id = Uuid::now_v7();
        let attachment_id = Uuid::now_v7();

        use_case
            .upload_attachment(
                actor_id,
                "build log.txt".to_string(),
                " text/plain ".to_string(),
                b"build log".to_vec(),
                64,
            )
            .await
            .unwrap();
        {
            let uploaded = repository
                .uploaded
                .lock()
                .expect("uploaded attachments should be lockable");
            assert_eq!(uploaded.len(), 1);
            assert_eq!(uploaded[0].0, actor_id);
            let command = &uploaded[0].1;
            assert_ne!(command.attachment_id, Uuid::nil());
            assert_eq!(command.file_name, "build log.txt");
            assert_eq!(command.content_type, "text/plain");
            assert_eq!(command.size_bytes, 9);
            assert_eq!(
                command.storage_key,
                format!("attachments/{}/build_log.txt", command.attachment_id)
            );
            assert_eq!(command.checksum, checksum(b"build log"));
            assert_eq!(command.bytes, b"build log".to_vec());
        }

        use_case.get(evidence_id).await.unwrap();
        assert_eq!(
            repository
                .requested_evidence
                .lock()
                .expect("requested evidence should be lockable")
                .as_slice(),
            [evidence_id]
        );

        use_case.get_attachment(attachment_id).await.unwrap();
        assert_eq!(
            repository
                .requested_attachments
                .lock()
                .expect("requested attachments should be lockable")
                .as_slice(),
            [attachment_id]
        );

        let downloaded = use_case.download_attachment(attachment_id).await.unwrap();
        assert_eq!(downloaded.bytes, b"build log".to_vec());
        assert_eq!(
            repository
                .downloaded_attachments
                .lock()
                .expect("downloaded attachments should be lockable")
                .as_slice(),
            [attachment_id]
        );

        assert!(
            use_case
                .upload_attachment(
                    actor_id,
                    "build.log".to_string(),
                    "text/plain".to_string(),
                    Vec::new(),
                    64,
                )
                .await
                .is_err()
        );
        assert!(
            use_case
                .upload_attachment(
                    actor_id,
                    "build.log".to_string(),
                    "text/plain".to_string(),
                    vec![0; 65],
                    64,
                )
                .await
                .is_err()
        );
        assert!(
            use_case
                .upload_attachment(
                    actor_id,
                    "bad/name.log".to_string(),
                    "text/plain".to_string(),
                    b"x".to_vec(),
                    64,
                )
                .await
                .is_err()
        );
        assert!(
            use_case
                .create(
                    actor_id,
                    Uuid::now_v7(),
                    None,
                    shared::CreateEvidenceRequest {
                        space: None,
                        document_id: None,
                        task_key: Some("SDLC-42".to_string()),
                        phase_key: None,
                        title: "File evidence".to_string(),
                        evidence_type: "uploaded_file".to_string(),
                        url: None,
                        attachment_id: Some(attachment_id.to_string()),
                        checksum: Some("sha256:client".to_string()),
                    },
                )
                .await
                .is_err()
        );
        assert!(
            use_case
                .create(
                    actor_id,
                    Uuid::now_v7(),
                    None,
                    shared::CreateEvidenceRequest {
                        space: None,
                        document_id: None,
                        task_key: None,
                        phase_key: None,
                        title: "Orphan evidence".to_string(),
                        evidence_type: "external_url".to_string(),
                        url: Some("https://ci.local/jobs/42".to_string()),
                        attachment_id: None,
                        checksum: None,
                    },
                )
                .await
                .is_err()
        );
    }

    #[test]
    fn wiki_helpers_prepare_content_and_storage_names() {
        assert_eq!(normalize_required("  title  ", "title").unwrap(), "title");
        assert_eq!(clamp_limit(Some(500), 100), 100);
        assert_eq!(markdown_to_text("# Title\n\n- Item"), "Title Item");
        let html =
            markdown_to_html("# Title\n\n<script>alert(1)</script>\n\n[Link](https://example.com)");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains(r#"<a href="https://example.com""#));
        assert!(!html.contains("<script"));
        assert!(!html.contains("alert(1)"));
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
            .list_recent(shared::AuditLogQuery { limit: None })
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
        assert!(!claims.token_id.is_empty());
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
        assert_ne!(access.token_id, refresh.token_id);
        assert!(!access.token_id.is_empty());
        assert!(!refresh.token_id.is_empty());
    }

    #[test]
    fn wiki_auth_token_pair_rotates_refresh_token_for_same_session() {
        let config = test_auth_config();
        let user_id = Uuid::now_v7();
        let session_id = Uuid::now_v7();

        let first = create_wiki_token_pair(&config, user_id, session_id)
            .expect("first token pair should be created");
        let second = create_wiki_token_pair(&config, user_id, session_id)
            .expect("second token pair should be created");

        assert_eq!(first.session_id, second.session_id);
        assert_ne!(first.access_token, second.access_token);
        assert_ne!(first.refresh_token, second.refresh_token);

        let first_refresh = decode_token(&config, &first.refresh_token, "refresh")
            .expect("first refresh token should decode");
        let second_refresh = decode_token(&config, &second.refresh_token, "refresh")
            .expect("second refresh token should decode");
        assert_eq!(first_refresh.jti, second_refresh.jti);
        assert_ne!(first_refresh.token_id, second_refresh.token_id);
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
