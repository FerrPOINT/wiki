# UI/API Contract - Wiki

## 1. Rule

Target frontend uses generated OpenAPI types for API calls after the contract stabilizes. Current MVP keeps thin handwritten auth/Wiki clients, and the backend uses SQLx/PostgreSQL persistence when `WIKI_DATABASE__URL` is set.

## 2. Required UI States

Every API-backed page has:

- loading state;
- empty state;
- permission denied state;
- validation error display;
- retry or refresh action where useful.

## 3. Route Mapping

| UI Route | API Group |
|---|---|
| `/spaces` | spaces |
| `/documents/:documentId` | documents/revisions |
| `/documents/new` | documents/drafts |
| `/tasks/:taskKey` | task dossiers |
| `/phases/:phaseId` | phase dossiers/evidence |
| `/evidence` | evidence/attachments |
| `/templates` | document templates |
| `/search` | search |
| `/audit-log` | audit |
| `/users` | users/roles |
| `/settings` | instance settings |
| `/admin` | admin overview |
