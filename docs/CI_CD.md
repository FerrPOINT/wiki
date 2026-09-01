# CI/CD — Wiki

## 1. Overview

Current Wiki CI is the pre-development quality gate for the MVP baseline. It verifies Rust backend, OpenAPI parity, SQLx migrations, dependency audit, React frontend and Chromium smoke E2E.

Release packaging, registry publishing, multi-browser E2E, mutation testing and production deployment automation are deferred release-hardening items.

## 2. Platform

- **GitHub Actions** is the active CI provider.
- **PostgreSQL 17.6-alpine** is used for migration checks and Docker-backed E2E.
- **pnpm 10** and **Node.js 22** are used for frontend checks.
- **Rust stable** is used for backend checks.

## 3. Active Workflow

The active workflow is `.github/workflows/ci.yml` and runs on pushes and pull requests to `main`.

| Job | Purpose | Blocking |
| --- | ------- | -------- |
| `backend` | Rust formatting, clippy and serial workspace tests | Yes |
| `openapi-check` | Regenerate OpenAPI from Rust handlers and compare with committed spec | Yes |
| `migrations` | Apply SQLx migrations to a clean PostgreSQL database and print status | Yes |
| `coverage` | Run backend tests through `cargo llvm-cov` and enforce 60% summary coverage | Yes |
| `audit` | Run `cargo audit` with the documented SQLx optional MySQL/RSA ignore | Yes |
| `frontend` | Generate API DTOs, typecheck, run Vitest, lint, format-check and build Vite app | Yes |
| `e2e` | Build frontend, start Docker PostgreSQL + backend, then run Chromium Playwright smoke | Yes |

## 4. Backend Gate

The backend job runs from `backend/`:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --test-threads=1
```

Serial tests are intentional because current integration coverage shares in-memory/runtime state in several suites. The documented local readiness gate uses the same serial mode.

## 5. OpenAPI Gate

The OpenAPI job regenerates `backend/openapi/openapi.json` from Rust route/DTO definitions and compares it with the repository-level contract:

```bash
mkdir -p openapi
cargo run -p api --bin openapi-gen -- openapi/openapi.json
diff -u ../openapi/openapi.json openapi/openapi.json
```

Any endpoint, DTO or schema change must update the generated root `openapi/openapi.json` in the same change.

## 6. Migration Gate

The migration job starts a clean PostgreSQL service and runs:

```bash
cargo run -p migration -- up
cargo run -p migration -- status
```

The runtime migrator loads migrations from `WIKI_MIGRATIONS_DIR` when set, otherwise from the repository `backend/migrations` directory. Docker images copy this directory into `/app/migrations`.

## 7. Coverage Gate

The coverage job uses `cargo llvm-cov`:

```bash
cargo llvm-cov --workspace --summary-only -- --test-threads=1
```

The active CI threshold is **60% total backend coverage**. Higher layer-specific targets remain release-hardening goals, not MVP pre-development blockers.

## 8. Audit Gate

The audit job runs:

```bash
cargo audit --ignore RUSTSEC-2023-0071
```

`RUSTSEC-2023-0071` is ignored because it is reported through optional SQLx MySQL/RSA packages retained in `Cargo.lock`; Wiki enables PostgreSQL-only SQLx features. Other non-ignored RustSec vulnerabilities block CI. The `h2` advisory is resolved by using `h2` `0.4.16`.

Frontend dependency audit is a release-hardening backlog item until the dependency policy is finalized.

## 9. Frontend Gate

The frontend job runs from `frontend/`:

```bash
pnpm install
pnpm generate:api
pnpm typecheck
pnpm test -- --run
pnpm lint
pnpm format:check
pnpm build
```

`pnpm generate:api` must keep `frontend/src/api/generated.ts` aligned with `openapi/openapi.json`.

## 10. E2E Gate

The E2E job:

1. Installs frontend dependencies.
2. Builds the Vite app.
3. Starts Docker Compose services `postgres` and `backend`.
4. Checks `http://localhost:3456/api/v1/health`.
5. Installs Chromium dependencies.
6. Runs `pnpm exec playwright test e2e/smoke.spec.ts --project=chromium`.
7. Tears down Docker volumes with `docker compose down -v`.

The frontend Playwright config starts a local `vite preview` server unless `PLAYWRIGHT_BASE_URL` is provided. `VITE_API_BASE_URL` points to the Docker backend.

## 11. Local Readiness Commands

```bash
# Backend
cd backend
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace -- --test-threads=1
cargo clippy --workspace --all-targets -- -D warnings
cargo audit --ignore RUSTSEC-2023-0071
```

```bash
# Frontend
cd frontend
pnpm typecheck
pnpm test -- --run
pnpm lint
pnpm format:check
pnpm build
pnpm test:e2e -- --project=chromium
pnpm shoot:evidence
```

```powershell
# PostgreSQL smoke on Windows host
pwsh -File scripts/postgres-smoke.ps1
pwsh -File scripts/postgres-smoke-wsl.ps1
```

Docker smoke is preferred when Docker Desktop is available. On this Windows host, the WSL PostgreSQL smoke is an accepted fallback and uses an isolated temporary database.

## 12. Release-Hardening Backlog

- Docker image publishing to GHCR.
- Branch protection policy enforcement.
- `pnpm audit` policy and lockfile vulnerability threshold.
- Trivy/container image scanning.
- Multi-browser Playwright matrix.
- Scheduled backup/restore drill.
- Production TLS/CORS/security-header verification.

## References

- `docs/TESTING.md`
- `docs/DEPLOYMENT.md`
- `docs/MIGRATIONS.md`
- `docs/SECURITY.md`
- `docs/CURRENT_STATE.md`
