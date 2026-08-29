# Third Party and SBOM - Wiki

## 1. Purpose

Track third-party dependencies, licenses and supply-chain risks for backend, frontend and infrastructure.

## 2. Backend Dependencies

Key Rust dependencies:

- axum
- tokio
- sea-orm
- sqlx
- comrak
- ammonia
- object_store
- tantivy
- utoipa
- apalis

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
