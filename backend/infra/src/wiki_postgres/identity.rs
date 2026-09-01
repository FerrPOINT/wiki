use super::{
    PostgresWikiBackend,
    mapping::{parse_uuid, user_response_from_row},
};
use app::wiki::{
    WikiAccessSessionCommand, WikiAuthRepository, WikiAuthRepositoryFuture, WikiAuthUseCase,
    WikiAuthUserRecord, WikiCreateUserCommand, WikiLogoutCommand, WikiRefreshSessionCommand,
    WikiRegisterAuthCommand, WikiSessionCommand, WikiSettingsRepository,
    WikiSettingsRepositoryFuture, WikiSettingsUseCase, WikiUpdateUserCommand, WikiUserRepository,
    WikiUserRepositoryFuture, WikiUserUseCase,
};
use shared::wiki_contract::*;
use sqlx::{Postgres, Row, postgres::PgRow};
use uuid::Uuid;

struct PostgresWikiUserRepository<'a> {
    backend: &'a PostgresWikiBackend,
    request_id: Option<&'a str>,
}

impl WikiUserRepository for PostgresWikiUserRepository<'_> {
    fn list_users<'a>(&'a self) -> WikiUserRepositoryFuture<'a, Vec<WikiUserResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                SELECT id, email, username, display_name, global_role, is_active
                FROM users
                ORDER BY lower(email)
                "#,
            )
            .fetch_all(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;

            Ok(rows.iter().map(user_response_from_row).collect())
        })
    }

    fn create_user<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateUserCommand,
    ) -> WikiUserRepositoryFuture<'a, WikiUserResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
            let user_id = Uuid::now_v7();

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
            .bind(&command.email)
            .bind(&command.username)
            .bind(&command.display_name)
            .bind(&command.password_hash)
            .bind(&command.global_role)
            .fetch_one(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "user.create",
                    "user",
                    user_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            Ok(user_response_from_row(&row))
        })
    }

    fn update_user<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpdateUserCommand,
    ) -> WikiUserRepositoryFuture<'a, WikiUserResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

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
            .bind(command.user_id)
            .bind(command.email.as_deref())
            .bind(command.username.as_deref())
            .bind(command.display_name.as_deref())
            .bind(command.global_role.as_deref())
            .bind(command.active)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("user", command.user_id))?;

            if command.active == Some(false) {
                sqlx::query(
                    "UPDATE auth_sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
                )
                .bind(command.user_id)
                .execute(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
            }

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "user.update",
                    "user",
                    command.user_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            Ok(user_response_from_row(&row))
        })
    }
}

struct PostgresWikiSettingsRepository<'a> {
    backend: &'a PostgresWikiBackend,
}

impl WikiSettingsRepository for PostgresWikiSettingsRepository<'_> {
    fn get_settings<'a>(&'a self) -> WikiSettingsRepositoryFuture<'a> {
        Box::pin(async move { Ok(self.backend.settings.clone()) })
    }
}

struct PostgresWikiAuthRepository<'a> {
    backend: &'a PostgresWikiBackend,
    request_id: Option<&'a str>,
}

fn auth_user_from_row(row: &PgRow) -> WikiAuthUserRecord {
    WikiAuthUserRecord {
        id: row.get("id"),
        email: row.get("email"),
        username: row.get("username"),
        display_name: row.get("display_name"),
        password_hash: row.get("password_hash"),
        global_role: row.get("global_role"),
        is_active: row.get("is_active"),
    }
}

async fn insert_session(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    session: &WikiSessionCommand,
) -> Result<(), shared::AppError> {
    sqlx::query(
        r#"
        INSERT INTO auth_sessions (
            id, user_id, access_token_hash, refresh_token_hash,
            expires_at, refresh_expires_at, created_at, last_used_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, now(), now())
        "#,
    )
    .bind(session.session_id)
    .bind(session.user_id)
    .bind(&session.access_token_hash)
    .bind(&session.refresh_token_hash)
    .bind(session.access_expires_at)
    .bind(session.refresh_expires_at)
    .execute(&mut **tx)
    .await
    .map_err(shared::AppError::database)?;
    Ok(())
}

