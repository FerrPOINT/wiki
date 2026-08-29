# ADR-0007: Runner Boundary Is Deferred

## Status

Deferred

## Context

Wiki MVP does not require external runners. Core operations are handled by API services and PostgreSQL transactions.

If future workers are added, they will process only Wiki maintenance jobs, not project code or external workflow execution.

## Decision

Do not introduce runner processes for MVP.

Future runner boundary, if approved, must be limited to:

- search rebuild;
- attachment cleanup;
- preview generation;
- maintenance jobs.

## Consequences

- MVP has fewer runtime components.
- API must not depend on worker availability for normal reads/writes.
- Worker security design remains documented but not implemented.
- Any future runner needs its own requirement and runbook.

## References

- `docs/RUNNER_ARCHITECTURE.md`
- `docs/contracts/RUNNER_PROTOCOL.md`
- `docs/PRODUCT_REQUIREMENTS.md`
