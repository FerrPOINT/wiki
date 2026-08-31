# Sequence - Workflow Phase Sync

```mermaid
sequenceDiagram
    participant W as UI or CLI
    participant A as Wiki API
    participant D as PostgreSQL
    W->>A: link document/evidence to task_key or phase_key
    A->>D: record task/phase link
    A->>D: check linked documents/materials
    A-->>W: accepted
```

## Rules

- Wiki stores task and phase links created through API/CLI clients.
- Wiki tracks documentation completeness from linked documents and materials.
- Completed phase can still receive additional materials, but the original completion snapshot stays auditable.

## Failure Modes

| Failure | Handling |
|---|---|
| Unknown phase | Create pending mapping or reject by project policy |
| Missing task/phase key | Create dossier projection from linked documents/materials |
| Missing materials | Accept update and show dashboard gap |
| Duplicate phase link | Return existing phase page state |

## Acceptance Criteria

- Phase page exists after the task/phase link is created.
- Missing required documents appear on the dashboard.
- Phase page links back to task page and related materials.
