mod connection;
mod documents;
mod identity;
mod mapping;
mod queries;
mod spaces;
pub use connection::connect_postgres_wiki_backend;

use app::wiki::{
    WikiSpaceAccess as SpaceAccess, build_wiki_search_criteria, checksum, clamp_limit,
    normalize_document_type, normalize_evidence_type, normalize_phase_key, normalize_required,
    normalize_space_key, normalize_task_key, safe_download_filename, space_role_allows,
};
use mapping::*;
use queries::*;
use shared::wiki_contract::*;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
struct PostgresWikiBackend {
    pool: PgPool,
    auth: shared::AuthConfig,
    storage: Arc<dyn domain::wiki::WikiAttachmentStorage>,
    max_upload_bytes: usize,
    settings: WikiSettingsSnapshot,
}

impl PostgresWikiBackend {
    async fn list_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<TaskPageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT task_key
            FROM task_dossiers
            WHERE space_id = $1
            ORDER BY task_key
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let task_key: String = row.get("task_key");
            tasks.push(self.task_page(&key, &task_key).await?);
        }
        Ok(TaskPageListResponse { tasks })
    }

    async fn get_task(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let task_key = normalize_task_key(task_key)?;
        self.task_page(&key, &task_key).await
    }

    async fn link_task_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let task_key = normalize_task_key(task_key)?;
        let document_id = self.resolve_document_id(&body.document_id).await?;
        if self.document_space_id(document_id).await? != space_id {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let task_id = self
            .upsert_task_dossier_tx(&mut tx, space_id, &task_key)
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
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "task.link_document",
            "task",
            task_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.task_page(&key, &task_key).await
    }

    async fn list_task_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let task = self.get_task(claims, space_key, task_key).await?;
        let mut documents = Vec::with_capacity(task.documents.len());
        for summary in task.documents {
            documents.push(
                self.document_response(parse_uuid(&summary.id, "document")?)
                    .await?,
            );
        }
        Ok(DocumentListResponse { documents })
    }

    async fn list_task_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_task(claims, space_key, task_key).await?.evidence,
        })
    }

    async fn list_phases(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<PhasePageListResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT phase_key
            FROM phase_dossiers
            WHERE space_id = $1
            ORDER BY phase_key
            "#,
        )
        .bind(space_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let mut phases = Vec::with_capacity(rows.len());
        for row in rows {
            let phase_key: String = row.get("phase_key");
            phases.push(self.phase_page(&key, &phase_key).await?);
        }
        Ok(PhasePageListResponse { phases })
    }

    async fn get_phase(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        self.ensure_space_access(claims, &key, SpaceAccess::View)
            .await?;
        let phase_key = normalize_phase_key(phase_key)?;
        self.phase_page(&key, &phase_key).await
    }

    async fn link_phase_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        let space_id = self
            .ensure_space_access(claims, &key, SpaceAccess::Edit)
            .await?;
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let phase_key = normalize_phase_key(phase_key)?;
        let document_id = self.resolve_document_id(&body.document_id).await?;
        if self.document_space_id(document_id).await? != space_id {
            return Err(shared::AppError::invalid_input(
                "document belongs to another space",
            ));
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(shared::AppError::database)?;
        let phase_id = self
            .upsert_phase_dossier_tx(&mut tx, space_id, &phase_key)
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
        self.insert_audit(
            &mut tx,
            Some(actor_id),
            "phase.link_document",
            "phase",
            phase_id,
        )
        .await?;
        tx.commit().await.map_err(shared::AppError::database)?;
        self.phase_page(&key, &phase_key).await
    }

    async fn list_phase_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        let phase = self.get_phase(claims, space_key, phase_key).await?;
        let mut documents = Vec::with_capacity(phase.documents.len());
        for summary in phase.documents {
            documents.push(
                self.document_response(parse_uuid(&summary.id, "document")?)
                    .await?,
            );
        }
        Ok(DocumentListResponse { documents })
    }

    async fn list_phase_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        Ok(EvidenceListResponse {
            evidence: self.get_phase(claims, space_key, phase_key).await?.evidence,
        })
    }

    async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let actor_id = parse_uuid(&claims.user_id, "user")?;
        let evidence_type = normalize_evidence_type(&body.evidence_type)?;
        match evidence_type {
            "external_url" if body.url.is_none() || body.attachment_id.is_some() => {
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
        .bind(body.url)
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

    async fn list_evidence(
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

    async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, shared::AppError> {
        let evidence_id = parse_uuid(evidence_id, "evidence")?;
        self.ensure_evidence_access(claims, evidence_id, SpaceAccess::View)
            .await?;
        self.get_evidence_by_id(evidence_id).await
    }

    async fn upload_attachment(
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

    async fn get_attachment(
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

    async fn download_attachment(
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

    async fn list_templates(&self) -> Result<TemplateListResponse, shared::AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, document_type, content_markdown
            FROM document_templates
            WHERE is_active = true
            ORDER BY lower(name)
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(TemplateListResponse {
            templates: rows.iter().map(template_response_from_row).collect(),
        })
    }

    async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, shared::AppError> {
        let actor_id = self.ensure_admin(claims).await?;
        let name = normalize_required(&body.name, "template name")?;
        let document_type = normalize_document_type(&body.document_type, false)?;
        let body_markdown = normalize_required(&body.body_markdown, "template body_markdown")?;
        let id = Uuid::now_v7();
        let row = sqlx::query(
            r#"
            INSERT INTO document_templates (
                id, name, document_type, content_markdown, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, true, now(), now())
            RETURNING id, name, document_type, content_markdown
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(document_type)
        .bind(body_markdown)
        .fetch_one(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        self.audit(Some(actor_id), "template.create", "template", id)
            .await?;
        Ok(template_response_from_row(&row))
    }

    async fn list_audit_log(
        &self,
        claims: &WikiClaims,
    ) -> Result<AuditLogResponse, shared::AppError> {
        self.ensure_admin(claims).await?;
        let rows = sqlx::query(
            r#"
            SELECT id, actor_id, action, entity_type, entity_id, created_at
            FROM audit_log
            ORDER BY created_at DESC
            LIMIT 200
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(AuditLogResponse {
            entries: rows.iter().map(audit_entry_from_row).collect(),
        })
    }

    async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, shared::AppError> {
        let criteria = build_wiki_search_criteria(
            query.q.as_deref(),
            query.space.as_deref(),
            query.task_key.as_deref(),
            query.phase_key.as_deref(),
            query.document_type.as_deref(),
            query.include_archived,
            query.limit,
        )?;
        if let Some(space_key) = criteria.space_key.as_deref() {
            self.ensure_space_access(claims, space_key, SpaceAccess::View)
                .await?;
        }
        let access_user_id = self.restricted_user_id(claims).await?;

        let document_rows = sqlx::query(SEARCH_DOCUMENTS_SQL)
            .bind(&criteria.needle)
            .bind(criteria.space_key.as_deref())
            .bind(criteria.task_key.as_deref())
            .bind(criteria.phase_key.as_deref())
            .bind(criteria.document_type)
            .bind(criteria.include_archived)
            .bind(access_user_id)
            .bind(criteria.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        let evidence_rows = sqlx::query(SEARCH_EVIDENCE_SQL)
            .bind(&criteria.evidence_like_pattern)
            .bind(criteria.space_key.as_deref())
            .bind(criteria.task_key.as_deref())
            .bind(criteria.phase_key.as_deref())
            .bind(access_user_id)
            .bind(criteria.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;

        let mut results = document_rows
            .iter()
            .map(search_result_from_row)
            .chain(evidence_rows.iter().map(search_result_from_row))
            .collect::<Vec<_>>();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        results.truncate(criteria.limit as usize);
        Ok(SearchResponse { results })
    }

    async fn ensure_admin(&self, claims: &WikiClaims) -> Result<Uuid, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let role = self.active_global_role(user_id).await?;
        if role == "admin" {
            Ok(user_id)
        } else {
            Err(shared::AppError::Forbidden)
        }
    }

    async fn active_global_role(&self, user_id: Uuid) -> Result<String, shared::AppError> {
        sqlx::query_scalar("SELECT global_role FROM users WHERE id = $1 AND is_active = true")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or(shared::AppError::Unauthorized)
    }

    async fn restricted_user_id(
        &self,
        claims: &WikiClaims,
    ) -> Result<Option<Uuid>, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let role = self.active_global_role(user_id).await?;
        if role == "admin" {
            Ok(None)
        } else {
            Ok(Some(user_id))
        }
    }

    async fn ensure_space_access(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        self.ensure_space_id_access(claims, space_id, required)
            .await?;
        Ok(space_id)
    }

    async fn ensure_space_id_access(
        &self,
        claims: &WikiClaims,
        space_id: Uuid,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let user_id = parse_uuid(&claims.user_id, "user")?;
        let row = sqlx::query(
            r#"
            SELECT u.global_role, sm.role AS space_role
            FROM users u
            LEFT JOIN space_members sm ON sm.user_id = u.id AND sm.space_id = $2
            WHERE u.id = $1 AND u.is_active = true
            "#,
        )
        .bind(user_id)
        .bind(space_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or(shared::AppError::Unauthorized)?;

        let global_role: String = row.get("global_role");
        let space_role: Option<String> = row.get("space_role");
        if global_role == "admin" || space_role_allows(space_role.as_deref(), required) {
            Ok(user_id)
        } else {
            Err(shared::AppError::Forbidden)
        }
    }

    async fn ensure_document_access(
        &self,
        claims: &WikiClaims,
        document_id: Uuid,
        required: SpaceAccess,
    ) -> Result<Uuid, shared::AppError> {
        let space_id = self.document_space_id(document_id).await?;
        self.ensure_space_id_access(claims, space_id, required)
            .await?;
        Ok(space_id)
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

    async fn user_response(&self, user_id: Uuid) -> Result<WikiUserResponse, shared::AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, username, display_name, global_role, is_active
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?
        .ok_or_else(|| shared::AppError::not_found("user", user_id))?;
        Ok(user_response_from_row(&row))
    }

    async fn space_id(&self, space_key: &str) -> Result<Uuid, shared::AppError> {
        let key = normalize_space_key(space_key)?;
        sqlx::query_scalar("SELECT id FROM spaces WHERE key = $1")
            .bind(&key)
            .fetch_optional(&self.pool)
            .await
            .map_err(shared::AppError::database)?
            .ok_or_else(|| shared::AppError::not_found("space", space_key))
    }

    async fn task_page(
        &self,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let task_row = sqlx::query(
            "SELECT id, title_snapshot FROM task_dossiers WHERE space_id = $1 AND task_key = $2",
        )
        .bind(space_id)
        .bind(task_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let Some(task_row) = task_row else {
            return Ok(TaskPageResponse {
                space_key: space_key.to_string(),
                task_key: task_key.to_string(),
                title: None,
                document_count: 0,
                evidence_count: 0,
                documents: Vec::new(),
                evidence: Vec::new(),
            });
        };
        let task_id: Uuid = task_row.get("id");
        let document_rows = sqlx::query(
            r#"
            SELECT d.id, d.slug, d.title, d.document_type, d.status, d.updated_at
            FROM document_task_links dtl
            JOIN documents d ON d.id = dtl.document_id
            WHERE dtl.task_dossier_id = $1 AND d.archived_at IS NULL
            ORDER BY d.updated_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents: Vec<_> = document_rows
            .iter()
            .map(document_summary_from_row)
            .collect();
        let evidence = self.evidence_for_target(Some(task_id), None).await?;
        let title_snapshot: Option<String> = task_row.get("title_snapshot");
        let title =
            title_snapshot.or_else(|| documents.first().map(|document| document.title.clone()));
        Ok(TaskPageResponse {
            space_key: space_key.to_string(),
            task_key: task_key.to_string(),
            title,
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        })
    }

    async fn phase_page(
        &self,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        let space_id = self.space_id(space_key).await?;
        let phase_row = sqlx::query(
            "SELECT id, phase_name FROM phase_dossiers WHERE space_id = $1 AND phase_key = $2",
        )
        .bind(space_id)
        .bind(phase_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let Some(phase_row) = phase_row else {
            return Ok(PhasePageResponse {
                space_key: space_key.to_string(),
                phase_key: phase_key.to_string(),
                title: Some(phase_key.to_string()),
                document_count: 0,
                evidence_count: 0,
                documents: Vec::new(),
                evidence: Vec::new(),
            });
        };
        let phase_id: Uuid = phase_row.get("id");
        let document_rows = sqlx::query(
            r#"
            SELECT d.id, d.slug, d.title, d.document_type, d.status, d.updated_at
            FROM document_phase_links dpl
            JOIN documents d ON d.id = dpl.document_id
            WHERE dpl.phase_dossier_id = $1 AND d.archived_at IS NULL
            ORDER BY d.updated_at DESC
            "#,
        )
        .bind(phase_id)
        .fetch_all(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        let documents: Vec<_> = document_rows
            .iter()
            .map(document_summary_from_row)
            .collect();
        let evidence = self.evidence_for_target(None, Some(phase_id)).await?;
        let phase_name: Option<String> = phase_row.get("phase_name");
        Ok(PhasePageResponse {
            space_key: space_key.to_string(),
            phase_key: phase_key.to_string(),
            title: phase_name.or_else(|| Some(phase_key.to_string())),
            document_count: documents.len(),
            evidence_count: evidence.len(),
            documents,
            evidence,
        })
    }

    async fn evidence_for_target(
        &self,
        task_dossier_id: Option<Uuid>,
        phase_dossier_id: Option<Uuid>,
    ) -> Result<Vec<EvidenceResponse>, shared::AppError> {
        let rows = sqlx::query(EVIDENCE_TARGET_SQL)
            .bind(task_dossier_id)
            .bind(phase_dossier_id)
            .fetch_all(&self.pool)
            .await
            .map_err(shared::AppError::database)?;
        Ok(rows.iter().map(evidence_response_from_row).collect())
    }

    async fn upsert_task_dossier_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        space_id: Uuid,
        task_key: &str,
    ) -> Result<Uuid, shared::AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO task_dossiers (id, space_id, task_key, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            ON CONFLICT (space_id, task_key)
            DO UPDATE SET updated_at = now()
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(space_id)
        .bind(task_key)
        .fetch_one(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(row.get("id"))
    }

    async fn upsert_phase_dossier_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        space_id: Uuid,
        phase_key: &str,
    ) -> Result<Uuid, shared::AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO phase_dossiers (id, space_id, phase_key, created_at, updated_at)
            VALUES ($1, $2, $3, now(), now())
            ON CONFLICT (space_id, phase_key)
            DO UPDATE SET updated_at = now()
            RETURNING id
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(space_id)
        .bind(phase_key)
        .fetch_one(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(row.get("id"))
    }

    async fn audit(
        &self,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), shared::AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, actor_id, action, entity_type, entity_id, request_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(format!("api-{}", Uuid::now_v7()))
        .execute(&self.pool)
        .await
        .map_err(shared::AppError::database)?;
        Ok(())
    }

    async fn insert_audit(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), shared::AppError> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (
                id, actor_id, action, entity_type, entity_id, request_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(actor_id)
        .bind(action)
        .bind(entity_type)
        .bind(entity_id)
        .bind(format!("api-{}", Uuid::now_v7()))
        .execute(&mut **tx)
        .await
        .map_err(shared::AppError::database)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WikiBackendPort for PostgresWikiBackend {
    async fn authenticate_access_token(&self, token: &str) -> Result<WikiClaims, shared::AppError> {
        PostgresWikiBackend::authenticate_access_token(self, token).await
    }

    async fn register(
        &self,
        body: WikiRegisterRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        PostgresWikiBackend::register(self, body).await
    }

    async fn login(&self, body: WikiLoginRequest) -> Result<WikiAuthResponse, shared::AppError> {
        PostgresWikiBackend::login(self, body).await
    }

    async fn refresh(
        &self,
        body: WikiRefreshRequest,
    ) -> Result<WikiAuthResponse, shared::AppError> {
        PostgresWikiBackend::refresh(self, body).await
    }

    async fn logout(&self, claims: &WikiClaims) -> Result<(), shared::AppError> {
        PostgresWikiBackend::logout(self, claims).await
    }

    async fn get_current_user(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserResponse, shared::AppError> {
        PostgresWikiBackend::get_current_user(self, claims).await
    }

    async fn list_users(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiUserListResponse, shared::AppError> {
        PostgresWikiBackend::list_users(self, claims).await
    }

    async fn create_user(
        &self,
        claims: &WikiClaims,
        body: WikiCreateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        PostgresWikiBackend::create_user(self, claims, body).await
    }

    async fn update_user(
        &self,
        claims: &WikiClaims,
        user_id: &str,
        body: WikiUpdateUserRequest,
    ) -> Result<WikiUserResponse, shared::AppError> {
        PostgresWikiBackend::update_user(self, claims, user_id, body).await
    }

    async fn get_settings(
        &self,
        claims: &WikiClaims,
    ) -> Result<WikiSettingsSnapshot, shared::AppError> {
        PostgresWikiBackend::get_settings(self, claims).await
    }

    async fn list_spaces(
        &self,
        claims: &WikiClaims,
    ) -> Result<SpaceListResponse, shared::AppError> {
        PostgresWikiBackend::list_spaces(self, claims).await
    }

    async fn create_space(
        &self,
        claims: &WikiClaims,
        body: CreateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::create_space(self, claims, body).await
    }

    async fn get_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::get_space(self, claims, space_key).await
    }

    async fn update_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: UpdateSpaceRequest,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::update_space(self, claims, space_key, body).await
    }

    async fn archive_space(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceResponse, shared::AppError> {
        PostgresWikiBackend::archive_space(self, claims, space_key).await
    }

    async fn list_space_members(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceMemberListResponse, shared::AppError> {
        PostgresWikiBackend::list_space_members(self, claims, space_key).await
    }

    async fn upsert_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
        body: UpsertSpaceMemberRequest,
    ) -> Result<SpaceMemberResponse, shared::AppError> {
        PostgresWikiBackend::upsert_space_member(self, claims, space_key, user_id, body).await
    }

    async fn delete_space_member(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        user_id: &str,
    ) -> Result<(), shared::AppError> {
        PostgresWikiBackend::delete_space_member(self, claims, space_key, user_id).await
    }

    async fn get_space_tree(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<SpaceTreeResponse, shared::AppError> {
        PostgresWikiBackend::get_space_tree(self, claims, space_key).await
    }

    async fn create_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        body: CreateDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::create_document(self, claims, space_key, body).await
    }

    async fn get_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::get_document(self, claims, document_id).await
    }

    async fn update_document_draft(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: UpdateDocumentDraftRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::update_document_draft(self, claims, document_id, body).await
    }

    async fn publish_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: PublishDocumentRequest,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        PostgresWikiBackend::publish_document(self, claims, document_id, body).await
    }

    async fn archive_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::archive_document(self, claims, document_id).await
    }

    async fn move_document(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        body: MoveDocumentRequest,
    ) -> Result<DocumentResponse, shared::AppError> {
        PostgresWikiBackend::move_document(self, claims, document_id, body).await
    }

    async fn list_document_revisions(
        &self,
        claims: &WikiClaims,
        document_id: &str,
    ) -> Result<DocumentRevisionListResponse, shared::AppError> {
        PostgresWikiBackend::list_document_revisions(self, claims, document_id).await
    }

    async fn get_document_revision(
        &self,
        claims: &WikiClaims,
        document_id: &str,
        revision_id: &str,
    ) -> Result<DocumentRevisionResponse, shared::AppError> {
        PostgresWikiBackend::get_document_revision(self, claims, document_id, revision_id).await
    }

    async fn list_tasks(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<TaskPageListResponse, shared::AppError> {
        PostgresWikiBackend::list_tasks(self, claims, space_key).await
    }

    async fn get_task(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<TaskPageResponse, shared::AppError> {
        PostgresWikiBackend::get_task(self, claims, space_key, task_key).await
    }

    async fn link_task_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<TaskPageResponse, shared::AppError> {
        PostgresWikiBackend::link_task_document(self, claims, space_key, task_key, body).await
    }

    async fn list_task_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        PostgresWikiBackend::list_task_documents(self, claims, space_key, task_key).await
    }

    async fn list_task_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        task_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        PostgresWikiBackend::list_task_evidence(self, claims, space_key, task_key).await
    }

    async fn list_phases(
        &self,
        claims: &WikiClaims,
        space_key: &str,
    ) -> Result<PhasePageListResponse, shared::AppError> {
        PostgresWikiBackend::list_phases(self, claims, space_key).await
    }

    async fn get_phase(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<PhasePageResponse, shared::AppError> {
        PostgresWikiBackend::get_phase(self, claims, space_key, phase_key).await
    }

    async fn link_phase_document(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
        body: LinkDocumentRequest,
    ) -> Result<PhasePageResponse, shared::AppError> {
        PostgresWikiBackend::link_phase_document(self, claims, space_key, phase_key, body).await
    }

    async fn list_phase_documents(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<DocumentListResponse, shared::AppError> {
        PostgresWikiBackend::list_phase_documents(self, claims, space_key, phase_key).await
    }

    async fn list_phase_evidence(
        &self,
        claims: &WikiClaims,
        space_key: &str,
        phase_key: &str,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        PostgresWikiBackend::list_phase_evidence(self, claims, space_key, phase_key).await
    }

    async fn create_evidence(
        &self,
        claims: &WikiClaims,
        body: CreateEvidenceRequest,
    ) -> Result<EvidenceResponse, shared::AppError> {
        PostgresWikiBackend::create_evidence(self, claims, body).await
    }

    async fn list_evidence(
        &self,
        claims: Option<&WikiClaims>,
        query: EvidenceQuery,
    ) -> Result<EvidenceListResponse, shared::AppError> {
        PostgresWikiBackend::list_evidence(self, claims, query).await
    }

    async fn get_evidence(
        &self,
        claims: &WikiClaims,
        evidence_id: &str,
    ) -> Result<EvidenceResponse, shared::AppError> {
        PostgresWikiBackend::get_evidence(self, claims, evidence_id).await
    }

    async fn upload_attachment(
        &self,
        claims: &WikiClaims,
        file_name: String,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentResponse, shared::AppError> {
        PostgresWikiBackend::upload_attachment(self, claims, file_name, content_type, bytes).await
    }

    async fn get_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentResponse, shared::AppError> {
        PostgresWikiBackend::get_attachment(self, claims, attachment_id).await
    }

    async fn download_attachment(
        &self,
        claims: &WikiClaims,
        attachment_id: &str,
    ) -> Result<AttachmentDownloadResponse, shared::AppError> {
        PostgresWikiBackend::download_attachment(self, claims, attachment_id).await
    }

    async fn list_templates(&self) -> Result<TemplateListResponse, shared::AppError> {
        PostgresWikiBackend::list_templates(self).await
    }

    async fn create_template(
        &self,
        claims: &WikiClaims,
        body: CreateTemplateRequest,
    ) -> Result<TemplateResponse, shared::AppError> {
        PostgresWikiBackend::create_template(self, claims, body).await
    }

    async fn list_audit_log(
        &self,
        claims: &WikiClaims,
    ) -> Result<AuditLogResponse, shared::AppError> {
        PostgresWikiBackend::list_audit_log(self, claims).await
    }

    async fn search(
        &self,
        claims: &WikiClaims,
        query: SearchQuery,
    ) -> Result<SearchResponse, shared::AppError> {
        PostgresWikiBackend::search(self, claims, query).await
    }
}
