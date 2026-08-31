# Project Administration - Wiki

## 1. Purpose

Project administration is deferred reference material. Wiki MVP manages spaces directly; future project mapping may connect spaces with external SDLC project keys, CI/CD identifiers, repositories and workflow phase rules.

`SPACE_ADMIN.md` describes MVP space administration. This document keeps CI/CD-style documentation parity and does not define MVP routes or API requirements.

## 2. Project Mapping

| Wiki Object   | External Mapping                                                    |
| ------------- | ------------------------------------------------------------------- |
| Space         | Product/team/project area                                           |
| Task dossier  | External task key                                                   |
| Phase dossier | project-workflow phase instance                                     |
| Source link   | Repository/PR/commit                                                |
| Evidence      | URL or uploaded file for CI/CD artifact, deployment or review proof |

One space may contain multiple external projects. One external project should map to one default space unless migration requires a temporary split.

## 3. Admin Capabilities

Future project administrators may:

- create and archive project mappings;
- configure task key patterns;
- configure default document templates per phase;
- configure required evidence by phase;
- map CI/CD project identifiers and repository URLs;
- manage external source names;
- view import errors for approved external source sync;
- view project-level audit events.

They cannot bypass system-wide retention, security, audit or secret policies.

## 4. Permissions

| Role                 | Capability                                                    |
| -------------------- | ------------------------------------------------------------- |
| System admin         | All spaces and mappings                                       |
| Space admin          | Mappings inside administered spaces                           |
| Editor               | Create documents/evidence inside permitted spaces             |
| Viewer               | View permitted documents and evidence                         |
| Future service token | Deferred; scoped source writes only after a separate approval |

All project admin actions produce audit entries.

## 5. Required Fields

| Field               | Description                         |
| ------------------- | ----------------------------------- |
| `space_id`          | Owning Wiki space                   |
| `project_key`       | Human-readable external project key |
| `source_system`     | Source system name                  |
| `display_name`      | UI label                            |
| `base_url`          | Optional source URL                 |
| `task_key_pattern`  | Optional validation regexp          |
| `default_templates` | Template ids by document/phase type |
| `required_evidence` | Evidence rules by phase             |

## 6. Operational Rules

- Archive disables new automatic dossier creation but does not delete existing documents.
- Changing `task_key_pattern` does not rewrite historical dossier keys.
- Deleting a mapping requires migration or explicit archive-first policy.
- Future source-sync failures are visible through audit or a separately approved operator view.

## 7. Acceptance Criteria

- A system admin can see all mappings.
- A space admin can configure mappings only for administered spaces.
- An approved source sync can create a task page in the configured default space.
- Future reporting can show evidence coverage by project mapping.
- Archived mappings remain visible in audit and historical documents.

## References

- `docs/SPACE_ADMIN.md`
- `docs/AUTHORIZATION.md`
- `docs/WEBHOOKS.md`
- `docs/contracts/AUTHZ_CONTRACT.md`
