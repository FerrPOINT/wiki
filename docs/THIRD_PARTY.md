# Third Party and SBOM - Wiki

## 1. Purpose

Track third-party dependencies, licenses and supply-chain risks for backend, frontend and infrastructure.

## 2. Target Backend Dependencies

Key Rust dependencies:

- axum
- tokio
- sqlx
- comrak
- ammonia
- utoipa
- sha2
- jsonwebtoken
- argon2

`sea-orm-migration` is removed from the Wiki backend. `sea-orm` may still exist only behind explicit `legacy-tracker` compatibility modules until those copied task-tracker internals are deleted.

## 3. Frontend Dependencies

Key npm dependencies:

- react
- vite
- typescript
- react-router
- @tanstack/react-query
- zustand
- radix primitives
- lucide-react
- tailwindcss

## 4. Rules

- Pin major versions.
- Review licenses before introducing new dependency.
- Update SBOM on release.
- Security advisories block production release until triaged.
