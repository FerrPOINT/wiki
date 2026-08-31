# Database Migrations — Wiki

## 1. Overview

Миграции управляют схемой PostgreSQL. Используется **SeaORM Migrator** (`sea-orm-migration` 1.1). Миграции — Rust-файлы с типизированным API, регистрируются в `migration/src/lib.rs`.

## 2. Tooling

| Tool | Purpose |
|------|---------|
| `sea-orm-migration` | Применение миграций при старте сервера |
| `cargo build --bin openapi-gen` | Генерация OpenAPI spec |
| `sea-orm-cli generate entity` | Генерация сущностей из схемы (опционально) |

## 3. Folder Structure

Target Wiki migration set:

```
backend/migration/src/
├── lib.rs                        # Migrator registration
├── m20260828_000001_identity.rs
├── m20260828_000002_spaces.rs
├── m20260828_000003_documents.rs
├── m20260828_000004_document_tree.rs
├── m20260828_000005_task_phase_links.rs
├── m20260828_000006_evidence_attachments.rs
├── m20260828_000007_templates.rs
├── m20260828_000008_audit.rs
├── m20260828_000009_search_indexes.rs
└── m20260828_000010_permissions_indexes.rs
```

The current repository still contains inherited task-tracker migration files. They are not the target Wiki schema and must be replaced or quarantined during backend migration before OpenAPI is regenerated.

## 4. Naming Convention

```
m{YYYYMMDD}_{NNNNNN}_{description}.rs
```

- Дата — дата создания миграции.
- NNNNNN — порядковый номер (6 цифр), строго последовательный.
- Description — snake_case.
- Пример: `m20260826_0000027_fk_indexes.rs`.

## 5. Migration Rules

### 5.1 Must

- Каждая миграция регистрируется в `migration/src/lib.rs` (`Vec<Box<dyn MigrationTrait>>`).
- Все изменения обратимы или безопасны для отката (`down` метод).
- Добавлять новые колонки nullable или с default.
- Создавать индексы concurrently в production.

### 5.2 Must Not

- Не изменять существующие миграции после коммита — только новая миграция.
- Не удалять миграции из `lib.rs` — только помечать как deprecated.
- Не использовать raw SQL без необходимости — предпочитать SeaORM API.

## 6. Applying Migrations

Миграции применяются автоматически при старте сервера:

```rust
// backend/infra/src/db.rs
migration::Migrator::up(&db_conn, None).await?;
```

Вручную (через migration CLI):

```bash
# Применить все миграции
DATABASE_URL=postgres://... cargo run -p migration -- up

# Откатить последнюю
DATABASE_URL=postgres://... cargo run -p migration -- down

# Пересоздать БД + применить все миграции
DATABASE_URL=postgres://... cargo run -p migration -- fresh

# Проверить статус
DATABASE_URL=postgres://... cargo run -p migration -- status
```

CI проверяет применение всех миграций на чистой PostgreSQL (job `migrations`).

## 7. History Table

`seaql_migrations` — автоматически создаётся SeaORM Migrator. Хранит версию и контрольную сумму каждой применённой миграции.

## 8. Creating a New Migration

```bash
# Создать файл
touch backend/migration/src/m20260101_0000028_description.rs
```

Шаблон:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // DDL operations
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rollback
    }
}
```

Регистрация в `lib.rs`:

```rust
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // ...
            Box::new(m20260101_0000028_description::Migration),
        ]
    }
}
```

## 9. Production

- Миграции применяются при старте сервера автоматически.
- В production откатываются через **compensating migration**, а не `down`.
- Перед деплоем: тест на пустой БД (`cargo run -p migration -- fresh`).
- Резервная копия перед миграцией обязательна.

## 10. Environments

| Environment | When |
|-------------|------|
| local | `Migrator::up` при старте dev-сервера |
| CI | `Migrator::up` на testcontainers PostgreSQL |
| production | `Migrator::up` при старте backend контейнера |
