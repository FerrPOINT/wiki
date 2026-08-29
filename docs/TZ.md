# Technical Specification - Wiki

## 1. Product Summary

Wiki is a self-hosted SDLC knowledge base. It stores documents, immutable revisions, task/phase links, evidence and attachments. It is a basic internal Wiki, not a full Confluence replacement.

## 2. Goals

- Give teams one place for SDLC documents.
- Preserve immutable history of published document revisions.
- Link documents and evidence to external task keys and workflow phase keys.
- Provide permission-filtered search.
- Provide one backend API and two clients: UI and CLI.
- Reuse the Rust/React/PostgreSQL shape of sibling SDLC projects.

## 3. Non-goals

- Replace task tracker ownership of task state.
- Replace workflow execution.
- Replace CI/CD execution or artifact production.
- Replace Git hosting or code review tooling.
- Provide enterprise Confluence features such as marketplace apps, complex macros or real-time collaborative editing.

## 4. Functional Requirements

| ID | Requirement | Priority |
|---|---|---|
| TZ-001 | Login/logout and current user | P0 |
| TZ-002 | Admin/editor/viewer roles | P0 |
| TZ-003 | Create and manage spaces | P0 |
| TZ-004 | Manage space members | P0 |
| TZ-005 | Create Markdown document drafts | P0 |
| TZ-006 | Publish immutable document revisions | P0 |
| TZ-007 | Archive documents | P0 |
| TZ-008 | Maintain parent/child page tree | P0 |
| TZ-009 | Link documents to task key | P0 |
| TZ-010 | Link documents/evidence to phase key | P0 |
| TZ-011 | Upload files and add URL evidence | P0 |
| TZ-012 | Search title/body with permission filtering | P0 |
| TZ-013 | Provide basic document templates | P0 |
| TZ-014 | Provide API, UI and CLI for MVP operations | P0 |
| TZ-015 | Audit write and access-management actions | P0 |

## 5. User Interfaces

Frontend routes are documented in `ROUTING.md`. CLI commands are documented in `CLI.md`. Both clients use the same public API and share the same domain operations.

## 6. Backend Requirements

- Rust backend with Axum.
- Layered domain/app/infra/api/server shape.
- PostgreSQL as source of truth.
- Local storage adapter for MVP attachments.
- Markdown rendering and HTML sanitization.
- OpenAPI contract after Wiki endpoint implementation.
- Structured errors and idempotency keys for mutating commands.

## 7. Data Requirements

Core entities:

- `User`;
- `Space`;
- `SpaceMember`;
- `Document`;
- `DocumentDraft`;
- `DocumentRevision`;
- `TaskDossier`;
- `PhaseDossier`;
- `EvidenceItem`;
- `Attachment`;
- `DocumentTemplate`;
- `AuditEntry`.

Data model details are normative in `DATA_MODEL.md`.

## 8. Security Requirements

- JWT/session authentication.
- Role-based access to spaces and admin actions.
- Permission filtering for search results.
- Sanitized Markdown/HTML rendering.
- Secret values are never returned by API.
- Audit trail for write operations.

## 9. Acceptance Criteria

- Admin creates a space and adds editor/viewer users.
- Editor creates a document, publishes a revision and sees revision history.
- Viewer can read permitted documents but cannot edit them.
- Editor moves pages inside a space tree.
- Document is linked to task key and appears on task page.
- Document/evidence is linked to phase key and appears on phase page.
- URL evidence and file evidence appear in evidence list with metadata.
- Search returns only permitted documents.
- UI and CLI perform MVP operations through `/api/v1`.

## References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/ARCHITECTURE_INDEX.md`
- `docs/API.md`
- `docs/ROUTING.md`
- `docs/TRACEABILITY.md`
