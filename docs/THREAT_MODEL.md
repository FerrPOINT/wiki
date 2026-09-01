# Threat Model - Wiki

## 1. Assets

- Document content and revision history.
- Evidence artifacts and checksums.
- Attachment bytes and storage keys.
- Search index and derived plain text.
- Session tokens.
- Space membership and permissions.
- Audit log.
- Backup archives.
- Runtime secrets.

## 2. Trust Boundaries

| Boundary | Risk |
|---|---|
| Browser -> API | XSS, CSRF, token theft |
| CLI -> API | leaked bearer token, unsafe local output path |
| API -> PostgreSQL | SQL injection, overbroad queries |
| API -> local storage | path traversal, public file exposure |
| API -> search index | cross-space data leakage through derived text |
| Operator -> backup storage | leaked dumps, stale credentials |

## 3. Actors

| Actor | Capability | Concern |
| ----- | ---------- | ------- |
| Anonymous user | login/register only | brute force, registration abuse |
| Viewer | read published content in allowed spaces | horizontal access attempts |
| Editor | create/update documents and evidence | stored XSS, unsafe uploads, accidental data exposure |
| Space admin | manage members in a space | privilege mistakes, unauthorized role escalation |
| System admin | manage users/settings/audit | broad blast radius, secret handling |
| External script | call API through CLI/token | token leakage, repeated writes |

## 4. Key Threats

| Threat | Mitigation |
|---|---|
| Cross-space data leak | service-layer authz, repository filters, tests |
| Stored XSS in Markdown | render with `comrak`, sanitize with `ammonia`, CSP |
| Search result leakage | index only authorized document projections and filter every query by visible spaces |
| Attachment path traversal | generated storage keys, filename sanitization, no direct filesystem paths in API |
| Malicious uploads | size limit, content-type validation, extension policy, optional AV scan |
| URL evidence abuse | store URL as metadata only; do not server-fetch arbitrary URLs in MVP |
| Evidence tampering | immutable events, checksums, audit |
| Token misuse | short-lived access tokens, revocation, audit |
| Brute force login/register | auth rate limits and disabled-registration policy |
| CSRF on mutations | SameSite/httpOnly cookies and CSRF token if cookie-auth mutations are enabled |
| Oversized Markdown or uploads | request limits, upload quotas, bounded rendering/search work |
| Audit/log disclosure | redact secrets and avoid raw bearer tokens, storage paths or DB URLs in logs |
| Backup leakage | encrypt backup storage and test restore without exposing dumps to application users |

## 5. Controls By Area

### Auth And Access

- All protected endpoints require a valid session/JWT.
- Space membership is checked before reading documents, evidence, attachments, trees and search results.
- Admin endpoints require system admin rights.
- Logout/revocation invalidates refresh/session state.

### Documents And Markdown

- Markdown source remains the canonical document body.
- Rendered HTML is derived and sanitized before display.
- Published revisions are immutable, so historical content cannot be silently rewritten.
- Moving pages must preserve the acyclic tree invariant and same-space boundary.

### Evidence And Attachments

- `external_url` evidence stores links but does not require backend crawling or webhook ingestion.
- `uploaded_file` evidence must reference a staged attachment with checksum metadata.
- Attachment download checks access through the owning document/task/phase context.
- Unsafe filenames, empty files and oversized uploads are rejected.

### Search And Audit

- Search is scoped by caller-visible spaces and supported filters.
- Draft content is not exposed through general search until published.
- Audit entries are append-only and must not include secrets or raw uploaded file bytes.

## 6. Verification

- Backend authz tests cover viewer/editor/admin boundaries.
- PostgreSQL smoke covers membership revocation and search index-plan evidence.
- Upload tests cover empty file, unsafe filename and configured size limits.
- Frontend tests cover permission/error states where role-specific behavior is visible.
- README/manifest screenshots verify that no deferred reports, notifications or integrations pages are exposed.

## 7. Review Triggers

- New file type allowed for upload.
- Public sharing feature.
- Document-level permissions.
- Server-side URL fetching, unfurling, previews or imports.
- New search source or binary attachment indexing.
- New webhook, notification, report or runner capability.
- Changes to auth token lifetime, cookie policy or registration policy.
- Backup/restore tooling change.

## 8. Deferred Security Work

- Malware scanning and quarantine for high-risk deployments.
- MFA/OIDC/LDAP if the product needs enterprise identity.
- Document-level permissions beyond space RBAC.
- Signed public links and external sharing policy.
- Server-side URL preview/unfurling, if later approved.
