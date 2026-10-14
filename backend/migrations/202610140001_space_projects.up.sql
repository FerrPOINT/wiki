-- Wiki owns its spaces and these associations. A project is an external identity,
-- not the parent or lifecycle owner of the space.
CREATE TABLE wiki_space_projects (
    space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE RESTRICT,
    registry_instance_id uuid NOT NULL CHECK (registry_instance_id <> '00000000-0000-0000-0000-000000000000'),
    namespace_id uuid NOT NULL CHECK (namespace_id <> '00000000-0000-0000-0000-000000000000'),
    created_by uuid REFERENCES users(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (space_id, registry_instance_id, namespace_id)
);
CREATE INDEX wiki_space_projects_project_idx
    ON wiki_space_projects (registry_instance_id, namespace_id, space_id);

-- Retain existing space/document IDs and owner-command receipts. Receipts record
-- provisioning; they no longer define the space's project collection.
INSERT INTO wiki_space_projects (space_id, registry_instance_id, namespace_id, created_at)
SELECT b.resource_id, b.registry_instance_id, b.namespace_id, s.created_at
FROM wiki_namespace_bindings b JOIN spaces s ON s.id = b.resource_id;

-- A historical task link must remember the task's actual project even after the
-- space is detached from that project or attached to a second project.
ALTER TABLE managed_task_document_links ADD COLUMN registry_instance_id uuid;
ALTER TABLE managed_task_document_links ADD COLUMN namespace_id uuid;
-- This migration holds the table lock; lifecycle guards must not reject the
-- metadata backfill of historical links in archived spaces.
ALTER TABLE managed_task_document_links DISABLE TRIGGER namespace_write_guard;
UPDATE managed_task_document_links l
SET registry_instance_id = b.registry_instance_id, namespace_id = b.namespace_id
FROM wiki_namespace_bindings b WHERE b.resource_id = l.space_id;
ALTER TABLE managed_task_document_links ENABLE TRIGGER namespace_write_guard;
ALTER TABLE managed_task_document_links ALTER COLUMN registry_instance_id SET NOT NULL;
ALTER TABLE managed_task_document_links ALTER COLUMN namespace_id SET NOT NULL;
ALTER TABLE managed_task_document_links ADD CONSTRAINT managed_link_project_non_nil CHECK (registry_instance_id <> '00000000-0000-0000-0000-000000000000' AND namespace_id <> '00000000-0000-0000-0000-000000000000');
CREATE INDEX managed_task_document_links_project_idx
    ON managed_task_document_links (registry_instance_id, namespace_id, dossier_id);

CREATE OR REPLACE FUNCTION wiki_namespace_write_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data jsonb; sid uuid; sids uuid[] := '{}';
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
    IF TG_TABLE_NAME='spaces' AND TG_OP='DELETE' AND EXISTS(SELECT 1 FROM spaces WHERE id=ANY(sids) AND namespace_managed) THEN
        RAISE EXCEPTION 'namespace_resource_read_only' USING ERRCODE='42501';
    END IF;
    -- Project archive does not archive a shared space. Its own archive continues
    -- to freeze content, including direct SQL writes racing with an API request.
    IF TG_TABLE_NAME<>'spaces' THEN
        PERFORM id FROM spaces WHERE id=ANY(sids) ORDER BY id FOR SHARE;
    END IF;
    IF TG_TABLE_NAME<>'spaces' AND EXISTS(SELECT 1 FROM spaces WHERE id=ANY(sids) AND archived_at IS NOT NULL) THEN
        RAISE EXCEPTION 'space_read_only' USING ERRCODE='42501';
    END IF;
    IF TG_OP='DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER space_projects_write_guard BEFORE INSERT OR UPDATE OR DELETE ON wiki_space_projects
    FOR EACH ROW EXECUTE FUNCTION wiki_namespace_write_guard();
