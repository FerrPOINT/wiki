# File Storage - Wiki

## 1. Overview

Wiki хранит вложения документов, материалы evidence и изображения. Backend абстрагирует хранилище через `FileStore` trait и может работать с локальной файловой системой, S3-compatible storage или MinIO.

## 2. Supported Backends

| Backend | Use Case |
|---|---|
| `filesystem` | Local dev, single-node deploy |
| `s3` | Production, scalable storage |
| `minio` | Self-hosted S3-compatible setup |

## 3. Configuration

```env
WIKI_FILE_STORAGE_BACKEND=s3
WIKI_FILE_STORAGE_BUCKET=wiki-artifacts
WIKI_FILE_STORAGE_REGION=ru-central1
WIKI_FILE_STORAGE_ENDPOINT=https://s3.example.com
WIKI_FILE_STORAGE_ACCESS_KEY=...
WIKI_FILE_STORAGE_SECRET_KEY=...
WIKI_FILE_STORAGE_PATH=/var/lib/wiki/uploads
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

1. Client uploads a file to `/api/v1/attachments` with target owner fields.
2. Server validates size, MIME, filename and owner permissions.
3. Server creates attachment UUIDv7 and storage key.
4. File is written to object storage.
5. Attachment metadata is saved in PostgreSQL.
6. Optional maintenance jobs may generate previews and thumbnails after the base storage flow is stable.

## 6. Storage Path Schema

```text
attachments/
  documents/{document_id}/{attachment_id}/{sanitized_filename}
  revisions/{revision_id}/{attachment_id}/{sanitized_filename}
  task-dossiers/{task_dossier_id}/{attachment_id}/{sanitized_filename}
  phase-dossiers/{phase_dossier_id}/{attachment_id}/{sanitized_filename}
  evidence/{evidence_id}/{attachment_id}/{sanitized_filename}
previews/
  attachments/{attachment_id}.webp
```

## 7. Attachment Entity

```rust
pub struct Attachment {
    pub id: Uuid,
    pub owner_entity_type: AttachmentOwnerType,
    pub owner_entity_id: Uuid,
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
| `POST` | `/api/v1/attachments` | Upload attachment for document/dossier/evidence |
| `GET` | `/api/v1/attachments/{id}` | Download attachment |
| `GET` | `/api/v1/attachments/{id}/preview` | Download preview |
| `DELETE` | `/api/v1/attachments/{id}` | Delete attachment metadata and storage object |
| `POST` | `/api/v1/evidence/{id}/attachments` | Attach file directly to evidence |
| `GET` | `/api/v1/documents/{id}/attachments` | List document attachments |

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
