use super::{
    PostgresWikiBackend, ensure_document_accepts_writes_tx,
    mapping::{parse_uuid, revision_response_from_row, to_iso},
};
use app::wiki::{
    WikiArchiveDocumentCommand, WikiCreateDocumentCommand, WikiDocumentRepository,
    WikiDocumentRepositoryFuture, WikiDocumentUseCase, WikiMoveDocumentCommand,
    WikiPublishDocumentCommand, WikiSpaceAccess as SpaceAccess, WikiUpdateDocumentDraftCommand,
    checksum, markdown_to_html, markdown_to_text, normalize_space_key,
};
use shared::wiki_contract::*;
use sqlx::Row;
use std::collections::BTreeSet;
use uuid::Uuid;

struct PostgresWikiDocumentRepository<'a> {
    backend: &'a PostgresWikiBackend,
}

impl WikiDocumentRepository for PostgresWikiDocumentRepository<'_> {
    fn create_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
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
            .bind(command.space_id)
            .bind(command.parent_id)
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
            .bind(command.document_id)
            .bind(command.space_id)
            .bind(command.parent_id)
            .bind(&command.slug)
            .bind(&command.title)
            .bind(&command.document_type)
            .bind(command.owner_id)
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
            .bind(command.document_id)
            .bind(command.owner_id)
            .bind(&command.content_markdown)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;

            if let Some(task_key) = &command.task_key {
                let task_id = self
                    .backend
                    .upsert_task_dossier_tx(&mut tx, command.space_id, task_key)
                    .await?;
                sqlx::query(
                    r#"
                    INSERT INTO document_task_links (space_id, document_id, task_dossier_id, created_by, created_at)
                    VALUES ($1, $2, $3, $4, now())
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(command.space_id)
                .bind(command.document_id)
                .bind(task_id)
                .bind(actor_id)
                .execute(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
            }
            if let Some(phase_key) = &command.phase_key {
                let phase_id = self
                    .backend
                    .upsert_phase_dossier_tx(&mut tx, command.space_id, phase_key)
                    .await?;
                sqlx::query(
                    r#"
                    INSERT INTO document_phase_links (space_id, document_id, phase_dossier_id, created_by, created_at)
                    VALUES ($1, $2, $3, $4, now())
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(command.space_id)
                .bind(command.document_id)
                .bind(phase_id)
                .bind(actor_id)
                .execute(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
            }

            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "document.create",
                    "document",
                    command.document_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            self.backend.document_response(command.document_id).await
        })
    }

    fn get_document<'a>(
        &'a self,
        document_id: Uuid,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentResponse> {
        Box::pin(async move { self.backend.document_response(document_id).await })
    }

    fn update_document_draft<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUpdateDocumentDraftCommand,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
            ensure_document_accepts_writes_tx(&mut tx, command.document_id).await?;
            sqlx::query(
                r#"
                UPDATE documents
                SET title = COALESCE($2, title),
                    status = 'draft',
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(command.document_id)
            .bind(command.title.as_deref())
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
            .bind(command.document_id)
            .bind(actor_id)
            .bind(&command.content_markdown)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "document.draft_update",
                    "document",
                    command.document_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            self.backend.document_response(command.document_id).await
        })
    }

    fn publish_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiPublishDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentRevisionResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
            ensure_document_accepts_writes_tx(&mut tx, command.document_id).await?;
            let row = sqlx::query(
                r#"
                SELECT d.title, COALESCE(dd.content_markdown, '') AS content_markdown
                FROM documents d
                LEFT JOIN document_drafts dd ON dd.document_id = d.id
                WHERE d.id = $1
                FOR UPDATE OF d
                "#,
            )
            .bind(command.document_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("document", command.document_id))?;
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
            .bind(command.document_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
            let content_text = markdown_to_text(&content_markdown);
            let content_html = markdown_to_html(&content_markdown);
            let content_checksum = checksum(content_markdown.as_bytes());

            let revision_row = sqlx::query(
                r#"
                INSERT INTO document_revisions (
                    id, document_id, version, title, content_markdown, content_html,
                    content_text, content_checksum, summary, author_id, published_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
                RETURNING id, document_id, version, title, content_markdown, content_html, summary, author_id, published_at
                "#,
            )
            .bind(command.revision_id)
            .bind(command.document_id)
            .bind(version)
            .bind(title)
            .bind(&content_markdown)
            .bind(content_html)
            .bind(content_text)
            .bind(content_checksum)
            .bind(command.summary.as_deref())
            .bind(actor_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;

            sqlx::query(
                r#"
                UPDATE documents
                SET current_revision_id = $2, status = 'published', updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(command.document_id)
            .bind(command.revision_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
            sqlx::query(
                "UPDATE document_drafts SET base_revision_id = $2, updated_at = now() WHERE document_id = $1",
            )
            .bind(command.document_id)
            .bind(command.revision_id)
            .execute(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "document.publish",
                    "document",
                    command.document_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            Ok(revision_response_from_row(&revision_row))
        })
    }

    fn archive_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiArchiveDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
            ensure_document_accepts_writes_tx(&mut tx, command.document_id).await?;
            let row = sqlx::query(
                r#"
                UPDATE documents
                SET status = 'archived', archived_at = now(), updated_at = now()
                WHERE id = $1
                RETURNING id
                "#,
            )
            .bind(command.document_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("document", command.document_id))?;
            let document_id: Uuid = row.get("id");
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "document.archive",
                    "document",
                    document_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            self.backend.document_response(document_id).await
        })
    }

    fn move_document<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiMoveDocumentCommand,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
            ensure_document_accepts_writes_tx(&mut tx, command.document_id).await?;
            let row = sqlx::query(
                r#"
                UPDATE documents
                SET parent_id = $2, updated_at = now()
                WHERE id = $1
                RETURNING id
                "#,
            )
            .bind(command.document_id)
            .bind(command.parent_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("document", command.document_id))?;
            let document_id: Uuid = row.get("id");
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "document.move",
                    "document",
                    document_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            self.backend.document_response(document_id).await
        })
    }

    fn list_revisions<'a>(
        &'a self,
        document_id: Uuid,
    ) -> WikiDocumentRepositoryFuture<'a, Vec<DocumentRevisionResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(
                r#"
                SELECT id, document_id, version, title, content_markdown, content_html, summary, author_id, published_at
                FROM document_revisions
                WHERE document_id = $1
                ORDER BY version DESC
                "#,
            )
            .bind(document_id)
            .fetch_all(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?;
            Ok(rows.iter().map(revision_response_from_row).collect())
        })
    }

    fn get_revision<'a>(
        &'a self,
        document_id: Uuid,
        revision_id: Uuid,
    ) -> WikiDocumentRepositoryFuture<'a, DocumentRevisionResponse> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT id, document_id, version, title, content_markdown, content_html, summary, author_id, published_at
                FROM document_revisions
                WHERE document_id = $1 AND id = $2
                "#,
            )
            .bind(document_id)
            .bind(revision_id)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("revision", revision_id))?;
            Ok(revision_response_from_row(&row))
        })
    }
}

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

        let parent_id = match body.parent_id.as_deref() {
            Some(parent_id) => {
                let resolved = self.resolve_document_id(parent_id).await?;
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

        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .create(actor_id, space_id, parent_id, body)
            .await
    }

    pub(super) async fn get_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository).get(document_id).await
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
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .update_draft(actor_id, document_id, body)
            .await
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
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .publish(actor_id, document_id, body)
            .await
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
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .archive(actor_id, document_id)
            .await
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
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .move_document(actor_id, document_id, parent_id)
            .await
    }

    pub(super) async fn list_document_revisions(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentRevisionListResponse, shared::AppError> {
        let document_id = self.resolve_document_id(document_id).await?;
        self.ensure_document_access(claims, document_id, SpaceAccess::View)
            .await?;
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .list_revisions(document_id)
            .await
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
        let repository = PostgresWikiDocumentRepository { backend: self };
        WikiDocumentUseCase::new(&repository)
            .get_revision(document_id, revision_id)
            .await
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
            body_html: current_revision
                .as_ref()
                .map(|revision| revision.body_html.clone())
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
            SELECT id, document_id, version, title, content_markdown, content_html, summary, author_id, published_at
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
