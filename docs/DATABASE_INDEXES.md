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
| `idempotency_records` | `idempotency_records_actor_key_idx` | unique btree | Retry lookup by actor and idempotency key |
| `idempotency_records` | `idempotency_records_expires_idx` | btree | TTL cleanup |
| `audit_log` | `audit_entity_idx` | btree | Entity audit trail |
| `audit_log` | `audit_actor_time_idx` | btree | Actor audit trail |

## 3. Composite Strategy

- Document tree: `(space_id, parent_id, position, title)` and separate unique indexes for root and child slugs because PostgreSQL treats `NULL` parent values as distinct.
- Current document by route: `(space_id, slug)` for top-level pages, plus parent-aware lookup for nested pages.
- Task dossier: `(space_id, task_key)` must be unique.
- Phase dossier: `(space_id, phase_key)` must be unique.
- Evidence feed: `(phase_dossier_id, created_at DESC)` and `(task_dossier_id, created_at DESC)`.
- Idempotency replay: `(actor_id, idempotency_key)` is unique so retry lookup is one indexed row; `expires_at` supports bounded cleanup.
- Archive filtering: partial indexes with `WHERE archived_at IS NULL`.

## 4. Full-text Search

```sql
CREATE INDEX document_revisions_search_idx
  ON document_revisions USING GIN (search_vector);

```

Document search uses PostgreSQL `websearch_to_tsquery('simple', q)` against `document_revisions.search_vector`. Title matches are weighted above body text in the generated vector, and the document query orders matching rows by `ts_rank_cd(...) DESC, updated_at DESC` before the MVP response limit is applied.

The release smoke must verify the normal filtered search shape with `EXPLAIN`: `q`, `space`, `task_key`, `phase_key`, `document_type` and `archived_at IS NULL`. The expected plan includes `document_revisions_search_idx`; this is covered by the env-gated `wiki_postgres_search_uses_fts_index_when_database_available` API test.

Evidence search remains a bounded MVP lookup over `evidence_items` title and URL plus indexed space/task/phase filters. Do not add a separate search service, trigram search, OCR or binary attachment indexing without a new approved requirement.

## 5. Migration Rules

- Use `CREATE INDEX CONCURRENTLY` for production migrations that touch large tables.
- Add indexes together with the endpoint/query that needs them.
- Use `EXPLAIN (ANALYZE, BUFFERS)` for search, tree and dossier feeds before release.
- Remove obsolete indexes only after confirming they are unused through `pg_stat_user_indexes`.

## 6. References

- `docs/DATA_MODEL.md`
- `docs/PERFORMANCE.md`
- `docs/MIGRATIONS.md`
