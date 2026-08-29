# ADR-0010: Background Jobs Are Deferred

## Status

Deferred

## Context

Base Wiki can work without a dedicated scheduler or queue. MVP operations are document editing, publishing, task/phase links, evidence, search and audit.

Adding a job runner too early would expand the runtime and make the product look larger than the required base application.

## Alternatives Considered

| Option | Pros | Cons |
|---|---|---|
| No dedicated worker in MVP | Simple deployment, fewer moving parts | Some maintenance tasks stay manual or synchronous |
| In-process lightweight tasks | Easy to add for search rebuild or cleanup | Must not become hidden workflow engine |
| Apalis worker | Durable retries and scheduling | Extra infrastructure before MVP needs it |

## Decision

Do not require Apalis or any external worker process for MVP.

If future requirements need durable jobs, Apalis can be reconsidered for search rebuild, cleanup or preview generation only.

## Consequences

- MVP deployment stays backend + frontend + PostgreSQL + local storage.
- Document publish and evidence creation stay normal API transactions.
- Search must remain correct without a separate worker dependency.
- Future worker work needs a new requirement and operational runbook.

## Related

- `docs/RUNNER_ARCHITECTURE.md`
- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/PRODUCT_REQUIREMENTS.md`
