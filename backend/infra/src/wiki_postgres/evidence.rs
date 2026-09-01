use super::{
    PostgresWikiBackend,
    mapping::{attachment_response_from_row, evidence_response_from_row, parse_uuid},
    queries::{ATTACHMENT_ONE_SQL, EVIDENCE_LIST_SQL, EVIDENCE_ONE_SQL},
};
use app::wiki::{
    WikiCreateEvidenceCommand, WikiEvidenceQueryCriteria, WikiEvidenceRepository,
    WikiEvidenceRepositoryFuture, WikiEvidenceUseCase, WikiSpaceAccess as SpaceAccess,
    WikiUploadAttachmentCommand, normalize_evidence_space_key, normalize_space_key,
};
use shared::wiki_contract::*;
use sqlx::Row;
use uuid::Uuid;

struct PostgresWikiEvidenceRepository<'a> {
    backend: &'a PostgresWikiBackend,
}

impl PostgresWikiEvidenceRepository<'_> {
    async fn get_evidence_by_id(
        &self,
        evidence_id: Uuid,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let row = sqlx::query(EVIDENCE_ONE_SQL)
            .bind(evidence_id)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("evidence", evidence_id))?;
        Ok(evidence_response_from_row(&row))
    }
}

impl WikiEvidenceRepository for PostgresWikiEvidenceRepository<'_> {
    fn create_evidence<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiCreateEvidenceCommand,
    ) -> WikiEvidenceRepositoryFuture<'a, EvidenceResponse> {
        Box::pin(async move {
            let mut tx = self
                .backend
                .pool
                .begin()
                .await
                .map_err(shared::AppError::database)?;
            let task_dossier_id = match &command.task_key {
                Some(task_key) => Some(
                    self.backend
                        .upsert_task_dossier_tx(&mut tx, command.space_id, task_key)
                        .await?,
                ),
                None => None,
            };
            let phase_dossier_id = match &command.phase_key {
                Some(phase_key) => Some(
                    self.backend
                        .upsert_phase_dossier_tx(&mut tx, command.space_id, phase_key)
                        .await?,
                ),
                None => None,
            };

            let mut stored_checksum = None;
            if let Some(attachment_id) = command.attachment_id {
                let attachment_row = sqlx::query(
                    "SELECT checksum FROM attachments WHERE id = $1 AND owner_entity_id IS NULL AND uploaded_by = $2",
                )
                .bind(attachment_id)
                .bind(actor_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(shared::AppError::database)?
                .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
                stored_checksum = Some(attachment_row.get::<String, _>("checksum"));

                sqlx::query(
                    r#"
                    UPDATE attachments
                    SET space_id = $2, owner_entity_type = 'evidence', owner_entity_id = $3
                    WHERE id = $1
                    "#,
                )
                .bind(attachment_id)
                .bind(command.space_id)
                .bind(command.evidence_id)
                .execute(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
            }

            let row = sqlx::query(
                r#"
                INSERT INTO evidence_items (
                    id, space_id, document_id, task_dossier_id, phase_dossier_id,
                    evidence_type, title, url, attachment_id, checksum, metadata,
                    created_by, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, '{}'::jsonb, $11, now())
                RETURNING id
                "#,
            )
            .bind(command.evidence_id)
            .bind(command.space_id)
            .bind(command.document_id)
            .bind(task_dossier_id)
            .bind(phase_dossier_id)
            .bind(&command.evidence_type)
            .bind(&command.title)
            .bind(command.url.as_deref())
            .bind(command.attachment_id)
            .bind(stored_checksum.as_deref())
            .bind(actor_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(shared::AppError::database)?;
            let evidence_id: Uuid = row.get("id");
            self.backend
                .insert_audit(
                    &mut tx,
                    Some(actor_id),
                    "evidence.create",
                    "evidence",
                    evidence_id,
                )
                .await?;
            tx.commit().await.map_err(shared::AppError::database)?;
            self.get_evidence_by_id(evidence_id).await
        })
    }

    fn list_evidence<'a>(
        &'a self,
        criteria: &'a WikiEvidenceQueryCriteria,
    ) -> WikiEvidenceRepositoryFuture<'a, Vec<EvidenceResponse>> {
        Box::pin(async move {
            let rows = sqlx::query(EVIDENCE_LIST_SQL)
                .bind(criteria.space_key.as_deref())
                .bind(criteria.document_id)
                .bind(criteria.task_key.as_deref())
                .bind(criteria.phase_key.as_deref())
                .bind(criteria.access_user_id)
                .bind(criteria.limit)
                .fetch_all(&self.backend.pool)
                .await
                .map_err(shared::AppError::database)?;
            Ok(rows.iter().map(evidence_response_from_row).collect())
        })
    }

    fn get_evidence<'a>(
        &'a self,
        evidence_id: Uuid,
    ) -> WikiEvidenceRepositoryFuture<'a, EvidenceResponse> {
        Box::pin(async move { self.get_evidence_by_id(evidence_id).await })
    }

    fn upload_attachment<'a>(
        &'a self,
        actor_id: Uuid,
        command: WikiUploadAttachmentCommand,
    ) -> WikiEvidenceRepositoryFuture<'a, AttachmentResponse> {
        Box::pin(async move {
            self.backend
                .storage
                .put(&command.storage_key, &command.bytes)
                .await?;

            let insert_result = async {
                let mut tx = self
                    .backend
                    .pool
                    .begin()
                    .await
                    .map_err(shared::AppError::database)?;
                let row = sqlx::query(
                    r#"
                    INSERT INTO attachments (
                        id, file_name, content_type, size_bytes, storage_key,
                        checksum, uploaded_by, uploaded_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, now())
                    RETURNING id, file_name, content_type, size_bytes, checksum, uploaded_by, uploaded_at
                    "#,
                )
                .bind(command.attachment_id)
                .bind(&command.file_name)
                .bind(&command.content_type)
                .bind(command.size_bytes)
                .bind(&command.storage_key)
                .bind(&command.checksum)
                .bind(actor_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(shared::AppError::database)?;
                self.backend
                    .insert_audit(
                        &mut tx,
                        Some(actor_id),
                        "attachment.upload",
                        "attachment",
                        command.attachment_id,
                    )
                    .await?;
                tx.commit().await.map_err(shared::AppError::database)?;
                Ok::<_, shared::AppError>(row)
            }
            .await;

            let row = match insert_result {
                Ok(row) => row,
                Err(err) => {
                    let _ = self.backend.storage.delete(&command.storage_key).await;
                    return Err(err);
                }
            };

            Ok(attachment_response_from_row(&row))
        })
    }

    fn get_attachment<'a>(
        &'a self,
        attachment_id: Uuid,
    ) -> WikiEvidenceRepositoryFuture<'a, AttachmentResponse> {
        Box::pin(async move {
            let row = sqlx::query(ATTACHMENT_ONE_SQL)
                .bind(attachment_id)
                .fetch_optional(&self.backend.pool)
                .await
                .map_err(shared::AppError::database)?
                .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
            Ok(attachment_response_from_row(&row))
        })
    }

    fn download_attachment<'a>(
        &'a self,
        attachment_id: Uuid,
    ) -> WikiEvidenceRepositoryFuture<'a, AttachmentDownloadResponse> {
        Box::pin(async move {
            let row = sqlx::query(
                r#"
                SELECT file_name, content_type, storage_key
                FROM attachments
                WHERE id = $1
                "#,
            )
            .bind(attachment_id)
            .fetch_optional(&self.backend.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
            let file_name: String = row.get("file_name");
            let content_type: String = row.get("content_type");
            let storage_key: String = row.get("storage_key");
            let bytes = self
                .backend
                .storage
                .get(&storage_key)
                .await
                .map_err(|err| {
                    if matches!(err, shared::AppError::NotFound(_)) {
                        shared::AppError::not_found("attachment file", attachment_id)
                    } else {
                        err
                    }
                })?;
            Ok(AttachmentDownloadResponse {
                file_name,
                content_type,
                bytes,
            })
        })
    }
}

