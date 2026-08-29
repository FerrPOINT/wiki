# Pagination & Bulk Operations - Wiki

## 1. Overview

Wiki uses pagination for document trees, search results, revision history, evidence feeds, comments and audit log.

## 2. Cursor Pagination

Default for large or frequently changing lists:

```http
GET /api/v1/search?q=release&cursor=eyJpZCI6IjAxOGYifQ&limit=20
```

```json
{
  "data": [],
  "next_cursor": "eyJpZCI6IjAxOGYifQ",
  "has_more": true
}
```

Rules:

- Cursor is base64url(JSON).
- Cursor contains the last sort value and ID.
- Cursor is opaque for clients.
- No arbitrary page jump.

## 3. Offset Pagination

Allowed for small admin dictionaries:

```http
GET /api/v1/spaces?limit=20&offset=0
```

```json
{
  "data": [],
  "total": 42,
  "limit": 20,
  "offset": 0
}
```

## 4. Keyset Pagination

Used for ordered trees and comments:

```http
GET /api/v1/spaces/ENG/tree?after_id=018f...&limit=50
```

Sort by stable tuple: `(position, id)` or `(created_at, id)`.

## 5. Limits

| Resource | Max Limit | Default |
|---|---|---|
| spaces | 100 | 20 |
| documents | 100 | 30 |
| document revisions | 100 | 20 |
| search results | 100 | 20 |
| evidence | 100 | 30 |
| comments | 50 | 20 |
| audit log | 200 | 50 |
| attachments | 50 | 20 |

## 6. Bulk Operations

Bulk endpoints are reserved for admin import/export after the base domain is implemented.

### Bulk Evidence Ingest

```json
POST /api/v1/evidence/bulk
Idempotency-Key: 018f...

{
  "items": [
    {
      "task_dossier_id": "018f...",
      "phase_key": "testing",
      "source_type": "ci_job",
      "source_ref": "backend-tests#1842"
    }
  ]
}
```

### Bulk Document Import

```json
POST /api/v1/spaces/ENG/documents/import
Idempotency-Key: 018f...

{
  "documents": [
    {
      "path": "requirements/wiki.md",
      "title": "Wiki Requirements",
      "content_markdown": "# Wiki"
    }
  ]
}
```

## 7. Response Format

```json
{
  "processed": 100,
  "succeeded": 98,
  "failed": 2,
  "errors": [
    { "id": "018f...", "error": "permission_denied" }
  ]
}
```

## 8. Request Size Limits

| Type | Limit |
|---|---|
| JSON body | 10 MiB |
| Bulk items | 100 |
| Single attachment | 50 MiB |
| Query params length | 4096 chars |

## 9. Deep Linking

- Search filters are encoded as URL query.
- Document location is encoded by document ID or stable path.
- Task links preserve external task key.
- Phase links preserve workflow run and phase key where possible.

## 10. References

- `docs/API.md`
- `docs/PERFORMANCE.md`
- `docs/TESTING.md`
