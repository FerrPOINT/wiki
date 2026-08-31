# Integration Protocol - Wiki

## 1. Status

Integration-specific protocols are deferred. Wiki MVP has one public API used by UI and CLI. External scripts may call the same document, task, phase and evidence endpoints, but there are no special integration endpoints, webhook receivers or integration tokens in the base product.

This file is kept for documentation parity and future planning only.

## 2. MVP Source References

MVP evidence can store source metadata as ordinary fields:

| Field | Purpose |
|---|---|
| `source_type` | Human-readable source category such as `ci_pipeline`, `deployment`, `pull_request` or `test_artifact` |
| `source_url` | External URL if the material lives outside Wiki |
| `checksum` | Optional checksum for uploaded files or stable artifacts |
| `task_key` | Optional external task key |
| `phase_key` | Optional workflow phase key |

The API treats these fields as metadata. Wiki does not call the external system, execute a workflow or mutate external state.

## 3. Auth

MVP authentication is the standard user/session/JWT mechanism defined in `docs/API.md` and `docs/SECURITY.md`. A script using CLI or HTTP API has the same permission checks as any other client.

## 4. Idempotency

Write calls that can be retried use the normal `Idempotency-Key` header. For evidence created from external URLs, clients should choose a stable key from source URL, task key, phase key and artifact checksum when available.

## 5. Deferred Protocol

A future source-sync protocol may add:

- signed inbound webhooks;
- source-specific credentials;
- event id deduplication;
- external project mapping;
- operator visibility for failed imports.

Those items are not required for MVP and must not add frontend routes or API promises until approved separately.

## 6. References

- `docs/API.md`
- `docs/WEBHOOKS.md`
- `docs/EXECUTION_AUTOMATION_IMPLEMENTATION_SPEC.md`
