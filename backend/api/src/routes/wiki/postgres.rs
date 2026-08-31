use super::*;
use app::wiki::{
    WikiSpaceAccess as SpaceAccess, build_wiki_search_criteria, checksum, clamp_limit,
    create_wiki_session_token_pair, create_wiki_token_pair, decode_token, default_username,
    global_role_from_request, hash_password, hash_token, markdown_to_text, normalize_document_type,
    normalize_evidence_type, normalize_phase_key, normalize_required, normalize_slug,
    normalize_space_key, normalize_space_role, normalize_task_key, safe_download_filename, snippet,
    space_role_allows, verify_password,
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::PgPoolOptions, postgres::PgRow};
use std::sync::Arc;

#[derive(Clone)]
struct PostgresWikiBackend {
    pool: PgPool,
    auth: shared::AuthConfig,
    storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
    max_upload_bytes: usize,
    settings: WikiSettingsResponse,
}

pub(super) async fn connect_persistent_backend(
    config: &shared::AppConfig,
    storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
) -> Result<WikiBackend, shared::AppError> {
    let backend = PostgresWikiBackend::connect(config, storage).await?;
    let settings = backend.settings.clone();
    Ok(WikiBackend::persistent(Arc::new(backend), settings))
}

impl PostgresWikiBackend {
    async fn connect(
        config: &shared::AppConfig,
        storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
    ) -> Result<Self, shared::AppError> {
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

        let backend = Self {
            pool,
            auth: config.auth.clone(),
            storage,
            max_upload_bytes: config.storage.max_upload_bytes,
            settings: WikiSettingsResponse::from_config(config),
        };
        backend.bootstrap(&config.bootstrap).await?;
        Ok(backend)
    }

    async fn bootstrap(&self, config: &shared::BootstrapConfig) -> Result<(), shared::AppError> {
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

    async fn get_settings(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiSettingsResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        Ok(self.settings.clone())
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

    async fn list_spaces(
        &self,
        claims: &WikiClaims,
    ) -> Result<SpaceListResponse, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let rows = sqlx::query(SPACE_LIST_SQL)
            .bind(user_id)
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
        let actor_id = self.ensure_admin(claims).await?;
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

    async fn get_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        self.ensure_space_access(claims, space_key, SpaceAccess::View)
            .await?;
        self.get_space_by_key(space_key).await
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
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
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
        let updated_space_id: Uuid = row.get("id");
        debug_assert_eq!(space_id, updated_space_id);
        self.audit(Some(actor_id), "space.update", "space", space_id)
            .await?;
        self.get_space_by_key(&key).await
    }

    async fn archive_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
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
        let archived_space_id: Uuid = row.get("id");
        debug_assert_eq!(space_id, archived_space_id);
        self.audit(Some(actor_id), "space.archive", "space", space_id)
            .await?;
        self.get_space_by_key(&key).await
    }

