<p align="center">
  <img src="docs/assets/wiki-readme-banner.svg" alt="Base Wiki - knowledge, revisions, evidence and audit" />
</p>

<p align="center">
  <a href="#capabilities"><img src="https://img.shields.io/badge/Capabilities-312e81?style=for-the-badge" alt="Capabilities" /></a>
  <a href="#quick-start"><img src="https://img.shields.io/badge/Quick_Start-4c1d95?style=for-the-badge" alt="Quick start" /></a>
  <a href="#visual-proof"><img src="https://img.shields.io/badge/Visual_Proof-075985?style=for-the-badge" alt="Visual proof" /></a>
  <a href="#safety"><img src="https://img.shields.io/badge/Safety-155e75?style=for-the-badge" alt="Safety" /></a>
  <a href="#quality"><img src="https://img.shields.io/badge/Quality-334155?style=for-the-badge" alt="Quality" /></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Axum-Rest_API-312e81?style=flat-square" alt="Axum REST API" />
  <img src="https://img.shields.io/badge/PostgreSQL-17-4169e1?style=flat-square&logo=postgresql&logoColor=white" alt="PostgreSQL 17" />
  <img src="https://img.shields.io/badge/React-19-38bdf8?style=flat-square&logo=react&logoColor=0f172a" alt="React 19" />
  <img src="https://img.shields.io/badge/CI-.github%2Fworkflows%2Fci.yml-15803d?style=flat-square" alt="Repository CI" />
</p>

> **Base Wiki** is a self-hosted knowledge hub for documents, immutable published revisions, evidence and audit. It links knowledge to external task and phase keys without owning those workflows, Git hosting or CI execution.

<a name="overview"></a>
## Overview

Wiki has one public REST API and two official clients: the React interface and the HTTP-only `wiki` CLI. The current MVP is intentionally bounded: it is a knowledge system, not a replacement for a task tracker, workflow engine or Forge CI/CD.

| Surface | Current behavior | Boundary |
|---|---|---|
| Spaces | Create, update, archive and manage membership; each space owns its page tree. | Archived spaces reject document, evidence and dossier-link writes. |
| Documents | Markdown drafts, publish, archive, move and immutable revision history. | A stale `base_revision_id` is rejected rather than overwriting a newer revision. |
| Evidence | Attach links or uploaded files to documents, external task keys or phase keys. | Local attachment storage; binary OCR/indexing is deferred. |
| Discovery | Search, templates, task/phase dossiers and bounded audit reads. | Wiki stores links to external work; it does not mutate external tasks or phases. |
| Interfaces | React UI, CLI, OpenAPI contract, health/readiness and Prometheus metrics. | UI and CLI use the same `/api/v1` contract. |

`Current` means implemented in source and covered by the repository's tests or runtime checks. The exhaustive cut is [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md); scope and deferred capabilities are in [docs/PRODUCT_REQUIREMENTS.md](docs/PRODUCT_REQUIREMENTS.md).

<a name="capabilities"></a>
## Capabilities

- **Knowledge lifecycle.** Create spaces and documents, edit Markdown drafts, publish immutable revisions, archive pages and move them inside a space tree.
- **Evidence without workflow ownership.** Link URL/file evidence to a document, task key or phase key, then retrieve metadata and authorized attachment downloads through the public API.
- **Access and accountability.** User, global-role and space-membership checks guard content; protected writes support idempotent retry and core mutations produce append-only audit entries with request correlation.
- **Shared identity integration.** In the Base umbrella runtime, Wiki can validate configured central-auth tokens and proxy configured central login. Local Wiki session behavior remains available where that bridge is not configured.
- **Operational interface.** `GET /api/v1/health` is liveness; `GET /api/v1/health/ready` additionally confirms runtime readiness. The CLI, OpenAPI artifact and React client share the same API boundary.

<a name="quick-start"></a>
## Quick Start

For the repository-local Compose profile, start with the checked-in template and replace its sentinel values with operator-owned secrets before starting anything. Do not commit `.env`.

```bash
cp .env.example .env
# Edit .env: set database credentials and a unique WIKI_JWT_SECRET.
docker compose up --build -d
curl -fsS http://127.0.0.1:3456/api/v1/health
curl -fsS http://127.0.0.1:3456/api/v1/health/ready
```

