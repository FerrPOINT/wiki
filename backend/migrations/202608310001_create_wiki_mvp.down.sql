DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS document_templates;
DROP TABLE IF EXISTS evidence_items;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS document_phase_links;
DROP TABLE IF EXISTS document_task_links;
DROP TABLE IF EXISTS phase_dossiers;
DROP TABLE IF EXISTS task_dossiers;
DROP TABLE IF EXISTS document_drafts;

ALTER TABLE IF EXISTS documents
    DROP CONSTRAINT IF EXISTS documents_current_revision_same_document_fk;

DROP TABLE IF EXISTS document_revisions;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS space_members;
DROP TABLE IF EXISTS spaces;
DROP TABLE IF EXISTS users;