    async fn list_space_members(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
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
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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

    async fn get_space_tree(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceTreeResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
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
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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

    async fn get_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
        self.document_response(document_id).await
    }

    async fn update_document_draft(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: UpdateDocumentDraftRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        let document_id = self.resolve_document_id(document_id).await?;
        let document_space_id = self
            .ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentRevisionListResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
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
        claims: &WikiClaims,
        document_id: &str,
        revision_id: &str,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
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

    async fn list_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<TaskPageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
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
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
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
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let task = self.get_task(claims, space_key, task_key).await?;
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
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_task(claims, space_key, task_key).await?.evidence,
        })
    }

    async fn list_phases(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<PhasePageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
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
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
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
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
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
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let phase = self.get_phase(claims, space_key, phase_key).await?;
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
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_phase(claims, space_key, phase_key).await?.evidence,
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
        self.ensure_space_id_access(claims, space_id, SpaceAccess::Edit)
            .await?;
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
                "SELECT checksum FROM attachments WHERE id = $1 AND owner_entity_id IS NULL AND uploaded_by = $2",
            )
            .bind(attachment_id)
            .bind(actor_id)
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
        claims: Option<&WikiClaims>,
        query: EvidenceQuery,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        let space_key = query
            .space
            .as_deref()
            .map(normalize_space_key)
            .transpose()?;
        let document_id = match query.document_id.as_deref() {
            Some(value) => {
                let document_id = self.resolve_document_id(value).await?;
                if let Some(claims) = claims {
                    self.ensure_document_access(claims, document_id, SpaceAccess::View)
                        .await?;
                }
                Some(document_id)
            }
            None => None,
        };
        let access_user_id = match claims {
            Some(claims) => {
                if let Some(space_key) = space_key.as_deref() {
                    self.ensure_space_access(claims, space_key, SpaceAccess::View)
                        .await?;
                }
                self.restricted_user_id(claims).await?
            }
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
            .bind(access_user_id)
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

    async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let evidence_id = parse_uuid(evidence_id, "evidence")?;
        self.ensure_evidence_access(claims, evidence_id, SpaceAccess::View)
            .await?;
        self.get_evidence_by_id(evidence_id).await
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
        self.storage.put(&storage_key, &bytes).await?;

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
                let _ = self.storage.delete(&storage_key).await;
                return Err(shared::AppError::database(err));
            }
        };

        self.audit(Some(actor_id), "attachment.upload", "attachment", id)
            .await?;
        Ok(attachment_response_from_row(&row))
    }

    async fn get_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        self.ensure_attachment_access(claims, attachment_id).await?;
        let row = sqlx::query(ATTACHMENT_ONE_SQL)
            .bind(attachment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        Ok(attachment_response_from_row(&row))
    }

    async fn download_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<Response, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        self.ensure_attachment_access(claims, attachment_id).await?;
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
        let bytes = self.storage.get(&storage_key).await.map_err(|err| {
            if matches!(err, shared::AppError::NotFound(_)) {
                shared::AppError::not_found("attachment file", attachment_id)
            } else {
                err
            }
        })?;
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

    async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, shared::AppError> {
        let criteria = build_wiki_search_criteria(
            query.q.as_deref(),
            query.space.as_deref(),
            query.task_key.as_deref(),
            query.phase_key.as_deref(),
            query.document_type.as_deref(),
            query.include_archived,
            query.limit,
        )?;
        if let Some(space_key) = criteria.space_key.as_deref() {
            self.ensure_space_access(claims, space_key, SpaceAccess::View)
                .await?;
        }
        let access_user_id = self.restricted_user_id(claims).await?;

        let document_rows = sqlx::query(SEARCH_DOCUMENTS_SQL)
            .bind(&criteria.needle)
            .bind(criteria.space_key.as_deref())
            .bind(criteria.task_key.as_deref())
            .bind(criteria.phase_key.as_deref())
            .bind(criteria.document_type)
            .bind(criteria.include_archived)
            .bind(access_user_id)
            .bind(criteria.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        let evidence_rows = sqlx::query(SEARCH_EVIDENCE_SQL)
            .bind(&criteria.evidence_like_pattern)
            .bind(criteria.space_key.as_deref())
            .bind(criteria.task_key.as_deref())
            .bind(criteria.phase_key.as_deref())
            .bind(access_user_id)
            .bind(criteria.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;

        let mut results = document_rows
            .iter()
            .map(search_result_from_row)
            .chain(evidence_rows.iter().map(search_result_from_row))
            .collect::<Vec<_>>();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(criteria.limit as usize);
        Ok(SearchResponse { results })
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

    async fn ensure_evidence_access(
        &self,
        claims: &WikiClaims,
        evidence_id: Uuid,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let space_id: Uuid =
            sqlx::query_scalar("SELECT space_id FROM evidence_items WHERE id = $1")
                .bind(evidence_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?
                .ok_or_else(|| shared::AppError::not_found("evidence", evidence_id))?;
        self.ensure_space_id_access(claims, space_id, required)
            .await?;
        Ok(space_id)
    }

    async fn ensure_attachment_access(
        &self,
        claims: &WikiClaims,
        attachment_id: Uuid,
    ) -> Result<(), shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let row = sqlx::query(
            r#"
            SELECT space_id, uploaded_by
            FROM attachments
            WHERE id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        let uploaded_by: Uuid = row.get("uploaded_by");
        let space_id: Option<Uuid> = row.get("space_id");
        match space_id {
            Some(space_id) => {
                self.ensure_space_id_access(claims, space_id, SpaceAccess::View)
                    .await?;
                Ok(())
            }
            None => {
                let role = self.active_global_role(user_id).await?;
                if role == "admin" || uploaded_by == user_id {
                    Ok(())
                } else {
                    Err(shared::AppError::Forbidden)
                }
            }
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
            .list_evidence(
                None,
                EvidenceQuery {
                    space: None,
                    document_id: Some(document_id.to_string()),
                    task_key: None,
                    phase_key: None,
                    limit: Some(100),
                },
            )
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
    JOIN users u ON u.id = $1 AND u.is_active = true
    LEFT JOIN space_members actor_member ON actor_member.space_id = s.id AND actor_member.user_id = u.id
    WHERE u.global_role = 'admin' OR actor_member.user_id IS NOT NULL
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
      AND ($5::uuid IS NULL OR EXISTS (
          SELECT 1
          FROM space_members sm
          WHERE sm.space_id = e.space_id AND sm.user_id = $5
      ))
    ORDER BY e.created_at DESC
    LIMIT $6
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
    WITH search_query AS (
        SELECT CASE
            WHEN NULLIF(btrim($1::text), '') IS NULL THEN NULL
            ELSE websearch_to_tsquery('simple', $1::text)
        END AS query
    )
    SELECT d.id,
           'document' AS result_type,
           d.title,
           s.key AS space_key,
           '/documents/' || d.slug AS url,
           COALESCE(NULLIF(cr.content_text, ''), NULLIF(dd.content_markdown, ''), d.title) AS snippet,
           d.updated_at
    FROM search_query sq
    CROSS JOIN documents d
    JOIN spaces s ON s.id = d.space_id
    LEFT JOIN document_drafts dd ON dd.document_id = d.id
    LEFT JOIN document_revisions cr ON cr.id = d.current_revision_id
    WHERE (
        sq.query IS NULL
        OR cr.search_vector @@ sq.query
        OR (
            cr.id IS NULL
            AND (
                setweight(to_tsvector('simple', coalesce(d.title, '')), 'A')
                || setweight(to_tsvector('simple', coalesce(dd.content_markdown, '')), 'B')
            ) @@ sq.query
        )
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
      AND ($7::uuid IS NULL OR EXISTS (
          SELECT 1
          FROM space_members sm
          WHERE sm.space_id = d.space_id AND sm.user_id = $7
      ))
    ORDER BY d.updated_at DESC
    LIMIT $8
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
        OR lower(e.title) LIKE $1 ESCAPE E'\\'
        OR lower(COALESCE(e.url, '')) LIKE $1 ESCAPE E'\\'
    )
      AND ($2::text IS NULL OR s.key = $2)
      AND ($3::text IS NULL OR td.task_key = $3)
      AND ($4::text IS NULL OR pd.phase_key = $4)
      AND ($5::uuid IS NULL OR EXISTS (
          SELECT 1
          FROM space_members sm
          WHERE sm.space_id = e.space_id AND sm.user_id = $5
      ))
    ORDER BY e.created_at DESC
    LIMIT $6
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

fn parse_uuid(value: &str, entity: &str) -> Result<Uuid, shared::AppError> {
    Uuid::parse_str(value).map_err(|_| shared::AppError::not_found(entity, value))
}

#[async_trait::async_trait]
impl WikiBackendPort for PostgresWikiBackend {
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
    ) -> Result<WikiSettingsResponse, shared::AppError> {
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
    ) -> Result<Response, shared::AppError> {
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
