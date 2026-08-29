# Threat Model - Wiki

## 1. Assets

- Document content and revision history.
- Evidence artifacts and checksums.
- Session tokens.
- Space membership and permissions.
- Audit log.
- Runtime secrets.

## 2. Trust Boundaries

| Boundary | Risk |
|---|---|
| Browser -> API | XSS, CSRF, token theft |
| API -> PostgreSQL | SQL injection, overbroad queries |
| API -> local storage | path traversal, public file exposure |

## 3. Key Threats

| Threat | Mitigation |
|---|---|
| Cross-space data leak | service-layer authz, repository filters, tests |
| Stored XSS in Markdown | render with `comrak`, sanitize with `ammonia`, CSP |
| Malicious uploads | MIME sniffing, extension blocklist, optional AV scan |
| Evidence tampering | immutable events, checksums, audit |
| Token misuse | short-lived access tokens, revocation, audit |

## 4. Review Triggers

- New file type allowed for upload.
- Public sharing feature.
- Document-level permissions.
