use super::PostgresWikiBackend;
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
        self.delete_expired_idempotency_records().await?;

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
        .map_err(shared::AppError::database)?;

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
        .map_err(shared::AppError::database)?
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
            .map_err(shared::AppError::database)?
            .ok_or_else(|| {
                shared::AppError::internal("completed idempotency record has no status")
            })?;
        let content_type = row
            .try_get::<Option<String>, _>("response_content_type")
            .map_err(shared::AppError::database)?;
        let body = row
            .try_get::<Option<Vec<u8>>, _>("response_body")
            .map_err(shared::AppError::database)?
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
        .map_err(shared::AppError::database)?
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
        .map_err(shared::AppError::database)?;
        Ok(())
    }

    async fn delete_expired_idempotency_records(&self) -> Result<(), shared::AppError> {
        sqlx::query("DELETE FROM idempotency_records WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(())
    }
}
