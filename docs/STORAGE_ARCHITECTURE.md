# Storage Architecture - Wiki

## 1. Data Classes

| Class | Storage |
|---|---|
| Structured metadata | PostgreSQL |
| Draft/revision text | PostgreSQL |
| Attachments | Local filesystem behind storage trait |
| Evidence artifacts | Local file attachment or external URL |
| Search index | PostgreSQL FTS |
| Audit/events | PostgreSQL append-only tables |

## 2. Object Keys

Object keys include owner type and owner ID. Keys are generated server-side and never trust client paths.

```text
attachments/documents/{document_id}/{attachment_id}/{filename}
attachments/evidence/{evidence_id}/{attachment_id}/{filename}
```

## 3. Integrity

- Store SHA-256 checksum.
- Validate size after upload.
- Verify object existence during restore.
- Do not delete evidence artifacts without retention/audit.

## 4. References

- `docs/STORAGE.md`
- `docs/contracts/DATA_LIFECYCLE.md`
