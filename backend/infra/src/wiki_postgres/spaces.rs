use super::{
    PostgresWikiBackend,
    mapping::{build_db_tree, parse_uuid, space_member_response_from_row, space_response_from_row},
    queries::{SPACE_LIST_SQL, SPACE_ONE_SQL},
};
use app::wiki::{
    WikiArchiveSpaceCommand, WikiCreateSpaceCommand, WikiDeleteSpaceMemberCommand,
    WikiSpaceAccess as SpaceAccess, WikiSpaceRepository, WikiSpaceRepositoryFuture,
    WikiSpaceUseCase, WikiUpdateSpaceCommand, WikiUpsertSpaceMemberCommand,
};
use shared::wiki_contract::*;
use sqlx::Row;
use uuid::Uuid;

struct PostgresWikiSpaceRepository<'a> {
    backend: &'a PostgresWikiBackend,
    request_id: Option<&'a str>,
}

async fn fetch_space_by_key(
    backend: &PostgresWikiBackend,
    key: &str,
) -> Result<SpaceResponse, shared::AppError> {
    let row = sqlx::query(SPACE_ONE_SQL)
        .bind(key)
        .fetch_optional(&backend.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("space", key))?;
    Ok(space_response_from_row(&row))
}

impl WikiSpaceRepository for PostgresWikiSpaceRepository<'_> {
    fn list_spaces<'a>(
        &'a self,
        user_id: Uuid,
    ) -> WikiSpaceRepositoryFuture<'a, Vec<SpaceResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(SPACE_LIST_SQL)
                .bind(user_id)
                .fetch_all(&self.backend.pool)
                .await
                .map_err(shared::AppError::database)?;
            Ok(rows.iter().map(space_response_from_row).collect())
        })
    }

    fn create_space<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateSpaceCommand,
    ) -> WikiSpaceRepositoryFuture<'a, SpaceResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            sqlx::query(
                r#"
                INSERT INTO spaces (id, key, name, description, owner_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, now(), now())
                "#,
            )
            .bind(command.space_id)
            .bind(&command.key)
            .bind(&command.name)
            .bind(&command.description)
            .bind(command.owner_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;

            sqlx::query(
                r#"
                INSERT INTO space_members (space_id, user_id, role, joined_at)
                VALUES ($1, $2, 'admin', now())
                "#,
            )
            .bind(command.space_id)
            .bind(command.owner_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "space.create",
                    "space",
                    command.space_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            fetch_space_by_key(self.backend, &command.key).await
        })
    }

    fn get_space<'a>(&'a self, key: &'a str) -> WikiSpaceRepositoryFuture<'a, SpaceResponse> {
        Box::pin(async move { fetch_space_by_key(self.backend, key).await })
    }

    fn update_space<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpdateSpaceCommand,
    ) -> WikiSpaceRepositoryFuture<'a, SpaceResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            let row = sqlx::query(
                r#"
                UPDATE spaces
                SET name = COALESCE($3, name),
                    description = COALESCE($4, description),
                    updated_at = now()
                WHERE id = $1 AND key = $2
                RETURNING id
                "#,
            )
            .bind(command.space_id)
            .bind(&command.key)
            .bind(command.name.as_deref())
            .bind(command.description.as_deref())
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("space", &command.key))?;
            let updated_space_id: Uuid = row.get("id");
            debug_assert_eq!(updated_space_id, command.space_id);

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "space.update",
                    "space",
                    command.space_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            fetch_space_by_key(self.backend, &command.key).await
        })
    }

    fn archive_space<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiArchiveSpaceCommand,
    ) -> WikiSpaceRepositoryFuture<'a, SpaceResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            let row = sqlx::query(
                r#"
                UPDATE spaces
                SET archived_at = now(), updated_at = now()
                WHERE id = $1 AND key = $2
                RETURNING id
                "#,
            )
            .bind(command.space_id)
            .bind(&command.key)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("space", &command.key))?;
            let archived_space_id: Uuid = row.get("id");
            debug_assert_eq!(archived_space_id, command.space_id);

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "space.archive",
                    "space",
                    command.space_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            fetch_space_by_key(self.backend, &command.key).await
        })
    }

    fn list_members<'a>(
        &'a self,
        space_id: Uuid,
    ) -> WikiSpaceRepositoryFuture<'a, Vec<SpaceMemberResponse>> {
        Box::pin(async move {
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
            .fetch_all(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(rows.iter().map(space_member_response_from_row).collect())
        })
    }

    fn upsert_member<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpsertSpaceMemberCommand,
    ) -> WikiSpaceRepositoryFuture<'a, SpaceMemberResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            let row = sqlx::query(
                r#"
                WITH target_user AS (
                    SELECT id, email, display_name
                    FROM users
                    WHERE id = $2
                ),
                upserted AS (
                    INSERT INTO space_members (space_id, user_id, role, joined_at)
                    SELECT $1, id, $3, now()
                    FROM target_user
                    ON CONFLICT (space_id, user_id)
                    DO UPDATE SET role = EXCLUDED.role
                    RETURNING user_id, role, joined_at
                )
                SELECT
                    u.id AS user_id,
                    u.email,
                    u.display_name,
                    upserted.role,
                    upserted.joined_at
                FROM upserted
                JOIN target_user u ON u.id = upserted.user_id
                "#,
            )
            .bind(command.space_id)
            .bind(command.user_id)
            .bind(&command.role)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("user", command.user_id))?;

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "space.member_upsert",
                    "space",
                    command.space_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;

            Ok(space_member_response_from_row(&row))
        })
    }

    fn delete_member<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiDeleteSpaceMemberCommand,
    ) -> WikiSpaceRepositoryFuture<'a, ()> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;

            let result =
                sqlx::query("DELETE FROM space_members WHERE space_id = $1 AND user_id = $2")
                    .bind(command.space_id)
                    .bind(command.user_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(shared::AppError::database)?;
            if result.rows_affected() == 0 {
                return Err(shared::AppError::not_found("space member", command.user_id));
            }

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "space.member_delete",
                    "space",
                    command.space_id,
                    self.request_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            Ok(())
        })
    }

    fn get_tree<'a>(
        &'a self,
        space_id: Uuid,
    ) -> WikiSpaceRepositoryFuture<'a, Vec<SpaceTreeNodeResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                SELECT id, parent_id, slug, title, document_type, status, position, updated_at
                FROM documents
                WHERE space_id = $1 AND archived_at IS NULL
                ORDER BY parent_id NULLS FIRST, position, title
                "#,
            )
            .bind(space_id)
            .fetch_all(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(build_db_tree(&rows, None))
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn list_spaces(
        &self,
        claims: &WikiClaims,
    ) -> Result<SpaceListResponse, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: None,
        };
        WikiSpaceUseCase::new(&repository).list(user_id).await
    }

    pub(super) async fn create_space(
        &self,
        claims: &WikiClaims,
        body: CreateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiSpaceUseCase::new(&repository)
            .create(actor_id, body)
            .await
    }

    pub(super) async fn get_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        self.ensure_space_access(claims, space_key, SpaceAccess::View)
            .await?;
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: None,
        };
        WikiSpaceUseCase::new(&repository).get(space_key).await
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
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiSpaceUseCase::new(&repository)
            .update(actor_id, space_id, space_key, body)
            .await
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
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiSpaceUseCase::new(&repository)
            .archive(actor_id, space_id, space_key)
            .await
    }

    pub(super) async fn list_space_members(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::Admin)
            .await?;
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: None,
        };
        WikiSpaceUseCase::new(&repository)
            .list_members(space_id)
            .await
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
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiSpaceUseCase::new(&repository)
            .upsert_member(actor_id, space_id, user_id, body)
            .await
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
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: claims.request_id.as_deref(),
        };
        WikiSpaceUseCase::new(&repository)
            .delete_member(actor_id, space_id, user_id)
            .await
    }

    pub(super) async fn get_space_tree(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceTreeResponse, shared::AppError> {
        let space_id = self
            .ensure_space_access(claims, space_key, SpaceAccess::View)
            .await?;
        let repository = PostgresWikiSpaceRepository {
            backend: self,
            request_id: None,
        };
        WikiSpaceUseCase::new(&repository)
            .tree(space_id, space_key)
            .await
    }
}
