# Database Migrations — Wiki

## 1. Overview

Миграции управляют схемой PostgreSQL. Целевой Wiki-подход - **SQLx migrations**: plain SQL-файлы, применяемые через `sqlx migrate` или thin migration runner в backend startup.

Текущий `backend/migration` на SeaORM унаследован из `task-tracker` и не является целевой Wiki-схемой. Его нельзя расширять новыми Wiki capability; на этапе backend migration он должен быть заменён clean SQLx migration set или явно изолирован как compatibility layer.

## 2. Tooling

| Tool | Purpose |
|------|---------|
| `sqlx migrate` | Создание, применение и проверка SQL migrations |
| `cargo build --bin openapi-gen` | Генерация OpenAPI spec |
| `psql` | Ручная диагностика схемы и индексов |

## 3. Folder Structure

Target Wiki migration set:

```
backend/migrations/
├── 20260828000001_identity.sql
├── 20260828000002_spaces.sql
├── 20260828000003_documents.sql
├── 20260828000004_document_tree.sql
├── 20260828000005_task_phase_links.sql
├── 20260828000006_evidence_attachments.sql
├── 20260828000007_templates.sql
├── 20260828000008_audit.sql
├── 20260828000009_search_indexes.sql
├── 20260828000010_permissions_indexes.sql
└── seeds/
```

The current repository still contains inherited task-tracker migration files. They are not the target Wiki schema and must be replaced or quarantined during backend migration before OpenAPI is regenerated.

## 4. Naming Convention

```
{YYYYMMDDHHMMSS}_{description}.sql
```

- Дата — дата создания миграции.
- Timestamp - монотонный UTC timestamp.
- Description - snake_case.
- Пример: `20260828000006_evidence_attachments.sql`.

## 5. Migration Rules

### 5.1 Must

- Каждая миграция содержит атомарный DDL-блок для одной логической области.
- Все изменения обратимы через отдельную compensating migration или безопасны для повторного применения в test reset flow.
- Добавлять новые колонки nullable или с explicit default/backfill plan.
- Создавать индексы concurrently в production.
- Использовать явный SQL, который можно проверить ревью и EXPLAIN.

### 5.2 Must Not

- Не изменять существующие миграции после коммита — только новая миграция.
- Не редактировать уже применённые migration-файлы.
- Не удалять старые migration-файлы без fresh-schema решения и отдельного migration note.
- Не добавлять новые SeaORM migrations для Wiki-сущностей.

## 6. Applying Migrations

Миграции применяются автоматически при старте сервера после подключения к PostgreSQL:

```rust
// target shape in backend/infra
sqlx::migrate!("./migrations").run(&pool).await?;
```

Вручную:

```bash
# Применить все миграции
DATABASE_URL=postgres://... sqlx migrate run --source backend/migrations

# Добавить новую миграцию
sqlx migrate add -r -s backend/migrations evidence_attachments

# Проверить статус
DATABASE_URL=postgres://... sqlx migrate info --source backend/migrations
```

CI проверяет применение всех миграций на чистой PostgreSQL (job `migrations`).

## 7. History Table

`_sqlx_migrations` создаётся SQLx. Таблица хранит version, description, checksum и время применения.

## 8. Creating a New Migration

```bash
sqlx migrate add -r -s backend/migrations document_revisions
```

## 9. Production

- Миграции применяются при старте сервера автоматически.
- В production откатываются через **compensating migration**, а не `down`.
- Перед деплоем: тест на пустой БД (`sqlx database reset` или testcontainer fresh run).
- Резервная копия перед миграцией обязательна.

## 10. Environments

| Environment | When |
|-------------|------|
| local | `sqlx migrate run` при старте dev-сервера |
| CI | `sqlx migrate run` на testcontainers PostgreSQL |
| production | startup migration runner или отдельный release step |
