//! Tracker is the sole project/space association owner; Wiki only acknowledges a durable projection.
use super::PostgresWikiBackend;
use pdlc_shared::{project_spaces::*, space_lifecycle::SpaceOwnerAuthority};
use shared::AppError;
use sqlx::Row;
use uuid::Uuid;
fn current(authority: &SpaceOwnerAuthority) -> Result<(), AppError> {
    if authority.is_current() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
fn configured(key: &str) -> Result<Uuid, AppError> {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|id: &Uuid| !id.is_nil())
        .ok_or_else(|| AppError::Unavailable("project_projection_not_configured".into()))
}
impl PostgresWikiBackend {
    pub(super) async fn apply_project_spaces_projection(
        &self,
        command: &ProjectSpacesProjection,
        authority: &SpaceOwnerAuthority,
    ) -> Result<ProjectSpacesProjectionReadback, AppError> {
        current(authority)?;
        let tracker = configured("WIKI_NAMESPACE__TRACKER_INSTANCE_ID")?;
        let registry = configured("WIKI_NAMESPACE__REGISTRY_INSTANCE_ID")?;
        if !command.valid()
            || command.project.tracker_instance_id != tracker
            || command
                .spaces
                .iter()
                .any(|s| s.registry_instance_id != registry)
            || command
                .legacy_project
                .as_ref()
                .is_some_and(|n| n.registry_instance_id != registry)
        {
            return Err(AppError::invalid_input("invalid_project_space_projection"));
        }
        let mut tx = self.pool.begin().await.map_err(AppError::database)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!("project-space-operation:{}", command.operation_id))
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        sqlx::query("SELECT singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE")
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::database)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
            .bind(format!(
                "project-spaces:{tracker}:{}",
                command.project.project_id
            ))
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        sqlx::query(
            "SELECT tracker_owned FROM wiki_project_projection_state WHERE singleton FOR UPDATE",
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::database)?;
        current(authority)?;
        if let Some(row)=sqlx::query("SELECT command,readback FROM wiki_project_space_projection_operations WHERE operation_id=$1").bind(command.operation_id).fetch_optional(&mut *tx).await.map_err(AppError::database)?{
            let old:ProjectSpacesProjection=serde_json::from_value(row.try_get("command").map_err(AppError::database)?).map_err(|_|AppError::Unavailable("invalid_project_projection_receipt".into()))?;
            let ack:ProjectSpacesProjectionReadback=serde_json::from_value(row.try_get("readback").map_err(AppError::database)?).map_err(|_|AppError::Unavailable("invalid_project_projection_receipt".into()))?;
            if old!=*command || !ack.matches(command){return Err(AppError::conflict("project_projection_operation_conflict"));}
            current(authority)?;tx.commit().await.map_err(AppError::database)?;return Ok(ack);
        }
        if let Some(value)=sqlx::query_scalar::<_,serde_json::Value>("SELECT command FROM wiki_project_space_projection_sets WHERE tracker_instance_id=$1 AND project_id=$2 FOR UPDATE").bind(tracker).bind(command.project.project_id).fetch_optional(&mut *tx).await.map_err(AppError::database)?{
            let old:ProjectSpacesProjection=serde_json::from_value(value).map_err(|_|AppError::Unavailable("invalid_project_projection_head".into()))?;
            if !old.valid() || old.revision>=command.revision || old.legacy_project!=command.legacy_project{return Err(AppError::conflict("project_projection_revision_conflict"));}
        }
        let mut ordered = command.spaces.clone();
        ordered.sort_by_key(|s| (s.registry_instance_id, s.space_id));
        let mut local = Vec::new();
        for space in &ordered {
            let row=sqlx::query("SELECT local_space_id,admin_revision FROM wiki_space_catalog WHERE registry_instance_id=$1 AND space_id=$2 FOR SHARE").bind(space.registry_instance_id).bind(space.space_id).fetch_optional(&mut *tx).await.map_err(AppError::database)?.ok_or_else(||AppError::conflict("space_representation_not_ready"))?;
            if row
                .try_get::<Option<i64>, _>("admin_revision")
                .map_err(AppError::database)?
                .is_none()
            {
                return Err(AppError::conflict("space_representation_not_confirmed"));
            }
            local.push(
                row.try_get::<Uuid, _>("local_space_id")
                    .map_err(AppError::database)?,
            );
        }
        current(authority)?;
        sqlx::query("SET LOCAL pdlc.wiki.projection_owner='tracker_v2'")
            .execute(&mut *tx)
            .await
            .map_err(AppError::database)?;
        if let Some(legacy) = &command.legacy_project {
            // Migration must first import the union of confirmed old links into Tracker.
            // Never discard an unresolved legacy edge merely to make the projection fit.
            let missing:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_space_projects WHERE registry_instance_id=$1 AND namespace_id=$2 AND project_id IS NULL AND NOT(space_id=ANY($3)))").bind(legacy.registry_instance_id).bind(legacy.namespace_id).bind(&local).fetch_one(&mut *tx).await.map_err(AppError::database)?;
            if missing {
                return Err(AppError::conflict(
                    "legacy_project_links_require_union_migration",
                ));
            }
            sqlx::query("INSERT INTO wiki_project_identity_mappings(tracker_instance_id,project_id,registry_instance_id,namespace_id) VALUES($1,$2,$3,$4) ON CONFLICT(tracker_instance_id,project_id) DO NOTHING").bind(tracker).bind(command.project.project_id).bind(legacy.registry_instance_id).bind(legacy.namespace_id).execute(&mut *tx).await.map_err(AppError::database)?;
            let mapping:(Uuid,Uuid)=sqlx::query_as("SELECT registry_instance_id,namespace_id FROM wiki_project_identity_mappings WHERE tracker_instance_id=$1 AND project_id=$2").bind(tracker).bind(command.project.project_id).fetch_one(&mut *tx).await.map_err(AppError::database)?;
            if mapping != (legacy.registry_instance_id, legacy.namespace_id) {
                return Err(AppError::conflict("legacy_project_identity_conflict"));
            }
            sqlx::query("UPDATE wiki_space_projects SET tracker_instance_id=$1,project_id=$2 WHERE registry_instance_id=$3 AND namespace_id=$4 AND project_id IS NULL").bind(tracker).bind(command.project.project_id).bind(legacy.registry_instance_id).bind(legacy.namespace_id).execute(&mut *tx).await.map_err(AppError::database)?;
        }
        sqlx::query("DELETE FROM wiki_space_projects WHERE tracker_instance_id=$1 AND project_id=$2 AND NOT(space_id=ANY($3))").bind(tracker).bind(command.project.project_id).bind(&local).execute(&mut *tx).await.map_err(AppError::database)?;
        for id in &local {
            sqlx::query("INSERT INTO wiki_space_projects(space_id,tracker_instance_id,project_id) SELECT $1,$2,$3 WHERE NOT EXISTS(SELECT 1 FROM wiki_space_projects WHERE space_id=$1 AND tracker_instance_id=$2 AND project_id=$3)").bind(id).bind(tracker).bind(command.project.project_id).execute(&mut *tx).await.map_err(AppError::database)?;
        }
        let ack = ProjectSpacesProjectionReadback::confirmed(command)
            .ok_or_else(|| AppError::invalid_input("invalid_projection"))?;
        let wire = serde_json::to_value(command)
            .map_err(|e| AppError::Internal(format!("project_projection_serialization:{e}")))?;
        let readback = serde_json::to_value(&ack)
            .map_err(|e| AppError::Internal(format!("project_projection_serialization:{e}")))?;
        sqlx::query("INSERT INTO wiki_project_space_projection_sets(tracker_instance_id,project_id,revision,operation_id,command) VALUES($1,$2,$3,$4,$5) ON CONFLICT(tracker_instance_id,project_id) DO UPDATE SET revision=EXCLUDED.revision,operation_id=EXCLUDED.operation_id,command=EXCLUDED.command").bind(tracker).bind(command.project.project_id).bind(command.revision).bind(command.operation_id).bind(&wire).execute(&mut *tx).await.map_err(AppError::database)?;
        sqlx::query("INSERT INTO wiki_project_space_projection_operations(operation_id,tracker_instance_id,project_id,revision,command,readback,owner_issuer,owner_subject) VALUES($1,$2,$3,$4,$5,$6,$7,$8)").bind(command.operation_id).bind(tracker).bind(command.project.project_id).bind(command.revision).bind(wire).bind(readback).bind(&authority.identity().issuer).bind(&authority.identity().subject).execute(&mut *tx).await.map_err(AppError::database)?;
        self.insert_audit(
            &mut tx,
            None,
            "project.spaces_projection.apply",
            "project",
            command.project.project_id,
            Some(&command.operation_id.to_string()),
        )
        .await?;
        sqlx::query("UPDATE wiki_project_projection_state SET tracker_owned=true WHERE singleton AND NOT tracker_owned").execute(&mut *tx).await.map_err(AppError::database)?;
        current(authority)?;
        tx.commit().await.map_err(AppError::database)?;
        Ok(ack)
    }
}
