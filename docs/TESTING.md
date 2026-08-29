# Testing Strategy - Wiki

## 1. Principles

- Tests cover meaningful user paths and domain invariants.
- Backend tests prefer real PostgreSQL for repositories and focused mocks for services.
- Frontend tests use Vitest and Testing Library; E2E uses Playwright.
- UI changes should be checked on mobile and desktop widths.
- Integration tests must cover auth, permissions and data isolation between spaces.

## 2. Backend Tests

### Unit

- `domain/` - document, revision, space key and dossier invariants.
- `app/` - services for publish, archive, evidence attach, permission checks.
- `shared/` - config/env parsing, ID helpers, error mapping.

### Integration

- Repository tests against PostgreSQL.
- API tests for spaces, documents, revisions, task dossiers, phase dossiers, evidence and attachments.
- Failing repository tests for 500/error envelope behavior.
- External link ingestion tests use service-level mocks only after the base Wiki domain exists.

### Coverage Priorities

| Area | Target |
|---|---|
| Domain invariants | high |
| Permission checks | high |
| Publish/revision flow | high |
| Evidence ingestion | high |
| Search indexing | medium |
| Admin settings | medium |

## 3. Frontend Tests

Unit/component tests:

- app shell navigation, account menu and logout;
- dashboard;
- document editor;
- revision history;
- evidence feed;
- task dossier page;
- phase dossier page;
- search empty/result/error states.

E2E smoke:

- login;
- open dashboard;
- create draft;
- publish document;
- attach evidence;
- search by task key;
- open phase dossier.

## 4. Commands

```bash
cd backend
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace -- --test-threads=1

cd frontend
pnpm typecheck
pnpm test
pnpm test:e2e
```

## 5. Fixtures

Baseline test fixtures:

- system admin;
- space `ENG`;
- document with two revisions;
- task dossier `SDLC-42`;
- phase dossier `implementation`;
- linked evidence artifact;
- viewer/editor users for permission checks.

## 6. Merge Checklist

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets` clean
- [ ] `cargo test --workspace` green
- [ ] `pnpm typecheck` clean
- [ ] `pnpm test` green
- [ ] `pnpm build` green
- [ ] Playwright critical path green
- [ ] Documentation updated

## 7. References

- `docs/ARCHITECTURE.md`
- `docs/API.md`
- `docs/DEPLOYMENT.md`
- `justfile`
