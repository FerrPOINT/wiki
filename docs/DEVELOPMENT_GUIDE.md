# Development Guide - Wiki

## 1. Local Setup

```bash
git clone git@github.com:FerrPOINT/wiki.git
cd wiki
cp .env.example .env
```

`.env` is read by Docker Compose. Host-side `cargo run` reads process environment variables and `backend/config/default.toml`; export overrides before starting the backend.

Backend:

```bash
cd backend
cargo build
export WIKI_JWT_SECRET=dev-secret-32-chars-minimum
export WIKI_DATABASE__URL=postgres://wiki:[CHANGE_ME]@localhost:3457/wiki
cargo run --bin server
```

PowerShell equivalent:

```powershell
cd backend
$env:WIKI_JWT_SECRET = "dev-secret-32-chars-minimum"
$env:WIKI_DATABASE__URL = "postgres://wiki:[CHANGE_ME]@localhost:3457/wiki"
cargo run --bin server
```

Frontend:

```bash
cd frontend
pnpm install
pnpm dev
```

## 2. Architecture Rule

The target backend is layered:

```text
api/routes -> app/services -> domain -> infra/repositories
```

The target frontend follows:

```text
app -> pages -> widgets -> features -> entities -> shared
```

## 3. Development Order

1. Add or update REQ-ID in `docs/PRODUCT_REQUIREMENTS.md`.
2. Update contract docs.
3. Add migration/domain/service/API.
4. Regenerate OpenAPI and frontend client after the backend Wiki API is implemented.
5. Add focused backend and frontend tests.
6. Update user/operations docs.

## 4. Current Migration Note

The repository is copied from `task-tracker`. Replace inherited modules instead of extending task-tracker concepts. The target nouns are spaces, documents, revisions, task dossiers, phase dossiers, evidence and attachments.

## 5. Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
pnpm typecheck
pnpm test
pnpm test:e2e
```
