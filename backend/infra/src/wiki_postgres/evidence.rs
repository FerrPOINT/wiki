use super::{
    PostgresWikiBackend,
    mapping::{attachment_response_from_row, evidence_response_from_row, parse_uuid},
    queries::{ATTACHMENT_ONE_SQL, EVIDENCE_LIST_SQL, EVIDENCE_ONE_SQL},
};
use app::wiki::{
    WikiSpaceAccess as SpaceAccess, checksum, clamp_limit, normalize_evidence_type,
    normalize_phase_key, normalize_required, normalize_space_key, normalize_task_key,
    safe_download_filename,
};
use shared::wiki_contract::*;
use sqlx::Row;
use uuid::Uuid;

impl PostgresWikiBackend {
    pub(super) async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let evidence_type = normalize_evidence_type(&body.evidence_type)?;
        let url = body
            .url
            .as_deref()
            .map(|value| normalize_required(value, "evidence url"))
            .transpose()?;
        match evidence_type {
            "external_url"
                if url.is_none() || body.attachment_id.is_some() || body.checksum.is_some() =>
            {
                return Err(shared::AppError::invalid_input(
                    "external_url evidence requires url only",
                ));
            }
            "uploaded_file" if body.attachment_id.is_none() || body.url.is_some() => {
                return Err(shared::AppError::invalid_input(
                    "uploaded_file evidence requires attachment_id only",
                ));
            }
            "external_url" | "uploaded_file" => {}
            _ => unreachable!("validated evidence type"),
        }

