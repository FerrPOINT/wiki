# CLI - Wiki

Консольный клиент `wiki` - второй официальный клиент продукта наряду с UI. Он управляет Wiki только через публичный HTTP API и не имеет специальных команд под отдельные типы потребителей.

## Глобальные опции

```text
--api-url   Базовый URL API (env: WIKI_API_URL, default: http://localhost:3456/api/v1)
--token     Bearer token (env: WIKI_TOKEN)
--output    json|table|compact (env: WIKI_OUTPUT, default: json)
```

## Output, Input And Exit Contract

`json` is canonical for automation and tests. `table` and `compact` are human presentation modes over the same response data.

Successful JSON output is the API response, pretty-printed without adding hidden fields. Empty `204` responses are normalized to:

```json
{
  "status": "ok"
}
```

API errors are parsed from the same envelope used by HTTP clients:

```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "space role is required"
  }
}
```

CLI stderr keeps the HTTP status, API code, message, optional request id and validation details, for example:

```text
API returned 400 Bad Request: VALIDATION_ERROR: Request validation failed; requestId=req-binary; details=q: too short
```

CLI exit codes for MVP:

| Code | Meaning |
| ---- | ------- |
| `0` | Command completed successfully, or help/version was printed |
| `1` | Runtime failure: API error, network failure, non-JSON API response, file read/write failure |
| `2` | CLI usage error emitted by `clap` before command execution |

Markdown input for `--from-file` accepts a filesystem path or `-` for stdin. Attachment download writes only to the path passed in `--out`.

Every CLI HTTP request sends `X-Request-ID` with a `wiki-cli-request-` prefix for backend log correlation. Mutating commands additionally send `Idempotency-Key`.

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
wiki space update SDLC --name "SDLC Wiki"
wiki space archive SDLC
wiki space get SDLC
wiki space tree SDLC
wiki space members SDLC
wiki space member-set SDLC --user <user-id> --role editor
wiki space member-remove SDLC --user <user-id>
```

### Users

```bash
wiki user list
wiki user create --email editor@example.com --username editor --password secret --name "Editor" --role user
wiki user update <user-id> --name "Editor Renamed" --role admin --active true
```

### Documents

```bash
wiki doc create --space SDLC --title "Requirements" --type requirements --from-file requirements.md
wiki doc get <document-id>
wiki doc draft <document-id> --title "Updated title" --from-file updated.md
wiki doc publish <document-id> --base-revision <revision-id> --summary "Clarified scope"
wiki doc archive <document-id>
wiki doc move <document-id> --parent <parent-document-id>
wiki doc history <document-id> --limit 20
wiki doc revision <document-id> <revision-id>
```

### Task Pages

```bash
wiki task list --space SDLC
wiki task get --space SDLC --key SDLC-42
wiki task docs --space SDLC --key SDLC-42
wiki task evidence --space SDLC --key SDLC-42
wiki task link-doc --space SDLC --key SDLC-42 --document <document-id>
```

### Phase Pages

```bash
wiki phase list --space SDLC
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
wiki evidence list --space SDLC --document <document-id> --limit 30
```

### Attachments

```bash
wiki attachment get <attachment-id>
wiki attachment download <attachment-id> --out ./artifact.bin
```

### Templates and Search

```bash
wiki template list
wiki template create --name "Release note" --type release_note --from-file release-note.md
wiki template apply requirements --space SDLC --title "Requirements"
wiki search query "authorization" --space SDLC --type requirements --limit 20
wiki search query "archived decision" --space SDLC --include-archived
```

### Audit and Settings

```bash
wiki audit list
wiki audit list --limit 25
wiki settings get
```

`wiki audit list` returns the API JSON as-is, including `request_id` for correlating CLI/UI/API write operations with backend logs. Without `--limit`, the API returns the latest 50 events; `--limit` is clamped server-side to `1..200`.

Bounded read commands use the same limits as the public API: `wiki doc history` defaults to 20 and clamps to `1..100`; `wiki evidence list` defaults to 30 and clamps to `1..100`; `wiki search query` defaults to 20 and clamps to `1..100`.

## Contract Freeze

CLI is ready for main development when:

- every command group maps to the public `/api/v1` API documented in `docs/API.md`;
- JSON is the default output for every command and table/compact modes are presentation-only;
- non-2xx API responses produce a non-zero exit code and render the API error code/message;
- write commands preserve `Idempotency-Key` behavior for safe retries;
- Markdown input works from a path or stdin through `--from-file -`;
- attachment download writes only to the requested local path and does not expose server storage keys.

## API Coverage Notes

| API area | CLI coverage |
| -------- | ------------ |
| Auth and current user | `wiki auth login/logout/whoami`; registration and token refresh are documented exceptions below |
| Users and roles | `wiki user`, `wiki space members/member-set/member-remove` |
| Spaces and tree | `wiki space list/create/update/archive/get/tree` |
| Documents and revisions | `wiki doc create/get/draft/publish/archive/move/history/revision` |
| Task dossiers | `wiki task list/get/docs/evidence/link-doc` |
| Phase dossiers | `wiki phase list/get/docs/evidence/link-doc` |
| Evidence and attachments | `wiki evidence`, `wiki attachment` |
| Templates/search/audit/settings | `wiki template`, `wiki search`, `wiki audit`, `wiki settings` |

Documented API-only or non-CLI MVP surfaces:

| Surface | Reason |
| ------- | ------ |
| `/api/v1/auth/register` | Public self-registration is a UI/API flow; admins can create users through `wiki user create`. |
| `/api/v1/auth/refresh` | CLI MVP accepts a bearer token through `--token`/`WIKI_TOKEN`; automated refresh can be added after token storage policy is approved. |
| `/api/v1/health`, `/api/v1/health/ready` | Operator probes are checked with HTTP tooling and deployment monitors. |
| `/metrics` | Prometheus scrape endpoint outside versioned `/api/v1` and outside OpenAPI v1. |
| `openapi/openapi.json` | Contract artifact consumed by generators/tests, not a runtime command. |

## Требования

- JSON output по умолчанию.
- Ненулевой exit code для ошибок.
- Ошибки совместимы с API error envelope.
- Markdown можно передать из файла или stdin.
- Повторяемые write-команды отправляют `Idempotency-Key`.
- CLI не зависит от PostgreSQL schema, server filesystem или internal Rust modules.
