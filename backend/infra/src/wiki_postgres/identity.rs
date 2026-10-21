use super::database_error;
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
    admission: Option<std::sync::Arc<shared::wiki_space_access::WikiSpaceAdmission>>,
}

#[derive(serde::Deserialize)]
struct CentralDirectoryUser {
    id: String,
    email: String,
    #[serde(default)]
    username: String,
    display_name: String,
    status: String,
}

impl CentralDirectoryUser {
    fn projected_display_name(&self) -> Result<&str, shared::AppError> {
        [&self.display_name, &self.username, &self.email]
            .into_iter()
            .map(|value| value.trim())
            .find(|value| !value.is_empty())
            .ok_or_else(|| {
                shared::AppError::Unavailable(
                    "Central Auth directory has no usable user name".into(),
                )
            })
    }
}

async fn persist_central_directory_user(
    pool: &sqlx::PgPool,
    entry: &CentralDirectoryUser,
) -> Result<(), shared::AppError> {
    if entry.status == "pending" {
        return Ok(());
    }
    if !matches!(entry.status.as_str(), "active" | "disabled") {
        return Err(shared::AppError::Unavailable(
            "invalid_central_identity_status".into(),
        ));
    }
    if entry.status == "disabled" {
        sqlx::query("UPDATE users SET is_active = false, updated_at = now() WHERE central_sub = $1 AND is_active")
            .bind(&entry.id).execute(pool).await.map_err(database_error)?;
        return Ok(());
    }
    let display_name = entry.projected_display_name()?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, password_hash, central_sub, global_role, is_active, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, '!', $5, 'user', true, now(), now()) \
         ON CONFLICT (central_sub) WHERE central_sub IS NOT NULL DO UPDATE \
         SET email = EXCLUDED.email, display_name = EXCLUDED.display_name, updated_at = now() \
         WHERE users.email IS DISTINCT FROM EXCLUDED.email \
            OR users.display_name IS DISTINCT FROM EXCLUDED.display_name"
    )
    .bind(id).bind(&entry.email).bind(format!("central-{}", id.simple()))
    .bind(display_name).bind(&entry.id)
    .execute(pool).await.map_err(database_error)?;
    Ok(())
}

impl PostgresWikiBackend {
    pub async fn sync_central_users(&self, token: &str) -> Result<(), shared::AppError> {
        let jwks = std::env::var("WIKI_AUTH__CENTRAL_JWKS_URI")
            .map_err(|_| shared::AppError::Unavailable("Central Auth is unavailable".into()))?;
        let mut url = reqwest::Url::parse(&jwks)
            .map_err(|_| shared::AppError::Unavailable("Central Auth URL is invalid".into()))?;
        url.set_path("/auth/directory");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|_| shared::AppError::Unavailable("Central Auth is unavailable".into()))?;

        for page in 0..100 {
            let response = client
                .get(url.clone())
                .query(&[("offset", page * 100)])
                .bearer_auth(token)
                .send()
                .await
                .map_err(|_| shared::AppError::Unavailable("Central Auth is unavailable".into()))?;
            if !response.status().is_success() {
                return Err(shared::AppError::Unavailable(
                    "Central Auth directory is unavailable".into(),
                ));
            }
            let batch = response
                .json::<Vec<CentralDirectoryUser>>()
                .await
                .map_err(|_| {
                    shared::AppError::Unavailable("Central Auth directory is invalid".into())
                })?;
            let count = batch.len();
            for entry in batch {
                persist_central_directory_user(&self.pool, &entry).await?;
            }
            if count < 100 {
                return Ok(());
            }
        }
        Err(shared::AppError::Unavailable(
            "Central Auth directory is too large".into(),
        ))
    }
}

