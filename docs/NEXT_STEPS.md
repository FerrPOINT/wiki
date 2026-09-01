# Wiki MVP Remaining Work

## 0. Current Baseline

Wiki is now a clean MVP-oriented application rather than a task-tracker variant:

- public API/OpenAPI exposes only Wiki MVP endpoints;
- UI and CLI are ordinary clients of the same `/api/v1`;
- frontend routes are limited to the approved MVP page set;
- non-MVP `/integrations`, `/reports` and `/notifications` screens are removed;
- copied tracker-only backend modules and old dependencies are removed;
- production runtime is PostgreSQL-backed through SQLx and `WikiBackendPort`;
- memory backend is explicit test/dev composition;
- SQLx migrations in `backend/migrations` are the canonical schema source;
- docs, README screenshots and screenshot manifest describe the current MVP scope.

The remaining work is hardening and release readiness, not expansion of product scope.

## 1. PostgreSQL Runtime Smoke

- Keep `pwsh -File scripts/postgres-smoke.ps1` as the Docker-backed release smoke on hosts where Docker Desktop is running.
- Use `pwsh -File scripts/postgres-smoke-wsl.ps1` on hosts with a local WSL PostgreSQL service when Docker Desktop is unavailable.
- The runner starts disposable `backend/docker-compose.test.yml` Postgres on `127.0.0.1:3458`, sets `WIKI_TEST_DATABASE_URL=postgres://wiki@127.0.0.1:3458/wiki_test` and runs `cargo test -p api wiki_postgres_ -- --test-threads=1 --nocapture`.
- The WSL runner creates a temporary isolated role/database, sets `WIKI_TEST_DATABASE_URL` to that database, runs the same filtered suite and drops only those temporary objects on exit.
- The filtered suite applies SQLx migrations from an empty database, verifies disabled public registration, checks persistence across router/backend rebuilds and verifies membership removal revokes space/document/search access.
- The suite also verifies MVP full-text search with `space`, `task_key`, `phase_key` and `document_type` filters and asserts the PostgreSQL plan uses `document_revisions_search_idx`.
- Save the successful smoke output with release evidence for the chosen runner.

## 2. Repository And API Hardening

- Keep focused PostgreSQL-backed tests for viewer/editor/space-admin/global-admin boundary combinations, and extend them when new endpoints are added.
- Keep negative tests for archived spaces/documents across document, evidence and task/phase link write commands.
- Keep attachment tests covering missing bytes, runtime size limits, unsafe names, unsafe storage keys, reused staged uploads and owner-space mismatch.
- Keep audit writes in the same transaction as the command that caused them.
- Keep route handlers behind app use cases/repository ports; handlers should not know concrete SQL/storage details.

## 3. Search

- Keep document search on PostgreSQL `tsvector`/GIN for MVP.
- Keep title/body weighting in the generated vector and SQL ordering; do not expose search score in the public API unless the product requirement changes.
- Run and archive the env-gated `EXPLAIN` evidence from `wiki_postgres_search_uses_fts_index_when_database_available`.
- Decide whether to move from `simple` to a Russian/English-aware text search configuration only after representative Wiki content is available.
- Keep search results permission-filtered and bounded.

## 4. CLI Parity

- Keep `backend/cli/src/main.rs` aligned with `docs/CLI.md`.
- Keep command groups matched to public API groups: auth, user, space, doc, task, phase, evidence, attachment, template, audit, search and settings.
- Keep JSON as the default output and non-zero exit code for API errors.
- Preserve idempotency key behavior for write commands.

## 5. Frontend Integration

- Keep visible UI text Russian by default while technical IDs/routes stay English.
- Keep component tests aligned with visible MVP states: loading, empty, validation error, permission denied and successful mutation.
- Extract repeated document editor, tree, revision panel and evidence feed pieces into reusable widgets/features once the page behavior stops moving.
- Regenerate screenshots after any UI or route change.

## 6. Documentation

- Update API docs after route or DTO changes.
- Update data model docs after migration changes.
- Keep deferred reports, notifications, webhooks, imports, approvals and real-time collaboration documented only as future/reference scope.
- Keep docs free of unresolved markers and one-screen placeholder pages.
- Preserve CI/CD-style documentation filename parity unless a Wiki-specific document intentionally differs.

## 7. Release Readiness

- Run backend final suite: `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace -- --test-threads=1 --nocapture`, `cargo clippy --workspace --all-targets -- -D warnings`.
- Run frontend final suite: `npm run typecheck`, `npm run test`, `npm run lint`, `npm run format:check`, `npm run build`, `npm run test:e2e -- --project=chromium`.
- Run screenshot script against `vite preview` and verify README/manifest references.
- Regenerate `openapi/openapi.json` after API changes and verify it has only MVP paths.
- Fix local Windows Rust toolchain by installing MSVC Build Tools, or keep backend verification documented as WSL-based.

## 8. Deferred After MVP

- Reports UI and report projections.
- Notification center, unread counters and delivery channels.
- Webhook ingestion and outbound delivery.
- Import/export bundles.
- Real-time collaborative editing.
- Comments, mentions and approval chains.
- OCR and binary attachment indexing.
