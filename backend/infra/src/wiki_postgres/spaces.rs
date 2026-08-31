use super::{
    PostgresWikiBackend,
    mapping::{
        build_db_tree, parse_uuid, space_member_response_from_row, space_response_from_row, to_iso,
    },
    queries::{SPACE_LIST_SQL, SPACE_ONE_SQL},
};
use app::wiki::{
    WikiSpaceAccess as SpaceAccess, normalize_required, normalize_space_key, normalize_space_role,
};
use shared::wiki_contract::*;
use sqlx::Row;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn list_spaces(
        &self,
        claims: &WikiClaims,
    ) -> Result<SpaceListResponse, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let rows = sqlx::query(SPACE_LIST_SQL)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(SpaceListResponse {
            spaces: rows.iter().map(space_response_from_row).collect(),
        })
    }

    pub(super) async fn create_space(
        &self,
        claims: &WikiClaims,
        body: CreateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let key = normalize_space_key(&body.key)?;
        let name = normalize_required(&body.name, "space name")?;
        let description = body.description.unwrap_or_default();
        let space_id = Uuid::now_v7();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let row = sqlx::query(
            r#"
            INSERT INTO spaces (id, key, name, description, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, now(), now())
            RETURNING id
            "#,
        )
        .bind(space_id)
        .bind(&key)
        .bind(name)
        .bind(description)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        let space_id: Uuid = row.get("id");

        sqlx::query(
            r#"
            INSERT INTO space_members (space_id, user_id, role, joined_at)
            VALUES ($1, $2, 'admin', now())
            "#,
        )
        .bind(space_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        self.insert_audit(&mut tx, Some(actor_id), "space.create", "space", space_id)
            .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.get_space_by_key(&key).await
    }

    pub(super) async fn get_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        self.ensure_space_access(claims, space_key, SpaceAccess::View)
            .await?;
        self.get_space_by_key(space_key).await
    }

    async fn get_space_by_key(&self, space_key: &str) -> Result<SpaceResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let row = sqlx::query(SPACE_ONE_SQL)
            .bind(&key)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
        Ok(space_response_from_row(&row))
    }

    pub(super) async fn update_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: UpdateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let row = sqlx::query(
            r#"
            UPDATE spaces
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = now()
            WHERE key = $1
            RETURNING id
            "#,
        )
        .bind(&key)
        .bind(
            body.name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(body.description.as_deref().map(str::trim))
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
        let updated_space_id: Uuid = row.get("id");
        debug_assert_eq!(space_id, updated_space_id);
        self.audit(Some(actor_id), "space.update", "space", space_id)
            .await?;
        self.get_space_by_key(&key).await
    }

    pub(super) async fn archive_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let key = normalize_space_key(space_key)?;
        let row = sqlx::query(
            "UPDATE spaces SET archived_at = now(), updated_at = now() WHERE key = $1 RETURNING id",
        )
        .bind(&key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("space", space_key))?;
        let archived_space_id: Uuid = row.get("id");
        debug_assert_eq!(space_id, archived_space_id);
        self.audit(Some(actor_id), "space.archive", "space", space_id)
            .await?;
        self.get_space_by_key(&key).await
    }

    pub(super) async fn list_space_members(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT sm.user_id, u.email, u.display_name, sm.role, sm.joined_at
            FROM space_members sm
            JOIN users u ON u.id = sm.user_id
            WHERE sm.space_id = $1
            ORDER BY lower(u.email)
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(SpaceMemberListResponse {
            members: rows.iter().map(space_member_response_from_row).collect(),
        })
    }

    pub(super) async fn upsert_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
        body: UpsertSpaceMemberRequest,
    ) -> Result<SpaceMemberResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let user_id = parse_uuid(user_id, "user")?;
        let role = normalize_space_role(&body.role)?;

        let row = sqlx::query(
            r#"
            INSERT INTO space_members (space_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (space_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            RETURNING user_id, role, joined_at
            "#,
        )
        .bind(space_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;

        self.audit(Some(actor_id), "space.member_upsert", "space", space_id)
            .await?;
        let user = self.user_response(user_id).await?;
        Ok(SpaceMemberResponse {
            user_id: row.get::<Uuid, _>("user_id").to_string(),
            email: user.email,
            display_name: user.display_name,
            role: row.get("role"),
            joined_at: to_iso(row.get("joined_at")),
        })
    }

    pub(super) async fn delete_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
    ) -> Result<(), shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let user_id = parse_uuid(user_id, "user")?;
        sqlx::query("DELETE FROM space_members WHERE space_id = $1 AND user_id = $2")
            .bind(space_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "space.member_delete", "space", space_id)
            .await?;
        Ok(())
    }

    pub(super) async fn get_space_tree(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceTreeResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT id, parent_id, slug, title, document_type, status, position, updated_at
            FROM documents
            WHERE space_id = $1 AND archived_at IS NULL
            ORDER BY parent_id NULLS FIRST, position, title
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents = build_db_tree(&rows, None);
        Ok(SpaceTreeResponse {
            space_key: key,
            documents,
        })
    }
}
