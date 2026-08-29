# Sequence: Git And Pull Request Ingress

## Purpose

Deferred reference flow for capturing repository, commit and pull request references as Wiki evidence without becoming a Git hosting service. MVP stores these references through ordinary API/CLI evidence writes.

## Flow

```text
API or CLI client
  -> POST /api/v1/evidence
  -> API verifies user/service token and idempotency key
  -> Application service normalizes repository, commit and PR fields
  -> Task page is found or created by external task key
  -> Source link and review evidence are upserted
  -> Search index is updated
```

## Rules

- Client-provided idempotency key prevents duplicate evidence on retry.
- Imported PR descriptions are sanitized before display.
- Commit SHA is stored as metadata and indexed as an exact-match facet.
- Missing task key creates unassigned source evidence only if project policy allows it.

## Failure Modes

| Failure | Handling |
|---|---|
| Invalid token | Reject `401`, audit security event |
| Unknown repository | Store source-link error or route to manual triage |
| Duplicate write | Return success without duplicate evidence |
| Search update failed | Return retryable error or roll back evidence write |

## References

- `docs/GIT_HOSTING.md`
- `docs/PULL_REQUESTS.md`
- `docs/contracts/EVENT_CONTRACT.md`
