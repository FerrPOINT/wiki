# ADR-0009: Canonical Registry And Source Priority

## Status

Accepted

## Context

Wiki stores SDLC knowledge, but it does not own task execution, boards, sprints or pipeline execution. The project needs one canonical vocabulary and authority order so external terms do not become internal product scope.

## Decision

Canonical Wiki nouns:

| Concept | Canon |
|---|---|
| Knowledge container | `space` |
| Page | `document` |
| Published version | `document_revision` |
| External task folder | `task_dossier` |
| Workflow phase folder | `phase_dossier` |
| Proof artifact/link/note | `evidence` |
| File metadata | `attachment` |
| External source reference | `source_reference` |
| Immutable action record | `audit_entry` |

Source priority:

1. Code, migrations and committed OpenAPI describe current runtime behavior.
2. ADRs define accepted architecture decisions.
3. `docs/contracts/*` define target observable contracts.
4. Narrative docs explain architecture and operations.
5. `docs/CURRENT_STATE.md` summarizes the verified snapshot.
6. Roadmap and plans are planning aids, not normative contracts.

## Consequences

- External tracker words may appear only as external source references, task keys or migration notes.
- Backend code must keep owned concepts in Wiki nouns.
- UI route names must follow `spaces`, `documents`, `tasks`, `phases`, `evidence`, `templates`, `audit-log`, `users`, `settings`, `search`, `admin`.
- OpenAPI must be regenerated after Wiki API implementation.
- Documentation reviews can reject stale vocabulary.

## References

- `docs/GLOSSARY.md`
- `docs/DOCUMENTATION_GOVERNANCE.md`
- `docs/architecture/transition-map.md`
- `docs/contracts/API_CONTRACT.md`
