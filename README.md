# Wiki

Self-hosted база знаний для SDLC: документы, ревизии, задачи, фазы workflow, материалы, вложения, поиск и аудит.

## Статус

Проект находится в стадии подготовки MVP. Кодовая база скопирована из `task-tracker`, но публичный Wiki router/OpenAPI, CLI surface, документация, frontend-shell и SQLx runtime persistence уже сведены к базовому Wiki scope.

| Capability | Статус |
|---|---|
| Документы требований и архитектурный каркас | Current |
| MVP route/page set и screenshot evidence | Current |
| Frontend pages/navigation под Wiki | Current, API-backed MVP pages |
| CLI command surface под Wiki | Current, HTTP client surface |
| Public Wiki API/OpenAPI | Current, Wiki MVP endpoints only |
| PostgreSQL domain/migrations под Wiki | Current baseline + auth/session migration |
| SQLx runtime persistence | Current for MVP API operations |
| Generated frontend OpenAPI client | Target |

Полный срез: [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md).

## Назначение

Wiki хранит важные материалы по задаче и каждой фазе выполненного workflow:

- требования и acceptance criteria;
- research notes и архитектурные решения;
- планы тестирования и материалы проверки;
- ссылки на PR, pipeline и релизные проверки;
- скриншоты и файлы;
- заметки к релизу и incident notes.

## Быстрый старт

```bash
cp .env.example .env
# замените [CHANGE_ME] и задайте WIKI_BOOTSTRAP__ADMIN_EMAIL/PASSWORD
docker compose up -d
curl http://127.0.0.1:3456/api/v1/health
```

Порты: frontend `19877`, backend `3456`, PostgreSQL `3457`, Redis `6379`.

Frontend:

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
target\debug\wiki.exe evidence add-link --space SDLC --document product-requirements --task SDLC-42 --phase testing --title "Smoke-проверка" --url "https://ci.local/jobs/42"
```

## Документация

По аудитории:

- Пользователь: [USER_GUIDE](docs/USER_GUIDE.md)
- Разработчик: [DEVELOPMENT_GUIDE](docs/DEVELOPMENT_GUIDE.md)
- Оператор: [OPERATIONS](docs/OPERATIONS.md), [TROUBLESHOOTING](docs/TROUBLESHOOTING.md)
- Безопасность: [SECURITY](docs/SECURITY.md), [THREAT_MODEL](docs/THREAT_MODEL.md)
- Продукт: [PRODUCT_REQUIREMENTS](docs/PRODUCT_REQUIREMENTS.md), [ROADMAP](docs/ROADMAP.md), [CURRENT_STATE](docs/CURRENT_STATE.md), [NEXT_STEPS](docs/NEXT_STEPS.md)

Архитектура:

- Входная точка: [ARCHITECTURE_INDEX](docs/ARCHITECTURE_INDEX.md)
- Narrative: [ARCHITECTURE](docs/ARCHITECTURE.md), [FUNCTIONAL_ARCHITECTURE](docs/FUNCTIONAL_ARCHITECTURE.md), [AUTHORIZATION](docs/AUTHORIZATION.md), [PAGE_DESIGN](docs/PAGE_DESIGN.md), [AUTOMATION](docs/AUTOMATION_ARCHITECTURE.md), [STORAGE](docs/STORAGE_ARCHITECTURE.md), [DELIVERY](docs/DELIVERY_ARCHITECTURE.md)
- Справочники: [API](docs/API.md), [DATA_MODEL](docs/DATA_MODEL.md), [ENV](docs/ENV.md), [CLI](docs/CLI.md), [LIBRARIES](docs/LIBRARIES.md), [TECH_CHOICES](docs/TECH_CHOICES.md)
- Контракты: [docs/contracts](docs/contracts)
- Sequence flows: [docs/architecture/sequences](docs/architecture/sequences)

Качество и SDLC:

- [TRACEABILITY](docs/TRACEABILITY.md)
- [TEST_PLAN](docs/TEST_PLAN.md)
- [ACCESSIBILITY](docs/ACCESSIBILITY.md)
- [RISK_REGISTER](docs/RISK_REGISTER.md)
- [SLO](docs/SLO.md)
- [METRICS](docs/METRICS.md)
- [DISASTER_RECOVERY](docs/DISASTER_RECOVERY.md)
- [INCIDENT_RESPONSE](docs/INCIDENT_RESPONSE.md)
- [THIRD_PARTY](docs/THIRD_PARTY.md)

## Frontend Pages

| Route | Назначение |
|---|---|
| `/login` | Вход пользователя |
| `/register` | Регистрация пользователя |
| `/` | Dashboard: последние документы и незакрытые связи |
| `/spaces` | Пространства и дерево документов |
| `/documents/new` | Создание документа |
| `/documents/:documentId` | Просмотр документа |
| `/tasks` / `/tasks/:taskKey` | Карточки задач и связанные документы |
| `/phases` / `/phases/:phaseId` | Карточки фаз workflow |
| `/evidence` | Реестр материалов |
| `/templates` | Шаблоны документов |
| `/audit-log` | Журнал аудита |
| `/users` | Пользователи и роли |
| `/settings` | Настройки инстанса |
| `/search` | Поиск по документам, задачам, фазам и материалам |
| `/admin` | Администрирование |

## Скриншоты страниц

Полный реестр и параметры пересъёмки: [docs/assets/screens/manifest.md](docs/assets/screens/manifest.md).

### Auth

![Login](docs/screenshots/01-login.png)

![Register](docs/screenshots/02-register.png)

### Core

![Dashboard](docs/screenshots/03-dashboard.png)

![Пространства](docs/screenshots/04-spaces.png)

![Создание документа](docs/screenshots/05-document-compose.png)

![Просмотр документа](docs/screenshots/06-document-view.png)

### Задачи и фазы workflow

![Задачи](docs/screenshots/07-task-dossiers.png)

![Карточка задачи](docs/screenshots/08-task-dossier-detail.png)

![Фазы](docs/screenshots/09-phase-dossiers.png)

![Карточка фазы](docs/screenshots/10-phase-dossier-detail.png)

### Материалы и операции

![Материалы](docs/screenshots/11-evidence.png)

![Шаблоны](docs/screenshots/12-templates.png)

![Аудит](docs/screenshots/13-audit-log.png)

### Administration

![Пользователи](docs/screenshots/14-users.png)

![Настройки](docs/screenshots/15-settings.png)

![Поиск](docs/screenshots/16-search.png)

![Администрирование](docs/screenshots/17-admin.png)

### Mobile Smoke

![Мобильный обзор](docs/screenshots/m-dashboard.png)

![Мобильные пространства](docs/screenshots/m-spaces.png)

![Мобильный документ](docs/screenshots/m-document-view.png)

![Мобильная задача](docs/screenshots/m-task-dossier.png)

![Мобильный поиск](docs/screenshots/m-search.png)

## Структура

```text
wiki/
├── backend/         # Rust workspace; public Wiki API with SQLx persistence and memory test fallback
├── frontend/        # React/Vite Wiki shell and API-backed MVP pages
├── cli/             # Codex/project helper skill notes
├── docs/            # requirements, architecture, contracts, operations, quality
├── openapi/         # Wiki MVP API artifact
├── scripts/         # helper scripts
└── docker-compose.yml
```

## Лицензия

MIT.
