# Database Standards

## 1. СУБД

- PostgreSQL 17.6+.
- Тип UUID — `UUID`.
- Для Wiki-owned таблиц PK создаётся приложением как UUIDv7; database default для PK не задаём, чтобы не смешивать версии UUID.

## 2. Миграции

- Целевой инструмент - `sqlx migrate` и plain SQL migrations по ADR-0001.
- Унаследованный `backend/migration` на SeaORM относится к task-tracker scaffold и должен быть заменён или изолирован до реализации Wiki persistence.
- Имя файла: `YYYYMMDDHHMMSS_description.up.sql` и `YYYYMMDDHHMMSS_description.down.sql`.
- Каждая миграция:
  - выполняется транзакционно через SQLx, если явно не указан no-transaction case
  - имеет обратный `.down.sql` для local/test reset
  - не удаляет данные без `WHERE` и бэкапа
- Запрещено:
  - изменять уже применённую миграцию
  - удалять столбцы с данными без explicit migration step
  - использовать `SELECT *` в миграциях

## 3. Именование

| Объект | Конвенция | Пример |
|---|---|---|
| Таблица | snake_case, множественное число | `document_revisions` |
| Столбец | snake_case | `created_at` |
| PK | `id` | `id UUID PRIMARY KEY` |
| FK | `<table>_id` | `space_id` |
| Индекс | `<table>_<columns>_idx` | `documents_space_slug_idx` |
| Constraint | `<table>_<columns>_<type>` | `documents_space_slug_unique` |
| Enum | `enum_<name>` | `enum_document_status` |

## 4. Типы данных

| Назначение | Тип | Примечание |
|---|---|---|
| ID | `UUID` | application-supplied UUIDv7 |
| Timestamp | `TIMESTAMPTZ` | всегда UTC |
| JSON | `JSONB` | для неструктурированных/расширяемых данных |
| Перечисления | `TEXT` + check / native `enum` | для маленьких стабильных списков — native enum; для часто меняющихся — lookup table |
| Деньги | `NUMERIC(19,4)` | если потребуется |
| Длительность | `INTERVAL` | reserved for future timing metadata |
| IP | `INET` | audit log |

## 5. Индексы

- Каждый FK — индекс.
- Частые фильтры и сортировки — покрывающие индексы.
- GIN для `JSONB` полей, по которым идёт поиск.
- Уникальные индексы для бизнес-ключей (`spaces_key_idx`, `documents_root_slug_idx`, `documents_child_slug_idx`, `task_dossiers_space_key_idx`).

## 6. Soft delete

- По умолчанию — hard delete.
- Для критичных сущностей (`documents`, `spaces`, `evidence_items`) - `archived_at TIMESTAMPTZ` + partial unique index.
- Восстановление архивных документов реализуется через status/archived metadata и audit log.

## 7. Constraints

- `NOT NULL` по умолчанию для обязательных полей.
- `DEFAULT` только для технических полей (`created_at`, `id`).
- `ON DELETE`:
  - `CASCADE` - для явно дочерних сущностей (`document_drafts` к `documents`)
  - `RESTRICT` - для ссылок на справочники, если удаление нарушает целостность
  - `SET NULL` - для опциональных FK (`summary_document_id`)

## 8. Partitioning

- Кандидаты на партиционирование:
  - `audit_log` — по `created_at` (range)
- До 10M+ строк не партиционируем.

## 9. SQL style

- Ключевые слова — uppercase.
- Идентификаторы — lowercase.
- Запросы форматировать с переносами:

```sql
SELECT d.id, d.title, r.version
FROM documents d
JOIN document_revisions r ON r.id = d.current_revision_id
WHERE d.space_id = $1
  AND d.archived_at IS NULL
ORDER BY d.updated_at DESC
LIMIT 50;
```

## 10. Seeds и fixtures

- Seed-данные для dev — `backend/migrations/seeds/`.
- Fixtures для тестов — `backend/tests/fixtures/`.
- Продакшен defaults (admin user, base spaces/templates) создаются seed/bootstrap flow, а не смешиваются с DDL baseline.

## 11. References

- `docs/DATA_MODEL.md` — полная физическая модель.
- `docs/MIGRATIONS.md` — процесс миграций.
- `docs/DATABASE_INDEXES.md` — перечень индексов.
- `docs/ARCHITECTURE.md` — persistence layer.
