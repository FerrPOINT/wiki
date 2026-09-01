use super::{
    PostgresWikiBackend,
    mapping::{parse_uuid, revision_response_from_row, to_iso},
};
use app::wiki::{
    WikiSpaceAccess as SpaceAccess, checksum, markdown_to_text, normalize_document_type,
    normalize_phase_key, normalize_required, normalize_slug, normalize_space_key,
    normalize_task_key, slugify,
};
use shared::wiki_contract::*;
use sqlx::Row;
use std::collections::BTreeSet;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn create_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: CreateDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        self.ensure_space_accepts_writes(space_id).await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let title = normalize_required(&body.title, "document title")?;
        let document_type = normalize_document_type(&body.document_type, true)?;
        let document_id = Uuid::now_v7();
        let mut slug = body.slug.unwrap_or_else(|| slugify(&title));
        slug = slugify(&slug);
        if slug.is_empty() {
            slug = format!("document-{}", document_id.simple());
            slug.truncate(17);
        }
        let slug = normalize_slug(&slug)?;

        let parent_id = match body.parent_id {
            Some(parent_id) => {
                let resolved = self.resolve_document_id(&parent_id).await?;
                let parent_space_id = self.document_space_id(resolved).await?;
                if parent_space_id != space_id {
                    return Err(shared::AppError::invalid_input(
                        "parent document belongs to another space",
                    ));
                }
                Some(resolved)
            }
            None => None,
        };

        let task_key = match body.task_key {
            Some(value) => Some(normalize_task_key(&value)?),
            None => None,
        };
        let phase_key = match body.phase_key {
            Some(value) => Some(normalize_phase_key(&value)?),
            None => None,
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let position: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM documents
            WHERE space_id = $1
              AND (($2::uuid IS NULL AND parent_id IS NULL) OR parent_id = $2)
            "#,
        )
        .bind(space_id)
        .bind(parent_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        sqlx::query(
            r#"
            INSERT INTO documents (
                id, space_id, parent_id, slug, title, document_type, status,
                owner_id, position, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'draft', $7, $8, now(), now())
            "#,
        )
        .bind(document_id)
        .bind(space_id)
        .bind(parent_id)
        .bind(&slug)
        .bind(title)
        .bind(document_type)
        .bind(actor_id)
        .bind(position as i32)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        sqlx::query(
            r#"
            INSERT INTO document_drafts (document_id, author_id, content_markdown, updated_at)
            VALUES ($1, $2, $3, now())
            "#,
        )
        .bind(document_id)
        .bind(actor_id)
        .bind(body.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        if let Some(task_key) = &task_key {
            let task_id = self
                .upsert_task_dossier_tx(&mut tx, space_id, task_key)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO document_task_links (space_id, document_id, task_dossier_id, created_by, created_at)
                VALUES ($1, $2, $3, $4, now())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(space_id)
            .bind(document_id)
            .bind(task_id)
            .bind(actor_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        }
        if let Some(phase_key) = &phase_key {
            let phase_id = self
                .upsert_phase_dossier_tx(&mut tx, space_id, phase_key)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO document_phase_links (space_id, document_id, phase_dossier_id, created_by, created_at)
                VALUES ($1, $2, $3, $4, now())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(space_id)
            .bind(document_id)
            .bind(phase_id)
            .bind(actor_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        }

        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "document.create",
            "document",
            document_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.document_response(document_id).await
    }

    pub(super) async fn get_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
        self.document_response(document_id).await
    }

    pub(super) async fn update_document_draft(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: UpdateDocumentDraftRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let title = body
            .title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE id = $1")
            .bind(document_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
        if exists.is_none() {
            return Err(shared::AppError::not_found("document", document_id));
        }
        sqlx::query(
            r#"
            UPDATE documents
            SET title = COALESCE($2, title),
                status = 'draft',
                archived_at = NULL,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(document_id)
        .bind(title)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        sqlx::query(
            r#"
            INSERT INTO document_drafts (document_id, author_id, content_markdown, updated_at)
            VALUES ($1, $2, $3, now())
            ON CONFLICT (document_id)
            DO UPDATE SET author_id = EXCLUDED.author_id,
                          content_markdown = EXCLUDED.content_markdown,
                          updated_at = now()
            "#,
        )
        .bind(document_id)
        .bind(actor_id)
        .bind(body.content_markdown)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "document.draft_update",
            "document",
            document_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.document_response(document_id).await
    }

    pub(super) async fn publish_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: PublishDocumentRequest,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let row = sqlx::query(
            r#"
            SELECT d.title, COALESCE(dd.content_markdown, '') AS content_markdown
            FROM documents d
            LEFT JOIN document_drafts dd ON dd.document_id = d.id
            WHERE d.id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;
        let title: String = row.get("title");
        let content_markdown: String = row.get("content_markdown");
        if content_markdown.trim().is_empty() {
            return Err(shared::AppError::invalid_input(
                "published content is required",
            ));
        }
        let version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM document_revisions WHERE document_id = $1",
        )
        .bind(document_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        let revision_id = Uuid::now_v7();
        let content_text = markdown_to_text(&content_markdown);
        let content_checksum = checksum(content_markdown.as_bytes());

        let revision_row = sqlx::query(
            r#"
            INSERT INTO document_revisions (
                id, document_id, version, title, content_markdown, content_html,
                content_text, content_checksum, summary, author_id, published_at
            )
            VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8, $9, now())
            RETURNING id, document_id, version, title, content_markdown, summary, author_id, published_at
            "#,
        )
        .bind(revision_id)
        .bind(document_id)
        .bind(version)
        .bind(title)
        .bind(&content_markdown)
        .bind(content_text)
        .bind(content_checksum)
        .bind(body.summary)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;

        sqlx::query(
            r#"
            UPDATE documents
            SET current_revision_id = $2, status = 'published', archived_at = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(document_id)
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        sqlx::query(
            "UPDATE document_drafts SET base_revision_id = $2, updated_at = now() WHERE document_id = $1",
        )
        .bind(document_id)
        .bind(revision_id)
        .execute(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "document.publish",
            "document",
            document_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        Ok(revision_response_from_row(&revision_row))
    }

    pub(super) async fn archive_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let row = sqlx::query(
            r#"
            UPDATE documents
            SET status = 'archived', archived_at = now(), updated_at = now()
            WHERE id = $1
            RETURNING id
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;
        let document_id: Uuid = row.get("id");
        self.audit(Some(actor_id), "document.archive", "document", document_id)
            .await?;
        self.document_response(document_id).await
    }

    pub(super) async fn move_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: MoveDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        let document_space_id = self
            .ensure_document_access(claims, document_id, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let parent_id = match body.parent_id {
            Some(parent_id) => {
                let parent_id = self.resolve_document_id(&parent_id).await?;
                if parent_id == document_id {
                    return Err(shared::AppError::invalid_input(
                        "document cannot be moved under itself",
                    ));
                }
                let parent_space_id = self.document_space_id(parent_id).await?;
                if parent_space_id != document_space_id {
                    return Err(shared::AppError::invalid_input(
                        "parent document belongs to another space",
                    ));
                }
                if self
                    .document_parent_chain_contains(parent_id, document_id)
                    .await?
                {
                    return Err(shared::AppError::invalid_input(
                        "document cannot be moved under its descendant",
                    ));
                }
                Some(parent_id)
            }
            None => None,
        };
        sqlx::query("UPDATE documents SET parent_id = $2, updated_at = now() WHERE id = $1")
            .bind(document_id)
            .bind(parent_id)
            .execute(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "document.move", "document", document_id)
            .await?;
        self.document_response(document_id).await
    }

    pub(super) async fn list_document_revisions(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentRevisionListResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT id, document_id, version, title, content_markdown, summary, author_id, published_at
            FROM document_revisions
            WHERE document_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(DocumentRevisionListResponse {
            revisions: rows.iter().map(revision_response_from_row).collect(),
        })
    }

    pub(super) async fn get_document_revision(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        revision_id: &str,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
        let revision_id = parse_uuid(revision_id, "revision")?;
        let row = sqlx::query(
            r#"
            SELECT id, document_id, version, title, content_markdown, summary, author_id, published_at
            FROM document_revisions
            WHERE document_id = $1 AND id = $2
            "#,
        )
        .bind(document_id)
        .bind(revision_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("revision", revision_id))?;
        Ok(revision_response_from_row(&row))
    }

    pub(super) async fn document_space_id(
        &self,
        document_id: Uuid,
    ) -> Result<Uuid, shared::AppError> {
        sqlx::query_scalar("SELECT space_id FROM documents WHERE id = $1")
            .bind(document_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("document", document_id))
    }

    async fn document_parent_chain_contains(
        &self,
        parent_id: Uuid,
        document_id: Uuid,
    ) -> Result<bool, shared::AppError> {
        let mut current_id = Some(parent_id);
        let mut visited = BTreeSet::new();
        while let Some(id) = current_id {
            if id == document_id {
                return Ok(true);
            }
            if !visited.insert(id) {
                return Err(shared::AppError::conflict("document tree contains a cycle"));
            }
            current_id = sqlx::query_scalar("SELECT parent_id FROM documents WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?
                .ok_or_else(|| shared::AppError::not_found("document", id))?;
        }
        Ok(false)
    }

    pub(super) async fn resolve_document_id(&self, value: &str) -> Result<Uuid, shared::AppError> {
        if let Ok(id) = Uuid::parse_str(value) {
            let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?;
            return exists.ok_or_else(|| shared::AppError::not_found("document", value));
        }

        let rows = sqlx::query(
            r#"
            SELECT id
            FROM documents
            WHERE slug = $1 AND archived_at IS NULL
            ORDER BY updated_at DESC
            LIMIT 2
            "#,
        )
        .bind(value)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        match rows.as_slice() {
            [row] => Ok(row.get("id")),
            [] => Err(shared::AppError::not_found("document", value)),
            _ => Err(shared::AppError::conflict(
                "document slug is ambiguous across spaces",
            )),
        }
    }

    pub(super) async fn document_response(
        &self,
        document_id: Uuid,
    ) -> Result<DocumentResponse, shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT d.id, s.key AS space_key, d.parent_id, d.slug, d.title,
                   d.document_type, d.status, d.current_revision_id, d.owner_id,
                   d.created_at, d.updated_at,
                   COALESCE(dd.content_markdown, '') AS draft_markdown
            FROM documents d
            JOIN spaces s ON s.id = d.space_id
            LEFT JOIN document_drafts dd ON dd.document_id = d.id
            WHERE d.id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;

        let current_revision_id: Option<Uuid> = row.get("current_revision_id");
        let current_revision = match current_revision_id {
            Some(revision_id) => Some(self.revision_response(document_id, revision_id).await?),
            None => None,
        };
        let task_keys = self.document_task_keys(document_id).await?;
        let phase_keys = self.document_phase_keys(document_id).await?;
        let evidence = self
            .list_evidence(
                None,
                EvidenceQuery {
                    space: None,
                    document_id: Some(document_id.to_string()),
                    task_key: None,
                    phase_key: None,
                    limit: Some(100),
                },
            )
            .await?
            .evidence;
        let owner_id: Uuid = row.get("owner_id");

        Ok(DocumentResponse {
            id: row.get::<Uuid, _>("id").to_string(),
            space_key: row.get("space_key"),
            parent_id: row
                .get::<Option<Uuid>, _>("parent_id")
                .map(|id| id.to_string()),
            slug: row.get("slug"),
            title: row.get("title"),
            document_type: row.get("document_type"),
            status: row.get("status"),
            body_markdown: current_revision
                .as_ref()
                .map(|revision| revision.body_markdown.clone())
                .unwrap_or_default(),
            draft_markdown: row.get("draft_markdown"),
            current_revision,
            task_keys,
            phase_keys,
            evidence,
            created_by: owner_id.to_string(),
            updated_by: owner_id.to_string(),
            created_at: to_iso(row.get("created_at")),
            updated_at: to_iso(row.get("updated_at")),
        })
    }

    async fn revision_response(
        &self,
        document_id: Uuid,
        revision_id: Uuid,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, document_id, version, title, content_markdown, summary, author_id, published_at
            FROM document_revisions
            WHERE document_id = $1 AND id = $2
            "#,
        )
        .bind(document_id)
        .bind(revision_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("revision", revision_id))?;
        Ok(revision_response_from_row(&row))
    }

    async fn document_task_keys(&self, document_id: Uuid) -> Result<Vec<String>, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT td.task_key
            FROM document_task_links dtl
            JOIN task_dossiers td ON td.id = dtl.task_dossier_id
            WHERE dtl.document_id = $1
            ORDER BY td.task_key
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(|row| row.get("task_key")).collect())
    }

    async fn document_phase_keys(
        &self,
        document_id: Uuid,
    ) -> Result<Vec<String>, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT pd.phase_key
            FROM document_phase_links dpl
            JOIN phase_dossiers pd ON pd.id = dpl.phase_dossier_id
            WHERE dpl.document_id = $1
            ORDER BY pd.phase_key
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(|row| row.get("phase_key")).collect())
    }
}
