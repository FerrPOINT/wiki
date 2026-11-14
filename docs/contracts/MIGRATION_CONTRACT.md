# Migration Contract - Wiki

## 1. Rules

- Migrations are versioned and ordered.
- Schema changes are reviewed with data model changes.
- Apply migrations explicitly during production release with `wiki-server migrate`;
  normal startup retains the same migration check and application bootstrap.
- Destructive migrations require backup and rollback note.
- Large indexes use concurrent creation where supported.

## 2. Required Evidence

- Fresh DB migration test.
- Existing DB migration test when changing released schema.
- Downgrade story or explicit irreversible note.
- Data model docs updated.
- `docs/MVP_READINESS.md` and `docs/CURRENT_STATE.md` updated when migration readiness changes.

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
- Table/index/constraint categories remain reflected in the data-model migration parity gate.

## 6. DB-only server command

`wiki-server migrate` loads the normal `AppConfig::from_env` configuration and
uses the same PostgreSQL pool options and SQLx migration owner as ordinary server
startup. `WIKI_DATABASE__URL` is required; its configured database/session schema
selection is preserved. `WIKI_MIGRATIONS_DIR` selects the canonical SQL directory;
when unset, the existing compiled backend/migrations path is used. This command
adds no schema override, new migration runner or alternate SQL catalog.

Successful migration closes the pool and exits with status 0. Failure exits with status 1. No document
seed, bootstrap user/space, attachment storage, reader preparation, HTTP listener
or background task is started. With no arguments the server serves normally;
`prepare-reader <subject-uuid> --token-file <private-file>` retains its existing
behavior. Unknown/incomplete commands and extra arguments exit with status 2 before config or
DB access. Database credentials and normal production configuration validations
remain required; the mode does not bypass them.
