use super::PostgresWikiBackend;
use super::database_error;
use super::mapping::parse_uuid;
use app::wiki::WIKI_COMMAND;
use shared::wiki_contract::{WikiIdempotencyReplay, WikiIdempotencyRequest, WikiIdempotencyStatus};
use shared::{AppError, wiki_space_access::WikiSpaceAdmission};
use sqlx::{Postgres, Row, Transaction};
use std::sync::{Arc, atomic::Ordering};
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
        // Completion is published by the content owner before COMMIT. The HTTP
        // adapter only verifies that its actual envelope matches that saved result.
        let matched: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM idempotency_records
             WHERE actor_id=$1 AND idempotency_key=$2 AND method=$3 AND path=$4
               AND request_hash=$5 AND state='completed' AND response_status=$6
               AND response_content_type IS NOT DISTINCT FROM $7 AND response_body=$8)",
        )
        .bind(parse_uuid(&request.actor_id, "user")?)
        .bind(&request.key)
        .bind(&request.method)
        .bind(&request.path)
        .bind(&request.request_hash)
        .bind(i32::from(replay.status_code))
        .bind(&replay.content_type)
        .bind(&replay.body)
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;
        if matched {
            Ok(())
        } else {
            Err(shared::AppError::Unavailable(
                "command_result_not_confirmed".into(),
            ))
        }
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

pub(super) fn has_command() -> bool {
    WIKI_COMMAND.try_with(|_| ()).is_ok()
}
pub(super) fn is_replay() -> bool {
    WIKI_COMMAND
        .try_with(|context| context.replaying)
        .unwrap_or(false)
}

/// Call after the owner has checked the current subject/resource authorization.
pub(super) async fn replay<T: serde::de::DeserializeOwned>(
    tx: &mut Transaction<'_, Postgres>,
    actor: Uuid,
) -> Result<Option<T>, AppError> {
    let Ok(context) = WIKI_COMMAND.try_with(Arc::clone) else {
        return Ok(None);
    };
    if parse_uuid(&context.request.actor_id, "user")? != actor {
        return Err(AppError::Forbidden);
    }
    let row=sqlx::query(
        "SELECT state,response_status,response_body FROM idempotency_records
         WHERE actor_id=$1 AND idempotency_key=$2 AND method=$3 AND path=$4 AND request_hash=$5 FOR UPDATE"
    ).bind(actor).bind(&context.request.key).bind(&context.request.method).bind(&context.request.path)
        .bind(&context.request.request_hash).fetch_optional(&mut **tx).await.map_err(database_error)?
        .ok_or_else(||AppError::Unavailable("command_reservation_not_confirmed".into()))?;
    if row.get::<String, _>("state") != "completed" {
        return Ok(None);
    }
    let status: Option<i32> = row.get("response_status");
    let body: Option<Vec<u8>> = row.get("response_body");
    let body = body.ok_or_else(|| AppError::internal("completed command has no result"))?;
    let result = if status == Some(204) {
        serde_json::from_slice(b"null")
    } else {
        serde_json::from_slice(&body)
    };
    result
        .map(Some)
        .map_err(|_| AppError::Unavailable("stored_command_result_incompatible".into()))
}

