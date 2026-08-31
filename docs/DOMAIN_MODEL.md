# Domain Model - Wiki

## 1. Bounded Contexts

| Контекст               | Ответственность                                  | Основные агрегаты                                |
| ---------------------- | ------------------------------------------------ | ------------------------------------------------ |
| Identity & Access      | Пользователи, роли, доступ к spaces              | User, SpaceMember                                |
| Knowledge Base         | Spaces, дерево страниц, документы, версии        | Space, Document, DocumentDraft, DocumentRevision |
| SDLC Links             | Связь документов/evidence с task key и phase key | TaskDossier, PhaseDossier                        |
| Evidence & Attachments | URL/file evidence и metadata файлов              | EvidenceItem, Attachment                         |
| Search                 | Индексация опубликованных документов             | SearchDocument                                   |
| Administration         | Audit, шаблоны и read-only runtime settings      | AuditEntry, DocumentTemplate                     |

## 2. Главные агрегаты

Кодовый baseline домена находится в `backend/domain/src/wiki.rs`. Старые task-tracker сущности остаются compatibility scaffold до замены app/infra слоя и не должны расширяться новыми Wiki capability.

### User

- Поля: `id`, `email`, `display_name`, `password_hash`, `role`, `is_active`.
- Инварианты:
  - email уникален;
  - пароль хранится только как hash;
  - disabled user не получает session.

### Space

- Поля: `id`, `key`, `name`, `description`, `owner_id`, `archived_at`.
- Инварианты:
  - `key` уникален глобально;
  - archived space не принимает новые documents/evidence;
  - доступ к space задаётся через `SpaceMember`.

### SpaceMember

- Поля: `space_id`, `user_id`, `role`, `joined_at`.
- Роли: `admin`, `editor`, `viewer`.
- Инварианты:
  - одна роль пользователя внутри одного space;
  - viewer не может создавать drafts, publish или evidence.

### Document

- Поля: `id`, `space_id`, `parent_id`, `slug`, `title`, `document_type`, `status`, `current_revision_id`, `owner_id`.
- Инварианты:
  - `(space_id, parent_id, slug)` уникален;
  - published document имеет `current_revision_id`;
  - archived document скрыт из обычного дерева.

### DocumentDraft

- Поля: `document_id`, `author_id`, `content_markdown`, `base_revision_id`, `updated_at`.
- Инварианты:
  - draft принадлежит одному document;
  - publish создаёт новую immutable revision;
  - draft content рендерится через sanitizer перед preview.

### DocumentRevision

- Поля: `id`, `document_id`, `version`, `title`, `content_markdown`, `content_html`, `content_text`, `content_checksum`, `summary`, `author_id`, `published_at`.
- Инварианты:
  - revision неизменяема после публикации;
  - version монотонно растёт внутри document;
  - `content_html` является производным от Markdown.

### TaskDossier

- Поля: `id`, `space_id`, `task_key`, `title_snapshot`, `external_url`.
- Инварианты:
  - один dossier на `(space_id, task_key)`;
  - Wiki не владеет статусом внешней задачи;
  - dossier показывает связанные documents и evidence.

### PhaseDossier

- Поля: `id`, `space_id`, `phase_key`, `phase_name`.
- Инварианты:
  - один dossier на `(space_id, phase_key)`;
  - Wiki не управляет переходами phase state;
  - dossier показывает связанные documents и evidence.

### EvidenceItem

- Поля: `id`, `space_id`, `task_dossier_id`, `phase_dossier_id`, `document_id`, `evidence_type`, `title`, `url`, `attachment_id`, `checksum`, `metadata`.
- Инварианты:
  - evidence связано минимум с document, task dossier или phase dossier;
  - `uploaded_file` evidence имеет attachment и не имеет url;
  - `external_url` evidence имеет url и не имеет attachment;
  - evidence другого space недоступно через связи текущего space.

### Attachment

- Поля: `id`, `space_id`, `owner_entity_type`, `owner_entity_id`, `file_name`, `content_type`, `size_bytes`, `storage_key`, `checksum`, `uploaded_by`, `uploaded_at`.
- Инварианты:
  - staged upload до создания evidence имеет пустые owner-поля;
  - claimed attachment имеет `space_id`, `owner_entity_type` и `owner_entity_id`;
  - bytes хранятся вне PostgreSQL;
  - download проверяет права на owner entity;
  - checksum вычисляется при загрузке.

### DocumentTemplate

- Поля: `id`, `space_id`, `name`, `document_type`, `content_markdown`, `is_active`.
- MVP document types: `requirements`, `research_note`, `implementation_note`, `test_plan`, `release_note`.

### AuditEntry

- Поля: `id`, `actor_id`, `action`, `entity_type`, `entity_id`, `diff`, `request_id`, `created_at`.
- Инварианты:
  - audit append-only;
  - secrets и bearer tokens не пишутся в audit;
  - write-действия создают audit entry в той же транзакционной границе, где это возможно.

## 3. Value Objects

| VO             | Пример               | Ограничения                                            |
| -------------- | -------------------- | ------------------------------------------------------ |
| `SpaceKey`     | `ENG`                | 2-32 uppercase letters, digits or hyphens              |
| `DocumentSlug` | `release-plan`       | 1-96 lowercase letters, digits or single hyphens       |
| `TaskKey`      | `SDLC-42`            | Non-empty external key without whitespace              |
| `PhaseKey`     | `implementation`     | 1-64 lowercase letters, digits, hyphens or underscores |
| `Checksum`     | `sha256:...`         | Target value object; baseline stores validated text    |
| `StorageKey`   | `documents/{id}/...` | Target value object; baseline stores validated text    |

## 4. Domain Events

| Событие                | Потребители   |
| ---------------------- | ------------- |
| `document.created`     | Audit         |
| `document.published`   | Audit, search |
| `document.archived`    | Audit, search |
| `document.moved`       | Audit         |
| `task_dossier.linked`  | Audit         |
| `phase_dossier.linked` | Audit         |
| `evidence.added`       | Audit         |
| `attachment.uploaded`  | Audit         |
| `space.member_changed` | Audit         |

## 5. Integration Boundary

- External task tracker, workflow and CI/CD systems остаются владельцами своих процессов.
- Wiki хранит только ключи, ссылки, snapshots и evidence, которые нужны для чтения истории работы.
- Любой внешний процесс использует публичный API или CLI, без отдельной доменной модели.

## 6. References

- `docs/PRODUCT_REQUIREMENTS.md`
- `docs/DATA_MODEL.md`
- `docs/API.md`
- `docs/ARCHITECTURE.md`
