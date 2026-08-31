# System Administration - Wiki

## 1. Scope

System administration in Wiki MVP covers users, roles, instance settings, audit visibility, storage checks and operational health. It does not include plugin systems, notification delivery, report builders, external source sync, marketplace features or workflow execution controls.

## 2. Users

| Field | Description |
|---|---|
| `id` | UUIDv7 |
| `username` | Unique login |
| `email` | Unique email |
| `display_name` | User-visible name |
| `active` | Whether the user can authenticate |
| `locale` | `ru` or `en` |
| `timezone` | User timezone |
| `created_at` | Creation timestamp |
| `last_login_at` | Last successful login |

MVP user management supports:

- create user;
- deactivate user;
- reactivate user;
- update display name, locale and timezone;
- reset password through an admin action or documented manual operation.

## 3. Roles

| Role | Scope | Description |
|---|---|---|
| `system_admin` | instance | Full administrative access |
| `space_owner` | space | Manage members, templates and settings in owned spaces |
| `editor` | space | Create drafts, publish documents and attach evidence |
| `viewer` | space | Read published documents and permitted evidence |

Roles are explicit and auditable. Groups, LDAP, SAML and SCIM are deferred until after MVP.

## 4. Instance Settings

| Setting | Default | Description |
|---|---|---|
| `application_title` | `Wiki` | Header/product name |
| `base_url` | `http://localhost:19877` | Public frontend URL |
| `default_locale` | `ru` | Default UI locale |
| `default_timezone` | `Europe/Moscow` | Default timezone |
| `default_role` | `viewer` | Space role for invited users unless overridden |
| `public_links_enabled` | `false` | Public document links are disabled in MVP |
| `max_attachment_size` | `50 MiB` | Per-file upload limit |

Settings changes require `system_admin` and produce audit entries.

## 5. Authentication

MVP authentication uses:

- local email/password login;
- Argon2id password hashing;
- short-lived access token;
- refresh token/session storage;
- logout and session revocation.

OAuth/OIDC, SAML, LDAP, TOTP and passwordless login are deferred.

## 6. Audit Log

System administrators can inspect:

- login/logout events;
- user create/update/deactivate events;
- role and membership changes;
- document create/edit/publish/archive events;
- evidence create/archive events;
- settings changes.

Audit entries are append-only. Any retention change requires a separate security decision.

## 7. Backup And Restore

Administrative backup scope:

- PostgreSQL database;
- attachment object storage;
- environment/config snapshot without plaintext secrets;
- OpenAPI and documentation artifacts from git.

Restore must validate attachment checksums and report missing storage objects.

## 8. Health And Maintenance

System admin and operators monitor:

- `/api/v1/health`;
- target `/api/v1/health/ready`;
- `/metrics`;
- PostgreSQL connectivity;
- storage availability;
- search freshness;
- failed uploads;
- audit write failures.

Maintenance actions are operational procedures first. Dedicated UI controls can be added after backend MVP is stable.

## 9. Deferred

Not in MVP:

- plugin marketplace or runtime plugin API;
- notification center and email delivery;
- inbound or outbound webhooks;
- report builders;
- import/export bundles;
- external identity provider sync;
- workflow execution or runner controls.

## 10. References

- `docs/AUTHORIZATION.md`
- `docs/SECURITY.md`
- `docs/OPERATIONS.md`
- `docs/BACKUP_RESTORE.md`
- `docs/PAGE_DESIGN.md`