Repository-local defaults are frontend `19877`, API `3456` and PostgreSQL `3457`. In the Base umbrella runtime, the same surfaces are published at frontend `7732`, API `7731` and loopback PostgreSQL `7733`; those are deployment-local coordinates, not public endpoints.

For source development and operational details, use [docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md), [docs/ENV.md](docs/ENV.md), [docs/OPERATIONS.md](docs/OPERATIONS.md) and [docs/CLI.md](docs/CLI.md).

<a name="visual-proof"></a>
## Visual Proof

The root README uses only reviewed blank-state or generic template evidence. It intentionally excludes document, space, dashboard, audit and user screens because their deterministic test fixtures show workflow identifiers, test account-like values or timeline metadata. The complete route inventory remains in [docs/assets/screens/manifest.md](docs/assets/screens/manifest.md).

### Login boundary

![Wiki login](docs/screenshots/01-login.png)

### Reusable document structures

![Wiki templates](docs/screenshots/12-templates.png)

### Login on mobile

![Wiki login on mobile](docs/screenshots/m-login.png)

The mobile proof is captured at `375x812`; the form has blank credentials and no browser or deployment chrome.

<a name="safety"></a>
## Safety Boundaries

- **Knowledge boundary.** Wiki stores pages, revisions, links and evidence. It does not execute pipelines, host Git repositories or advance external task/phase state.
- **Access boundary.** Permissions are checked before documents, evidence, attachments, trees and search results are read. Archived spaces/documents reject further protected writes.
- **Content boundary.** Published Markdown is rendered to sanitized HTML. Attachment names and storage keys are validated; secrets and tokens are excluded from search, audit and rendered user HTML.
- **Deployment boundary.** Production startup rejects weak auth configuration, wildcard or non-HTTPS CORS origins, insecure refresh cookies and an empty database URL. TLS, ingress, backups and network policy remain operator responsibilities.
- **Operational boundary.** Liveness and readiness are distinct probes. Readiness confirms the Wiki runtime dependencies, not every external identity, mail or future integration provider.

Read [docs/SECURITY.md](docs/SECURITY.md) and [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) before a shared deployment.

<a name="quality"></a>
## Quality and Verification

| Gate | Command |
|---|---|
| README contract tests | `python3 -m unittest scripts.tests.test_verify_readme -v` |
| README assets and anchors | `python3 scripts/verify_readme.py` |
| Backend format, lint and tests | `cd backend && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace -- --test-threads=1` |
| Frontend API/type/test/lint/build | `cd frontend && pnpm openapi:check && pnpm typecheck && pnpm test -- --run && pnpm lint && pnpm format:check && pnpm build` |
| Browser route smoke | `cd frontend && pnpm test:e2e -- --project=chromium` |
| Compose contract | `docker compose config -q` |
| Runtime probes | `curl -fsS http://127.0.0.1:3456/api/v1/health` and `curl -fsS http://127.0.0.1:3456/api/v1/health/ready` |

GitHub Actions runs backend, OpenAPI, migration, coverage, dependency-audit, frontend and browser-E2E gates. The independent README job prevents broken anchors, missing reviewed evidence and local-path/placeholder leaks from reaching `main`.

## Documentation Map

- **Current scope:** [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md), [docs/PRODUCT_REQUIREMENTS.md](docs/PRODUCT_REQUIREMENTS.md)
- **Architecture and contracts:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/API.md](docs/API.md), [docs/DATA_MODEL.md](docs/DATA_MODEL.md)
- **Operators:** [docs/OPERATIONS.md](docs/OPERATIONS.md), [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md), [docs/ENV.md](docs/ENV.md)
- **Engineering:** [docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md), [docs/TEST_PLAN.md](docs/TEST_PLAN.md), [docs/TRACEABILITY.md](docs/TRACEABILITY.md)
- **Security:** [docs/SECURITY.md](docs/SECURITY.md), [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md)

<a name="license"></a>
## License

FerrPOINT Proprietary Source-Available Evaluation License v1.0. This repository is not open source. Viewing and evaluation are allowed under [LICENSE](LICENSE); commercial, production, resale, redistribution and SaaS/hosting use require a written FerrPOINT license. See [NOTICE](NOTICE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
