# File Storage - Wiki

## 1. Overview

Wiki хранит вложения документов и материалы evidence. Backend абстрагирует хранилище через `FileStore` trait. MVP начинает с локальной файловой системы; S3-compatible storage или MinIO остаются расширением за тем же trait.

## 2. Supported Backends

| Backend | Use Case |
|---|---|
| `filesystem` | MVP local dev and single-node deploy |
| `s3` | Future production scalable storage |
| `minio` | Future self-hosted S3-compatible setup |

## 3. Configuration

```env
WIKI_STORAGE__DIR=/var/lib/wiki/uploads
WIKI_STORAGE__MAX_UPLOAD_BYTES=26214400
```

## 4. FileStore Trait

```rust
#[async_trait]
pub trait FileStore: Send + Sync {
    async fn put(&self, key: &str, content: Bytes, content_type: &str) -> Result<(), FileStoreError>;
    async fn get(&self, key: &str) -> Result<Bytes, FileStoreError>;
    async fn delete(&self, key: &str) -> Result<(), FileStoreError>;
    async fn exists(&self, key: &str) -> Result<bool, FileStoreError>;
    fn public_url(&self, key: &str) -> Option<String>;
}
```

## 5. Upload Flow

1. Client uploads a file to `/api/v1/attachments`.
2. Server validates size, MIME and filename.
3. Server creates attachment UUIDv7 and staged storage key.
4. File is written to object storage.
5. Attachment metadata is saved in PostgreSQL without owner fields yet.
6. When `uploaded_file` evidence is created, backend claims the attachment in the same transaction by setting `space_id`, `owner_entity_type = evidence` and `owner_entity_id`.
7. Optional maintenance jobs may clean expired staged files after the base storage flow is stable.

## 6. Storage Path Schema

```text
attachments/
  documents/{document_id}/{attachment_id}/{sanitized_filename}
  revisions/{revision_id}/{attachment_id}/{sanitized_filename}
  evidence/{evidence_id}/{attachment_id}/{sanitized_filename}
```

## 7. Attachment Entity

```rust
pub struct Attachment {
    pub id: Uuid,
    pub space_id: Option<Uuid>,
    pub owner_entity_type: Option<AttachmentOwnerType>,
    pub owner_entity_id: Option<Uuid>,
    pub file_name: String,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub uploaded_by: Uuid,
    pub uploaded_at: DateTime<Utc>,
}
```

## 8. Security

- Reject path traversal, null bytes and control characters in file names.
- Do not trust client-provided MIME type; detect by magic bytes where possible.
- Block executable/script uploads by default.
- Optionally scan with ClamAV before publishing downloads to other users.
- Store private files behind authenticated download endpoints or signed URLs.
- Never expose raw filesystem paths or bucket credentials.

## 9. API Endpoints

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/api/v1/attachments` | Upload attachment metadata and bytes |
| `GET` | `/api/v1/attachments/{attachment_id}` | Read attachment metadata |
| `GET` | `/api/v1/attachments/{attachment_id}/download` | Download attachment bytes |

Attachment delete, preview generation and document attachment listing are deferred until after the base document/evidence lifecycle is implemented.

## 10. Quotas

| Entity | Default Limit |
|---|---|
| Per attachment | 50 MiB |
| Per document | 500 MiB |
| Per task dossier | 2 GiB |
| Per space | 50 GiB |

## 11. Backup

- S3/MinIO: bucket versioning and lifecycle policies.
- Filesystem: backup volume plus checksum validation.
- Restore must validate that every `attachments.storage_key` exists or report missing artifacts.

## 12. References

- `docs/DATA_MODEL.md`
- `docs/SECURITY.md`
- `docs/BACKUP_RESTORE.md`
