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

- Run a fresh disposable database with `WIKI_TEST_DATABASE_URL`.
- Apply SQLx migrations from an empty database.
- Verify production backend construction through `server`/`infra`.
- Verify disabled public registration returns the expected API error.
- Verify documents, revisions, dossiers, evidence, attachments, templates and audit persist across router/backend rebuilds.
- Verify membership removal revokes access to spaces, documents, evidence, attachment downloads and search results.

## 2. Repository And API Hardening

- Add focused PostgreSQL-backed tests for viewer/editor/admin boundary combinations.
- Add negative tests for archived spaces/documents across all write commands.
- Expand attachment tests for missing bytes, unsafe storage keys, reused staged uploads and owner-space mismatch.
- Keep audit writes in the same transaction as the command that caused them.
- Keep route handlers behind app use cases/repository ports; handlers should not know concrete SQL/storage details.

## 3. Search

- Tune PostgreSQL full-text search ranking.
- Record `EXPLAIN` evidence for expected MVP search filters: q, space, task key, phase key and document type.
- Decide and document the default language configuration for Russian/English content.
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
