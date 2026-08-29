# Runner Architecture - Wiki

## 1. Status

Dedicated Wiki runners are deferred. The MVP should work without separate worker processes.

This document exists for parity with the CI/CD documentation set and for future design notes only.

## 2. MVP Boundary

MVP runtime:

- backend API;
- frontend static app;
- CLI as HTTP client;
- PostgreSQL;
- local attachment storage.

No MVP capability requires a runner. Search indexing can be implemented synchronously during publish or through a simple in-process job if needed.

## 3. Deferred Runner Jobs

Future runner jobs may be useful for:

- rebuilding search projection;
- cleaning orphaned uploaded files;
- generating attachment previews;
- moving from local storage to object storage;
- longer maintenance tasks.

They are not required for document create/edit/publish, task/phase links, evidence or basic search.

## 4. Trust Boundary

If runners are added later:

- workers must not execute project source code;
- workers must use scoped service identity;
- worker logs must redact secrets and private document body;
- attachment processing must enforce size, MIME and timeout limits;
- jobs must be safe to retry.

## 5. Observability

Future worker metrics:

- queue depth;
- job duration;
- failed job count;
- search rebuild lag;
- cleanup failures.

These metrics are deferred with the worker implementation.

## 6. Acceptance Criteria for Future Runner

- Base API keeps working when the worker is down.
- A failed worker job cannot corrupt document revisions.
- Retry does not duplicate evidence or attachments.
- Operators can see failed jobs without reading raw secrets.

## References

- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/contracts/RUNNER_PROTOCOL.md`
- `docs/OPERATIONS.md`