        let document_id = match body.document_id.as_deref() {
            Some(value) => Some(self.resolve_document_id(value).await?),
            None => None,
        };
        let document_space_id = match document_id {
            Some(id) => Some(self.document_space_id(id).await?),
            None => None,
        };
        let space_key = body
            .space
            .as_deref()
            .map(normalize_space_key)
            .transpose()?
            .unwrap_or_else(|| "SDLC".to_string());
        let space_id = if let Some(document_space_id) = document_space_id {
            let requested_space_id = self.space_id(&space_key).await?;
            if requested_space_id != document_space_id {
                return Err(shared::AppError::invalid_input(
                    "document belongs to another space",
                ));
            }
            requested_space_id
        } else {
            self.space_id(&space_key).await?
        };
        self.ensure_space_id_access(claims, space_id, SpaceAccess::Edit)
            .await?;
        self.ensure_space_accepts_writes(space_id).await?;
        let task_key = body
            .task_key
            .as_deref()
            .map(normalize_task_key)
            .transpose()?;
        let phase_key = body
            .phase_key
            .as_deref()
            .map(normalize_phase_key)
            .transpose()?;
        if document_id.is_none() && task_key.is_none() && phase_key.is_none() {
            return Err(shared::AppError::invalid_input(
                "evidence must target a document, task or phase",
            ));
        }
        let title = normalize_required(&body.title, "evidence title")?;
        let evidence_id = Uuid::now_v7();
        let attachment_id = body
            .attachment_id
            .as_deref()
            .map(|value| parse_uuid(value, "attachment"))
            .transpose()?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let task_dossier_id = match &task_key {
            Some(task_key) => Some(
                self.upsert_task_dossier_tx(&mut tx, space_id, task_key)
                    .await?,
            ),
            None => None,
        };
        let phase_dossier_id = match &phase_key {
            Some(phase_key) => Some(
                self.upsert_phase_dossier_tx(&mut tx, space_id, phase_key)
                    .await?,
            ),
            None => None,
        };
        let mut stored_checksum = body.checksum;
        if let Some(attachment_id) = attachment_id {
            let attachment_row = sqlx::query(
                "SELECT checksum FROM attachments WHERE id = $1 AND owner_entity_id IS NULL AND uploaded_by = $2",
            )
            .bind(attachment_id)
            .bind(actor_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
            stored_checksum = Some(attachment_row.get("checksum"));
            sqlx::query(
                r#"
                UPDATE attachments
                SET space_id = $2, owner_entity_type = 'evidence', owner_entity_id = $3
                WHERE id = $1
                "#,
            )
            .bind(attachment_id)
            .bind(space_id)
            .bind(evidence_id)
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
        .bind(evidence_id)
        .bind(space_id)
        .bind(document_id)
        .bind(task_dossier_id)
        .bind(phase_dossier_id)
        .bind(evidence_type)
        .bind(title)
        .bind(url)
        .bind(attachment_id)
        .bind(stored_checksum)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(shared::AppError::database)?;
        let evidence_id: Uuid = row.get("id");
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "evidence.create",
            "evidence",
            evidence_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.get_evidence_by_id(evidence_id).await
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
        let task_key = query
            .task_key
            .as_deref()
            .map(normalize_task_key)
            .transpose()?;
        let phase_key = query
            .phase_key
            .as_deref()
            .map(normalize_phase_key)
            .transpose()?;
        let limit = clamp_limit(query.limit, 100);
        let rows = sqlx::query(EVIDENCE_LIST_SQL)
            .bind(space_key)
            .bind(document_id)
            .bind(task_key)
            .bind(phase_key)
            .bind(access_user_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(EvidenceListResponse {
            evidence: rows.iter().map(evidence_response_from_row).collect(),
        })
    }

    async fn get_evidence_by_id(
        &self,
        evidence_id: Uuid,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let row = sqlx::query(EVIDENCE_ONE_SQL)
            .bind(evidence_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("evidence", evidence_id))?;
        Ok(evidence_response_from_row(&row))
    }

    pub(super) async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let evidence_id = parse_uuid(evidence_id, "evidence")?;
        self.ensure_evidence_access(claims, evidence_id, SpaceAccess::View)
            .await?;
        self.get_evidence_by_id(evidence_id).await
    }

    pub(super) async fn upload_attachment(
        &self,
        claims: &WikiClaims,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        if bytes.is_empty() {
            return Err(shared::AppError::invalid_input("file is required"));
        }
        if bytes.len() > self.max_upload_bytes {
            return Err(shared::AppError::invalid_input("file is too large"));
        }
        let id = Uuid::now_v7();
        let safe_name = safe_download_filename(&file_name);
        let storage_key = format!("attachments/{id}/{safe_name}");
        self.storage.put(&storage_key, &bytes).await?;

        let checksum = checksum(&bytes);
        let size_bytes = bytes.len() as i64;
        let row = match sqlx::query(
            r#"
            INSERT INTO attachments (
                id, file_name, content_type, size_bytes, storage_key,
                checksum, uploaded_by, uploaded_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, now())
            RETURNING id, file_name, content_type, size_bytes, checksum, uploaded_by, uploaded_at
            "#,
        )
        .bind(id)
        .bind(file_name)
        .bind(content_type)
        .bind(size_bytes)
        .bind(&storage_key)
        .bind(checksum)
        .bind(actor_id)
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => row,
            Err(err) => {
                let _ = self.storage.delete(&storage_key).await;
                return Err(shared::AppError::database(err));
            }
        };

        self.audit(Some(actor_id), "attachment.upload", "attachment", id)
            .await?;
        Ok(attachment_response_from_row(&row))
    }

    pub(super) async fn get_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        self.ensure_attachment_access(claims, attachment_id).await?;
        let row = sqlx::query(ATTACHMENT_ONE_SQL)
            .bind(attachment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        Ok(attachment_response_from_row(&row))
    }

    pub(super) async fn download_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentDownloadResponse, shared::AppError> {
        let attachment_id = parse_uuid(attachment_id, "attachment")?;
        self.ensure_attachment_access(claims, attachment_id).await?;
        let row = sqlx::query(
            r#"
            SELECT file_name, content_type, storage_key
            FROM attachments
            WHERE id = $1
            "#,
        )
        .bind(attachment_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("attachment", attachment_id))?;
        let file_name: String = row.get("file_name");
        let content_type: String = row.get("content_type");
        let storage_key: String = row.get("storage_key");
        let bytes = self.storage.get(&storage_key).await.map_err(|err| {
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
