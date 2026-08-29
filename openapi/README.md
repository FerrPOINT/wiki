# OpenAPI Workflow - Wiki

1. Backend handlers and DTOs are the source of truth for API schema.
2. Rust code uses `utoipa` / `utoipa-axum` (`#[derive(ToSchema)]`, `#[utoipa::path]`).
3. `cargo run --bin openapi-gen` writes `openapi/openapi.json` without starting a server.
4. `pnpm generate:api` consumes `openapi/openapi.json` and writes `frontend/src/api/generated.ts`.
5. Frontend uses `openapi-fetch` with generated `paths` and `components`.
6. `pnpm build` regenerates the client before `tsc` and `vite build`.

## Commands

```bash
# Regenerate OpenAPI schema from backend
cd /opt/dev/wiki/backend
cargo build --bin openapi-gen
./target/debug/openapi-gen /opt/dev/wiki/openapi/openapi.json

# Regenerate frontend client
cd /opt/dev/wiki/frontend
pnpm generate:api

# Full frontend checks
pnpm typecheck && pnpm test -- --run && pnpm build

# Start backend + DB
cd /opt/dev/wiki
docker compose up -d postgres redis backend
```

## Current State

`openapi/openapi.json` is inherited from the task-tracker base. Replace it after implementing Wiki endpoints for spaces, documents, revisions, task dossiers, phase dossiers, evidence, attachments, search and admin operations.

## Notes

- Do not edit generated OpenAPI JSON manually.
- Regenerate `frontend/src/api/generated.ts` whenever backend schemas change.
- Add `VITE_API_BASE_URL=http://127.0.0.1:3456/api/v1` to `frontend/.env` for local dev.
