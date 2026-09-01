# System Administration - Wiki

## 1. Scope

System administration in Wiki MVP covers users, roles, a safe read-only instance settings snapshot, audit visibility, storage checks and operational health. It does not include plugin systems, notification delivery, report builders, external source sync, marketplace features or workflow execution controls.

## 2. Users

| Field          | Description                       |
| -------------- | --------------------------------- |
| `id`           | UUIDv7                            |
| `username`     | Unique login                      |
| `email`        | Unique email                      |
| `display_name` | User-visible name                 |
| `active`       | Whether the user can authenticate |
| `created_at`   | Creation timestamp                |
| `global_role`  | `admin` or `user`                 |

MVP user management supports:

- create user;
- deactivate user;
- reactivate user;
- update email, username, display name and global role.

## 3. Roles

| Role           | Scope    | Description                                          |
| -------------- | -------- | ---------------------------------------------------- |
| `system_admin` | instance | Full administrative access                           |
| `admin`        | space    | Manage members and space metadata                    |
| `editor`       | space    | Create drafts, publish documents and attach evidence |
| `viewer`       | space    | Read published documents and permitted evidence      |

Roles are explicit and auditable. Groups, LDAP, SAML and SCIM are deferred until after MVP.

## 4. Instance Settings

| Setting                | Source                            | Description                               |
| ---------------------- | --------------------------------- | ----------------------------------------- |
| `instance_name`        | fixed MVP value                   | Header/product name                       |
| `api_base_path`        | fixed MVP value                   | Public API base path                      |
| `default_space_key`    | bootstrap/default                 | Default SDLC knowledge space              |
| `registration_enabled` | `WIKI_AUTH__REGISTRATION_ENABLED` | Whether public registration is allowed    |
| `public_links_enabled` | fixed MVP value                   | Public document links are disabled in MVP |
| `search_backend`       | fixed MVP value                   | PostgreSQL full-text search               |
| `storage_backend`      | fixed MVP value                   | Local attachment storage adapter          |
| `max_upload_bytes`     | `WIKI_STORAGE__MAX_UPLOAD_BYTES`  | Per-file upload limit; default is 25 MiB  |
| `default_language`     | fixed MVP value                   | Default UI language                       |
| `timezone`             | fixed MVP value                   | Default displayed timezone                |

The current MVP exposes `GET /api/v1/settings` as an admin-only read-only snapshot. Editable settings and their audit entries require a separate requirement before implementation.

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
- future settings changes after editable settings are approved.

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
