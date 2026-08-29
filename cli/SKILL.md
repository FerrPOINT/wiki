---
name: wiki-cli
description: Manage Wiki spaces, documents, task links, phase links and evidence via the Wiki HTTP API.
---

# Wiki CLI

CLI для управления Wiki через API. Бинарник: `wiki` (Rust, собирается из `backend/cli/`).

## Когда использовать

- Создать или обновить документ из файла.
- Создать или открыть связь с внешней задачей.
- Создать или открыть связь с фазой workflow.
- Добавить материал: ссылку, файл, pipeline, deployment, test run.
- Найти документы по тексту, task key, phase code или тегу.

## Сборка

```bash
cd backend
cargo build --bin wiki
```

## Переменные окружения

| Env | CLI arg | Значение |
|---|---|---|
| `WIKI_API_URL` | `--api-url` | URL API, например `http://localhost:3456/api/v1` |
| `WIKI_TOKEN` | `--token` | Bearer token |
| `WIKI_OUTPUT` | `--output` | `json`, `table`, `compact` |

## Планируемые команды

```bash
wiki auth login --email admin@example.com --password secret
wiki auth whoami

wiki space list
wiki space create --key SDLC --name "SDLC Knowledge Base"
wiki space tree SDLC

wiki doc create --space SDLC --title "Requirements" --type requirements --from-file requirements.md
wiki doc get <document-id>
wiki doc draft <document-id> --from-file updated.md
wiki doc publish <document-id> --summary "Clarified acceptance criteria"
wiki doc history <document-id>

wiki task upsert --space SDLC --source task-tracker --key PROJ-123 --title "Add audit trail"
wiki task get --space SDLC --source task-tracker --key PROJ-123

wiki phase upsert --task PROJ-123 --workflow-run run-456 --phase-code 2.REQUIREMENTS --phase-name "Требования"
wiki phase complete --task PROJ-123 --phase 10.QA --verdict PASS

wiki evidence add-link --task PROJ-123 --phase 10.QA --type test_run --title "Playwright smoke" --url "https://ci.local/jobs/42"
wiki evidence add-file --task PROJ-123 --phase 10.QA --type screenshot --file ./screen.png

wiki search query "audit trail" --space SDLC
```

## Контракт CLI

- JSON output по умолчанию.
- Все write-команды поддерживают idempotency key.
- Ошибки возвращаются с ненулевым exit code и machine-readable stderr.
- CLI общается только с HTTP API и не использует прямой доступ к PostgreSQL или storage.
