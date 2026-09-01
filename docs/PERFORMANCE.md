# Performance - Wiki

## 1. Goals

- P95 API response for common reads < 200 ms at 100 RPS.
- Open document with current revision < 250 ms.
- Render 500-node space tree < 500 ms.
- Search across 100k published revisions < 500 ms in MVP PostgreSQL mode.
- Upload metadata response < 300 ms for supported file sizes.

## 2. Database

Core indexes:

- `spaces(key)` for route lookup.
- `documents(space_id, parent_id, position)` for trees.
- `documents(space_id, parent_id, slug)` unique for stable paths.
- `document_revisions(document_id, version DESC)` for history.
- `document_revisions(search_vector)` GIN for MVP search.
- `task_dossiers(space_id, task_key)` unique for task lookup.
- `phase_dossiers(space_id, phase_key)` unique for phase lookup.
- `document_task_links(task_dossier_id, document_id)` for task documents.
- `document_phase_links(phase_dossier_id, document_id)` for phase documents.
- `evidence_items(phase_dossier_id, created_at DESC)` for phase feed.
- `attachments(owner_entity_type, owner_entity_id)` for attachment listing.

Avoid N+1 by loading document metadata, current revision, author and permissions in bounded queries.

## 3. API

- Cursor pagination for search, evidence feeds and audit log.
- ETag for document view and immutable revision responses.
- Compression for JSON and rendered HTML.
- Request timeout: 30 seconds, DB query timeout: 5 seconds.
- Bulk endpoints are outside MVP.

## 4. Search

- MVP: PostgreSQL `tsvector` plus permission filtering.
- Search stays in PostgreSQL FTS for MVP.
- Search index updates during publish or through a simple internal service.

## 5. Frontend

- Route-level lazy loading.
- Virtualize long document trees, search results and evidence feeds.
- Keep document editor state local and persist drafts explicitly.
- Use TanStack Query stale times tuned per entity.

## 6. Deferred Background Jobs

- Generate file previews.
- Verify external evidence URLs.
- Rebuild search projection.
- Clean orphaned uploads.

These jobs are future work and must not be required for MVP request latency.

## 7. Monitoring

Key metrics:

- `http_request_duration_seconds`
- `db_query_duration_seconds`
- `document_open_duration_seconds`
- `search_query_duration_seconds`
- `storage_operation_duration_seconds`
- `frontend_query_cache_refetch_total`

## 8. Load Testing

Scenarios:

- login and open dashboard;
- open space tree;
- open document and revision history;
- search by text and task key;
- attach evidence file;
- open phase dossier.

## 9. References

- `docs/CACHING.md`
- `docs/DATABASE_INDEXES.md`
- `docs/MONITORING.md`
