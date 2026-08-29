# Authorization Contract - Wiki

## 1. Principals

- User principal.
- Integration token principal.
- System admin principal.

## 2. Scope

Authorization scope is `space_id` first. Entity permissions resolve through the owning space.

## 3. Required Checks

| Operation | Permission |
|---|---|
| read document | `space.read` |
| create document | `document.create` |
| publish document | `document.publish` |
| archive document | `document.archive` |
| add evidence | `evidence.add` |
| upload attachment | `attachment.upload` |
| manage members | `space.manage` |
| read audit | `audit.read` |

## 4. Failure Semantics

- Return `401` when principal is missing/invalid.
- Return `403` when action is known but disallowed.
- Return `404` when revealing entity existence would leak data.
