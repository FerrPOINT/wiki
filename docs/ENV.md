# Environment Variables - Wiki

## 1. Required

| Variable | Description |
|---|---|
| `WIKI_DATABASE__URL` | PostgreSQL connection string for SQLx runtime persistence and compose |
| `WIKI_JWT_SECRET` | Backwards-compatible alias for `WIKI_AUTH__JWT_SECRET` |
| `WIKI_AUTH__JWT_SECRET` | JWT/session signing secret when using nested config |

For Docker Compose backend use `postgres://wiki:...@postgres:5432/wiki`. For host-side `cargo run` against compose Postgres use `postgres://wiki:...@localhost:3457/wiki`.

## 2. HTTP

| Variable | Default | Description |
|---|---|---|
| `WIKI_SERVER__ADDRESS` | `0.0.0.0` | Bind host |
| `WIKI_SERVER__PORT` | `3456` | Backend API port |
| `WIKI_SERVER__CORS_ALLOWED_ORIGINS` | local dev origins | Allowed origins |
| `WIKI_SERVER__AUTH_RATE_BURST` | `5` | Auth rate-limit burst |
| `WIKI_SERVER__AUTH_RATE_PERIOD_SECS` | `15` | Auth rate-limit period |
| `WIKI_SERVER__GENERAL_RATE_BURST` | `60` | General API rate-limit burst |
| `WIKI_SERVER__GENERAL_RATE_PERIOD_SECS` | `60` | General API rate-limit period |

## 3. Storage

| Variable | Default | Description |
|---|---|---|
| `WIKI_STORAGE__DIR` | `/var/lib/wiki/uploads` | Local storage path |
| `WIKI_STORAGE__MAX_UPLOAD_BYTES` | `26214400` | Max upload size in bytes |

## 4. Bootstrap Admin

| Variable | Description |
|---|---|
| `WIKI_BOOTSTRAP__ADMIN_EMAIL` | Optional first admin email; must be set together with password |
| `WIKI_BOOTSTRAP__ADMIN_PASSWORD` | Optional first admin password; never use a committed/default value |
| `WIKI_BOOTSTRAP__ADMIN_USERNAME` | Optional first admin username |
| `WIKI_BOOTSTRAP__ADMIN_DISPLAY_NAME` | Optional first admin display name |

Backwards-compatible aliases are also accepted: `WIKI_ADMIN_EMAIL`, `WIKI_ADMIN_PASSWORD`, `WIKI_ADMIN_USERNAME`, `WIKI_ADMIN_DISPLAY_NAME`.

When email and password are set, backend startup creates or updates that admin, ensures the `SDLC` space exists and seeds default document templates. Without these variables PostgreSQL starts with no default account.

## 5. Future External Sources

| Variable | Description |
|---|---|
| `WIKI_TASK_TRACKER_URL` | Optional task tracker base URL |
| `WIKI_CICD_URL` | Optional CI/CD base URL |
| `WIKI_PROJECT_WORKFLOW_URL` | Optional project-workflow URL |

These variables are not required for MVP.

## 6. Frontend

| Variable | Default | Description |
|---|---|---|
| `VITE_API_BASE_URL` | `http://127.0.0.1:3456/api/v1` | API base path |

## 7. Security Notes

- Never commit real secrets.
- Rotate secrets independently per environment.
- Use different secrets per environment.
