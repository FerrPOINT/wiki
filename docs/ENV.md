# Environment Variables - Wiki

## 1. Required

| Variable | Description |
|---|---|
| `WIKI_DATABASE_URL` | PostgreSQL connection string |
| `WIKI_AUTH_SECRET` | JWT/session signing secret |
| `WIKI_ADMIN_EMAIL` | Initial system admin email |
| `WIKI_ADMIN_PASSWORD` | Initial system admin password |

## 2. HTTP

| Variable | Default | Description |
|---|---|---|
| `WIKI_HTTP_HOST` | `0.0.0.0` | Bind host |
| `WIKI_HTTP_PORT` | `3456` | Backend API port |
| `WIKI_PUBLIC_URL` | `http://localhost:19877` | Public frontend URL |
| `WIKI_CORS_ORIGINS` | local dev origins | Allowed origins |

## 3. Storage

| Variable | Default | Description |
|---|---|---|
| `WIKI_FILE_STORAGE_BACKEND` | `filesystem` | `filesystem`, `s3`, `minio` |
| `WIKI_FILE_STORAGE_PATH` | `/var/lib/wiki/uploads` | Local storage path |
| `WIKI_FILE_STORAGE_BUCKET` | | S3/MinIO bucket |
| `WIKI_FILE_STORAGE_ENDPOINT` | | S3-compatible endpoint |
| `WIKI_FILE_STORAGE_ACCESS_KEY` | | Storage access key |
| `WIKI_FILE_STORAGE_SECRET_KEY` | | Storage secret |

## 4. Future External Sources

| Variable | Description |
|---|---|
| `WIKI_TASK_TRACKER_URL` | Optional task tracker base URL |
| `WIKI_CICD_URL` | Optional CI/CD base URL |
| `WIKI_PROJECT_WORKFLOW_URL` | Optional project-workflow URL |

These variables are not required for MVP.

## 5. Frontend

| Variable | Default | Description |
|---|---|---|
| `VITE_API_BASE_URL` | `http://127.0.0.1:3456/api/v1` | API base path |

## 6. Security Notes

- Never commit real secrets.
- Rotate secrets independently per environment.
- Use different secrets per environment.
