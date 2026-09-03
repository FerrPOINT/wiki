# Database Migrations — Wiki

## 1. Overview

Миграции управляют схемой PostgreSQL. Wiki-подход - **SQLx migrations**: plain SQL-файлы, применяемые через `sqlx migrate`, startup runner или thin `cargo run -p migration` runner.

Clean Wiki baseline lives in `backend/migrations/202608310001_create_wiki_mvp.*.sql` and creates a fresh MVP schema without task-tracker tables. `202608310002_add_auth_runtime.*.sql` adds usernames and auth session storage used by the SQLx runtime adapter. `202609030001_add_idempotency_records.*.sql` adds server-side replay storage for protected domain/admin writes with `Idempotency-Key`. `backend/migration` is a thin SQLx command wrapper over this directory, not a separate schema source.

## 2. Tooling

| Tool | Purpose |
|------|---------|
| `sqlx migrate` | Создание, применение и проверка SQL migrations |
| `cargo run -p migration -- up/status/down/fresh` | Local/CI runner над теми же SQLx migrations |
| `cargo build --bin openapi-gen` | Генерация OpenAPI spec |
| `psql` | Ручная диагностика схемы и индексов |

## 3. Folder Structure

Canonical Wiki migration set:

```
backend/migrations/
├── 202608310001_create_wiki_mvp.up.sql
├── 202608310001_create_wiki_mvp.down.sql
├── 202608310002_add_auth_runtime.up.sql
├── 202608310002_add_auth_runtime.down.sql
├── 202609030001_add_idempotency_records.up.sql
├── 202609030001_add_idempotency_records.down.sql
└── seeds/
```

`backend/migration` contains only the SQLx runner source. It must not define DDL or carry task-tracker migrations.

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
- Не добавлять ORM-specific migrations или task-tracker DDL.

## 6. Applying Migrations

Текущий startup runner в PostgreSQL runtime adapter читает canonical directory из
`WIKI_MIGRATIONS_DIR` или из соседнего `backend/migrations` относительно crate:

```rust
let migrator = sqlx::migrate::Migrator::new(migrations_dir()).await?;
migrator.run(&pool).await?;
```

Вручную через runner:

```bash
cd backend
DATABASE_URL=postgres://... cargo run -p migration -- up
DATABASE_URL=postgres://... cargo run -p migration -- status
```

Вручную через `sqlx-cli`:

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
| CI | `cargo run -p migration -- up/status` на clean PostgreSQL |
| production | startup migration runner или отдельный release step |
