<p align="center">
  <img src="docs/assets/wiki-readme-banner.svg" alt="Base Wiki - knowledge, revisions, evidence and audit" />
</p>

<p align="center">
  <a href="#overview"><img src="https://img.shields.io/badge/Overview-312e81?style=for-the-badge" alt="Overview" /></a>
  <a href="#capabilities"><img src="https://img.shields.io/badge/Capabilities-4c1d95?style=for-the-badge" alt="Capabilities" /></a>
  <a href="#routes"><img src="https://img.shields.io/badge/Routes-075985?style=for-the-badge" alt="Routes" /></a>
  <a href="#quick-start"><img src="https://img.shields.io/badge/Quick_Start-155e75?style=for-the-badge" alt="Quick start" /></a>
  <a href="#visual-proof"><img src="https://img.shields.io/badge/Visual_Proof-0f766e?style=for-the-badge" alt="Visual proof" /></a>
  <a href="#cli"><img src="https://img.shields.io/badge/CLI-334155?style=for-the-badge" alt="CLI" /></a>
  <a href="#quality"><img src="https://img.shields.io/badge/Quality-52525b?style=for-the-badge" alt="Quality" /></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Axum-Rest_API-312e81?style=flat-square" alt="Axum REST API" />
  <img src="https://img.shields.io/badge/SQLx-1D4ED8?style=flat-square" alt="SQLx" />
  <img src="https://img.shields.io/badge/PostgreSQL-17-4169e1?style=flat-square&logo=postgresql&logoColor=white" alt="PostgreSQL 17" />
  <img src="https://img.shields.io/badge/React-19-38bdf8?style=flat-square&logo=react&logoColor=0f172a" alt="React 19" />
  <img src="https://img.shields.io/badge/OpenAPI-6BA539?style=flat-square&logo=openapiinitiative&logoColor=white" alt="OpenAPI" />
  <img src="https://img.shields.io/badge/source--available-not%20open%20source-7F1D1D?style=flat-square" alt="Not open source" />
</p>

---

## 🎯 Позиционирование

**Wiki** — self-hosted knowledge base платформы Base для FerrPOINT: spaces, documents, revisions, task dossiers, workflow phases, evidence, attachments, search и audit.

Репозиторий сокращён до Wiki MVP runtime: public API/OpenAPI, CLI surface, frontend shell и SQLx/PostgreSQL persistence. Скопированные task-tracker backend-модули и старые зависимости удалены из активного workspace.

<a name="overview"></a>

## 📌 Snapshot

| Поле | Значение |
|---|---|
| Статус | MVP baseline: Wiki API/OpenAPI, CLI surface, frontend shell и SQLx persistence на месте |
| Backend | Rust 2024, Axum, SQLx runtime persistence |
| Data | PostgreSQL 17 и filesystem attachment storage |
| Frontend | React 19, Vite, Tailwind CSS |
| API | Canonical Wiki MVP contract: [openapi/openapi.json](openapi/openapi.json) |
| Порты | repository-local: frontend `19877`, backend `3456`, PostgreSQL `3457`; Base umbrella: frontend `7732`, API `7731`, PostgreSQL loopback `7733` |
| License | FerrPOINT Proprietary Source-Available Evaluation License v1.0 |

<a name="capabilities"></a>

## ✨ Возможности

| Feature | Описание |
|---|---|
| Spaces and documents | Spaces и document tree для requirements, architecture notes, decisions и release materials. |
| Document lifecycle | Create/view/edit/publish/archive/move flows, revision-aware backend endpoints и generated frontend API types. |
| Base dossiers | Task и phase dossiers, связанные с evidence и workflow context. |
| Evidence registry | External links и uploaded files, прикреплённые к documents, tasks или phases. |
| Operations | Templates, audit log, users/settings/admin pages, global search и API health/readiness probes. |
| CLI | HTTP-only `wiki` binary для тех же public API операций, что и UI. |
| Documentation | Architecture, operations, threat model, traceability и visual screenshot evidence. |

## 🔧 Стек

| Zone | Tech | Роль |
|---|---|---|
| API | Rust + Axum | Wiki MVP routes и public API |
| Persistence | SQLx + PostgreSQL | runtime data и migrations |
| Attachment storage | local filesystem | uploaded evidence files |
| Frontend | React + Vite + Tailwind | Wiki shell и API-backed MVP pages |
| Contract | OpenAPI | generated frontend types |
| Docs | contracts, security, traceability | source of truth для scope |

