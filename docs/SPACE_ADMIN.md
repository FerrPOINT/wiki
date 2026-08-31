# Space Administration - Wiki

## 1. Purpose

Space administration covers permissions and templates for a Wiki space.

## 2. Space Settings

| Field | Description |
|---|---|
| `key` | Stable short key, for example `ENG` |
| `name` | Human-readable name |
| `description` | Purpose of the space |
| `owner_id` | Space owner |
| `default_template_id` | Optional default document template |

## 3. Roles

| Role | Description |
|---|---|
| `admin` | Full control over space settings and members |
| `editor` | Create, edit, publish documents and evidence |
| `viewer` | Read published documents and allowed evidence |

## 4. Permissions

| Permission | Admin | Editor | Viewer |
|---|---|---|---|
| View published documents | yes | yes | yes |
| Create documents | yes | yes | no |
| Edit drafts | yes | yes | no |
| Publish revisions | yes | yes | no |
| Archive documents | yes | yes | no |
| Manage members | yes | no | no |
| Manage templates | yes | no | no |
| Attach evidence | yes | yes | no |

## 5. Templates

Recommended default templates:

- Requirements.
- Research note.
- Implementation note.
- Test plan.
- Release note.

## 6. Deferred

Not part of MVP:

- external source sync;
- notification settings and delivery;
- import/export bundles;
- retention policy UI.

## 7. Retention

- Archived documents stay searchable only with `include_archived=true`.
- Audit log retention should be longer than document retention.

## 8. References

- `docs/DATA_MODEL.md`
- `docs/SECURITY.md`
- `docs/API.md`
