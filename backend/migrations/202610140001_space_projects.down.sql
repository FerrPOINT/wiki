-- Reverting is safe only before the new relationship model has been used.
DO $$ BEGIN
    IF EXISTS(SELECT 1 FROM wiki_space_projects l WHERE NOT EXISTS(
        SELECT 1 FROM wiki_namespace_bindings b WHERE b.resource_id=l.space_id
        AND b.registry_instance_id=l.registry_instance_id AND b.namespace_id=l.namespace_id))
    OR EXISTS(SELECT 1 FROM wiki_namespace_bindings b WHERE NOT EXISTS(
        SELECT 1 FROM wiki_space_projects l WHERE l.space_id=b.resource_id
        AND l.registry_instance_id=b.registry_instance_id AND l.namespace_id=b.namespace_id))
    OR EXISTS(SELECT 1 FROM managed_task_document_links l WHERE NOT EXISTS(
        SELECT 1 FROM wiki_namespace_bindings b WHERE b.resource_id=l.space_id
        AND b.registry_instance_id=l.registry_instance_id AND b.namespace_id=l.namespace_id)) THEN
        RAISE EXCEPTION 'space_projects_rollback_would_discard_associations';
    END IF;
END $$;
DROP TABLE wiki_space_projects;
DROP INDEX managed_task_document_links_project_idx;
ALTER TABLE managed_task_document_links DROP COLUMN registry_instance_id;
ALTER TABLE managed_task_document_links DROP COLUMN namespace_id;
CREATE OR REPLACE FUNCTION wiki_namespace_write_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data jsonb; sid uuid; sids uuid[] := '{}'; lifecycle text;
BEGIN
    IF TG_TABLE_NAME='spaces' AND TG_OP='UPDATE' THEN
        IF OLD.namespace_managed AND NOT NEW.namespace_managed THEN RAISE EXCEPTION 'namespace_marker_immutable' USING ERRCODE='42501'; END IF;
        IF NOT OLD.namespace_managed AND NEW.namespace_managed AND to_jsonb(OLD)-'namespace_managed'=to_jsonb(NEW)-'namespace_managed' AND EXISTS(SELECT 1 FROM wiki_namespace_bindings WHERE resource_id=NEW.id) THEN RETURN NEW; END IF;
    END IF;
    FOR data IN SELECT value FROM jsonb_array_elements(CASE TG_OP WHEN 'INSERT' THEN jsonb_build_array(to_jsonb(NEW)) WHEN 'DELETE' THEN jsonb_build_array(to_jsonb(OLD)) ELSE jsonb_build_array(to_jsonb(OLD),to_jsonb(NEW)) END)
    LOOP
        IF TG_TABLE_NAME='spaces' THEN sid := (data->>'id')::uuid;
        ELSIF data ? 'space_id' THEN sid := (data->>'space_id')::uuid;
        ELSE SELECT space_id INTO sid FROM documents WHERE id=(data->>'document_id')::uuid;
        END IF;
        sids := array_append(sids,sid);
    END LOOP;
    IF EXISTS(SELECT 1 FROM spaces s WHERE s.id=ANY(sids) AND s.namespace_managed AND NOT EXISTS(SELECT 1 FROM wiki_namespace_bindings b WHERE b.resource_id=s.id)) THEN
        RAISE EXCEPTION 'namespace_projection_missing' USING ERRCODE='42501';
    END IF;
    IF TG_TABLE_NAME='document_revisions' AND TG_OP<>'INSERT' AND EXISTS(SELECT 1 FROM spaces WHERE id=ANY(sids) AND namespace_managed) THEN
        RAISE EXCEPTION 'published_revision_immutable' USING ERRCODE='42501';
    END IF;
    FOR lifecycle IN SELECT state FROM wiki_namespace_bindings WHERE resource_id=ANY(sids) ORDER BY resource_id FOR SHARE
    LOOP
        IF lifecycle <> 'active' OR (TG_TABLE_NAME='spaces' AND TG_OP='DELETE') THEN
            RAISE EXCEPTION 'namespace_resource_read_only' USING ERRCODE='42501';
        END IF;
    END LOOP;
    IF TG_OP='DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END $$;