impl WikiUserRepository for PostgresWikiUserRepository<'_> {
    fn list_users(&self) -> WikiUserRepositoryFuture<'_, Vec<WikiUserResponse>> {
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
            .map_err(database_error)?;

            if let Some(scope) = &self.admission {
                scope.require_current()?;
            }
            Ok(rows.iter().map(user_response_from_row).collect())
        })
    }

    fn create_user(
        &self,
        actor_id: Uuid,
        command: WikiCreateUserCommand,
    ) -> WikiUserRepositoryFuture<'_, WikiUserResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .begin_content_write(
                    actor_id,
                    self.admission.as_ref(),
                    &[],
                    super::SpaceAccess::Admin,
                )
                .await?;
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
            .map_err(database_error)?;
            if command.global_role == "admin" {
                sqlx::query("INSERT INTO wiki_confirmed_global_roles(user_id,role,granted_by) VALUES($1,'admin',$2)")
                    .bind(user_id).bind(actor_id).execute(&mut *tx).await.map_err(database_error)?;
            }

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
            super::write_authorization::commit(tx, self.admission.as_ref()).await?;

            Ok(user_response_from_row(&row))
        })
    }

    fn update_user(
        &self,
        actor_id: Uuid,
        command: WikiUpdateUserCommand,
    ) -> WikiUserRepositoryFuture<'_, WikiUserResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .begin_content_write(
                    actor_id,
                    self.admission.as_ref(),
                    &[],
                    super::SpaceAccess::Admin,
                )
                .await?;

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
            .map_err(database_error)?
            .ok_or_else(|| shared::AppError::not_found("user", command.user_id))?;

            if let Some(role) = command.global_role.as_deref() {
                sqlx::query("DELETE FROM wiki_confirmed_global_roles WHERE user_id=$1")
                    .bind(command.user_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(database_error)?;
                if role == "admin" {
                    sqlx::query("INSERT INTO wiki_confirmed_global_roles(user_id,role,granted_by) VALUES($1,'admin',$2)")
                        .bind(command.user_id).bind(actor_id).execute(&mut *tx).await.map_err(database_error)?;
                }
            }
            if command.active == Some(false) {
                sqlx::query(
                    "UPDATE auth_sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
                )
                .bind(command.user_id)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
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
            super::write_authorization::commit(tx, self.admission.as_ref()).await?;

            Ok(user_response_from_row(&row))
        })
    }
}

struct PostgresWikiSettingsRepository<'a> {
    backend: &'a PostgresWikiBackend,
}

impl WikiSettingsRepository for PostgresWikiSettingsRepository<'_> {
    fn get_settings(&self) -> WikiSettingsRepositoryFuture<'_> {
        Box::pin(async move { Ok(self.backend.settings.clone()) })
    }
}

impl PostgresWikiAuthRepository<'_> {
    async fn existing_central_user(
        &self,
        subject: &str,
    ) -> Result<WikiAuthUserRecord, shared::AppError> {
        // Older PAT introspection has verified identity/scopes but no name.
        // Reuse the exact active subject only; no email link or profile mutation.
        let row = sqlx::query("SELECT id,email,username,display_name,password_hash,global_role,is_active FROM users WHERE central_sub=$1 AND is_active")
            .bind(subject).fetch_optional(&self.backend.pool).await.map_err(database_error)?
            .ok_or(shared::AppError::Unauthorized)?;
        Ok(auth_user_from_row(&row))
    }

    /// Central subjects never auto-link to historical local email rows.
    async fn find_or_link_central_user(
        &self,
        ctx: &pdlc_auth_core::AuthContext,
        display_name: &str,
    ) -> Result<WikiAuthUserRecord, shared::AppError> {
        self.resolve_central_profile(&WikiCentralProfile {
            subject: ctx.user_id.clone(),
            email: ctx.email.clone(),
            display_name: Some(display_name.into()),
            session_id: ctx.session_id.clone(),
        })
        .await
    }

    async fn resolve_central_profile(
        &self,
        profile: &WikiCentralProfile,
    ) -> Result<WikiAuthUserRecord, shared::AppError> {
        let display_name = profile.display_name.as_deref().unwrap_or_default().trim();
        if display_name.is_empty() {
            return Err(shared::AppError::Unavailable(
                "Central Auth returned no verified display name".into(),
            ));
        }
        let email = profile
            .email
            .as_deref()
            .unwrap_or_default()
            .to_lowercase()
            .trim()
            .to_string();
        if email.is_empty() {
            return Err(shared::AppError::Unauthorized);
        }
        if profile.subject.trim().is_empty() {
            return Err(shared::AppError::Unauthorized);
        }
        let id = uuid::Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, username, display_name, password_hash, central_sub,
                global_role, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, '!', $5, 'user', true, now(), now())
            ON CONFLICT (central_sub) WHERE central_sub IS NOT NULL DO NOTHING
            "#,
        )
        .bind(id)
        .bind(&email)
        .bind(format!("central-{}", id.simple()))
        .bind(display_name)
        .bind(&profile.subject)
        .execute(&self.backend.pool)
        .await
        .map_err(database_error)?;
        sqlx::query("UPDATE users SET display_name = $2, updated_at = now() WHERE central_sub = $1 AND display_name IS DISTINCT FROM $2")
            .bind(&profile.subject).bind(display_name).execute(&self.backend.pool).await.map_err(database_error)?;
        let row = sqlx::query(
            "SELECT id, email, username, display_name, password_hash, global_role, is_active \
             FROM users WHERE central_sub = $1",
        )
        .bind(&profile.subject)
        .fetch_one(&self.backend.pool)
        .await
        .map_err(database_error)?;
        if !row.get::<bool, _>("is_active") {
            return Err(shared::AppError::Unauthorized);
        }
        Ok(auth_user_from_row(&row))
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
    .map_err(database_error)?;
    Ok(())
}

