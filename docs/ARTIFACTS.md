# Artifacts - Wiki

## 1. Definition

Artifacts are files or external URLs attached as evidence: CI logs, build archives, screenshots, QA files and release bundles.

## 2. Artifact Metadata

| Field | Description |
|---|---|
| `id` | Attachment/evidence ID |
| `source_type` | `ci_job`, `artifact`, `external_url`, `manual_file` |
| `source_ref` | External reference |
| `storage_key` | Object storage key |
| `checksum` | SHA-256 when available |
| `size_bytes` | File size |
| `content_type` | MIME type |
| `collected_at` | Capture time |

## 3. Lifecycle

- Upload.
- Validate.
- Optional scan.
- Attach to evidence.
- Index metadata.
- Retain/archive/delete by policy.

## 4. UI

Artifacts appear on evidence and phase dossier pages with status, source, checksum and download/open action.