impl PostgresWikiBackend {
    pub(super) async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let document_id = match body.document_id.as_deref() {
            Some(value) => Some(self.resolve_document_id(value).await?),
            None => None,
        };
        let document_space = match document_id {
            Some(id) => Some(self.document_space_context(id).await?),
            None => None,
        };
        let document_space_key = document_space.as_ref().map(|(_, key)| key.as_str());
        let space_key = normalize_evidence_space_key(body.space.as_deref(), document_space_key)?;
        let space_id = self.space_id(&space_key).await?;
        if document_space.is_some_and(|(document_space_id, _)| document_space_id != space_id) {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }

        self.ensure_space_id_access(claims, space_id, SpaceAccess::Edit)
            .await?;
        self.ensure_space_accepts_writes(space_id).await?;
        let repository = PostgresWikiEvidenceRepository { backend: self };
        WikiEvidenceUseCase::new(&repository)
            .create(actor_id, space_id, document_id, body)
            .await
    }

    pub(super) async fn list_evidence(
        &self,
        claims: Option<&WikiClaims>,
        query: EvidenceQuery,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        let space_key = query
            .space
            .as_deref()
            .map(normalize_space_key)
            .transpose()?;
        let document_id = match query.document_id.as_deref() {
            Some(value) => {
                let document_id = self.resolve_document_id(value).await?;
                if let Some(claims) = claims {
                    self.ensure_document_access(claims, document_id, SpaceAccess::View)
                        .await?;
                }
                Some(document_id)
            }
            None => None,
        };
        let access_user_id = match claims {
            Some(claims) => {
                if let Some(space_key) = space_key.as_deref() {
                    self.ensure_space_access(claims, space_key, SpaceAccess::View)
                        .await?;
                }
                self.restricted_user_id(claims).await?
            }
            None => None,
        };
        let repository = PostgresWikiEvidenceRepository { backend: self };
        WikiEvidenceUseCase::new(&repository)
            .list(
                space_key.as_deref(),
                document_id,
                query.task_key.as_deref(),
                query.phase_key.as_deref(),
                access_user_id,
                query.limit,
            )
            .await
    }

    pub(super) async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let evidence_id = parse_uuid(evidence_id, "evidence")?;
        self.ensure_evidence_access(claims, evidence_id, SpaceAccess::View)
            .await?;
        let repository = PostgresWikiEvidenceRepository { backend: self };
        WikiEvidenceUseCase::new(&repository).get(evidence_id).await
    }

    pub(super) async fn upload_attachment(
        &self,
        claims: &WikiClaims,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let repository = PostgresWikiEvidenceRepository { backend: self };
        WikiEvidenceUseCase::new(&repository)
            .upload_attachment(
                actor_id,
                file_name,
                content_type,
                bytes,
                self.max_upload_bytes,
            )
            .await
    }

    pub(super) async fn get_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        self.ensure_attachment_access(claims, attachment_id).await?;
        let repository = PostgresWikiEvidenceRepository { backend: self };
        WikiEvidenceUseCase::new(&repository)
            .get_attachment(attachment_id)
            .await
    }

    pub(super) async fn download_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentDownloadResponse, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        self.ensure_attachment_access(claims, attachment_id).await?;
        let repository = PostgresWikiEvidenceRepository { backend: self };
        WikiEvidenceUseCase::new(&repository)
            .download_attachment(attachment_id)
            .await
    }

    async fn document_space_context(
        &self,
        document_id: Uuid,
    ) -> Result<(Uuid, String), shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT d.space_id, s.key AS space_key
            FROM documents d
            JOIN spaces s ON s.id = d.space_id
            WHERE d.id = $1
            "#,
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("document", document_id))?;
        Ok((row.get("space_id"), row.get("space_key")))
    }

    async fn ensure_evidence_access(
        &self,
        claims: &WikiClaims,
        evidence_id: Uuid,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let space_id: Uuid =
            sqlx::query_scalar("SELECT space_id FROM evidence_items WHERE id = $1")
                .bind(evidence_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(shared::AppError::database)?
                .ok_or_else(|| shared::AppError::not_found("evidence", evidence_id))?;
        self.ensure_space_id_access(claims, space_id, required)
            .await?;
        Ok(space_id)
    }

    async fn ensure_attachment_access(
        &self,
        claims: &WikiClaims,
        attachment_id: Uuid,
    ) -> Result<(), shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let row = sqlx::query(
            r#"
            SELECT space_id, uploaded_by
            FROM attachments
            WHERE id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        let uploaded_by: Uuid = row.get("uploaded_by");
        let space_id: Option<Uuid> = row.get("space_id");
        match space_id {
            Some(space_id) => {
                self.ensure_space_id_access(claims, space_id, SpaceAccess::View)
                    .await?;
                Ok(())
            }
            None => {
                let role = self.active_global_role(user_id).await?;
                if role == "admin" || uploaded_by == user_id {
                    Ok(())
                } else {
                    Err(shared::AppError::Forbidden)
                }
            }
        }
    }
}