impl WikiAuthRepository for PostgresWikiAuthRepository<'_> {
    fn authenticate_access_session(
        &self,
        command: WikiAccessSessionCommand,
    ) -> WikiAuthRepositoryFuture<'_, bool> {
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
            .map_err(database_error)?;

            if found.is_some() {
                sqlx::query("UPDATE auth_sessions SET last_used_at = now() WHERE id = $1")
                    .bind(command.session_id)
                    .execute(&self.backend.pool)
                    .await
                    .map_err(database_error)?;
            }

            Ok(found.is_some())
        })
    }

    fn register_user(
        &self,
        command: WikiRegisterAuthCommand,
    ) -> WikiAuthRepositoryFuture<'_, WikiAuthUserRecord> {
        Box::pin(async move {
            let mut tx = self.backend.pool.begin().await.map_err(database_error)?;

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
            .map_err(database_error)?;

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
            tx.commit().await.map_err(database_error)?;

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
                WHERE lower(email) = lower($1) AND central_sub IS NULL
                "#,
            )
            .bind(email)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(database_error)?;

            Ok(row.as_ref().map(auth_user_from_row))
        })
    }

    fn create_login_session(
        &self,
        session: WikiSessionCommand,
    ) -> WikiAuthRepositoryFuture<'_, ()> {
        Box::pin(async move {
            let mut tx = self.backend.pool.begin().await.map_err(database_error)?;

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
            tx.commit().await.map_err(database_error)?;
            Ok(())
        })
    }

    fn find_refresh_session(
        &self,
        command: WikiRefreshSessionCommand,
    ) -> WikiAuthRepositoryFuture<'_, Option<WikiAuthUserRecord>> {
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
            .map_err(database_error)?;

            Ok(row.as_ref().map(auth_user_from_row))
        })
    }

    fn rotate_session(&self, session: WikiSessionCommand) -> WikiAuthRepositoryFuture<'_, ()> {
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
            .map_err(database_error)?;
            Ok(())
        })
    }

    fn revoke_sessions(&self, command: WikiLogoutCommand) -> WikiAuthRepositoryFuture<'_, ()> {
        Box::pin(async move {
            let mut tx = self.backend.pool.begin().await.map_err(database_error)?;

            if let Some(session_id) = command.session_id {
                sqlx::query(
                    "UPDATE auth_sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2",
                )
                .bind(session_id)
                .bind(command.user_id)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
            } else {
                sqlx::query(
                    "UPDATE auth_sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
                )
                .bind(command.user_id)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
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
            tx.commit().await.map_err(database_error)?;
            Ok(())
        })
    }

    fn get_current_user(&self, user_id: Uuid) -> WikiAuthRepositoryFuture<'_, WikiAuthUserRecord> {
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
            .map_err(database_error)?
            .ok_or_else(|| shared::AppError::not_found("user", user_id))?;

            let mut user = auth_user_from_row(&row);
            if super::central_mode() {
                user.global_role = self.backend.active_global_role(user_id).await?;
            }
            Ok(user)
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn resolve_central_profile(
        &self,
        profile: WikiCentralProfile,
    ) -> Result<WikiClaims, shared::AppError> {
        if super::namespace::registered_machine(&profile.subject) {
            return Err(shared::AppError::Forbidden);
        }
        let repository = PostgresWikiAuthRepository {
            backend: self,
            request_id: None,
        };
        let record = match profile.display_name {
            Some(_) => repository.resolve_central_profile(&profile).await?,
            None => repository.existing_central_user(&profile.subject).await?,
        };
        Ok(WikiClaims {
            user_id: record.id.to_string(),
            session_id: profile.session_id,
            request_id: None,
            admission: None,
        })
    }

    pub(super) async fn authenticate_access_token(
        &self,
        token: &str,
    ) -> Result<WikiClaims, shared::AppError> {
        if self.admission_active().await? {
            let verified = super::central_auth::verified_for_request(token)
                .await
                .map_err(super::admission::auth_error)?;
            let ctx = &verified.context;
            if super::namespace::registered_machine(&ctx.user_id)
                || ctx.role.as_deref() == Some("service_account")
                || verified.subject_role.as_deref() == Some("service_account")
            {
                return Err(shared::AppError::Forbidden);
            }
            let repository = PostgresWikiAuthRepository {
                backend: self,
                request_id: None,
            };
            let record = match verified.display_name.as_deref() {
                Some(name) => repository.find_or_link_central_user(ctx, name).await?,
                None => repository.existing_central_user(&ctx.user_id).await?,
            };
            let mut claims = super::central_auth::claims_for(ctx, record.id.to_string());
            claims.admission = Some(
                self.confirmed_admission(&verified, record.id, record.global_role)
                    .await?,
            );
            return Ok(claims);
        }
        // Subject and live verified display name come from Central Auth.
        if let Some((ctx, name)) =
            crate::wiki_postgres::central_auth::try_central_with_name(token).await?
        {
            if super::namespace::registered_machine(&ctx.user_id)
                || ctx.role.as_deref() == Some("service_account")
            {
                return Err(shared::AppError::Forbidden);
            }
            let repository = PostgresWikiAuthRepository {
                backend: self,
                request_id: None,
            };
            let record = match name {
                Some(name) => repository.find_or_link_central_user(&ctx, &name).await?,
                None => repository.existing_central_user(&ctx.user_id).await?,
            };
            return Ok(crate::wiki_postgres::central_auth::claims_for(
                &ctx,
                record.id.to_string(),
            ));
        }
        if std::env::var_os("WIKI_AUTH__CENTRAL_JWKS_URI").is_some() {
            return Err(shared::AppError::Unauthorized);
        }
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
        // Central fleet auth first; local password login remains the fallback
        // (transition period, see central_login.rs).
        if let Some(pair) =
            crate::wiki_postgres::central_login::try_central_login(&body.email, &body.password)
                .await
        {
            return Ok(crate::wiki_postgres::central_login::central_response(
                pair,
                &body.email,
            ));
        }
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
        let mut user = WikiAuthUseCase::new(&repository, &self.auth)
            .current_user(claims)
            .await?;
        if let Some(scope) = self.read_admission(Some(claims)).await? {
            user.is_system_admin = scope.confirmed_global_admin();
            user.role = if user.is_system_admin {
                "admin"
            } else {
                "user"
            }
            .into();
        }
        Ok(user)
    }

    pub(super) async fn list_users(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserListResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let repository = PostgresWikiUserRepository {
            backend: self,
            request_id: None,
            admission: self.read_admission(Some(claims)).await?,
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
            admission: self.read_admission(Some(claims)).await?,
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
            admission: self.read_admission(Some(claims)).await?,
        };
        WikiUserUseCase::new(&repository)
            .update(actor_id, user_id, body)
            .await
    }
}

