use super::{PostgresWikiBackend, mapping::audit_entry_from_row};
use shared::wiki_contract::*;
use sqlx::Postgres;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn list_audit_log(
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

    pub(super) async fn audit(
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

    pub(super) async fn insert_audit(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
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
