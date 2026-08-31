# API Edge Cases - Wiki

## 1. Overview

Нестандартные сценарии API и ожидаемое поведение для Wiki: документы, версии, task/phase links, evidence, attachments, search и access control.

## 2. Auth

| Scenario                               | Behavior                                       |
| -------------------------------------- | ---------------------------------------------- |
| Expired access token                   | `401`, клиент делает refresh                   |
| Invalid refresh token                  | `401`, требуется повторный login               |
| Refresh token reused                   | Инвалидируем token family и пишем audit event  |
| User removed from space during session | Следующий запрос к space возвращает `403`      |
| Password changed elsewhere             | Все refresh tokens пользователя инвалидируются |

## 3. Document Edits

| Scenario                                  | Behavior                                                    |
| ----------------------------------------- | ----------------------------------------------------------- |
| Two users publish from same base revision | `409 REVISION_CONFLICT` with current revision metadata      |
| User edits archived document              | `409 DOCUMENT_ARCHIVED`                                     |
| Slug already exists under same parent     | `409 SLUG_EXISTS`                                           |
| Parent document belongs to another space  | `422 INVALID_PARENT`                                        |
| Move parent under its own descendant      | `400 VALIDATION_ERROR`; existing parent stays unchanged     |
| Markdown contains unsafe HTML             | HTML sanitized; blocked fragments listed in warning details |
| Empty title after trim                    | `422 VALIDATION_ERROR`                                      |

## 4. Dossiers

| Scenario                                     | Behavior                                         |
| -------------------------------------------- | ------------------------------------------------ |
| Same task key linked twice                   | Existing task page/link is returned idempotently |
| Task key has invalid format for the space    | `422 INVALID_TASK_KEY`                           |
| Same document linked to the same phase twice | Existing phase link is returned idempotently     |
| Phase key has invalid format for the space   | `422 INVALID_PHASE_KEY`                          |
| Link crosses space boundary                  | `422 SPACE_BOUNDARY_VIOLATION`                   |

## 5. Evidence

| Scenario                             | Behavior                                                             |
| ------------------------------------ | -------------------------------------------------------------------- |
| Evidence references missing artifact | `422 ARTIFACT_NOT_FOUND` for local storage, warning for external URL |
| Duplicate CI artifact sent twice     | Idempotency key or checksum dedup prevents duplicate                 |
| Checksum mismatch on upload          | `409 CHECKSUM_MISMATCH`                                              |
| External URL is private/unreachable  | Save URL, mark verification state as `unverified`                    |
| Evidence delete requested            | Soft archive plus audit; hard delete only by retention policy        |

## 6. Files

| Scenario                   | Behavior                         |
| -------------------------- | -------------------------------- |
| File exceeds limit         | `413 PAYLOAD_TOO_LARGE`          |
| MIME type mismatch         | `400 MIME_MISMATCH`              |
| Unsafe filename            | Sanitized or rejected with `422` |
| Storage quota exceeded     | `507 INSUFFICIENT_STORAGE`       |
| Object storage unavailable | `503 STORAGE_UNAVAILABLE`        |

## 7. Search

| Scenario                               | Behavior                                     |
| -------------------------------------- | -------------------------------------------- |
| Empty query                            | Return recent documents visible to user      |
| Query too broad                        | Cursor pagination, max limit enforced        |
| Archived document match                | Hidden unless `include_archived=true`        |
| User lacks access to matching document | Result is filtered out                       |
| Index lag                              | API may include `index_lag_seconds` metadata |

## 8. Database

| Scenario                    | Behavior                                   |
| --------------------------- | ------------------------------------------ |
| Unique constraint violation | `409` with stable code                     |
| Foreign key violation       | `422` with dependency hint                 |
| Connection pool exhausted   | `503`, retry recommended                   |
| Serialization conflict      | `409`, client may retry idempotent request |

## 9. References

- `docs/API.md`
- `docs/ERROR_HANDLING.md`
- `docs/TESTING.md`
- `docs/RESILIENCE.md`
