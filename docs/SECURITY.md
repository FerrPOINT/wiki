# Security — Wiki

## 1. Overview

Wiki — self-hosted приложение с конфиденциальными данными проектов. Безопасность встроена на всех уровнях: transport, auth, storage, application, operations.

## 2. Authentication

- Passwords hashed with **argon2id**.
- JWT access token (15 min) + httpOnly refresh cookie (7 days, rotation).
- User deactivation revokes existing access and refresh sessions; reactivation requires a new login.
- Failed login lockout после 5 попыток на 15 минут.
- MFA/TOTP — не реализовано (future).
- OAuth/OpenID/LDAP — не реализовано (future).

## 3. Authorization

- Role-based access control (RBAC) per space.
- Document-level security schemes (future).
- Permission checks на service layer, повторно — на repository layer.
- No data returned until permission verified.

## 4. Transport

- HTTPS/TLS everywhere в production.
- HSTS header.
- Secure, SameSite=Lax/Strict, httpOnly cookies.
- No sensitive data в URL query params.

## 5. Input Validation

- Strict DTO validation на входе backend/frontend validators.
- Для attachments проверяются размер, непустой `content_type` и безопасное имя файла; strict MIME allowlist остаётся deferred policy, если продукту потребуется ограничивать классы файлов.
- Filename sanitization для download headers и storage keys.
- SQL только через parameterized queries.
- No `eval`, no dynamic SQL.

## 6. XSS / CSP

- CSP target policy:
  ```
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: blob: {storage-origin};
  connect-src 'self' {api-origin};
  font-src 'self';
  object-src 'none';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
  ```
- API runtime sets the MVP self-hosted CSP with `img-src 'self' data: blob:`, `object-src 'none'`, `frame-ancestors 'none'`, `base-uri 'self'` and `form-action 'self'`. Dedicated external storage origins must be added only when a non-local storage adapter is introduced.
- User-generated content escaped при render.
- Markdown рендерится через controlled renderer, HTML проходит sanitizer.

## 7. CSRF

- SameSite cookies.
- Stateless CSRF token для mutation endpoints при необходимости.

## 8. CORS

- Strict whitelist:
  ```
  WIKI_CORS_ALLOWED_ORIGINS=https://wiki.example.com
  ```
- No wildcard (`*`) в production.

## 9. Secrets Management

- All secrets via env vars.
- No secrets in git.
- `.env.example` contains placeholders only.
- Rotate JWT/refresh secrets periodically.
- Database credentials separate from app config.

## 10. File Upload Security

- Size limits per type.
- Magic bytes validation.
- ClamAV virus scan — не реализовано (future).
- Quarantine bucket — не реализовано (future).
- No direct execution of uploaded files.

## 11. Rate Limiting

- `tower_governor` per IP and per user.
- Stricter limits for auth endpoints.
- Stricter limits for auth and upload endpoints.

| Endpoint | Limit |
|----------|-------|
| Login | 5/min |
| Register | 3/min |
| API general | 100/min |
| Search | 60/min |

## 12. Audit Logging

- Login/logout events.
- Permission changes.
- Space/member/role modifications.
- Admin actions.
- Stored in `audit_log` table, retained 1 year.

## 13. Dependency Security

- `cargo audit --ignore RUSTSEC-2023-0071` в CI; ignore documented because SQLx keeps optional MySQL/RSA packages in `Cargo.lock` while Wiki enables PostgreSQL-only SQLx features.
- `h2` advisory is resolved through `h2` `0.4.16`.
- `pnpm audit` policy is a release-hardening backlog item until frontend dependency thresholds are fixed.
- Dependabot/Renovate alerts.
- Pin major versions.

## 14. Container Security

- MVP images contain only runtime artifacts and must not bake secrets into image layers.
- Backend image copies SQLx migrations to `/app/migrations` and uses `WIKI_MIGRATIONS_DIR`.
- Backend image runs the API process as a dedicated non-root `wiki` user (`10001:10001`) and owns `/var/lib/wiki/uploads` for attachment storage.
- Read-only filesystem, distroless backend image and Trivy scan are release-hardening items.

## 15. Network

- PostgreSQL доступен только в internal network.
- Traefik на edge.
- Firewall: expose only frontend/reverse-proxy ports and required backend admin ports; keep PostgreSQL internal.

## 16. Incident Response

- Rotate compromised secrets.
- Revoke sessions via logout or user deactivation in the admin panel.
- Block users.
- Export audit log.

## 17. Security Headers

API responses set and test the baseline browser/security headers:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
Content-Security-Policy: ...
```

## 18. Penetration Testing

- Internal security review перед релизом.
- OWASP ZAP scan в CI.
- Bug bounty — future.

## 19. Data Privacy

- No personal data in logs.
- GDPR/CCPA delete account endpoint (future).
- Data retention policies.

## 20. References

- `docs/API.md` — auth flow.
- `docs/SYSTEM_ADMIN.md` — users/groups/permissions.
- `docs/STORAGE.md` — attachment security.
- `docs/ERROR_HANDLING.md` — error disclosure.
- `docs/ARCHITECTURE.md`
- `docs/DEPLOYMENT.md`
