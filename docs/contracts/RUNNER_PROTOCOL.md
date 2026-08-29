# Runner Protocol - Wiki

## 1. Status

Runner protocol is deferred. The Wiki MVP must not require external runners or queue workers.

This file remains in the contracts set for parity with the CI/CD project, but it is not a current implementation contract.

## 2. MVP Runtime

MVP uses:

- API process;
- PostgreSQL;
- local attachment storage;
- frontend static assets;
- CLI as HTTP client.

Document publish, evidence creation and search updates should work without a separate runner.

## 3. Future Job Types

If a runner is introduced later, accepted job types may be:

| Job type | Payload |
|---|---|
| `search_rebuild` | `space_id` or `document_revision_id` |
| `attachment_cleanup` | `attachment_id` or `space_id` |
| `preview_generate` | `attachment_id` |

No future runner job may execute repository code or mutate external systems as part of the Wiki base product.

## 4. Future Lease Shape

```json
{
  "workerId": "wiki-worker-01",
  "capabilities": ["search_rebuild"],
  "maxJobs": 1
}
```

```json
{
  "leaseId": "018f...",
  "jobType": "search_rebuild",
  "payload": {
    "spaceId": "018f..."
  },
  "expiresAt": "2026-08-28T12:00:00Z"
}
```

## 5. Safety Rules

- Jobs must be idempotent.
- Job logs must redact secrets.
- Failed jobs must not corrupt published revisions.
- API must continue to serve read/write requests when worker is offline.
- Retry must not duplicate evidence or attachments.

## 6. References

- `docs/RUNNER_ARCHITECTURE.md`
- `docs/AUTOMATION_ARCHITECTURE.md`
- `docs/PRODUCT_REQUIREMENTS.md`
