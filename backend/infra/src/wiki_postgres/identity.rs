use super::{
    PostgresWikiBackend,
    mapping::{parse_uuid, user_response_from_row},
};
use app::wiki::{
    WikiCreateUserCommand, WikiSettingsRepository, WikiSettingsRepositoryFuture,
    WikiSettingsUseCase, WikiUpdateUserCommand, WikiUserRepository, WikiUserRepositoryFuture,
    WikiUserUseCase, create_wiki_session_token_pair, create_wiki_token_pair, decode_token,
    hash_password, hash_token, normalize_required, verify_password,
};
use shared::wiki_contract::*;
use sqlx::{Row, postgres::PgRow};
use uuid::Uuid;

struct PostgresWikiUserRepository<'a> {
    backend: &'a PostgresWikiBackend,
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
                .insert_audit(&mut tx, Some(actor_id), "user.create", "user", user_id)
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

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "user.update",
                    "user",
                    command.user_id,
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

impl PostgresWikiBackend {
    pub(super) async fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<WikiClaims, shared::AppError> {
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

    pub(super) async fn register(
        &self,
        body: WikiRegisterRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        if !self.auth.registration_enabled {
            return Err(shared::AppError::Forbidden);
        }

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

    pub(super) async fn login(
        &self,
        body: WikiLoginRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
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

    pub(super) async fn refresh(
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

        let token_pair = create_wiki_token_pair(&self.auth, user_id, session_id)?;

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
        .bind(hash_token(&token_pair.access_token))
        .bind(hash_token(&token_pair.refresh_token))
        .bind(token_pair.access_expires_at)
        .bind(token_pair.refresh_expires_at)
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        Ok(WikiAuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: "Bearer".to_string(),
            user_id: user_id.to_string(),
            email: row.get("email"),
            username: row.get("username"),
            display_name: row.get("display_name"),
            expires_in: token_pair.expires_in,
        })
    }

    pub(super) async fn logout(&self, claims: &WikiClaims) -> Result<(), shared::AppError> {
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

    pub(super) async fn get_current_user(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserResponse, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        self.user_response(user_id).await
    }

    pub(super) async fn list_users(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserListResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let repository = PostgresWikiUserRepository { backend: self };
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
        let repository = PostgresWikiUserRepository { backend: self };
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
        let repository = PostgresWikiUserRepository { backend: self };
        WikiUserUseCase::new(&repository)
            .update(actor_id, user_id, body)
            .await
    }

    async fn issue_tokens(
        &self,
        user_id: Uuid,
        user: &PgRow,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        let token_pair = create_wiki_session_token_pair(&self.auth, user_id)?;

        sqlx::query(
            r#"
            INSERT INTO auth_sessions (
                id, user_id, access_token_hash, refresh_token_hash,
                expires_at, refresh_expires_at, created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now(), now())
            "#,
        )
        .bind(token_pair.session_id)
        .bind(user_id)
        .bind(hash_token(&token_pair.access_token))
        .bind(hash_token(&token_pair.refresh_token))
        .bind(token_pair.access_expires_at)
        .bind(token_pair.refresh_expires_at)
        .execute(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        Ok(WikiAuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            token_type: "Bearer".to_string(),
            user_id: user_id.to_string(),
            email: user.get("email"),
            username: user.get("username"),
            display_name: user.get("display_name"),
            expires_in: token_pair.expires_in,
        })
    }
}
