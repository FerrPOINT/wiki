# CLI - Wiki

Консольный клиент `wiki` - второй официальный клиент продукта наряду с UI. Он управляет Wiki только через публичный HTTP API и не имеет специальных команд под отдельные типы потребителей.

## Глобальные опции

```text
--api-url   Базовый URL API (env: WIKI_API_URL, default: http://localhost:3456/api/v1)
--token     Bearer token (env: WIKI_TOKEN)
--output    json|table|compact (env: WIKI_OUTPUT, default: json)
```

## Команды MVP

### Auth

```bash
wiki auth login --email user@example.com --password secret
wiki auth logout
wiki auth whoami
```

### Spaces

```bash
wiki space list
wiki space create --key SDLC --name "SDLC Knowledge Base"
wiki space get SDLC
wiki space tree SDLC
wiki space members SDLC
```

### Documents

```bash
wiki doc create --space SDLC --title "Requirements" --type requirements --from-file requirements.md
wiki doc get <document-id>
wiki doc draft <document-id> --from-file updated.md
wiki doc publish <document-id> --summary "Clarified scope"
wiki doc archive <document-id>
wiki doc move <document-id> --parent <parent-document-id>
wiki doc history <document-id>
```

### Task Pages

```bash
wiki task get --space SDLC --key SDLC-42
wiki task docs --space SDLC --key SDLC-42
wiki task evidence --space SDLC --key SDLC-42
wiki task link-doc --space SDLC --key SDLC-42 --document <document-id>
```

### Phase Pages

```bash
wiki phase get --space SDLC --key implementation
wiki phase docs --space SDLC --key implementation
wiki phase evidence --space SDLC --key implementation
wiki phase link-doc --space SDLC --key implementation --document <document-id>
```

### Evidence

```bash
wiki evidence add-link --space SDLC --document <document-id> --task SDLC-42 --phase testing --title "Smoke test" --url "https://ci.local/jobs/42"
wiki evidence add-file --space SDLC --document <document-id> --task SDLC-42 --phase testing --file ./screen.png
wiki evidence get <evidence-id>
wiki evidence list --space SDLC --document <document-id>
```

### Templates and Search

```bash
wiki template list
wiki template apply requirements --space SDLC --title "Requirements"
wiki search query "authorization" --space SDLC --type requirements
```

### Settings

```bash
wiki settings get
```

## Требования

- JSON output по умолчанию.
- Ненулевой exit code для ошибок.
- Ошибки совместимы с API error envelope.
- Markdown можно передать из файла или stdin.
- Повторяемые write-команды отправляют `Idempotency-Key`.
- CLI не зависит от PostgreSQL schema, server filesystem или internal Rust modules.
