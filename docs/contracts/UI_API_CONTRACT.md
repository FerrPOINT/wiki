# UI/API Contract - Wiki

## 1. Rule

Current frontend uses generated OpenAPI DTO types for API calls plus thin handwritten auth/Wiki endpoint wrappers. The backend uses SQLx/PostgreSQL persistence when `WIKI_DATABASE__URL` is set. Full operation-client generation remains a target state after the app/infra repository boundary stabilizes.

## 2. Required UI States

Every API-backed page has:

- loading state;
- empty state;
- permission denied state;
- validation error display;
- retry or refresh action where useful.

## 3. Route Mapping

| UI Route                 | API Group                            |
| ------------------------ | ------------------------------------ |
| `/spaces`                | spaces                               |
| `/documents/:documentId` | documents/revisions                  |
| `/documents/new`         | documents/drafts                     |
| `/tasks/:taskKey`        | task dossiers                        |
| `/phases/:phaseId`       | phase dossiers/evidence              |
| `/evidence`              | evidence/attachments                 |
| `/templates`             | document templates                   |
| `/search`                | search                               |
| `/audit-log`             | audit                                |
| `/users`                 | users/roles                          |
| `/settings`              | `GET /settings`                      |
| `/admin`                 | users/spaces/audit/settings overview |
