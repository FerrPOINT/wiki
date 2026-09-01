# Traceability Matrix - Wiki

## 1. Purpose

Traceability links requirements to API, data model, UI, CLI, tests and evidence. The 100% pre-development readiness gate is defined in `docs/MVP_READINESS.md`.

## 2. Matrix

| REQ           | Capability       | API                                                                   | Data                                                       | UI/CLI                                                        | Verification                    |
| ------------- | ---------------- | --------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------- |
| REQ-AUTH-001  | Auth             | `/auth/login`, `/auth/logout`, `/users/me`                            | `users`, sessions/tokens                                   | `/login`, account menu, `wiki auth`                           | auth API + UI smoke             |
| REQ-AUTH-002  | Roles            | `/users`, `/spaces/{space_key}/members`                               | `users`, `space_members`, `audit_log`                      | `/users`, `/admin`, `wiki user`, `wiki space member-set/remove` | RBAC policy tests               |
| REQ-AUTH-003  | Registration     | `/auth/register`, `/auth/refresh`                                     | `users`, sessions/tokens                                   | `/register`                                                   | auth API + register component   |
| REQ-SPC-001   | Spaces           | `/spaces`, `/spaces/{space_key}`                                      | `spaces`, `space_members`                                  | `/spaces`, `wiki space list/get`                              | API + component                 |
| REQ-SPC-002   | Space management | `/spaces`, `/spaces/{space_key}`, `/spaces/{space_key}/archive`       | `spaces`, `audit_log`                                      | `/spaces`, `/settings`, `/admin`, `wiki space create/update/archive` | admin API tests                 |
| REQ-SPC-003   | Space members    | `/spaces/{space_key}/members`                                         | `space_members`, `audit_log`                               | `/users`, `wiki space members/member-set/member-remove`       | RBAC integration tests          |
| REQ-DOC-001   | Documents        | `/spaces/{space_key}/documents`                                       | `documents`, `document_drafts`                             | `/documents/new`, `wiki doc create`                           | create API + editor test        |
| REQ-DOC-002   | Document view    | `/documents/{document_id}`                                            | `documents`, `document_revisions`                          | `/documents/:documentId`, `wiki doc get`                      | view API + visual smoke         |
| REQ-DOC-003   | Draft edit       | `/documents/{document_id}/draft`                                      | `document_drafts`                                          | `/documents/:documentId`, `wiki doc draft`                    | draft API + editor test         |
| REQ-DOC-004   | Publish          | `/documents/{document_id}/publish`                                    | `document_revisions`, `audit_log`                          | revision panel, `wiki doc publish`                            | publish flow test               |
| REQ-DOC-005   | Revision history | `/documents/{document_id}/revisions?limit=20`                         | `document_revisions`                                       | revision panel, `wiki doc history --limit`, `wiki doc revision` | revision API limit/history tests |
| REQ-DOC-006   | Archive          | `/documents/{document_id}/archive`                                    | `documents`, `audit_log`                                   | document actions, `wiki doc archive`                          | archive visibility tests        |
| REQ-TREE-001  | Page tree        | `/spaces/{space_key}/tree`                                            | `documents.parent_id`                                      | `/spaces`, `wiki space tree`                                  | tree API + component            |
| REQ-TREE-002  | Move page        | `/documents/{document_id}/move`                                       | `documents.parent_id`, `audit_log`                         | document actions, `wiki doc move`                             | move validation tests           |
| REQ-TASK-001  | Task link        | `/spaces/{space_key}/tasks/{task_key}/links/documents`                | `task_dossiers`, `document_task_links`                     | `/tasks/:taskKey`, `wiki task link-doc`                       | link API tests                  |
| REQ-TASK-002  | Task page        | `/spaces/{space_key}/tasks/{task_key}`                                | `task_dossiers`, links, evidence                           | `/tasks`, `/tasks/:taskKey`, `wiki task get/docs/evidence`    | dossier API + visual smoke      |
| REQ-PHASE-001 | Phase link       | `/spaces/{space_key}/phases/{phase_key}/links/documents`, `/evidence` | `phase_dossiers`, `document_phase_links`, `evidence_items` | `/phases/:phaseId`, `wiki phase link-doc`                     | phase link tests                |
| REQ-PHASE-002 | Phase page       | `/spaces/{space_key}/phases/{phase_key}`                              | `phase_dossiers`, links, evidence                          | `/phases`, `/phases/:phaseId`, `wiki phase get/docs/evidence` | dossier API + visual smoke      |
| REQ-EVID-001  | Link evidence    | `/evidence`                                                           | `evidence_items`                                           | `/evidence`, `wiki evidence add-link`                         | evidence API tests              |
| REQ-EVID-002  | File evidence    | `/attachments`, `/attachments/{attachment_id}/download`, `/evidence`  | `attachments`, `evidence_items`                            | `/evidence`, `wiki evidence add-file`, `wiki attachment get/download` | upload/download tests           |
| REQ-EVID-003  | Evidence list    | `/evidence?limit=30`, owner evidence endpoints                        | `evidence_items`                                           | `/evidence`, document/task/phase pages, `wiki evidence list --limit` | list/filter/limit tests         |
| REQ-SRCH-001  | Search           | `/search?limit=20`                                                    | PostgreSQL FTS projection                                  | `/search`, `wiki search query --limit`                        | search API limit tests          |
| REQ-SRCH-002  | Search filters   | `/search` query filters                                               | FTS projection + ACL joins                                 | `/search` filters, `wiki search query` filters                | permission/filter tests         |
| REQ-TPL-001   | Templates        | `/templates`                                                          | `document_templates`                                       | `/templates`, `wiki template list/create/apply`               | template API + component        |
| REQ-SET-001   | Settings         | `/settings`                                                           | runtime config snapshot                                    | `/settings`, `/admin`, `wiki settings get`                    | settings API + UI/CLI smoke     |
| REQ-OPS-001   | Runtime health   | `/health`, `/health/ready`                                            | API runtime readiness state                                | API-only deployment probes; no frontend page                  | health/readiness API tests      |
| REQ-AUD-001   | Audit            | `/audit-log`                                                          | `audit_log`                                                | `/audit-log`, `/admin`, `wiki audit list --limit`             | audit write/limit tests         |
| REQ-API-001   | API              | `/api/v1`, `openapi/openapi.json`                                     | DTOs + OpenAPI schemas                                     | UI and CLI clients                                            | OpenAPI parity check            |
| REQ-CLI-001   | CLI              | Same `/api/v1` endpoints                                              | none direct                                                | `wiki` binary                                                 | CLI command tests               |
| REQ-UI-001    | UI               | Same `/api/v1` endpoints                                              | none direct                                                | MVP route set in `docs/ROUTING.md`                            | Vitest, Playwright, screenshots |

