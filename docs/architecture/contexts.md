# Architecture Contexts - Wiki

| Context | Owns | Does Not Own |
|---|---|---|
| Identity & Access | users, sessions, roles | document content |
| Knowledge Base | spaces, documents, revisions, drafts | external task lifecycle |
| Task Knowledge | task dossiers by task key | task tracker source of truth |
| Workflow Evidence | phase dossiers by phase key, evidence | workflow execution engine |
| Storage | attachments | document semantics |
| Search | indexing and query | authorization decisions |
| Administration | templates, audit, settings | business content authoring |

## Context Boundaries

Identity authorizes access but does not interpret document semantics. Knowledge Base owns document lifecycle and revision immutability. Task Knowledge groups documents/evidence by task key but never changes external task status. Workflow Evidence groups documents/evidence by phase key but never executes workflow transitions.

## Context Integration

| Producer | Consumer | Event/Data |
|---|---|---|
| Knowledge Base | Search | `document.published` |
| Administration | Identity & Access | role and permission updates |

## Acceptance Criteria

- Each API endpoint belongs to one primary context.
- Cross-context calls happen through application services or events.
- Search is a projection and can be rebuilt.
- Context ownership is reflected in `DOMAIN_MODEL.md` and `DATA_MODEL.md`.
