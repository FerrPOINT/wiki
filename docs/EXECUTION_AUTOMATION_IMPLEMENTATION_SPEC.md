# Execution Automation Implementation Spec - Wiki

## 1. Status

Execution automation is deferred. MVP does not implement webhook ingestion, outgoing webhook delivery, external event replay, notification fanout or worker queues.

This document exists only to reserve the future design area and keep parity with the CI/CD documentation set.

## 2. MVP Rule

All MVP writes happen through the public API:

- UI calls API;
- CLI calls API;
- backend validates permissions and writes audit.

There is no separate execution automation plane in the base product.

## 3. Deferred Worker Types

| Worker | Status |
|---|---|
| `search-rebuild` | Deferred |
| `preview-generator` | Deferred |
| `retention-cleanup` | Deferred |
| `webhook-delivery` | Deferred |

## 4. Future Entry Criteria

Automation work can start only after:

- document create/edit/publish is implemented;
- task and phase links are implemented;
- evidence URL/file flow is implemented;
- audit log is implemented;
- API and CLI cover the same MVP operations.

## 5. Future Rules

- Every automated write must be idempotent.
- Every automated write must create audit.
- Duplicate event replay must not create duplicate documents/evidence.
- Worker failure must not corrupt published revisions.
- Worker outage must not break base read/write API.

## 6. References

- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/RUNNER_ARCHITECTURE.md`
- `docs/API.md`