impl WikiAuthRepository for PostgresWikiAuthRepository<'_> {
    fn authenticate_access_session<'a>(
        &'a self,
        command: WikiAccessSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, bool> {
        Box::pin(async move {
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
            .bind(command.session_id)
            .bind(command.user_id)
            .bind(&command.access_token_hash)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;

            if found.is_some() {
                sqlx::query("UPDATE auth_sessions SET last_used_at = now() WHERE id = $1")
                    .bind(command.session_id)
                    .execute(&self.backend.pool)
                    .await
                    .map_err(shared::AppError::database)?;
            }

            Ok(found.is_some())
        })
    }

    fn register_user<'a>(
        &'a self,
        command: WikiRegisterAuthCommand,
    ) -> WikiAuthRepositoryFuture<'a, WikiAuthUserRecord> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            let row = sqlx::query(
                r#"
                INSERT INTO users (
                    id, email, username, display_name, password_hash,
                    global_role, is_active, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, 'user', true, now(), now())
                RETURNING id, email, username, display_name, password_hash, global_role, is_active
                "#,
            )
            .bind(command.user_id)
            .bind(&command.email)
            .bind(&command.username)
            .bind(&command.display_name)
            .bind(&command.password_hash)
            .fetch_one(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(command.user_id),
                    "auth.register",
                    "user",
                    command.user_id,
                    self.request_id,
                )
                .await?;
            insert_session(&mut tx, &command.session).await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            Ok(auth_user_from_row(&row))
        })
    }

    fn find_user_by_email<'a>(
        &'a self,
        email: &'a str,
    ) -> WikiAuthRepositoryFuture<'a, Option<WikiAuthUserRecord>> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT id, email, username, display_name, password_hash, global_role, is_active
                FROM users
                WHERE lower(email) = lower($1)
                "#,
            )
            .bind(email)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;

            Ok(row.as_ref().map(auth_user_from_row))
        })
    }

    fn create_login_session<'a>(
        &'a self,
        session: WikiSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, ()> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            insert_session(&mut tx, &session).await?;
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(session.user_id),
                    "auth.login",
                    "user",
                    session.user_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            Ok(())
        })
    }

    fn find_refresh_session<'a>(
        &'a self,
        command: WikiRefreshSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, Option<WikiAuthUserRecord>> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT
                    u.id, u.email, u.username, u.display_name,
                    u.password_hash, u.global_role, u.is_active
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
            .bind(command.session_id)
            .bind(command.user_id)
            .bind(&command.refresh_token_hash)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;

            Ok(row.as_ref().map(auth_user_from_row))
        })
    }

    fn rotate_session<'a>(
        &'a self,
        session: WikiSessionCommand,
    ) -> WikiAuthRepositoryFuture<'a, ()> {
        Box::pin(async move {
            sqlx::query(
                r#"
                UPDATE auth_sessions
                SET access_token_hash = $1,
                    refresh_token_hash = $2,
                    expires_at = $3,
                    refresh_expires_at = $4,
                    last_used_at = now()
                WHERE id = $5 AND user_id = $6
                "#,
            )
            .bind(&session.access_token_hash)
            .bind(&session.refresh_token_hash)
            .bind(session.access_expires_at)
            .bind(session.refresh_expires_at)
            .bind(session.session_id)
            .bind(session.user_id)
            .execute(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(())
        })
    }

    fn revoke_sessions<'a>(
        &'a self,
        command: WikiLogoutCommand,
    ) -> WikiAuthRepositoryFuture<'a, ()> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            if let Some(session_id) = command.session_id {
                sqlx::query(
                    "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2",
                )
                .bind(session_id)
                .bind(command.user_id)
                .execute(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
            } else {
                sqlx::query(
                    "UPDATE auth_sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
                )
                .bind(command.user_id)
                .execute(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
            }

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(command.user_id),
                    "auth.logout",
                    "user",
                    command.user_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            Ok(())
        })
    }

    fn get_current_user<'a>(
        &'a self,
        user_id: Uuid,
    ) -> WikiAuthRepositoryFuture<'a, WikiAuthUserRecord> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT id, email, username, display_name, password_hash, global_role, is_active
                FROM users
                WHERE id = $1
                "#,
            )
            .bind(user_id)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("user", user_id))?;

            Ok(auth_user_from_row(&row))
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<WikiClaims, shared::AppError> {
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: None,
        };
        WikiAuthUseCase::new(&repository, &self.auth)
            .authenticate_access_token(token)
            .await
    }

    pub(super) async fn register(
        &self,
        request_id: Option<String>,
        body: WikiRegisterRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: request_id.as_deref(),
        };
        WikiAuthUseCase::new(&repository, &self.auth)
            .register(body)
            .await
    }

    pub(super) async fn login(
        &self,
        request_id: Option<String>,
        body: WikiLoginRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: request_id.as_deref(),
        };
        WikiAuthUseCase::new(&repository, &self.auth)
            .login(body)
            .await
    }

    pub(super) async fn refresh(
        &self,
        body: WikiRefreshRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: None,
        };
        WikiAuthUseCase::new(&repository, &self.auth)
            .refresh(body)
            .await
    }

    pub(super) async fn logout(&self, claims: &WikiClaims) -> Result<(), shared::AppError> {
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiAuthUseCase::new(&repository, &self.auth)
            .logout(claims)
            .await
    }

    pub(super) async fn get_current_user(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: None,
        };
        WikiAuthUseCase::new(&repository, &self.auth)
            .current_user(claims)
            .await
    }

    pub(super) async fn list_users(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserListResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let repository = PostgresWikiUserRepository {
            backend: self,
            request_id: None,
        };
        WikiUserUseCase::new(&repository).list().await
    }

    pub(super) async fn get_settings(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiSettingsSnapshot, shared::AppError> {
        self.ensure_admin(claims).await?;
        let repository = PostgresWikiSettingsRepository { backend: self };
        WikiSettingsUseCase::new(&repository).get().await
    }

    pub(super) async fn create_user(
        &self,
        claims: &WikiClaims,
        body: WikiCreateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let repository = PostgresWikiUserRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiUserUseCase::new(&repository)
            .create(actor_id, body)
            .await
    }

    pub(super) async fn update_user(
        &self,
        claims: &WikiClaims,
        user_id: &str,
        body: WikiUpdateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let user_id = parse_uuid(user_id, "user")?;
        let repository = PostgresWikiUserRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiUserUseCase::new(&repository)
            .update(actor_id, user_id, body)
            .await
    }
}
