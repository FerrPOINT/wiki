# Wiki

Self-hosted SDLC knowledge base для FerrPOINT: spaces, documents, revisions, task dossiers, workflow phases, evidence, attachments, search and audit.

| Поле     | Значение                                                                                      |
| -------- | --------------------------------------------------------------------------------------------- |
| Статус   | MVP baseline: Wiki API/OpenAPI, CLI surface, frontend shell and SQLx persistence are in place |
| Backend  | Rust 2024, Axum, SQLx runtime persistence, PostgreSQL 17, Redis 8                             |
| Frontend | React 19, Vite, Tailwind CSS                                                                  |
| API      | Canonical Wiki MVP contract in [openapi/openapi.json](openapi/openapi.json)                   |
| Порты    | Frontend `19877`, backend `3456`, PostgreSQL `3457`, Redis `6379`                             |
| Лицензия | [FerrPOINT Proprietary Source-Available Evaluation License v1.0](LICENSE)                     |

## Что есть

- Spaces and document tree for requirements, architecture notes, decisions and release materials.
- Document create/view flows, revision-aware backend endpoints and generated frontend API types.
- Task and phase dossiers linked to evidence, checks and SDLC workflow context.
- Evidence registry for links, screenshots, files, PRs, pipeline runs and release checks.
- Templates, audit log, users/settings/admin pages and global search surface.
- CLI binary `wiki` for spaces, documents and evidence operations.
- Architecture, operations, threat model, traceability and visual screenshot evidence.

## Границы

- The repository was split from `task-tracker`; remaining legacy shape is being narrowed to Wiki scope.
- The current baseline is API-backed MVP, not a finished enterprise knowledge platform.
- Before shared deployments, replace all `[CHANGE_ME]` values, set `WIKI_JWT_SECRET`, configure bootstrap admin credentials and review CORS/cookie/TLS settings.
- PostgreSQL and Redis are local/dev defaults; treat exposed ports as intentional deployment choices.

Полный срез: [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md).

## Быстрый старт

```bash
cp .env.example .env
# Replace [CHANGE_ME] values and set WIKI_BOOTSTRAP__ADMIN_EMAIL/PASSWORD
docker compose up -d
curl http://127.0.0.1:3456/api/v1/health
```

Frontend dev:

```bash
cd frontend
pnpm install
pnpm dev
```

CLI:

```bash
cd backend
cargo build --bin wiki

set WIKI_API_URL=http://localhost:3456/api/v1
set WIKI_TOKEN=<jwt_token>

target\debug\wiki.exe space list
target\debug\wiki.exe doc create --space SDLC --title "Requirements" --from-file requirements.md
```

## Frontend routes

| Route                                                 | Назначение              |
| ----------------------------------------------------- | ----------------------- |
| `/login`, `/register`                                 | Auth                    |
| `/`                                                   | Dashboard               |
| `/spaces`, `/documents/new`, `/documents/:documentId` | Spaces and documents    |
| `/tasks`, `/tasks/:taskKey`                           | Task dossiers           |
| `/phases`, `/phases/:phaseId`                         | Workflow phase dossiers |
| `/evidence`, `/templates`, `/audit-log`               | Evidence and operations |
| `/users`, `/settings`, `/admin`                       | Administration          |
| `/search`                                             | Global search           |

## Структура

```text
wiki/
├── backend/     # Rust workspace: public Wiki API, SQLx persistence, CLI
├── frontend/    # React/Vite Wiki shell and API-backed MVP pages
├── cli/         # helper skill notes
├── docs/        # requirements, architecture, contracts, operations, quality
├── openapi/     # Wiki MVP API artifact
├── scripts/     # helper scripts
└── docker-compose.yml
```

## Документы

- [docs/USER_GUIDE.md](docs/USER_GUIDE.md) - пользовательские сценарии.
- [docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md) - разработка.
- [docs/OPERATIONS.md](docs/OPERATIONS.md) и [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) - эксплуатация.
- [docs/SECURITY.md](docs/SECURITY.md) и [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) - безопасность.
- [docs/ARCHITECTURE_INDEX.md](docs/ARCHITECTURE_INDEX.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/contracts](docs/contracts) - архитектура и контракты.
- [docs/API.md](docs/API.md), [docs/DATA_MODEL.md](docs/DATA_MODEL.md), [docs/ENV.md](docs/ENV.md), [docs/CLI.md](docs/CLI.md) - справочники.
- [docs/TEST_PLAN.md](docs/TEST_PLAN.md), [docs/TRACEABILITY.md](docs/TRACEABILITY.md), [docs/RISK_REGISTER.md](docs/RISK_REGISTER.md) - качество.

Скриншоты и параметры пересъемки: [docs/assets/screens/manifest.md](docs/assets/screens/manifest.md).

## Лицензия

Proprietary source-available. Not open source.

Viewing/evaluation only.

Commercial, production, resale, redistribution, SaaS/hosting use require written license from FerrPOINT. См. [LICENSE](LICENSE), [NOTICE](NOTICE) и [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