impl PostgresWikiBackend {
    pub(super) async fn finish_command<T: serde::Serialize>(
        &self,
        mut tx: Transaction<'_, Postgres>,
        admission: Option<&Arc<WikiSpaceAdmission>>,
        status: u16,
        result: &T,
    ) -> Result<(), AppError> {
        let binding = if let Ok(context) = WIKI_COMMAND.try_with(Arc::clone) {
            let body = if status == 204 {
                Vec::new()
            } else {
                serde_json::to_vec(result)
                    .map_err(|_| AppError::internal("command result serialization failed"))?
            };
            let content_type = (status != 204).then_some("application/json");
            let id: Option<Uuid> = sqlx::query_scalar(
                "UPDATE idempotency_records SET state='completed',response_status=$6,
                 response_content_type=$7,response_body=$8,updated_at=now()
                 WHERE actor_id=$1 AND idempotency_key=$2 AND method=$3 AND path=$4
                   AND request_hash=$5 AND state='processing' RETURNING id",
            )
            .bind(parse_uuid(&context.request.actor_id, "user")?)
            .bind(&context.request.key)
            .bind(&context.request.method)
            .bind(&context.request.path)
            .bind(&context.request.request_hash)
            .bind(i32::from(status))
            .bind(content_type)
            .bind(&body)
            .fetch_optional(&mut *tx)
            .await
            .map_err(database_error)?;
            let id =
                id.ok_or_else(|| AppError::Unavailable("command_result_not_reserved".into()))?;
            Some((context, id, body, content_type))
        } else {
            None
        };
        match super::write_authorization::commit(tx, admission).await {
            Ok(()) => Ok(()),
            Err(error) => {
                if let Some((context, id, body, content_type)) = binding {
                    if context.commit_attempted.load(Ordering::SeqCst) {
                        match self
                            .command_commit_readback(id, status, content_type, &body)
                            .await
                        {
                            Ok(true) => return Ok(()),
                            Ok(false) => context.commit_attempted.store(false, Ordering::SeqCst),
                            Err(_) => {
                                tracing::warn!(command_id=%id,"Wiki command COMMIT unresolved; reservation retained")
                            }
                        }
                    }
                }
                Err(error)
            }
        }
    }

    async fn command_commit_readback(
        &self,
        id: Uuid,
        status: u16,
        content_type: Option<&str>,
        body: &[u8],
    ) -> Result<bool, AppError> {
        let fresh = self
            .pool
            .options()
            .clone()
            .max_connections(1)
            .min_connections(0)
            .connect_with((*self.pool.connect_options()).clone())
            .await
            .map_err(database_error)?;
        let outcome=async {
            let mut tx=fresh.begin().await.map_err(database_error)?;
            sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED").execute(&mut *tx).await.map_err(database_error)?;
            // The result row is updated by the original content transaction. Its
            // conflicting lock waits for that exact COMMIT, including a lost reply.
            let row=sqlx::query("SELECT state,response_status,response_content_type,response_body FROM idempotency_records WHERE id=$1 FOR UPDATE")
                .bind(id).fetch_optional(&mut *tx).await.map_err(database_error)?
                .ok_or_else(||AppError::Unavailable("original_command_reservation_missing".into()))?;
            let state:String=row.get("state");
            let stored_status:Option<i32>=row.get("response_status");
            let stored_type:Option<String>=row.get("response_content_type");
            let stored_body:Option<Vec<u8>>=row.get("response_body");
            let outcome=match state.as_str() {
                "completed" if stored_status==Some(i32::from(status))
                    && stored_type.as_deref()==content_type && stored_body.as_deref()==Some(body)=>true,
                "processing" if stored_status.is_none() && stored_type.is_none() && stored_body.is_none()=>false,
                _=>return Err(AppError::Unavailable("original_command_result_inconsistent".into())),
            };
            tx.rollback().await.map_err(database_error)?;
            Ok(outcome)
        }.await;
        fresh.close().await;
        outcome
    }
}

/// Called immediately before the actual COMMIT, after proof/role checks.
pub(super) fn mark_commit_attempt() {
    if let Ok(context) = WIKI_COMMAND.try_with(Arc::clone) {
        context.commit_attempted.store(true, Ordering::SeqCst);
    }
}

impl PostgresWikiBackend {
    /// Authorization is current, but a replay must not re-run domain preconditions or effects.
    pub(super) async fn replay_command<T: serde::de::DeserializeOwned>(
        &self,
        claims: &shared::WikiClaims,
        spaces: &[Uuid],
        required: super::SpaceAccess,
    ) -> Result<Option<T>, AppError> {
        if !WIKI_COMMAND
            .try_with(|context| context.replaying)
            .unwrap_or(false)
        {
            return Ok(None);
        }
        let actor = parse_uuid(&claims.user_id, "user")?;
        let mut tx = self
            .begin_replay_read(actor, claims.admission.as_ref(), spaces, required)
            .await?;
        let response = replay(&mut tx, actor).await?;
        tx.rollback().await.map_err(database_error)?;
        response
            .map(Some)
            .ok_or_else(|| AppError::Unavailable("command_replay_no_longer_confirmed".into()))
    }
}
