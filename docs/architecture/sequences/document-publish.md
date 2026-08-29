# Sequence - Document Publish

```mermaid
sequenceDiagram
    participant U as User
    participant F as Frontend
    participant A as API
    participant D as PostgreSQL
    U->>F: publish draft
    F->>A: POST /documents/{id}/publish
    A->>D: validate permission and base revision
    A->>D: insert document_revision
    A->>D: update current_revision_id
    A->>D: record audit event
    A-->>F: published revision
```

## Rules

- Publishing requires editor permission in the space.
- Base revision is checked to prevent stale overwrite.
- Published revision content is immutable.
- Restoring an older revision creates a new revision.
- Search index and audit data are updated as part of the publish flow.

## Failure Modes

| Failure | Handling |
|---|---|
| Missing permission | `403 permission_denied` |
| Stale draft | `409 revision_conflict` |
| Invalid Markdown metadata | `422 validation_failed` |
| Search index update failed | publish rolls back or returns explicit retryable error |

## Acceptance Criteria

- Document current revision changes atomically with revision insert.
- Audit log records actor, document id and revision id.
- Search can find the new revision after publish completes.