<a name="quick-start"></a>

## ⚡ Быстрый старт

```bash
cp .env.example .env
# Заменить [CHANGE_ME] значения и задать WIKI_BOOTSTRAP__ADMIN_EMAIL/PASSWORD
docker compose up --build -d
curl -fsS http://127.0.0.1:3456/api/v1/health
curl -fsS http://127.0.0.1:3456/api/v1/health/ready
```

Frontend dev:

```bash
cd frontend
pnpm install
pnpm dev
```

PostgreSQL API smoke (disposable test DB + env-gated `wiki_postgres_` API tests, включая persistence, membership revocation и FTS index-plan evidence):

```powershell
pwsh -File scripts/postgres-smoke.ps1
```

Если Docker Desktop недоступен, но в WSL есть локальный PostgreSQL — тот же smoke против изолированной временной БД:

```powershell
pwsh -File scripts/postgres-smoke-wsl.ps1
```

Backup/restore drill на том же Windows/WSL fallback-пути:

```powershell
pwsh -File scripts/backup-restore-smoke-wsl.ps1
```

<a name="routes"></a>

## 🧭 Фронтенд-роуты

| Route | Назначение |
|---|---|
| `/login`, `/register` | Auth |
| `/` | Dashboard |
| `/spaces`, `/documents/new`, `/documents/:documentId` | Spaces и documents |
| `/tasks`, `/tasks/:taskKey` | Task dossiers |
| `/phases`, `/phases/:phaseId` | Workflow phase dossiers |
| `/evidence`, `/templates`, `/audit-log` | Evidence и operations |
| `/users`, `/settings`, `/admin` | Administration |
| `/search` | Global search |

Операционные probe `/api/v1/health` и `/api/v1/health/ready` — API-only, без frontend-скриншотов.

<a name="visual-proof"></a>

## 🖼️ Визуальные доказательства

Скриншоты — реальные страницы продукта. Desktop full-page, mobile `375x812`. Полный реестр и параметры пересъёмки: [docs/assets/screens/manifest.md](docs/assets/screens/manifest.md).

### Вход

![Вход](docs/screenshots/01-login.png)

### Регистрация

![Регистрация](docs/screenshots/02-register.png)

### Дашборд

![Дашборд](docs/screenshots/03-dashboard.png)

### Пространства

![Пространства](docs/screenshots/04-spaces.png)

### Создание документа

![Создание документа](docs/screenshots/05-document-compose.png)

### Просмотр документа

![Просмотр документа](docs/screenshots/06-document-view.png)

### Task-досье

![Task-досье](docs/screenshots/07-task-dossiers.png)

### Карточка task-досье

![Карточка task-досье](docs/screenshots/08-task-dossier-detail.png)

### Phase-досье

![Phase-досье](docs/screenshots/09-phase-dossiers.png)

### Карточка phase-досье

![Карточка phase-досье](docs/screenshots/10-phase-dossier-detail.png)

### Evidence

![Evidence](docs/screenshots/11-evidence.png)

### Шаблоны

![Шаблоны](docs/screenshots/12-templates.png)

### Журнал аудита

![Журнал аудита](docs/screenshots/13-audit-log.png)

### Пользователи

![Пользователи](docs/screenshots/14-users.png)

### Настройки

![Настройки](docs/screenshots/15-settings.png)

### Поиск

![Поиск](docs/screenshots/16-search.png)

### Администрирование

![Администрирование](docs/screenshots/17-admin.png)

### Мобильный интерфейс (375×812)

|   |   |
| :---: | :---: |
| ![Дашборд на мобильном](docs/screenshots/m-dashboard.png) | ![Вход на мобильном](docs/screenshots/m-login.png) |
| ![Документ на мобильном](docs/screenshots/m-document-view.png) | ![Поиск на мобильном](docs/screenshots/m-search.png) |
| ![Пространства на мобильном](docs/screenshots/m-spaces.png) | ![Досье задачи на мобильном](docs/screenshots/m-task-dossier.png) |

<a name="cli"></a>

## 🖥️ CLI

```bash
cd backend
cargo build --bin wiki

export WIKI_API_URL=http://localhost:3456/api/v1
export WIKI_TOKEN=<jwt_token>

./target/debug/wiki space list
./target/debug/wiki user list
./target/debug/wiki doc create --space BASE --title "Requirements" --from-file requirements.md
./target/debug/wiki space member-set BASE --user <user-id> --role editor
./target/debug/wiki attachment download <attachment-id> --out artifact.bin
./target/debug/wiki audit list --limit 25
./target/debug/wiki settings get
```

