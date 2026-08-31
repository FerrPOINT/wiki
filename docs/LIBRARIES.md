# Libraries - Wiki

## 1. Backend Rust

| Area | Library | Role |
|---|---|---|
| HTTP API | `axum`, `tower`, `tower-http` | REST routing, middleware, tracing, CORS, compression |
| Async runtime | `tokio` | Tasks, IO, timers |
| Database | `sqlx` | Explicit SQL, migrations, transactions and search queries |
| Serialization | `serde`, `serde_json` | API DTOs, metadata JSON |
| Auth | `argon2`, `jsonwebtoken` | Password hashing, JWT/session tokens |
| Config | `config` | Typed TOML/process-env configuration with `WIKI_` prefix |
| Observability | `tracing`, `metrics`, `metrics-exporter-prometheus` | Logs, spans, Prometheus metrics |
| OpenAPI | `utoipa`, `utoipa-axum`, `utoipa-swagger-ui` | Generated schema and Swagger UI |
| Testing | `mockall`, `testcontainers`, `wiremock`, `rstest` | Trait mocks, DB tests, external HTTP mocks |

## 2. Wiki-specific Rust Building Blocks

| Need | Library | Notes |
|---|---|---|
| Markdown render | `comrak` | CommonMark/GFM parsing and HTML rendering |
| HTML sanitize | `ammonia` | Remove unsafe HTML after Markdown render |
| Storage MVP | local filesystem behind trait | Simple self-hosted attachments |
| Search MVP | PostgreSQL FTS via `sqlx` | Good enough for first release |
| Diff revisions | `similar` or `similar-asserts` | Text diff between document revisions |
| Slugs | `slug` or local normalizer | Stable document path segments |
| Checksums | `sha2` | Attachment/evidence integrity |
| MIME sniffing | `infer` | Validate uploaded file type |

## 3. Frontend

| Area | Library | Role |
|---|---|---|
| App | `react`, `react-dom`, `vite`, `typescript` | SPA foundation |
| Routing | `react-router` | Public/protected route tree |
| Server state | `@tanstack/react-query` | Queries, mutations, cache invalidation |
| Client state | `zustand` | Small UI stores |
| Forms | native React forms or local validators | Keep MVP forms simple |
| UI primitives | `@radix-ui/react-*` | Dialogs, dropdowns, tabs, menus |
| Icons | `lucide-react` | Buttons and navigation icons |
| Styling | `tailwindcss`, `clsx`, `tailwind-merge`, `class-variance-authority` | Tokens and reusable components |
| Toasts | `sonner` | Success/error feedback |
| Dates | `date-fns` | Relative and absolute dates |
| Tests | `vitest`, Testing Library, Playwright | Unit/component/E2E |

## 4. Infrastructure

| Tool | Role |
|---|---|
| PostgreSQL | Primary database, FTS for MVP |
| Local filesystem | MVP attachment bytes |
| Docker Compose | Local and self-hosted deployment |
| Prometheus/Grafana | Metrics and dashboards |
| Nginx/Caddy | Static frontend, reverse proxy, TLS |

## 5. Current Cleanup Note

The repository is copied from `task-tracker`, so dependency files may still contain libraries that are not needed for Wiki long-term. Remove task-tracker-only dependencies when replacing the backend/frontend domain modules.

## 6. References

- `docs/ARCHITECTURE.md`
- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/FRONTEND_ARCHITECTURE.md`
