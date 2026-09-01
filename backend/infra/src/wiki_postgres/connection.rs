use super::PostgresWikiBackend;
use app::wiki::{default_username, hash_password};
use shared::wiki_contract::{WikiBackendPort, WikiSettingsSnapshot};
use shared::{AppConfig, AppError, BootstrapConfig};
use sqlx::{Row, postgres::PgPoolOptions};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use uuid::Uuid;

pub async fn connect_postgres_wiki_backend(
    config: &AppConfig,
    storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
) -> Result<(Arc<dyn WikiBackendPort>, WikiSettingsSnapshot), AppError> {
    if config.database.url.trim().is_empty() {
        return Err(AppError::invalid_input(
            "WIKI_DATABASE__URL is required for PostgreSQL Wiki runtime",
        ));
    }

    let backend = PostgresWikiBackend::connect(config, storage).await?;
    let settings = backend.settings.clone();
    Ok((Arc::new(backend), settings))
}

impl PostgresWikiBackend {
    async fn connect(
        config: &AppConfig,
        storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
    ) -> Result<Self, AppError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(Duration::from_secs(config.database.connect_timeout_seconds))
            .idle_timeout(Duration::from_secs(config.database.idle_timeout_seconds))
            .connect(&config.database.url)
            .await
            .map_err(AppError::database)?;

        let migrator = sqlx::migrate::Migrator::new(migrations_dir())
            .await
            .map_err(AppError::database)?;
        migrator.run(&pool).await.map_err(AppError::database)?;

        let backend = Self {
            pool,
            auth: config.auth.clone(),
            storage,
            max_upload_bytes: config.storage.max_upload_bytes,
            settings: WikiSettingsSnapshot::from_config(config),
        };
        backend.bootstrap(&config.bootstrap).await?;
        Ok(backend)
    }

    async fn bootstrap(&self, config: &BootstrapConfig) -> Result<(), AppError> {
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
                .map_err(AppError::database)?;
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
                .map_err(AppError::database)?;
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
                .map_err(AppError::database)?;

                self.audit(Some(admin_id), "wiki.bootstrap", "space", space_id)
                    .await?;
            }
            (None, None) => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM users")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(AppError::database)?;
                if count == 0 {
                    tracing::warn!(
                        "Wiki database has no users; set WIKI_BOOTSTRAP__ADMIN_EMAIL and WIKI_BOOTSTRAP__ADMIN_PASSWORD or register the first user"
                    );
                }
            }
            _ => {
                return Err(AppError::invalid_input(
                    "bootstrap admin email and password must be set together",
                ));
            }
        }

        Ok(())
    }

    async fn seed_templates(&self) -> Result<(), AppError> {
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
            .map_err(AppError::database)?;
        }
        Ok(())
    }
}

fn migrations_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("WIKI_MIGRATIONS_DIR") {
        return PathBuf::from(path);
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("migrations")
}
