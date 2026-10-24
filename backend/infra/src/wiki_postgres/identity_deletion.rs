//! Durable owner fences. Every owned reference writer is guarded by the additive identity fence migration.
use pdlc_shared::identity_lifecycle::{
    DeletionRequest, OwnerFreezeReceipt, OwnerReferenceSummary, OwnerReleaseReceipt,
};
use serde_json::Value;
use shared::identity_deletion::IdentityDeletionWindow;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

const OWNER: &str = "wiki";
const SOURCES: &[&str] = &[
    "users",
    "wiki_identity_aliases",
    "spaces",
    "space_members",
    "documents",
    "document_revisions",
    "document_drafts",
    "document_task_links",
    "document_phase_links",
    "attachments",
    "evidence_items",
    "audit_log",
    "auth_sessions",
    "idempotency_records",
    "managed_task_document_links",
    "wiki_space_projects",
    "wiki_confirmed_global_roles",
    "wiki_namespace_bindings",
    "wiki_project_space_projection_sets",
    "wiki_space_owner_operations",
    "wiki_project_space_projection_operations",
];

#[derive(Debug)]
pub enum OwnerError {
    Invalid,
    Conflict,
    Unbound,
    Expired,
    Storage,
}
fn storage(error: sqlx::Error) -> OwnerError {
    if error.as_database_error().is_some_and(|error| {
        error.code().as_deref() == Some("42501")
            && error.message() == "pdlc_request_authority_expired"
    }) {
        OwnerError::Expired
    } else {
        OwnerError::Storage
    }
}