#[cfg(test)]
mod central_directory_tests {
    use super::*;

    async fn profile_backend() -> PostgresWikiBackend {
        let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("disposable QA database required");
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let database: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(database.ends_with("_test"), "refuse working database");
        let migrations = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations");
        sqlx::migrate::Migrator::new(migrations)
            .await
            .unwrap()
            .run(&pool)
            .await
            .unwrap();
        let config = shared::AppConfig::default();
        PostgresWikiBackend {
            pool,
            task_reader: None,
            auth: config.auth.clone(),
            storage: std::sync::Arc::new(crate::wiki_storage::LocalWikiAttachmentStorage::new(
                std::env::temp_dir().join(format!("wiki-profile-{}", Uuid::now_v7())),
            )),
            max_upload_bytes: config.storage.max_upload_bytes,
            staged_attachment_ttl_hours: config.maintenance.staged_attachment_ttl_hours,
            maintenance_batch_size: config.maintenance.batch_size,
            settings: WikiSettingsSnapshot::from_config(&config),
        }
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn current_human_role_controls_admin_writes_without_changing_product_history() {
        let backend = profile_backend().await;
        let local = Uuid::now_v7();
        let space = Uuid::now_v7();
        sqlx::query("INSERT INTO users(id,email,username,display_name,password_hash,global_role,central_sub) VALUES($1,$2,$3,'Role fixture','!','admin',$4)")
            .bind(local).bind(format!("{local}@example.test")).bind(local.to_string()).bind(local.to_string())
            .execute(&backend.pool).await.unwrap();
        sqlx::query(
            "INSERT INTO spaces(id,key,name,owner_id) VALUES($1,$2,'Working documents',$3)",
        )
        .bind(space)
        .bind(format!("Q{}", &space.simple().to_string()[..12]).to_uppercase())
        .bind(local)
        .execute(&backend.pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'admin')")
            .bind(space)
            .bind(local)
            .execute(&backend.pool)
            .await
            .unwrap();
        let claims = WikiClaims {
            user_id: local.to_string(),
            session_id: None,
            request_id: None,
            admission: None,
        };
        let mut actor = context(&format!("{local}@example.test"));
        actor.user_id = local.to_string();
        actor.role = Some("user".into());
        pdlc_auth_core::human_request::with_human_request(local, &actor, None, async {
            assert!(matches!(
                backend.ensure_admin(&claims).await,
                Err(shared::AppError::Forbidden)
            ));
            assert!(matches!(
                backend
                    .begin_content_write(local, None, &[space], super::super::SpaceAccess::Admin)
                    .await,
                Err(shared::AppError::Forbidden)
            ));
            let tx = backend
                .begin_content_write(local, None, &[space], super::super::SpaceAccess::Edit)
                .await
                .unwrap();
            tx.rollback().await.unwrap();
        })
        .await
        .unwrap();
        sqlx::query("UPDATE users SET global_role='user' WHERE id=$1")
            .bind(local)
            .execute(&backend.pool)
            .await
            .unwrap();
        actor.role = Some("admin".into());
        pdlc_auth_core::human_request::with_human_request(local, &actor, None, async {
            assert_eq!(backend.ensure_admin(&claims).await.unwrap(), local);
            let tx = backend
                .begin_content_write(local, None, &[space], super::super::SpaceAccess::Admin)
                .await
                .unwrap();
            tx.rollback().await.unwrap();
        })
        .await
        .unwrap();
        let retained: String = sqlx::query_scalar("SELECT global_role FROM users WHERE id=$1")
            .bind(local)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(
            retained, "user",
            "current permission must not rewrite role history"
        );
        backend.pool.close().await;
    }

    fn context(email: &str) -> pdlc_auth_core::AuthContext {
        pdlc_auth_core::AuthContext {
            user_id: Uuid::now_v7().to_string(),
            email: Some(email.into()),
            role: None,
            scopes: Default::default(),
            session_id: Some(Uuid::now_v7().to_string()),
            token: String::new(),
        }
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn first_central_profile_uses_verified_name_without_directory_sync_or_email_link() {
        let backend = profile_backend().await;
        let legacy_id = Uuid::now_v7();
        let email = format!("{legacy_id}@example.test");
        sqlx::query("INSERT INTO users (id, email, username, display_name, password_hash, global_role, is_active) VALUES ($1, $2, $3, 'Historical author', '!', 'user', true)")
            .bind(legacy_id).bind(&email).bind(format!("legacy-{legacy_id}"))
            .execute(&backend.pool).await.unwrap();
        let repository = PostgresWikiAuthRepository {
            backend: &backend,
            request_id: None,
        };
        let ctx = context(&email);
        let record = repository
            .find_or_link_central_user(&ctx, "  QA Отображаемое имя  ")
            .await
            .unwrap();
        assert_eq!(record.global_role, "user");
        assert_ne!(record.id, legacy_id);
        assert_eq!(record.display_name, "QA Отображаемое имя");
        assert_eq!(record.email, email);
        let historical: (String, Option<String>, String) = sqlx::query_as(
            "SELECT display_name, central_sub, global_role FROM users WHERE id = $1",
        )
        .bind(legacy_id)
        .fetch_one(&backend.pool)
        .await
        .unwrap();
        assert_eq!(
            historical,
            ("Historical author".into(), None, "user".into())
        );
        backend.pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn verified_name_refresh_is_subject_scoped_and_repeatable() {
        let backend = profile_backend().await;
        let repository = PostgresWikiAuthRepository {
            backend: &backend,
            request_id: None,
        };
        let email = format!("{}@example.test", Uuid::now_v7());
        let ctx = context(&email);
        let other = context(&email);
        let original = repository
            .find_or_link_central_user(&ctx, "Before")
            .await
            .unwrap();
        let other_record = repository
            .find_or_link_central_user(&other, "Other")
            .await
            .unwrap();
        let renamed = repository
            .find_or_link_central_user(&ctx, "After")
            .await
            .unwrap();
        assert_eq!(renamed.global_role, "user");
        sqlx::query("UPDATE users SET global_role='admin' WHERE id=$1")
            .bind(renamed.id)
            .execute(&backend.pool)
            .await
            .unwrap();
        let historical_admin = repository
            .find_or_link_central_user(&ctx, "After")
            .await
            .unwrap();
        assert_eq!(
            historical_admin.global_role, "admin",
            "profile refresh preserves history; central authority is request-scoped"
        );
        assert_eq!(original.id, renamed.id);
        assert_eq!(renamed.display_name, "After");
        let timestamp: chrono::DateTime<chrono::Utc> =
            sqlx::query_scalar("SELECT updated_at FROM users WHERE id = $1")
                .bind(renamed.id)
                .fetch_one(&backend.pool)
                .await
                .unwrap();
        let repeated = repository
            .find_or_link_central_user(&ctx, " After ")
            .await
            .unwrap();
        assert_eq!(repeated.id, original.id);
        let unchanged: chrono::DateTime<chrono::Utc> =
            sqlx::query_scalar("SELECT updated_at FROM users WHERE id = $1")
                .bind(repeated.id)
                .fetch_one(&backend.pool)
                .await
                .unwrap();
        assert_eq!(timestamp, unchanged);
        let other_name: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
            .bind(other_record.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(other_name, "Other");
        backend.pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn missing_verified_name_cannot_create_or_update_a_profile() {
        let backend = profile_backend().await;
        let repository = PostgresWikiAuthRepository {
            backend: &backend,
            request_id: None,
        };
        let ctx = context(&format!("{}@example.test", Uuid::now_v7()));
        assert!(
            repository
                .find_or_link_central_user(&ctx, " \t\n")
                .await
                .is_err()
        );
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE central_sub = $1")
            .bind(&ctx.user_id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let original = repository
            .find_or_link_central_user(&ctx, "Verified")
            .await
            .unwrap();
        assert!(
            repository
                .find_or_link_central_user(&ctx, "")
                .await
                .is_err()
        );
        let name: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
            .bind(original.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(name, "Verified");
        backend.pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn older_pat_reuses_only_exact_active_profile_without_mutation() {
        let backend = profile_backend().await;
        let repository = PostgresWikiAuthRepository {
            backend: &backend,
            request_id: None,
        };
        let ctx = context(&format!("{}@example.test", Uuid::now_v7()));
        assert!(
            repository
                .existing_central_user(&ctx.user_id)
                .await
                .is_err()
        );
        let original = repository
            .find_or_link_central_user(&ctx, "Verified name")
            .await
            .unwrap();
        let before: (String, String, chrono::DateTime<chrono::Utc>) =
            sqlx::query_as("SELECT display_name,global_role,updated_at FROM users WHERE id=$1")
                .bind(original.id)
                .fetch_one(&backend.pool)
                .await
                .unwrap();
        let record = repository
            .existing_central_user(&ctx.user_id)
            .await
            .unwrap();
        assert_eq!(record.id, original.id);
        let other = context(ctx.email.as_deref().unwrap());
        assert!(
            repository
                .existing_central_user(&other.user_id)
                .await
                .is_err()
        );
        let after: (String, String, chrono::DateTime<chrono::Utc>) =
            sqlx::query_as("SELECT display_name,global_role,updated_at FROM users WHERE id=$1")
                .bind(original.id)
                .fetch_one(&backend.pool)
                .await
                .unwrap();
        assert_eq!(before, after);
        sqlx::query("UPDATE users SET is_active=false WHERE id=$1")
            .bind(original.id)
            .execute(&backend.pool)
            .await
            .unwrap();
        assert!(
            repository
                .existing_central_user(&ctx.user_id)
                .await
                .is_err()
        );
        backend.pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn central_login_never_grants_admin_or_reactivates_a_disabled_local_profile() {
        let backend = profile_backend().await;
        let repository = PostgresWikiAuthRepository {
            backend: &backend,
            request_id: None,
        };
        let ctx = context(&format!("{}@example.test", Uuid::now_v7()));
        let record = repository
            .find_or_link_central_user(&ctx, "Ordinary")
            .await
            .unwrap();
        let role: String = sqlx::query_scalar("SELECT global_role FROM users WHERE id=$1")
            .bind(record.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(role, "user");
        repository
            .find_or_link_central_user(&ctx, "Renamed")
            .await
            .unwrap();
        let role: String = sqlx::query_scalar("SELECT global_role FROM users WHERE id=$1")
            .bind(record.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(role, "user");
        // An explicit existing role is preserved, not overwritten by JIT.
        sqlx::query("UPDATE users SET global_role='admin' WHERE id=$1")
            .bind(record.id)
            .execute(&backend.pool)
            .await
            .unwrap();
        repository
            .find_or_link_central_user(&ctx, "Explicit admin")
            .await
            .unwrap();
        let role: String = sqlx::query_scalar("SELECT global_role FROM users WHERE id=$1")
            .bind(record.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(role, "admin");
        sqlx::query("UPDATE users SET global_role='user',is_active=false WHERE id=$1")
            .bind(record.id)
            .execute(&backend.pool)
            .await
            .unwrap();
        assert!(matches!(
            repository
                .find_or_link_central_user(&ctx, "Still disabled")
                .await,
            Err(shared::AppError::Unauthorized)
        ));
        let state: (String, bool) =
            sqlx::query_as("SELECT global_role,is_active FROM users WHERE id=$1")
                .bind(record.id)
                .fetch_one(&backend.pool)
                .await
                .unwrap();
        assert_eq!(state, ("user".into(), false));
        backend.pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn directory_projection_does_not_grant_or_replace_local_roles() {
        let backend = profile_backend().await;
        let mut user = entry("Directory user", "ordinary");
        persist_central_directory_user(&backend.pool, &user)
            .await
            .unwrap();
        let role: String = sqlx::query_scalar("SELECT global_role FROM users WHERE central_sub=$1")
            .bind(&user.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(role, "user");
        sqlx::query("UPDATE users SET global_role='admin' WHERE central_sub=$1")
            .bind(&user.id)
            .execute(&backend.pool)
            .await
            .unwrap();
        user.display_name = "Renamed directory user".into();
        persist_central_directory_user(&backend.pool, &user)
            .await
            .unwrap();
        let role: String = sqlx::query_scalar("SELECT global_role FROM users WHERE central_sub=$1")
            .bind(&user.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(role, "admin");
        backend.pool.close().await;
    }

    #[tokio::test]
    async fn central_projection_distinguishes_database_failure_from_invalid_claims() {
        let pool = sqlx::PgPool::connect_lazy("postgres://unused@127.0.0.1:1/unused").unwrap();
        pool.close().await;
        let config = shared::AppConfig::default();
        let backend = PostgresWikiBackend {
            task_reader: None,
            pool,
            auth: config.auth.clone(),
            storage: std::sync::Arc::new(crate::LocalWikiAttachmentStorage::new("unused")),
            max_upload_bytes: config.storage.max_upload_bytes,
            staged_attachment_ttl_hours: config.maintenance.staged_attachment_ttl_hours,
            maintenance_batch_size: config.maintenance.batch_size,
            settings: WikiSettingsSnapshot::from_config(&config),
        };
        let repository = PostgresWikiAuthRepository {
            backend: &backend,
            request_id: None,
        };
        let mut context = pdlc_auth_core::AuthContext {
            user_id: Uuid::now_v7().to_string(),
            role: None,
            scopes: Default::default(),
            session_id: None,
            email: Some("qa-projection@example.test".into()),
            token: String::new(),
        };
        assert!(matches!(
            repository
                .find_or_link_central_user(&context, "Verified name")
                .await,
            Err(shared::AppError::Database(_))
        ));
        assert!(matches!(
            repository
                .find_or_link_central_user(&context, " \t\n")
                .await,
            Err(shared::AppError::Unavailable(_))
        ));
        context.email = None;
        assert!(matches!(
            repository
                .find_or_link_central_user(&context, "Verified name")
                .await,
            Err(shared::AppError::Unauthorized)
        ));
        context.email = Some("qa-projection@example.test".into());
        context.user_id.clear();
        assert!(matches!(
            repository
                .find_or_link_central_user(&context, "Verified name")
                .await,
            Err(shared::AppError::Unauthorized)
        ));
    }

    fn entry(name: &str, username: &str) -> CentralDirectoryUser {
        let id = Uuid::now_v7().to_string();
        CentralDirectoryUser {
            email: format!("{id}@example.test"),
            id,
            username: username.into(),
            display_name: name.into(),
            status: "active".into(),
        }
    }

    #[test]
    fn display_name_uses_trimmed_name_then_username_then_legacy_email() {
        assert_eq!(
            entry("  Name  ", "login").projected_display_name().unwrap(),
            "Name"
        );
        for name in ["", " \t\n"] {
            assert_eq!(
                entry(name, "  login  ").projected_display_name().unwrap(),
                "login"
            );
        }
        let legacy: CentralDirectoryUser = serde_json::from_value(serde_json::json!({
            "id": "legacy", "email": "legacy@example.test", "display_name": " ", "status": "active"
        }))
        .unwrap();
        assert_eq!(
            legacy.projected_display_name().unwrap(),
            "legacy@example.test"
        );
    }

    #[test]
    fn directory_without_any_usable_name_is_rejected() {
        let mut user = entry(" ", "\t");
        user.email = "\n".into();
        assert!(matches!(
            user.projected_display_name(),
            Err(shared::AppError::Unavailable(_))
        ));
    }

    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn mixed_central_directory_preserves_identity_and_disabled_state_in_postgres() {
        let url = std::env::var("WIKI_TEST_DATABASE_URL").expect("disposable QA database required");
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let migrations = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../migrations");
        sqlx::migrate::Migrator::new(migrations)
            .await
            .unwrap()
            .run(&pool)
            .await
            .unwrap();
        let mut batch = vec![
            entry("Named", "named"),
            entry("", "empty"),
            entry(" \t", "spaced"),
            entry("", ""),
        ];
        for user in &batch {
            persist_central_directory_user(&pool, user).await.unwrap();
        }
        async fn snapshot(
            pool: &sqlx::PgPool,
            batch: &[CentralDirectoryUser],
        ) -> Vec<(Uuid, String, bool, chrono::DateTime<chrono::Utc>)> {
            let ids: Vec<_> = batch.iter().map(|user| user.id.clone()).collect();
            sqlx::query_as("SELECT id, display_name, is_active, updated_at FROM users WHERE central_sub = ANY($1) ORDER BY central_sub")
                .bind(ids).fetch_all(pool).await.unwrap()
        }
        let before = snapshot(&pool, &batch).await;
        assert_eq!(before.len(), 4);
        assert!(
            before
                .iter()
                .all(|(_, name, active, _)| !name.trim().is_empty() && *active)
        );
        for user in &batch {
            persist_central_directory_user(&pool, user).await.unwrap();
        }
        assert_eq!(snapshot(&pool, &batch).await, before);
        let disabled_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE central_sub = $1")
            .bind(&batch[2].id)
            .fetch_one(&pool)
            .await
            .unwrap();
        batch[2].status = "disabled".into();
        batch[2].display_name.clear();
        batch[2].username.clear();
        batch[2].email.clear();
        for user in &batch {
            persist_central_directory_user(&pool, user).await.unwrap();
        }
        let disabled: (Uuid, bool) =
            sqlx::query_as("SELECT id, is_active FROM users WHERE central_sub = $1")
                .bind(&batch[2].id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(disabled.0, disabled_id);
        assert!(!disabled.1);
        let after = snapshot(&pool, &batch).await;
        for user in &batch {
            persist_central_directory_user(&pool, user).await.unwrap();
        }
        assert_eq!(snapshot(&pool, &batch).await, after);
        pool.close().await;
    }
    #[tokio::test]
    #[ignore = "requires a disposable PostgreSQL database in WIKI_TEST_DATABASE_URL"]
    async fn directory_refresh_preserves_local_disable_and_does_not_activate_pending_identity() {
        let backend = profile_backend().await;
        let user = entry("Confirmed display name", "same-name");
        persist_central_directory_user(&backend.pool, &user)
            .await
            .unwrap();
        sqlx::query("UPDATE users SET is_active=false WHERE central_sub=$1")
            .bind(&user.id)
            .execute(&backend.pool)
            .await
            .unwrap();
        persist_central_directory_user(&backend.pool, &user)
            .await
            .unwrap();
        let active: bool = sqlx::query_scalar("SELECT is_active FROM users WHERE central_sub=$1")
            .bind(&user.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert!(
            !active,
            "directory refresh cannot undo an explicit product disable"
        );
        let mut pending = entry("Pending profile", "pending-profile");
        pending.status = "pending".into();
        persist_central_directory_user(&backend.pool, &pending)
            .await
            .unwrap();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE central_sub=$1")
            .bind(&pending.id)
            .fetch_one(&backend.pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        pending.status = "unknown-status".into();
        assert!(
            persist_central_directory_user(&backend.pool, &pending)
                .await
                .is_err()
        );
    }
}
