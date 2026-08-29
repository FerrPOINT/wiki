# ADR-0004: PostgreSQL As Primary Data Store

## Status

Accepted

## Context

Wiki stores relational permissions, document revisions, evidence metadata, audit entries and search projections. MVP needs one reliable source of truth with strong transactions and easy self-hosting.

## Alternatives Considered

| Option | Pros | Cons |
|---|---|---|
| PostgreSQL only | Strong transactions, FTS, JSONB, mature backup tooling | Some workloads need careful indexing |
| PostgreSQL + Elasticsearch | Strong search capabilities | More infrastructure and permission-filtering complexity |
| SQLite | Very simple local deployment | Harder multi-user concurrency and operational backup model |
| Document database | Flexible document shape | Harder relational authorization and traceability |

## Decision

Use PostgreSQL as the primary store. Store Markdown source, revision metadata, relationships and audit log in relational tables with JSONB only for bounded metadata. Use PostgreSQL full-text search for MVP.

## Consequences

- Backup/restore and migration flows are straightforward.
- Search can start without operating a separate service.
- Authorization joins stay close to stored data.
- Large binary attachments stay outside PostgreSQL and are referenced by metadata.
- A separate search backend requires a future approved requirement.

## References

- `docs/DATA_MODEL.md`
- `docs/STORAGE.md`
- `docs/STORAGE_ARCHITECTURE.md`
- `docs/DATABASE_INDEXES.md`
