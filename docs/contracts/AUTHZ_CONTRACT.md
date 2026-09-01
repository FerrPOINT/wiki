# Authorization Contract - Wiki

## 1. Principals

- User principal.
- System admin principal.

Scoped API-token principals are deferred and require a separate contract update.

## 2. Scope

Authorization scope is `space_id` first. Entity permissions resolve through the owning space.

## 3. Required Checks

| Operation                                | Permission         |
| ---------------------------------------- | ------------------ |
| read document                            | `space.read`       |
| create document                          | `document.create`  |
| publish document                         | `document.publish` |
| archive document                         | `document.archive` |
| add URL evidence                         | `evidence.add`     |
| stage attachment upload                  | `attachment.stage` |
| claim staged attachment as file evidence | `attachment.claim` |
| manage members                           | `space.manage`     |
| read audit                               | `audit.read`       |

## 4. Failure Semantics

- Return `401` when principal is missing/invalid.
- Return `403` when action is known but disallowed.
- Return `404` when revealing entity existence would leak data.
