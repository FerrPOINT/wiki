# OpenAPI Workflow - Wiki

1. Backend handlers and DTOs are the source of truth for API schema.
2. Rust code uses `utoipa` / `utoipa-axum` (`#[derive(ToSchema)]`, `#[utoipa::path]`).
3. `cargo run --bin openapi-gen` writes `openapi/openapi.json` without starting a server.
4. `pnpm generate:api` will consume `openapi/openapi.json` and write `frontend/src/api/generated.ts` after generated-client support is enabled in the frontend.
5. Frontend temporarily uses a thin handwritten API shell; the generated client becomes the source once Wiki schemas stabilize.
6. `pnpm build` currently runs `tsc --noEmit && vite build`.

## Commands

```bash
# Regenerate OpenAPI schema from backend
cd /opt/dev/wiki/backend
cargo build --bin openapi-gen
./target/debug/openapi-gen /opt/dev/wiki/openapi/openapi.json

# Regenerate frontend client after generated-client support is enabled
cd /opt/dev/wiki/frontend
pnpm generate:api

# Full frontend checks
pnpm typecheck && pnpm test -- --run && pnpm build

# Start backend + DB
cd /opt/dev/wiki
docker compose up -d postgres backend
```

## Current State

`openapi/openapi.json` is the committed Wiki MVP public API artifact. It represents the approved route/DTO contract for the API router. Production server composition wires the same routes to PostgreSQL through `shared::wiki_contract::WikiBackendPort`; the memory backend remains a test/dev adapter.

## Notes

- Do not edit generated OpenAPI JSON manually.
- Regenerate `frontend/src/api/generated.ts` whenever backend schemas change.
- Add `VITE_API_BASE_URL=http://127.0.0.1:3456/api/v1` to `frontend/.env` for local dev.
