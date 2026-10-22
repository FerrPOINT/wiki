//! Request proof and current role locks belong to the same SQLx write transaction.
//! Owner lifecycle/projection and personal/global maintenance use their own ports.
use super::database_error;
use super::{PostgresWikiBackend, SpaceAccess, space_role_allows};
use shared::{AppError, wiki_space_access::WikiSpaceAdmission};
use sqlx::{Postgres, Row, Transaction};
use std::sync::Arc;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn begin_content_write(
        &self,
        actor: Uuid,
        admission: Option<&Arc<WikiSpaceAdmission>>,
        spaces: &[Uuid],
        required: SpaceAccess,
    ) -> Result<Transaction<'_, Postgres>, AppError> {
        if required == SpaceAccess::View {
            return Err(AppError::Forbidden);
        }
        if let Some(scope) = admission {
            if !scope.matches_user(actor) || !scope.allows_method("PUT") {
                return Err(AppError::Forbidden);
            }
        }
        let mut tx = self.pool.begin().await.map_err(database_error)?;
        let active: bool = sqlx::query_scalar(
            "SELECT active FROM wiki_space_admission_state WHERE singleton FOR SHARE",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(database_error)?;
        if active && admission.is_none() {
            return Err(AppError::Forbidden);
        }
        let mut spaces = spaces.to_vec();
        spaces.sort_unstable();
        spaces.dedup();
        if spaces.iter().any(Uuid::is_nil) {
            return Err(AppError::Forbidden);
        }
        let mut archived = false;
        // Match lifecycle lock order. Lock all target resources before profile/role:
        // an archive/row wait must not freeze an already obsolete role decision.
        for space in &spaces {
            let catalog:Option<(String,Option<i64>)>=sqlx::query_as("SELECT state,admin_revision FROM wiki_space_catalog WHERE local_space_id=$1 FOR SHARE")
                .bind(space).fetch_optional(&mut *tx).await.map_err(database_error)?;
            if let Some(scope) = admission {
                if !scope.allows(*space, true)
                    || (required == SpaceAccess::Admin && !scope.allows_admin(*space))
                {
                    return Err(AppError::Forbidden);
                }
                let revision = scope.catalog_revision(*space)?;
                if catalog.as_ref() != Some(&("active".into(), Some(revision))) {
                    return Err(AppError::conflict("space_confirmation_changed"));
                }
            }
            let target_lock = if required == SpaceAccess::Admin {
                "SELECT archived_at IS NULL FROM spaces WHERE id=$1 FOR UPDATE"
            } else {
                "SELECT archived_at IS NULL FROM spaces WHERE id=$1 FOR SHARE"
            };
            let available: Option<bool> = sqlx::query_scalar(target_lock)
                .bind(space)
                .fetch_optional(&mut *tx)
                .await
                .map_err(database_error)?;
            match available {
                Some(true) => {}
                Some(false) => archived = true,
                None => return Err(AppError::Forbidden),
            }
        }
        let user = sqlx::query(
            "SELECT global_role,central_sub FROM users WHERE id=$1 AND is_active FOR SHARE",
        )
        .bind(actor)
        .fetch_optional(&mut *tx)
        .await
        .map_err(database_error)?
        .ok_or(AppError::Unauthorized)?;
        if let Some(scope) = admission {
            let identity = scope.subject_identity()?;
            if user.get::<Option<String>, _>("central_sub").as_deref()
                != Some(identity.subject.as_str())
            {
                return Err(AppError::Forbidden);
            }
        }
        let global: String = user.get("global_role");
        let global_admin =
            global == "admin" && admission.is_none_or(|scope| scope.confirmed_global_admin());
        if spaces.is_empty() && required == SpaceAccess::Admin && !global_admin {
            return Err(AppError::Forbidden);
        }
        for space in &spaces {
            let role: Option<String> = sqlx::query_scalar(
                "SELECT role FROM space_members WHERE space_id=$1 AND user_id=$2 FOR SHARE",
            )
            .bind(space)
            .bind(actor)
            .fetch_optional(&mut *tx)
            .await
            .map_err(database_error)?;
            if !global_admin && !space_role_allows(role.as_deref(), required) {
                return Err(AppError::Forbidden);
            }
        }
        if let Some(scope) = admission {
            scope.require_current()?;
        }
        // Native writers retain the public validation error. Unauthorized callers
        // are rejected above; admitted central writes remain fenced by the scope.
        if archived {
            return Err(AppError::invalid_input(
                "archived space does not accept write commands",
            ));
        }
        Ok(tx)
    }
}

pub(super) async fn commit(
    tx: Transaction<'_, Postgres>,
    admission: Option<&Arc<WikiSpaceAdmission>>,
) -> Result<(), AppError> {
    // All row waits and content/audit writes have completed. The original
    // monotonic proof is not renewed by this final transaction boundary.
    if let Some(scope) = admission {
        scope.require_current()?;
    }
    tx.commit().await.map_err(database_error)
}