## 3. Readiness Coverage Rule

For pre-development readiness, every MVP capability must have all of these fields accounted for in `docs/MVP_READINESS.md`:

| Field | Meaning |
| ----- | ------- |
| User scenario | What a person or operator does with the capability |
| API | Public endpoint group or explicit API-only artifact |
| CLI | Matching CLI group or explicit API-only exception |
| UI | Frontend route, page area or explicit no-route note |
| Data | Primary table/entity or runtime state |
| Evidence | Test class, screenshot, manifest or operational check |

## 4. Evidence Rule

A REQ is complete only when docs, API/schema, tests and user-facing route are all updated or explicitly marked not applicable.

## 5. Status Labels

| Status               | Meaning                                             |
| -------------------- | --------------------------------------------------- |
| `Current`            | Implemented and verified in repository              |
| `Target approved`    | Accepted requirement, not fully implemented         |
| `Configuration only` | UI/config exists but runtime behavior is incomplete |
| `Blocked`            | Cannot progress without dependency or decision      |

## 6. Evidence Types

| Evidence | Examples                                          |
| -------- | ------------------------------------------------- |
| Docs     | PRD, API doc, contract, ADR                       |
| Code     | backend route/service, frontend page, CLI command |
| Tests    | unit, integration, E2E, visual smoke              |
| Screens  | screenshot manifest and README gallery            |
| Ops      | runbook, metrics, alert, backup/restore drill     |

## 7. Review Checklist

- Every P0/P1 row has a user-facing route or explicit API-only note.
- Every implemented route has at least one linked requirement.
- Every screenshot in README is listed in `assets/screens/manifest.md`.
- Every row in `docs/MVP_READINESS.md` has a matching requirement, API or explicit API-only rationale.
- API-only probes, metrics and OpenAPI are not required to have UI routes or screenshots.
- Target rows do not claim current behavior without tests or manual verification.
