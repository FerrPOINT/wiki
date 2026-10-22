use super::PostgresWikiBackend;
use super::database_error;
use chrono::{Duration, Utc};
use shared::wiki_contract::WikiMaintenanceReport;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn run_maintenance(&self) -> Result<WikiMaintenanceReport, shared::AppError> {
        let expired_staged_attachments_deleted = self.enqueue_expired_staged_attachments().await?;
        let mut file_delete_failures = 0;
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        super::write_authorization::bind_current_clock(&mut tx).await?;
        let pending = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT attachment_id, storage_key FROM attachment_file_cleanup ORDER BY created_at, attachment_id LIMIT $1 FOR UPDATE SKIP LOCKED"
        ).bind(i64::from(self.maintenance_batch_size)).fetch_all(&mut *tx).await.map_err(database_error)?;
        for (attachment_id, storage_key) in &pending {
            if let Err(err) = self.storage.delete(storage_key).await {
                file_delete_failures += 1;
                tracing::warn!(
                    attachment_id = %attachment_id,
                    error = %err,
                    "failed to delete expired staged attachment file"
                );
                continue;
            }
            sqlx::query("DELETE FROM attachment_file_cleanup WHERE attachment_id=$1")
                .bind(attachment_id)
                .execute(&mut *tx)
                .await
                .map_err(database_error)?;
        }
        tx.commit().await.map_err(database_error)?;

        let expired_idempotency_records_deleted = self.delete_expired_idempotency_records().await?;

        Ok(WikiMaintenanceReport {
            expired_staged_attachments_deleted,
            expired_staged_attachment_file_delete_failures: file_delete_failures,
            expired_idempotency_records_deleted,
        })
    }

    async fn enqueue_expired_staged_attachments(&self) -> Result<u64, shared::AppError> {
        let cutoff = Utc::now() - Duration::hours(i64::from(self.staged_attachment_ttl_hours));
        let result = sqlx::query(
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
            ), removed AS (
                DELETE FROM attachments a
                USING expired
                WHERE a.id = expired.id
                RETURNING a.id, a.storage_key
            )
            INSERT INTO attachment_file_cleanup(attachment_id, storage_key)
            SELECT id, storage_key FROM removed
            "#,
        )
        .bind(cutoff)
        .bind(i64::from(self.maintenance_batch_size))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;
        Ok(result.rows_affected())
    }
}
