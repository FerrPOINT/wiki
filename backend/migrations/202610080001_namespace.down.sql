DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM wiki_namespace_bindings)
       OR EXISTS (SELECT 1 FROM spaces WHERE namespace_managed)
       OR EXISTS (SELECT 1 FROM task_dossiers WHERE tracker_instance_id IS NOT NULL OR task_id IS NOT NULL)
       OR EXISTS (SELECT 1 FROM managed_task_document_links)
    THEN
        RAISE EXCEPTION 'namespace migration cannot be downgraded while managed namespace data exists; use a compensating migration';
    END IF;
END $$;

DROP TRIGGER IF EXISTS wiki_namespace_resource_marker ON wiki_namespace_bindings;
DROP FUNCTION IF EXISTS wiki_mark_namespace_resource();

DO $$
DECLARE table_name text;
BEGIN
    FOREACH table_name IN ARRAY ARRAY['spaces','space_members','documents','document_revisions','document_drafts',
        'task_dossiers','phase_dossiers','document_task_links','document_phase_links','evidence_items','attachments','managed_task_document_links']
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS namespace_write_guard ON %I', table_name);
    END LOOP;
END $$;

DROP FUNCTION IF EXISTS wiki_namespace_write_guard();
DROP TABLE IF EXISTS managed_task_document_links;
DROP TABLE IF EXISTS wiki_namespace_bindings;
ALTER TABLE spaces DROP COLUMN IF EXISTS namespace_managed;
ALTER TABLE task_dossiers DROP CONSTRAINT IF EXISTS task_ref_complete;
DROP INDEX IF EXISTS task_dossiers_legacy_space_key_idx;
DROP INDEX IF EXISTS task_dossiers_managed_task_idx;
ALTER TABLE task_dossiers DROP COLUMN IF EXISTS task_id;
ALTER TABLE task_dossiers DROP COLUMN IF EXISTS tracker_instance_id;
CREATE UNIQUE INDEX IF NOT EXISTS task_dossiers_space_key_idx ON task_dossiers(space_id, task_key);
