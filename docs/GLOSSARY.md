# Glossary - Wiki

| Term | Meaning |
|---|---|
| Space | Logical area for documents and permissions |
| Document | Wiki page with draft and published revisions |
| Revision | Immutable published version of a document |
| Draft | Editable unpublished document content |
| Task dossier | Knowledge folder linked to an external task |
| Phase dossier | Knowledge folder for a workflow phase |
| Evidence | URL or file proving work was done |
| Attachment | File stored by the Wiki storage adapter |
| Source system | External origin such as task tracker, workflow or CI/CD |
| Current | Verified capability in repository |
| Target | Approved but not fully implemented capability |

## Extended Terms

| Term | Meaning |
|---|---|
| Review | Deferred human verification workflow, not MVP |
| Template | Reusable document structure for requirements and notes |
| Source link | Repository, commit, branch or PR reference stored as metadata |
| Integration source | Deferred external system connection |
| Webhook delivery | Deferred outgoing signed event |
| Workflow documentation DSL | Deferred rules for required documents/evidence by phase |
| Runner | Deferred background worker concept, not required for MVP |
| Dead letter | Deferred failed async job queue |
| Published revision | Immutable document version visible as current or historical content |

## Naming Rules

- Use `space`, not project, when Wiki owns the container.
- Use `task dossier`, not issue, when Wiki stores external task knowledge.
- Use `phase dossier`, not sprint, when Wiki stores workflow-phase evidence.
- Use `evidence`, not artifact, when the record can be a file or URL.
