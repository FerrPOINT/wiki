# Authorization - Wiki

## 1. Model

Authorization is space-first. A user receives permissions through space membership and optional system role.

## 2. Roles

| Role | Scope | Description |
|---|---|---|
| System admin | global | Full instance administration |
| Space owner | space | Manage settings, members, templates, retention |
| Editor | space | Create/edit/publish documents and evidence |
| Viewer | space | Read published documents and allowed evidence |

## 3. Permissions

| Permission | Owner | Editor | Viewer |
|---|---|---|---|
| `space.read` | yes | yes | yes |
| `space.manage` | yes | no | no |
| `document.create` | yes | yes | no |
| `document.update_draft` | yes | yes | no |
| `document.publish` | yes | yes | no |
| `document.archive` | yes | yes | no |
| `evidence.add` | yes | yes | no |
| `attachment.upload` | yes | yes | no |
| `audit.read` | yes | no | no |

## 4. Rules

- Repository queries must filter by authorized space.
- Missing permission should return `404` where entity existence would leak data.
- Scoped API tokens are deferred and must use the same space permissions as user sessions if approved later.
- Admin APIs require system admin even if user owns a space.

## 5. Tests

- Viewer cannot publish.
- Editor cannot manage members.
- User from another space cannot read document.
- A user from another space cannot write evidence or attachments into an inaccessible space.
