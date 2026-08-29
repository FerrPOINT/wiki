# Migration Execution Spec - Wiki

## 1. Goal

Replace inherited task-tracker schema with Wiki schema safely and repeatably.

## 2. Required Tables

Initial Wiki migration must create:

- `users`
- `spaces`
- `space_members`
- `documents`
- `document_drafts`
- `document_revisions`
- `task_dossiers`
- `phase_dossiers`
- `evidence_items`
- `attachments`
- `document_templates`
- `tags`
- `document_tags`
- `comments`
- `audit_log`
- `outbox_events`

## 3. Execution Rules

- Migrations run once and are tracked by version.
- No runtime `CREATE TABLE IF NOT EXISTS` outside migrations.
- Add indexes with the feature that uses them.
- Destructive changes require backup/restore plan.
- Seed only non-secret baseline data.

## 4. Validation

- Fresh database migrates successfully.
- Existing database refuses incompatible downgrade.
- Integration tests verify constraints and indexes.
