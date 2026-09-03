use super::PostgresWikiBackend;
use chrono::{Duration, Utc};
use shared::wiki_contract::WikiMaintenanceReport;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn run_maintenance(&self) -> Result<WikiMaintenanceReport, shared::AppError> {
        let expired_staged_attachments = self.delete_expired_staged_attachments().await?;
        let mut file_delete_failures = 0;

        for (attachment_id, storage_key) in &expired_staged_attachments {
            if let Err(err) = self.storage.delete(storage_key).await {
                file_delete_failures += 1;
                tracing::warn!(
                    attachment_id = %attachment_id,
                    error = %err,
                    "failed to delete expired staged attachment file"
                );
            }
        }

        let expired_idempotency_records_deleted = self.delete_expired_idempotency_records().await?;

        Ok(WikiMaintenanceReport {
            expired_staged_attachments_deleted: expired_staged_attachments.len() as u64,
            expired_staged_attachment_file_delete_failures: file_delete_failures,
            expired_idempotency_records_deleted,
        })
    }

    async fn delete_expired_staged_attachments(
        &self,
    ) -> Result<Vec<(Uuid, String)>, shared::AppError> {
        let cutoff = Utc::now() - Duration::hours(i64::from(self.staged_attachment_ttl_hours));
        sqlx::query_as::<_, (Uuid, String)>(
            r#"
            WITH expired AS (
                SELECT id, storage_key
                FROM attachments
                WHERE space_id IS NULL
                  AND owner_entity_type IS NULL
                  AND owner_entity_id IS NULL
                  AND uploaded_at < $1
                ORDER BY uploaded_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            DELETE FROM attachments a
            USING expired
            WHERE a.id = expired.id
            RETURNING a.id, a.storage_key
            "#,
        )
        .bind(cutoff)
        .bind(i64::from(self.maintenance_batch_size))
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)
    }
}
