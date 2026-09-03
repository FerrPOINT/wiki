# Authorization - Wiki

## 1. Model

Authorization is space-first. A user receives permissions through space membership and optional system role.

## 2. Roles

| Role         | Scope  | Description                                   |
| ------------ | ------ | --------------------------------------------- |
| System admin | global | Full instance administration                  |
| Space admin  | space  | Manage space metadata and members             |
| Editor       | space  | Create/edit/publish documents and evidence    |
| Viewer       | space  | Read published documents and allowed evidence |

## 3. Permissions

| Permission              | Admin | Editor | Viewer |
| ----------------------- | ----- | ------ | ------ |
| `space.read`            | yes   | yes    | yes    |
| `space.manage`          | yes   | no     | no     |
| `document.create`       | yes   | yes    | no     |
| `document.update_draft` | yes   | yes    | no     |
| `document.publish`      | yes   | yes    | no     |
| `document.archive`      | yes   | yes    | no     |
| `evidence.add`          | yes   | yes    | no     |
| `attachment.stage`      | yes   | yes    | yes    |
| `attachment.claim`      | yes   | yes    | no     |
| `audit.read`            | yes   | no     | no     |

## 4. Rules

- Repository queries must filter by authorized space.
- Document read responses expose draft content only to users who can edit the document; viewers receive published body fields and `can_edit=false`.
- Missing permission should return `404` where entity existence would leak data.
- Scoped API tokens are deferred and must use the same space permissions as user sessions if approved later.
- Instance admin APIs such as `/users`, `/settings` and `/audit-log` require system admin even if the user is a space admin.

## 5. Tests

- Viewer cannot publish.
- Viewer cannot read unpublished document draft content through `GET /documents/{document_id}`.
- Editor cannot manage members.
- User from another space cannot read document.
- Viewer can stage attachment upload but cannot claim it as file evidence without edit rights.
- User from another space cannot read documents, evidence, search results or claimed attachments from an inaccessible space.
