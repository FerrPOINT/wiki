//! Admin is the sole metadata owner. Wiki persists the original local UUID and exact v2 receipt.
use super::PostgresWikiBackend;
use shared::{AppError, space_owner::*};
use sqlx::Row;
use uuid::Uuid;
fn configuration() -> Result<(Uuid, Uuid), AppError> {
    let id = |key| {
        std::env::var(key)
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .filter(|id| !id.is_nil())
            .ok_or_else(|| AppError::Unavailable("space_owner_not_configured".into()))
    };
    Ok((
        id("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")?,
        id("WIKI_NAMESPACE__INSTANCE_ID")?,
    ))
}
impl PostgresWikiBackend {
    pub(super) async fn apply_space_owner_v2(
        &self,
        command: &SpaceOwnerCommandV2,
        authority: &WikiSpaceOwnerAuthority,
    ) -> Result<SpaceOwnerReadbackV2, AppError> {
        authority.require_current()?;
        let (registry, instance) = configuration()?;
        if !command.valid_for(registry, instance) {
            return Err(AppError::invalid_input("invalid_space_owner_v2_command"));
        }
        // Directory refresh is an identity projection, never a role assignment. Do it
        // outside the owner transaction and only when a new local representation needs it.
        let mapped:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_space_catalog WHERE registry_instance_id=$1 AND space_id=$2)").bind(registry).bind(command.space.space_id).fetch_one(&self.pool).await.map_err(AppError::database)?;
        if !mapped {
            let present: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE central_sub=$1 AND is_active)",
            )
            .bind(&command.metadata.responsible_subject)
            .fetch_one(&self.pool)
            .await
            .map_err(AppError::database)?;
            if !present {
                self.sync_central_users(authority.token()).await?;
                authority.require_current()?;
            }
        }
        let mut tx = self.pool.begin().await.map_err(AppError::database)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("space-owner-operation:{}", command.operation_id))
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        sqlx::query("SELECT singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE")
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::database)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!(
                "shared-space:{registry}:{}",
                command.space.space_id
            ))
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        authority.require_current()?;
        if let Some(row) = sqlx::query(
            "SELECT command,readback FROM wiki_space_owner_operations WHERE operation_id=$1",
        )
        .bind(command.operation_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::database)?
        {
            let old: SpaceOwnerCommandV2 =
                serde_json::from_value(row.try_get("command").map_err(AppError::database)?)
                    .map_err(|_| AppError::Unavailable("invalid_space_owner_receipt".into()))?;
            let ack: SpaceOwnerReadbackV2 =
                serde_json::from_value(row.try_get("readback").map_err(AppError::database)?)
                    .map_err(|_| AppError::Unavailable("invalid_space_owner_receipt".into()))?;
            if old != *command
                || !ack.matches(command)
                || ack
                    .local_resource
                    .as_ref()
                    .is_none_or(|r| r.kind != shared::resource_context::ResourceKind::WikiSpace)
            {
                return Err(AppError::conflict("space_operation_payload_conflict"));
            }
            authority.require_current()?;
            tx.commit().await.map_err(AppError::database)?;
            return Ok(ack);
        }
        let existing=sqlx::query("SELECT local_space_id,admin_revision FROM wiki_space_catalog WHERE registry_instance_id=$1 AND space_id=$2 FOR UPDATE").bind(registry).bind(command.space.space_id).fetch_optional(&mut *tx).await.map_err(AppError::database)?;
        authority.require_current()?;
        sqlx::query("SET LOCAL pdlc.wiki.lifecycle_owner='admin_v2'")
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        let local;
        if let Some(row) = existing {
            local = row.try_get("local_space_id").map_err(AppError::database)?;
            let previous: Option<i64> =
                row.try_get("admin_revision").map_err(AppError::database)?;
            if previous.is_some_and(|revision| revision > command.revision) {
                return Err(AppError::conflict("stale_space_revision"));
            }
            let prior:Option<serde_json::Value>=sqlx::query_scalar("SELECT command FROM wiki_space_owner_operations WHERE registry_instance_id=$1 AND space_id=$2 ORDER BY revision DESC LIMIT 1").bind(registry).bind(command.space.space_id).fetch_optional(&mut *tx).await.map_err(AppError::database)?;
            if let Some(prior) = prior {
                let prior: SpaceOwnerCommandV2 = serde_json::from_value(prior)
                    .map_err(|_| AppError::Unavailable("invalid_space_owner_receipt".into()))?;
                if !command.follows(&prior) {
                    return Err(AppError::conflict("space_owner_revision_conflict"));
                }
            }
        } else {
            if command.desired_state != DesiredSpaceState::Active {
                return Err(AppError::conflict("space_representation_not_created"));
            }
            let owner: Uuid =
                sqlx::query_scalar("SELECT id FROM users WHERE central_sub=$1 AND is_active")
                    .bind(&command.metadata.responsible_subject)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(AppError::database)?
                    .ok_or_else(|| {
                        AppError::conflict("active_responsible_wiki_profile_required")
                    })?;
            local = Uuid::now_v7();
            // Full UUID, lossless and independent of human names/renames. Legacy keys are retained.
            let key = command
                .space
                .space_id
                .simple()
                .to_string()
                .to_ascii_uppercase();
            sqlx::query(
                "INSERT INTO spaces(id,key,name,description,owner_id) VALUES($1,$2,$3,$4,$5)",
            )
            .bind(local)
            .bind(key)
            .bind(&command.metadata.name)
            .bind(&command.metadata.description)
            .bind(owner)
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
            sqlx::query("INSERT INTO space_members(space_id,user_id,role) VALUES($1,$2,'admin')")
                .bind(local)
                .bind(owner)
                .execute(&mut *tx)
                .await
                .map_err(AppError::database)?;
            sqlx::query("INSERT INTO wiki_space_catalog(registry_instance_id,space_id,local_space_id,state,admin_revision,operation_id) VALUES($1,$2,$3,'provisioning',NULL,$4)").bind(registry).bind(command.space.space_id).bind(local).bind(command.operation_id).execute(&mut *tx).await.map_err(AppError::database)?;
        }
        let archived = command.desired_state == DesiredSpaceState::Archived;
        let state = if archived { "archived" } else { "active" };
        sqlx::query("UPDATE wiki_space_catalog SET admin_revision=$3,state=$4,operation_id=$5 WHERE registry_instance_id=$1 AND space_id=$2").bind(registry).bind(command.space.space_id).bind(command.revision).bind(state).bind(command.operation_id).execute(&mut *tx).await.map_err(AppError::database)?;
        // Responsible metadata never silently transfers existing product roles/ownership.
        sqlx::query("UPDATE spaces SET name=$2,description=$3,archived_at=CASE WHEN $4 THEN coalesce(archived_at,now()) ELSE NULL END,updated_at=now() WHERE id=$1").bind(local).bind(&command.metadata.name).bind(&command.metadata.description).bind(archived).execute(&mut *tx).await.map_err(AppError::database)?;
        let ack = SpaceOwnerReadbackV2 {
            schema_version: 2,
            space: command.space.clone(),
            participant_instance_id: instance,
            operation_id: command.operation_id,
            revision: command.revision,
            state: command.desired_state,
            drained: true,
            local_resource: Some(shared::resource_context::ResourceRef {
                kind: shared::resource_context::ResourceKind::WikiSpace,
                instance_id: instance,
                resource_id: local,
            }),
        };
        // FOR UPDATE waited for all in-flight content SQL holding the catalog's share
        // lock. Wiki owns no model-job runner; other participants must confirm their own drain.
        sqlx::query("INSERT INTO wiki_space_owner_operations(operation_id,registry_instance_id,space_id,revision,command,readback,owner_issuer,owner_subject) VALUES($1,$2,$3,$4,$5,$6,$7,$8)").bind(command.operation_id).bind(registry).bind(command.space.space_id).bind(command.revision).bind(serde_json::to_value(command).map_err(|_|AppError::Internal("invalid_space_owner_command".into()))?).bind(serde_json::to_value(&ack).map_err(|_|AppError::Internal("invalid_space_owner_readback".into()))?).bind(&authority.identity().issuer).bind(&authority.identity().subject).execute(&mut *tx).await.map_err(AppError::database)?;
        let request = command.operation_id.to_string();
        self.insert_audit(
            &mut tx,
            None,
            "space.owner_v2.apply",
            "space",
            local,
            Some(&request),
        )
        .await?;
        authority.require_current()?;
        tx.commit().await.map_err(AppError::database)?;
        Ok(ack)
    }
}
