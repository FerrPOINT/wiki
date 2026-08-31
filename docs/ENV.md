# Environment Variables - Wiki

## 1. Required

| Variable | Description |
|---|---|
| `WIKI_DATABASE__URL` | PostgreSQL connection string for target persistence and compose |
| `WIKI_JWT_SECRET` | Backwards-compatible alias for `WIKI_AUTH__JWT_SECRET` |
| `WIKI_AUTH__JWT_SECRET` | JWT/session signing secret when using nested config |

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

## 4. Future External Sources

| Variable | Description |
|---|---|
| `WIKI_TASK_TRACKER_URL` | Optional task tracker base URL |
| `WIKI_CICD_URL` | Optional CI/CD base URL |
| `WIKI_PROJECT_WORKFLOW_URL` | Optional project-workflow URL |

These variables are not required for MVP.

Initial system admin seed variables (`WIKI_ADMIN_EMAIL`, `WIKI_ADMIN_PASSWORD`) are target variables for the PostgreSQL migration phase. The current in-memory API shell uses demo credentials documented in `docs/DEPLOYMENT.md`.

## 5. Frontend

| Variable | Default | Description |
|---|---|---|
| `VITE_API_BASE_URL` | `http://127.0.0.1:3456/api/v1` | API base path |

## 6. Security Notes

- Never commit real secrets.
- Rotate secrets independently per environment.
- Use different secrets per environment.
