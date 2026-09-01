# ADR-0005: Workspace Layered Architecture

## Status

Accepted

## Context

Wiki needs a clean domain boundary so SDLC knowledge-base behavior does not drift into task execution, board management or pipeline execution.

## Decision

Use a Cargo workspace with explicit backend layers:

```text
domain -> app -> infra -> api -> server
```

Frontend follows:

```text
app -> pages -> widgets -> features -> entities -> shared
```

Domain/application contracts use Wiki nouns: `Space`, `Document`, `DocumentRevision`, `TaskDossier`, `PhaseDossier`, `Evidence`, `Attachment`, `Template`, `AuditEntry`, `SourceReference`.

## Consequences

- Backend crates keep Wiki-owned modules and ports as their public surface.
- Domain tests can run without Axum, SQLx or filesystem.
- API and CLI use the same application contracts.
- Frontend pages can be built before backend completion with a thin API shell.
- Refactors must preserve the dependency direction.

## References

- `docs/ARCHITECTURE.md`
- `docs/architecture/backend-boundaries.md`
- `docs/architecture/frontend-boundaries.md`
- `docs/DOMAIN_MODEL.md`
