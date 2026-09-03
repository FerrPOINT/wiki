# Pagination - Wiki

## 1. Overview

Wiki MVP uses bounded `limit` windows for large API reads where the public route already exposes query parameters: search results, revision history, evidence feeds and audit log.

Cursor, offset and keyset pagination below define the future standard for larger datasets. They are not active MVP route promises until the matching OpenAPI query parameters exist.

## 2. Cursor Pagination

Future default for large or frequently changing lists:

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

Future option for small admin dictionaries:

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

Future option for ordered trees and append-only feeds:

```http
GET /api/v1/spaces/ENG/tree?after_id=018f...&limit=50
```

Sort by stable tuple: `(position, id)` or `(created_at, id)`.

## 5. Limits

| MVP resource | Max Limit | Default |
|---|---|---|
| document revisions | 100 | 20 |
| search results | 100 | 20 |
| evidence | 100 | 30 |
| audit log | 200 | 50 |

Future list resources such as spaces, user dictionaries, document tree windows and attachment collections must define their own OpenAPI parameters before being treated as paginated MVP endpoints.

## 6. Future Bulk Operations

Bulk endpoints are reserved for admin import/export after the base domain is implemented.

No bulk endpoints are part of MVP. Future bulk evidence ingest or document import must define route shape, max item count, idempotency, partial failure response and permission model before implementation.

## 7. Future Bulk Response Format

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
| Future bulk items | 100 |
| Single attachment | 25 MiB default, configurable |
| Query params length | 4096 chars |

## 9. Deep Linking

- Search filters are encoded as URL query.
- Document location is encoded by document ID or stable path.
- Task links preserve external task key.
- Phase links preserve external phase key.

## 10. References

- `docs/API.md`
- `docs/PERFORMANCE.md`
- `docs/TESTING.md`