#[derive(Clone)]
pub(super) struct IdentityDeletionStore {
    pool: PgPool,
}
impl IdentityDeletionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// An explicit deployment assertion, never inferred from email or UUID similarity.
    pub async fn bind_legacy_issuer(&self, issuer: &str) -> Result<(), OwnerError> {
        let url = reqwest::Url::parse(issuer).map_err(|_| OwnerError::Invalid)?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(OwnerError::Invalid);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        lock(&mut tx).await?;
        let bound: Option<String> =
            sqlx::query_scalar("SELECT issuer FROM wiki_identity_legacy_issuer WHERE singleton")
                .fetch_optional(&mut *tx)
                .await
                .map_err(storage)?;
        match bound {
            Some(bound) if bound != issuer => return Err(OwnerError::Conflict),
            Some(_) => (),
            None => {
                sqlx::query(
                    "INSERT INTO wiki_identity_legacy_issuer(singleton,issuer) VALUES(true,$1)",
                )
                .bind(issuer)
                .execute(&mut *tx)
                .await
                .map_err(storage)?;
            }
        }
        tx.commit().await.map_err(storage)
    }

    async fn begin(
        &self,
        request: &DeletionRequest,
        window: &IdentityDeletionWindow,
    ) -> Result<Transaction<'_, Postgres>, OwnerError> {
        if !request.valid() {
            return Err(OwnerError::Invalid);
        }
        let mut tx = self.pool.begin().await.map_err(storage)?;
        sqlx::query(
            "SELECT set_config('lock_timeout','8s',true),set_config('statement_timeout','8s',true)",
        )
        .execute(&mut *tx)
        .await
        .map_err(storage)?;
        let observed_at = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&mut *tx)
            .await
            .map_err(storage)?;
        let until = window
            .commit_valid_until(observed_at)
            .map_err(|_| OwnerError::Expired)?;
        let fence = Uuid::new_v4();
        sqlx::query("INSERT INTO wiki_request_commit_fences(fence_id,valid_until) VALUES($1,$2)")
            .bind(fence)
            .bind(until)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
        sqlx::query("DELETE FROM wiki_request_commit_fences WHERE fence_id=$1")
            .bind(fence)
            .execute(&mut *tx)
            .await
            .map_err(storage)?;
        lock(&mut tx).await?;
        let bound: Option<String> =
            sqlx::query_scalar("SELECT issuer FROM wiki_identity_legacy_issuer WHERE singleton")
                .fetch_optional(&mut *tx)
                .await
                .map_err(storage)?;
        if bound.as_deref() != Some(request.identity.issuer.as_str()) {
            return Err(OwnerError::Unbound);
        }
        Ok(tx)
    }

    pub async fn check(
        &self,
        request: &DeletionRequest,
        window: &IdentityDeletionWindow,
    ) -> Result<OwnerReferenceSummary, OwnerError> {
        let mut tx = self.begin(request, window).await?;
        if !window.is_current() {
            return Err(OwnerError::Expired);
        }
        verify_operation(&mut tx, request).await?;
        let reference_count = count(&mut tx, request).await?;
        let revision = revision(&mut tx).await?;
        if !window.is_current() {
            return Err(OwnerError::Expired);
        }
        tx.commit().await.map_err(storage)?;
        Ok(OwnerReferenceSummary {
            request: request.clone(),
            owner: OWNER.into(),
            revision,
            reference_count,
            fencing_supported: true,
        })
    }

    pub async fn freeze(
        &self,
        request: &DeletionRequest,
        window: &IdentityDeletionWindow,
    ) -> Result<OwnerFreezeReceipt, OwnerError> {
        let mut tx = self.begin(request, window).await?;
        if !window.is_current() {
            return Err(OwnerError::Expired);
        }
        if let Some(row) = verify_operation(&mut tx, request).await? {
            if row.try_get::<String, _>("state").map_err(storage)? != "frozen" {
                return Err(OwnerError::Conflict);
            }
            let result = OwnerFreezeReceipt {
                request: request.clone(),
                owner: OWNER.into(),
                fence: row.try_get("fence").map_err(storage)?,
                revision: row.try_get::<i64, _>("revision").map_err(storage)? as u64,
                reference_count: row.try_get::<i64, _>("reference_count").map_err(storage)? as u64,
            };
            if !window.is_current() {
                return Err(OwnerError::Expired);
            }
            tx.commit().await.map_err(storage)?;
            return Ok(result);
        }
        let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_identity_deletion_operations WHERE issuer=$1 AND subject=$2 AND state='frozen')")
            .bind(&request.identity.issuer).bind(&request.identity.subject).fetch_one(&mut *tx).await.map_err(storage)?;
        if active {
            return Err(OwnerError::Conflict);
        }
        let reference_count = count(&mut tx, request).await?;
        let revision = revision(&mut tx).await?;
        let fence = Uuid::new_v4();
        sqlx::query("INSERT INTO wiki_identity_deletion_operations(operation_id,issuer,subject,inventory_revision,request,state,fence,revision,reference_count) VALUES($1,$2,$3,$4,$5,'frozen',$6,$7,$8)")
            .bind(request.operation_id).bind(&request.identity.issuer).bind(&request.identity.subject)
            .bind(request.inventory_revision).bind(serde_json::to_value(request).map_err(|_|OwnerError::Invalid)?)
            .bind(fence).bind(revision as i64).bind(reference_count as i64).execute(&mut *tx).await.map_err(storage)?;
        if !window.is_current() {
            return Err(OwnerError::Expired);
        }
        tx.commit().await.map_err(storage)?;
        Ok(OwnerFreezeReceipt {
            request: request.clone(),
            owner: OWNER.into(),
            fence,
            revision,
            reference_count,
        })
    }

    pub async fn release(
        &self,
        request: &DeletionRequest,
        window: &IdentityDeletionWindow,
    ) -> Result<OwnerReleaseReceipt, OwnerError> {
        let mut tx = self.begin(request, window).await?;
        if !window.is_current() {
            return Err(OwnerError::Expired);
        }
        // The trusted coordinator serializes cancellation against finalization.
        // A release receipt is durable even if the original freeze never arrived.
        verify_operation(&mut tx, request).await?;
        let revision = revision(&mut tx).await?;
        sqlx::query("INSERT INTO wiki_identity_deletion_operations(operation_id,issuer,subject,inventory_revision,request,state,fence,revision,reference_count) VALUES($1,$2,$3,$4,$5,'released',NULL,$6,0) ON CONFLICT(operation_id) DO UPDATE SET state='released',fence=NULL")
            .bind(request.operation_id).bind(&request.identity.issuer).bind(&request.identity.subject)
            .bind(request.inventory_revision).bind(serde_json::to_value(request).map_err(|_|OwnerError::Invalid)?)
            .bind(revision as i64).execute(&mut *tx).await.map_err(storage)?;
        if !window.is_current() {
            return Err(OwnerError::Expired);
        }
        tx.commit().await.map_err(storage)?;
        Ok(OwnerReleaseReceipt {
            request: request.clone(),
            owner: OWNER.into(),
            cancelled: true,
        })
    }
}
async fn lock(tx: &mut Transaction<'_, Postgres>) -> Result<(), OwnerError> {
    // Bound every owner transaction before it waits on a live writer or scans
    // the reference inventory. The HTTP boundary also bounds the whole call.
    sqlx::query(
        "SELECT set_config('lock_timeout','8s',true),set_config('statement_timeout','8s',true)",
    )
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    sqlx::query("SELECT pg_advisory_xact_lock(714402216003::bigint)")
        .execute(&mut **tx)
        .await
        .map_err(storage)?;
    let result =
        sqlx::query("UPDATE wiki_identity_fence_epoch SET revision=revision+1 WHERE singleton")
            .execute(&mut **tx)
            .await
            .map_err(storage)?;
    if result.rows_affected() != 1 {
        return Err(OwnerError::Storage);
    }
    Ok(())
}
async fn verify_operation(
    tx: &mut Transaction<'_, Postgres>,
    request: &DeletionRequest,
) -> Result<Option<sqlx::postgres::PgRow>, OwnerError> {
    let row=sqlx::query("SELECT request,state,fence,revision,reference_count FROM wiki_identity_deletion_operations WHERE operation_id=$1")
        .bind(request.operation_id).fetch_optional(&mut **tx).await.map_err(storage)?;
    if let Some(row) = &row {
        let value: Value = row.try_get("request").map_err(storage)?;
        if value != serde_json::to_value(request).map_err(|_| OwnerError::Invalid)? {
            return Err(OwnerError::Conflict);
        }
    }
    Ok(row)
}
async fn revision(tx: &mut Transaction<'_, Postgres>) -> Result<u64, OwnerError> {
    let value: i64 = sqlx::query_scalar("SELECT nextval('wiki_identity_reference_revision')")
        .fetch_one(&mut **tx)
        .await
        .map_err(storage)?;
    Ok(value as u64)
}
async fn count(
    tx: &mut Transaction<'_, Postgres>,
    request: &DeletionRequest,
) -> Result<u64, OwnerError> {
    let mut count = 0_u64;
    for source in SOURCES {
        let unknown:bool=sqlx::query_scalar(&format!("SELECT EXISTS(SELECT 1 FROM {source} t CROSS JOIN LATERAL wiki_identity_row_refs('{source}',to_jsonb(t)) r WHERE NOT r.resolved OR COALESCE(length(btrim(r.issuer)),0)=0)"))
            .fetch_one(&mut **tx).await.map_err(storage)?;
        if unknown {
            return Err(OwnerError::Unbound);
        }
        let sql = format!(
            "SELECT COUNT(*) FROM {source} t WHERE EXISTS(SELECT 1 FROM wiki_identity_row_refs('{source}',to_jsonb(t)) r WHERE r.issuer=$1 AND r.subject=$2)"
        );
        let value: i64 = sqlx::query_scalar(&sql)
            .bind(&request.identity.issuer)
            .bind(&request.identity.subject)
            .fetch_one(&mut **tx)
            .await
            .map_err(storage)?;
        count = count.checked_add(value as u64).ok_or(OwnerError::Storage)?;
    }
    Ok(count)
}

impl From<OwnerError> for shared::AppError {
    fn from(error: OwnerError) -> Self {
        match error {
            OwnerError::Invalid => Self::invalid_input("invalid_identity_deletion_request"),
            OwnerError::Conflict => Self::Conflict("identity_deletion_operation_conflict".into()),
            OwnerError::Expired => Self::Unauthorized,
            OwnerError::Unbound => Self::Unavailable("identity_binding_unknown".into()),
            OwnerError::Storage => {
                Self::Unavailable("identity_deletion_storage_unavailable".into())
            }
        }
    }
}
impl super::PostgresWikiBackend {
    pub(super) fn deletion_store(&self) -> IdentityDeletionStore {
        IdentityDeletionStore::new(self.pool.clone())
    }
}
