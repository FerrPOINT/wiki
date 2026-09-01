# Migration Execution Spec - Wiki

## 1. Goal

Create and evolve the Wiki schema safely and repeatably through reviewed SQLx migrations.

## 2. Required Tables

Initial Wiki MVP migration must create:

- `users`
- `auth_sessions`
- `spaces`
- `space_members`
- `documents`
- `document_drafts`
- `document_revisions`
- `task_dossiers`
- `phase_dossiers`
- `document_task_links`
- `document_phase_links`
- `attachments`
- `evidence_items`
- `document_templates`
- `audit_log`

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
