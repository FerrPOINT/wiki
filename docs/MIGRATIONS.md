# Database Migrations — Wiki

## 1. Overview

Миграции управляют схемой PostgreSQL. Wiki-подход - **SQLx migrations**: plain SQL-файлы, применяемые через `sqlx migrate` или thin migration runner в backend startup.

Clean Wiki baseline lives in `backend/migrations/202608310001_create_wiki_mvp.*.sql` and creates a fresh MVP schema without task-tracker tables. Текущий `backend/migration` на SeaORM унаследован из `task-tracker` и остаётся только compatibility/quarantine layer до удаления старых infra-модулей; его нельзя расширять новыми Wiki capability.

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
├── 202608310001_create_wiki_mvp.up.sql
├── 202608310001_create_wiki_mvp.down.sql
└── seeds/
```

The current repository still contains inherited task-tracker migration files under `backend/migration`. They are not the target Wiki schema and must be removed or isolated after SQLx repositories replace the old infra layer.

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

- Каждая migration pair содержит атомарный DDL-блок для одной логической области.
- Для local/test есть `.down.sql`; production rollback идёт через compensating migration.
- Добавлять новые колонки nullable или с explicit default/backfill plan.
- Создавать индексы concurrently в production.
- Использовать явный SQL, который можно проверить ревью и EXPLAIN.

### 5.2 Must Not

- Не изменять существующие миграции после коммита — только новая миграция.
- Не редактировать уже применённые migration-файлы.
- Не удалять старые migration-файлы без fresh-schema решения и отдельного migration note.
- Не добавлять новые SeaORM migrations для Wiki-сущностей.

## 6. Applying Migrations

Целевой startup runner после подключения к PostgreSQL:

```rust
// target shape in backend/infra
sqlx::migrate!("./migrations").run(&pool).await?;
```

Вручную:

```bash
# Применить все миграции
DATABASE_URL=postgres://... sqlx migrate run --source backend/migrations

# Добавить новую миграцию после baseline
sqlx migrate add -r -s backend/migrations document_revision_restore

# Проверить статус
DATABASE_URL=postgres://... sqlx migrate info --source backend/migrations
```

CI проверяет применение всех миграций на чистой PostgreSQL (job `migrations`).

## 7. History Table

`_sqlx_migrations` создаётся SQLx. Таблица хранит version, description, checksum и время применения.

## 8. Creating a New Migration

```bash
sqlx migrate add -r -s backend/migrations document_revision_restore
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
