# Data Lifecycle Contract - Wiki

## 1. Lifecycle States

| Entity | States |
|---|---|
| Space | active, archived |
| Document | draft, published, archived |
| Revision | immutable |
| Evidence | active, archived |
| Attachment | active, quarantined, archived, deleted |

## 2. Retention

- Published revisions are retained by default.
- Evidence is retained at least as long as linked task/phase audit needs.
- Hard deletion requires audit and retention policy.

## 3. Restore

Restore validates PostgreSQL metadata and object storage keys. Missing objects are reported, not silently ignored.

## 4. Deletion Rules

| Entity | Soft Delete | Hard Delete |
|---|---|---|
| Space | archive | system admin with retention check |
| Document | archive | only after all revisions pass retention policy |
| Revision | no | legal/admin policy only |
| Evidence | archive | after linked audit retention expires |
| Attachment | archive/quarantine | after checksum and backup policy |

Published revisions are immutable business records. User-visible correction creates a newer revision instead of rewriting history.

## 5. Backup Sets

- PostgreSQL dump for relational metadata.
- Object storage backup for attachments.
- Configuration backup without plaintext secrets.
- Git repository backup for docs, migrations and OpenAPI.

## 6. Acceptance Criteria

- Restore drill can recover document metadata and attachment objects.
- Search index can be rebuilt after restore.
- Archive operations are reversible where policy allows.
- Hard delete creates audit entry with reason and actor.
