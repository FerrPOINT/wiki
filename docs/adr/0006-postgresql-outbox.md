# ADR-0006: PostgreSQL Outbox Is Deferred

## Status

Deferred

## Context

The base Wiki MVP needs audit and PostgreSQL full-text search, but it does not need durable external event delivery or a separate job runner to satisfy the core flows.

Transactional outbox is useful once the product adds durable async processing, but adopting it in MVP would add tables, workers and operational concerns too early.

## Alternatives Considered

| Option | Pros | Cons |
|---|---|---|
| Synchronous audit/search updates | Simple, enough for MVP | Less flexible for long jobs |
| In-process best-effort events | Easy to add | Not durable across crashes |
| PostgreSQL outbox | Durable and transactional | Requires relay worker and monitoring |

## Decision

Do not require PostgreSQL outbox in MVP.

MVP may update audit and search projection synchronously inside application services. If future requirements add durable async processing, PostgreSQL outbox can be reconsidered.

## Consequences

- MVP deployment stays smaller.
- No worker is required for normal document/evidence writes.
- Long-running jobs remain deferred.
- Future outbox work requires migration, monitoring and retry policy.

## References

- `docs/EVENTS.md`
- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/RUNNER_ARCHITECTURE.md`
