use super::PostgresWikiBackend;
use super::database_error;
use super::mapping::parse_uuid;
use shared::wiki_contract::{WikiIdempotencyReplay, WikiIdempotencyRequest, WikiIdempotencyStatus};
use sqlx::Row;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn begin_idempotent_request(
        &self,
        request: WikiIdempotencyRequest,
    ) -> Result<WikiIdempotencyStatus, shared::AppError> {
        let actor_id = parse_uuid(&request.actor_id, "user")?;
        // Reuse only this expired, confirmed-completed key. Unknown processing
        // outcomes belong to inventory/readback, never to time-based retry.
        sqlx::query("DELETE FROM idempotency_records WHERE actor_id=$1 AND idempotency_key=$2 AND state='completed' AND expires_at<now()")
            .bind(actor_id).bind(&request.key).execute(&self.pool).await.map_err(database_error)?;

        let inserted = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO idempotency_records (
                id, actor_id, idempotency_key, method, path, request_hash,
                state, created_at, updated_at, expires_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6,
                'processing', now(), now(), now() + interval '24 hours'
            )
            ON CONFLICT (actor_id, idempotency_key) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(actor_id)
        .bind(&request.key)
        .bind(&request.method)
        .bind(&request.path)
        .bind(&request.request_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        if inserted.is_some() {
            return Ok(WikiIdempotencyStatus::Started);
        }

        let row = sqlx::query(
            r#"
            SELECT method, path, request_hash, state, response_status,
                   response_content_type, response_body
            FROM idempotency_records
            WHERE actor_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(actor_id)
        .bind(&request.key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?
        .ok_or_else(|| {
            shared::AppError::internal("idempotency record disappeared during lookup")
        })?;

        let stored_method: String = row.get("method");
        let stored_path: String = row.get("path");
        let stored_hash: String = row.get("request_hash");
        if stored_method != request.method
            || stored_path != request.path
            || stored_hash != request.request_hash
        {
            return Err(shared::AppError::conflict(
                "idempotency key was reused for a different request",
            ));
        }

        let state: String = row.get("state");
        if state == "processing" {
            return Err(shared::AppError::conflict(
                "idempotent request is already processing",
            ));
        }

        let status_code = row
            .try_get::<Option<i32>, _>("response_status")
            .map_err(database_error)?
            .ok_or_else(|| {
                shared::AppError::internal("completed idempotency record has no status")
            })?;
        let content_type = row
            .try_get::<Option<String>, _>("response_content_type")
            .map_err(database_error)?;
        let body = row
            .try_get::<Option<Vec<u8>>, _>("response_body")
            .map_err(database_error)?
            .ok_or_else(|| {
                shared::AppError::internal("completed idempotency record has no body")
            })?;

        Ok(WikiIdempotencyStatus::Replay(WikiIdempotencyReplay {
            status_code: status_code as u16,
            content_type,
            body,
        }))
    }

    pub(super) async fn complete_idempotent_request(
        &self,
        request: WikiIdempotencyRequest,
        replay: WikiIdempotencyReplay,
    ) -> Result<(), shared::AppError> {
        let actor_id = parse_uuid(&request.actor_id, "user")?;
        let rows = sqlx::query(
            r#"
            UPDATE idempotency_records
            SET state = 'completed',
                response_status = $6,
                response_content_type = $7,
                response_body = $8,
                updated_at = now()
            WHERE actor_id = $1
              AND idempotency_key = $2
              AND method = $3
              AND path = $4
              AND request_hash = $5
              AND state = 'processing'
            "#,
        )
        .bind(actor_id)
        .bind(&request.key)
        .bind(&request.method)
        .bind(&request.path)
        .bind(&request.request_hash)
        .bind(i32::from(replay.status_code))
        .bind(&replay.content_type)
        .bind(&replay.body)
        .execute(&self.pool)
        .await
        .map_err(database_error)?
        .rows_affected();

        if rows == 0 {
            return Err(shared::AppError::internal(
                "idempotency record could not be completed",
            ));
        }

        Ok(())
    }

    pub(super) async fn abandon_idempotent_request(
        &self,
        request: WikiIdempotencyRequest,
    ) -> Result<(), shared::AppError> {
        let actor_id = parse_uuid(&request.actor_id, "user")?;
        sqlx::query(
            r#"
            DELETE FROM idempotency_records
            WHERE actor_id = $1
              AND idempotency_key = $2
              AND method = $3
              AND path = $4
              AND request_hash = $5
              AND state = 'processing'
            "#,
        )
        .bind(actor_id)
        .bind(&request.key)
        .bind(&request.method)
        .bind(&request.path)
        .bind(&request.request_hash)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;
        Ok(())
    }

    pub(super) async fn delete_expired_idempotency_records(&self) -> Result<u64, shared::AppError> {
        let rows = sqlx::query(
            "WITH expired AS (
                SELECT id FROM idempotency_records WHERE state='completed' AND expires_at<now()
                ORDER BY expires_at,id LIMIT $1 FOR UPDATE SKIP LOCKED
            ) DELETE FROM idempotency_records WHERE id IN(SELECT id FROM expired)",
        )
        .bind(i64::from(self.maintenance_batch_size))
        .execute(&self.pool)
        .await
        .map_err(database_error)?
        .rows_affected();
        Ok(rows)
    }
}
