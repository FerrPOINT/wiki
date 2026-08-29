# Traceability Matrix - Wiki

## 1. Purpose

Traceability links requirements to API, data model, UI, tests and evidence.

## 2. Matrix

| REQ | Capability | API | Data | UI Page | Tests |
|---|---|---|---|---|---|
| WIKI-REQ-001 | Spaces | `/spaces` | `spaces`, `space_members` | `/spaces` | API + component |
| WIKI-REQ-002 | Documents | `/documents` | `documents`, `document_drafts` | `/documents/:id` | API + editor |
| WIKI-REQ-003 | Revisions | `/documents/{id}/publish` | `document_revisions` | revision panel | publish flow |
| WIKI-REQ-004 | Task links | `/spaces/{space}/tasks/{task_key}` | `task_dossiers`, `document_task_links` | `/tasks/:taskKey` | API + component |
| WIKI-REQ-005 | Phase links | `/spaces/{space}/phases/{phase_key}` | `phase_dossiers`, `document_phase_links` | `/phases/:phaseId` | API + component |
| WIKI-REQ-006 | Evidence | `/evidence` | `evidence_items` | evidence feed | API + E2E |
| WIKI-REQ-007 | Attachments | `/attachments` | `attachments` | document/evidence panels | upload tests |
| WIKI-REQ-008 | Search | `/search` | search index | `/search` | search tests |
| WIKI-REQ-009 | Templates | `/templates` | `document_templates` | `/templates` | API + component |
| WIKI-REQ-010 | Audit | `/audit-log` | `audit_log` | `/audit-log` | policy tests |

## 3. Evidence Rule

A REQ is complete only when docs, API/schema, tests and user-facing route are all updated or explicitly marked not applicable.

## 4. Status Labels

| Status | Meaning |
|---|---|
| `Current` | Implemented and verified in repository |
| `Target approved` | Accepted requirement, not fully implemented |
| `Configuration only` | UI/config exists but runtime behavior is incomplete |
| `Blocked` | Cannot progress without dependency or decision |

## 5. Evidence Types

| Evidence | Examples |
|---|---|
| Docs | PRD, API doc, contract, ADR |
| Code | backend route/service, frontend page, CLI command |
| Tests | unit, integration, E2E, visual smoke |
| Screens | screenshot manifest and README gallery |
| Ops | runbook, metrics, alert, backup/restore drill |

## 6. Review Checklist

- Every P0/P1 row has a user-facing route or explicit API-only note.
- Every implemented route has at least one linked requirement.
- Every screenshot in README is listed in `assets/screens/manifest.md`.
- Target rows do not claim current behavior without tests or manual verification.
