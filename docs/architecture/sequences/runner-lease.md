# Sequence: Wiki Runner Lease

## Purpose

This is deferred reference material. Wiki MVP does not require a runner protocol, worker queue or runner lease. Base operations are handled by the API service and PostgreSQL transactions.

If a future runner is approved, this sequence can be used to coordinate background jobs with at-most-one active worker lease.

## Flow

```text
Runner heartbeat
  -> Runner asks for queued work through a future internal endpoint
  -> API selects a compatible queued job
  -> Lease is created with expiry and fencing token
  -> Runner processes job
  -> Runner reports progress or completion
  -> API commits result and emits audit/metrics events
```

## Lease Semantics

- Poll is long-polling and may return `204`.
- Lease owner is checked on every mutation.
- Expired lease fences the old runner.
- Retry creates a new attempt record.
- Completion is idempotent for identical result payloads.

## Job Examples

| Job | Trigger |
|---|---|
| Search reindex | Manual or scheduled maintenance |
| File preview | Attachment uploaded |
| Import/export bundle | Separately approved portability feature |
| Outbound delivery | Separately approved webhook feature |

## References

- `docs/RUNNER_ARCHITECTURE.md`
- `docs/contracts/RUNNER_PROTOCOL.md`
- `docs/RESILIENCE.md`
