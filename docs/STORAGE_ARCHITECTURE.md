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

Object keys are generated server-side from the attachment ID and sanitized filename. Clients never send or receive raw storage paths.

```text
attachments/{attachment_id}/{filename}
```

Owner-specific object key layouts can be introduced later if document or revision attachments become part of the public API.

## 3. Integrity

- Store SHA-256 checksum.
- Validate size after upload.
- Verify object existence during restore.
- Do not delete evidence artifacts without retention/audit.

## 4. References

- `docs/STORAGE.md`
- `docs/contracts/DATA_LIFECYCLE.md`