## 🏗️ Архитектура

```mermaid
flowchart TD
    UI[React Wiki shell] --> API[Axum Wiki API]
    CLI[wiki CLI] --> API
    API --> Services[Wiki application services]
    Services --> Store[SQLx persistence]
    Store --> DB[(PostgreSQL)]
    Services --> Evidence[Evidence + audit]
    OpenAPI[OpenAPI contract] --> Gen[Generated frontend types]
    API --> OpenAPI
```

<a name="safety"></a>

## 🧱 Границы

- Текущий baseline — API-backed MVP, не готовая enterprise knowledge platform.
- Reports, notifications, webhooks, import/export bundles, OCR и real-time collaboration отложены.
- Перед shared deployments замените все `[CHANGE_ME]`, задайте `WIKI_ENVIRONMENT=production`, `WIKI_JWT_SECRET`, bootstrap-admin credentials и проверьте CORS/cookie/TLS.
- Опубликованный Markdown рендерится в sanitized HTML; имена и storage-ключи вложений валидируются.
- Production startup отклоняет слабую auth-конфигурацию, wildcard/non-HTTPS CORS, insecure refresh cookies и пустой database URL.

Полный current-state срез: [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md).

<a name="quality"></a>

## 🛡️ Качество и проверки

| Проверка | Команда |
|---|---|
| README contract tests | `python3 -m unittest scripts.tests.test_verify_readme -v` |
| README assets и anchors | `python3 scripts/verify_readme.py` |
| Backend | `cd backend && cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace -- --test-threads=1` |
| Frontend | `cd frontend && pnpm openapi:check && pnpm typecheck && pnpm test -- --run && pnpm lint && pnpm build` |
| Browser route smoke | `cd frontend && pnpm test:e2e -- --project=chromium` |
| Compose contract | `docker compose config -q` |
| Runtime probes | `curl -fsS http://127.0.0.1:3456/api/v1/health` и `.../health/ready` |

GitHub Actions прогоняет backend, OpenAPI, migration, coverage, dependency-audit, frontend и browser-E2E gates; независимый README job не пускает в `main` битые anchors, отсутствующее reviewed evidence и утечки путей/плейсхолдеров.

## 🗂️ Карта проекта

```text
wiki/
├── backend/     # Rust workspace: public Wiki API, SQLx persistence, CLI
├── frontend/    # React/Vite Wiki shell и API-backed MVP pages
├── cli/         # helper skill notes
├── docs/        # requirements, architecture, contracts, operations, quality
├── openapi/     # Wiki MVP API artifact
├── scripts/     # helper scripts
└── docker-compose.yml
```

## 📚 Документы

- [docs/USER_GUIDE.md](docs/USER_GUIDE.md) — пользовательские сценарии.
- [docs/MVP_READINESS.md](docs/MVP_READINESS.md) — 100% readiness gate перед main development.
- [docs/DEVELOPMENT_GUIDE.md](docs/DEVELOPMENT_GUIDE.md) — разработка.
- [docs/OPERATIONS.md](docs/OPERATIONS.md), [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) — эксплуатация.
- [docs/SECURITY.md](docs/SECURITY.md), [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) — безопасность.
- [docs/ARCHITECTURE_INDEX.md](docs/ARCHITECTURE_INDEX.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/contracts](docs/contracts) — архитектура и контракты.
- [docs/API.md](docs/API.md), [docs/DATA_MODEL.md](docs/DATA_MODEL.md), [docs/ENV.md](docs/ENV.md), [docs/CLI.md](docs/CLI.md) — справочники.
- [docs/TEST_PLAN.md](docs/TEST_PLAN.md), [docs/TRACEABILITY.md](docs/TRACEABILITY.md), [docs/RISK_REGISTER.md](docs/RISK_REGISTER.md) — качество.

Скриншоты и параметры пересъёмки: [docs/assets/screens/manifest.md](docs/assets/screens/manifest.md).

<a name="license"></a>

## 🔒 Лицензия

Proprietary source-available. Not open source. Viewing/evaluation only.

Commercial, production, resale, redistribution, SaaS/hosting use require written license from FerrPOINT. См. [LICENSE](LICENSE), [NOTICE](NOTICE) и [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
