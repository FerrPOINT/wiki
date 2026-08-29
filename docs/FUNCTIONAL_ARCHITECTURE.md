# Functional Architecture - Wiki

## 1. Product Boundary

Wiki is a lightweight SDLC knowledge base. It stores documents, revisions, attachments and evidence around tasks and workflow phases.

Wiki does not own task state, workflow execution, CI/CD execution or Git hosting. UI and CLI are clients of the same public API.

## 2. Capability Map

| Capability | Owner Context | MVP |
|---|---|---|
| Auth and current user | Identity & Access | yes |
| Users and roles | Identity & Access | yes |
| Space management | Knowledge Base | yes |
| Space members | Identity & Access | yes |
| Document tree | Knowledge Base | yes |
| Markdown draft/edit | Knowledge Base | yes |
| Revision publishing | Knowledge Base | yes |
| Task links | SDLC Links | yes |
| Phase links | SDLC Links | yes |
| URL/file evidence | Evidence & Attachments | yes |
| Attachments | Evidence & Attachments | yes |
| Search | Search | yes |
| Templates | Administration | yes |
| Audit log | Administration | yes |

## 3. Core Flow

1. Admin creates a space and adds members.
2. Editor creates a document from a template or blank page.
3. Editor publishes a revision.
4. Editor links the document to a task key or phase key when needed.
5. Editor adds URL/file evidence to a document, task or phase.
6. Viewer opens the document, task page, phase page or search results within permissions.
7. Wiki records write actions in audit log.

## 4. Cross-cutting Invariants

- Published revisions are immutable.
- Drafts and published revisions are separate records.
- Permissions are checked by space.
- Search results are filtered by permissions.
- Files are stored outside PostgreSQL and referenced by storage key.
- External systems are represented only by keys, URLs and evidence metadata.

## 5. Deferred Capabilities

- Comments and mentions.
- Approval chains.
- Advanced reports.
- Import/export bundles.
- Real-time collaborative editing.

## 6. References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/DOMAIN_MODEL.md`
- `docs/DATA_MODEL.md`
- `docs/API.md`
