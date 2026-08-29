# Migration Contract - Wiki

## 1. Rules

- Migrations are versioned and ordered.
- Schema changes are reviewed with data model changes.
- No application startup schema bootstrap in production.
- Destructive migrations require backup and rollback note.
- Large indexes use concurrent creation where supported.

## 2. Required Evidence

- Fresh DB migration test.
- Existing DB migration test when changing released schema.
- Downgrade story or explicit irreversible note.
- Data model docs updated.

## 3. Migration Classes

| Class | Example | Required Review |
|---|---|---|
| Additive | new table, nullable column, new index | backend owner |
| Backfill | populate dossier links or search metadata | backend + operator |
| Constraint | new unique/FK/check constraint | backend + product owner |
| Destructive | drop column/table/object key rewrite | architecture + operator approval |

## 4. Compatibility

- Application version N must tolerate old data during rolling update when rolling update is supported.
- Generated OpenAPI/client update follows backend compatibility, not the other way around.
- Long-running backfills must be resumable.
- Failed migration leaves a clear operator diagnostic.

## 5. Acceptance Criteria

- Migration applies to empty database.
- Migration applies to previous released database fixture.
- Rollback/restore instruction exists for destructive or irreversible changes.
- `DATA_MODEL.md`, `DATABASE_INDEXES.md` and `CURRENT_STATE.md` are updated.
