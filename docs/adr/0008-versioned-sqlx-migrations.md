# ADR-0008: Versioned SQLx Migrations

## Status

Accepted

## Context

Wiki owns a fresh schema for knowledge-base data. Runtime bootstrap logic is risky because it hides schema state and makes repeatable deployment harder. The project needs versioned migrations that can be reviewed, applied and tested.

## Decision

Use versioned SQL migrations managed by SQLx-compatible tooling. Migrations live in the canonical backend migrations directory and are applied explicitly during deployment or local setup.

Migration policy:

- no destructive rewrite without backup/migration plan;
- forward migrations are immutable after merge;
- data backfills are idempotent or guarded;
- indexes for large tables are planned with lock impact in mind;
- rollback strategy is documented per release.

## Consequences

- Schema changes become reviewable artifacts.
- CI can apply migrations to an empty database and a migrated fixture.
- Operators can reason about upgrade order.
- Generated OpenAPI and data-model docs can reference stable schema names.
- Emergency fixes may require additive migrations instead of editing history.

## References

- `docs/MIGRATIONS.md`
- `docs/MIGRATION_EXECUTION_SPEC.md`
- `docs/DATABASE_STANDARDS.md`
- `docs/DATABASE_INDEXES.md`
