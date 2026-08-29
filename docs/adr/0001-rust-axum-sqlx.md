# ADR-0001: Rust + Axum + SQLx For Backend

## Status

Accepted

## Context

Wiki needs a compact self-hosted backend for documents, immutable revisions, task-linked pages, phase-linked pages, evidence, attachments, search and audit. The sibling SDLC projects already use Rust, Axum and PostgreSQL, so keeping the same stack reduces operational variance.

The backend must provide predictable resource usage, explicit error handling, strong typing around domain state and safe async I/O for API and storage calls.

## Alternatives Considered

| Option | Pros | Cons |
|---|---|---|
| Java + Spring Boot | Mature DI, broad enterprise ecosystem | Heavier runtime for small self-hosted deployments |
| Go + chi/sqlc | Simple deployment, fast builds | Weaker domain invariants and less reuse from sibling Rust projects |
| Node.js + NestJS | TypeScript end-to-end | Runtime safety and concurrency profile are less attractive for backend core |
| Rust + Axum + SQLx | Strong types, Tokio/Tower ecosystem, explicit SQL | Higher onboarding and compile-time cost |

## Decision

Use Rust edition 2024, Axum, Tokio and SQLx for Wiki backend API and persistence. The long-term backend structure is `domain -> app -> infra -> api -> server`; domain code must not depend on HTTP, SQL or filesystem implementations.

SQLx is the preferred low-level PostgreSQL access layer for migrations and explicit queries. If SeaORM is retained temporarily from the inherited codebase, it must stay inside `infra` and must not leak into domain or application contracts.

## Consequences

- Domain invariants can be expressed with Rust enums, value objects and result types.
- API handlers remain thin and delegate to application services.
- SQL remains visible and reviewable in migration and repository code.
- Backend and CLI can share DTO and error handling patterns.
- Developers need Rust familiarity, and CI must include `cargo fmt`, `cargo clippy` and tests.

## References

- `docs/ARCHITECTURE.md`
- `docs/DATA_MODEL.md`
- `docs/LIBRARIES.md`
- `docs/adr/0004-postgresql-only.md`
