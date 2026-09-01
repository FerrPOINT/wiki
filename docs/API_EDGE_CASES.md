# API Edge Cases - Wiki

## 1. Overview

Нестандартные сценарии API и ожидаемое поведение для Wiki: документы, версии, task/phase links, evidence, attachments, search и access control.

Этот документ описывает текущий MVP-контракт. Богатые domain-specific коды можно добавить позже, но текущий runtime возвращает единый envelope `{ "error": { "code", "message" } }` с кодами `VALIDATION_ERROR`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `CONFLICT` и `INTERNAL_ERROR`.

## 2. Auth

| Scenario                               | Behavior                                  |
| -------------------------------------- | ----------------------------------------- |
| Missing/invalid access token           | `401 UNAUTHORIZED`                        |
| Invalid refresh token                  | `401 UNAUTHORIZED`, требуется новый login |
| Registration disabled                  | `403 FORBIDDEN`                           |
| User removed from space during session | Следующий запрос к space возвращает `403` |
| Viewer tries write action              | `403 FORBIDDEN`                           |

## 3. Document Edits

| Scenario                                 | Behavior                                                |
| ---------------------------------------- | ------------------------------------------------------- |
| Empty title after trim                   | `400 VALIDATION_ERROR`                                  |
| Empty publish content                    | `400 VALIDATION_ERROR`                                  |
| Document not found                       | `404 NOT_FOUND`                                         |
| Slug already exists under same space     | `409 CONFLICT`                                          |
| Archived document write                  | `400 VALIDATION_ERROR`; draft/publish/move are rejected |
| Parent document belongs to another space | `400 VALIDATION_ERROR`                                  |
| Move parent under itself/descendant      | `400 VALIDATION_ERROR`; existing parent stays unchanged |
| Markdown contains unsafe HTML            | HTML is sanitized before render/search projection       |

## 4. Dossiers

| Scenario                                     | Behavior                                         |
| -------------------------------------------- | ------------------------------------------------ |
| Same task key linked twice                   | Existing task page/link is returned idempotently |
| Task key has invalid format                  | `400 VALIDATION_ERROR`                           |
| Same document linked to the same phase twice | Existing phase link is returned idempotently     |
| Phase key has invalid format                 | `400 VALIDATION_ERROR`                           |
| Link crosses space boundary                  | `400 VALIDATION_ERROR`                           |

## 5. Evidence

| Scenario                                      | Behavior                                                       |
| --------------------------------------------- | -------------------------------------------------------------- |
| Evidence has no document/task/phase target    | `400 VALIDATION_ERROR`                                         |
| `external_url` has no URL                     | `400 VALIDATION_ERROR`                                         |
| `external_url` includes attachment/checksum   | `400 VALIDATION_ERROR`                                         |
| `uploaded_file` has no attachment             | `400 VALIDATION_ERROR`                                         |
| `uploaded_file` includes URL/client checksum  | `400 VALIDATION_ERROR`                                         |
| Missing, already claimed or чужой attachment  | `404 NOT_FOUND`                                                |
| Evidence document and explicit space mismatch | `400 VALIDATION_ERROR`                                         |
| Evidence has document but no explicit space   | API uses the document's space                                  |
| External URL is private/unreachable           | URL is stored as user-supplied evidence; verification deferred |
| Evidence delete requested                     | Deferred; MVP creates/lists evidence only                      |

## 6. Files

| Scenario                                             | Behavior                                      |
| ---------------------------------------------------- | --------------------------------------------- |
| Empty multipart upload                               | `400 VALIDATION_ERROR`                        |
| File exceeds configured limit                        | `400 VALIDATION_ERROR`                        |
| Unsafe filename                                      | Sanitized for download name or rejected `400` |
| Staged file downloaded by non-owner                  | `403 FORBIDDEN` before it is claimed          |
| Claimed file downloaded by user without space access | `403 FORBIDDEN`                               |
| Object storage unavailable                           | `500 INTERNAL_ERROR`; operator checks storage |

## 7. Search

| Scenario                               | Behavior                                  |
| -------------------------------------- | ----------------------------------------- |
| Empty query                            | Return recent visible documents/evidence  |
| Query too broad                        | Bounded `limit` is enforced               |
| Archived document match                | Hidden unless `include_archived=true`     |
| User lacks access to matching document | Result is filtered out                    |
| Invalid space filter                   | `400 VALIDATION_ERROR` or `403 FORBIDDEN` |

## 8. Database

| Scenario                    | Behavior                                                  |
| --------------------------- | --------------------------------------------------------- |
| Unique constraint violation | `409 CONFLICT`                                            |
| Missing dependency          | `404 NOT_FOUND` when mapped before write                  |
| Unmapped database error     | `500 INTERNAL_ERROR`; infrastructure details stay in logs |
| Serialization conflict      | `500 INTERNAL_ERROR` until retryable conflicts are mapped |

## 9. Settings And Operations

| Scenario | Behavior |
| -------- | -------- |
| Non-admin requests settings | `403 FORBIDDEN` |
| Settings response would include secret-like value | Field is omitted from response and logs |
| API process is live but dependencies are not ready | `/api/v1/health` succeeds, `/api/v1/health/ready` returns not ready |
| Metrics endpoint is unavailable behind edge proxy | API contract remains valid; operator checks deployment routing |

## 10. References

- `docs/API.md`
- `docs/ERROR_HANDLING.md`
- `docs/MVP_READINESS.md`
- `docs/TESTING.md`
- `docs/RESILIENCE.md`
