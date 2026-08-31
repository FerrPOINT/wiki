# Database Indexes - Wiki

## 1. Overview

Индексы PostgreSQL должны покрывать частые операции Wiki: открытие документа, построение дерева space, поиск, открытие task/phase dossier, listing evidence и audit.

## 2. Per-table Indexes

| Table | Index | Type | Purpose |
|---|---|---|---|
| `users` | `users_email_idx` | unique btree | Login by email |
| `users` | `users_active_idx` | partial btree | Active account listing |
| `spaces` | `spaces_key_idx` | unique btree | Space routing |
| `spaces` | `spaces_owner_idx` | btree | Owner spaces |
| `space_members` | `space_members_user_idx` | btree | Spaces available to user |
| `space_members` | primary key `(space_id, user_id)` | unique btree | Permission lookup |
| `documents` | `documents_root_slug_idx` / `documents_child_slug_idx` | unique btree | Tree uniqueness |
| `documents` | `documents_space_parent_position_idx` | btree | Tree listing |
| `documents` | `documents_current_revision_idx` | btree | Fast open |
| `documents` | `documents_live_idx` | partial btree | Live pages by space |
| `document_revisions` | `document_revisions_document_version_idx` | unique btree | Revision history |
| `document_revisions` | `document_revisions_search_idx` | GIN tsvector | Full-text search |
| `document_drafts` | `document_drafts_author_idx` | btree | My drafts |
| `task_dossiers` | `task_dossiers_space_key_idx` | unique btree | Task key lookup |
| `task_dossiers` | `task_dossiers_space_idx` | btree | Space task list |
| `phase_dossiers` | `phase_dossiers_space_key_idx` | unique btree | Phase key lookup |
| `document_task_links` | `document_task_links_task_idx` | btree | Task documents |
| `document_phase_links` | `document_phase_links_phase_idx` | btree | Phase documents |
| `evidence_items` | `evidence_task_idx` | btree | Task evidence |
| `evidence_items` | `evidence_phase_idx` | btree | Phase evidence |
| `evidence_items` | `evidence_document_idx` | btree | Document evidence |
| `attachments` | `attachments_owner_idx` | btree | Attachments by owner |
| `attachments` | `attachments_checksum_idx` | btree | Dedup/integrity |
| `attachments` | `attachments_staged_idx` | partial btree | Cleanup staged uploads |
| `audit_log` | `audit_entity_idx` | btree | Entity audit trail |
| `audit_log` | `audit_actor_time_idx` | btree | Actor audit trail |

## 3. Composite Strategy

- Document tree: `(space_id, parent_id, position, title)` and separate unique indexes for root and child slugs because PostgreSQL treats `NULL` parent values as distinct.
- Current document by route: `(space_id, slug)` for top-level pages, plus parent-aware lookup for nested pages.
- Task dossier: `(space_id, task_key)` must be unique.
- Phase dossier: `(space_id, phase_key)` must be unique.
- Evidence feed: `(phase_dossier_id, created_at DESC)` and `(task_dossier_id, created_at DESC)`.
- Archive filtering: partial indexes with `WHERE archived_at IS NULL`.

## 4. Full-text Search

```sql
CREATE INDEX document_revisions_search_idx
  ON document_revisions USING GIN (search_vector);

```

Keep MVP search inside PostgreSQL. A separate search service requires a new approved requirement.

## 5. Migration Rules

- Use `CREATE INDEX CONCURRENTLY` for production migrations that touch large tables.
- Add indexes together with the endpoint/query that needs them.
- Use `EXPLAIN (ANALYZE, BUFFERS)` for search, tree and dossier feeds before release.
- Remove obsolete indexes only after confirming they are unused through `pg_stat_user_indexes`.

## 6. References

- `docs/DATA_MODEL.md`
- `docs/PERFORMANCE.md`
- `docs/MIGRATIONS.md`
