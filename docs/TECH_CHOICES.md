# Tech Choices - Wiki

## 1. Backend

| Choice | Decision |
|---|---|
| Language | Rust |
| HTTP | Axum + Tower |
| Persistence | PostgreSQL |
| ORM/query | SQLx |
| Migrations | Versioned SQL migrations |
| Auth | Argon2id + JWT/session |
| OpenAPI | utoipa / utoipa-axum |
| Storage | Local filesystem behind a trait |

## 2. Wiki-specific Choices

| Need | Decision |
|---|---|
| Document source | Markdown |
| Markdown rendering | comrak |
| HTML sanitizing | ammonia |
| Search MVP | PostgreSQL FTS |
| Revision diff | similar |
| File integrity | sha2 checksums |
| MIME detection | infer |

## 3. Frontend

| Choice | Decision |
|---|---|
| UI | React + Vite |
| Styling | Tailwind CSS |
| Components | Radix/local shadcn-style components |
| Server state | TanStack Query |
| Client state | Zustand |
| Routing | react-router |
| Tests | Vitest + Playwright |

## 4. Rationale

The stack stays close to `task-tracker` and `CI-CD` to reuse operational knowledge, while adding Wiki-specific libraries only where the base product needs them: Markdown, sanitization, local attachment storage, search and revision diffing.
