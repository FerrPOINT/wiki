use super::{PostgresWikiBackend, mapping::audit_entry_from_row};
use app::wiki::{
    WikiAuditCommand, WikiAuditRepository, WikiAuditRepositoryFuture, WikiAuditUseCase,
};
use shared::wiki_contract::*;
use sqlx::Postgres;
use uuid::Uuid;

struct PostgresWikiAuditRepository<'a> {
    backend: &'a PostgresWikiBackend,
    request_id: Option<&'a str>,
}

impl WikiAuditRepository for PostgresWikiAuditRepository<'_> {
    fn list_recent_entries<'a>(
        &'a self,
        limit: usize,
    ) -> WikiAuditRepositoryFuture<'a, Vec<AuditEntryResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                SELECT id, actor_id, action, entity_type, entity_id, request_id, created_at
                FROM audit_log
                ORDER BY created_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit as i64)
            .fetch_all(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(rows.iter().map(audit_entry_from_row).collect())
        })
    }

    fn record_entry<'a>(&'a self, command: WikiAuditCommand) -> WikiAuditRepositoryFuture<'a, ()> {
        Box::pin(async move {
            sqlx::query(
                r#"
                INSERT INTO audit_log (
                    id, actor_id, action, entity_type, entity_id, request_id, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, now())
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(command.actor_id)
            .bind(&command.action)
            .bind(&command.entity_type)
            .bind(command.entity_id)
            .bind(audit_request_id(self.request_id))
            .execute(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(())
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn list_audit_log(
        &self,
        claims: &WikiClaims,
    ) -> Result<AuditLogResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let repository = PostgresWikiAuditRepository {
            backend: self,
            request_id: None,
        };
        WikiAuditUseCase::new(&repository).list_recent().await
    }

    pub(super) async fn audit(
        &self,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), shared::AppError> {
        let repository = PostgresWikiAuditRepository {
            backend: self,
            request_id: None,
        };
        WikiAuditUseCase::new(&repository)
            .record(actor_id, action, entity_type, entity_id)
            .await
    }

    pub(super) async fn insert_audit(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
        request_id: Option<&str>,
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
        .bind(audit_request_id(request_id))
        .execute(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(())
    }
}

fn audit_request_id(request_id: Option<&str>) -> String {
    request_id
        .filter(|value| !value.trim().is_empty() && value.len() <= 128)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("api-{}", Uuid::now_v7().simple()))
}
