ALTER TABLE spaces ADD COLUMN namespace_managed boolean NOT NULL DEFAULT false;
CREATE TABLE wiki_namespace_bindings (
    resource_id uuid PRIMARY KEY REFERENCES spaces(id) ON DELETE RESTRICT,
    registry_instance_id uuid NOT NULL,
    namespace_id uuid NOT NULL,
    generation bigint NOT NULL CHECK(generation > 0),
    state text NOT NULL CHECK(state IN ('active','archived')),
    command jsonb NOT NULL,
    UNIQUE(registry_instance_id,namespace_id)
);
ALTER TABLE task_dossiers ADD COLUMN tracker_instance_id uuid;
ALTER TABLE task_dossiers ADD COLUMN task_id uuid;
ALTER TABLE task_dossiers ADD CONSTRAINT task_ref_complete CHECK ((tracker_instance_id IS NULL) = (task_id IS NULL));
DROP INDEX task_dossiers_space_key_idx;
CREATE UNIQUE INDEX task_dossiers_legacy_space_key_idx ON task_dossiers(space_id,task_key) WHERE task_id IS NULL;
CREATE UNIQUE INDEX task_dossiers_managed_task_idx ON task_dossiers(space_id,tracker_instance_id,task_id) WHERE task_id IS NOT NULL;
CREATE TABLE managed_task_document_links (
    id uuid PRIMARY KEY,
    space_id uuid NOT NULL REFERENCES spaces(id) ON DELETE RESTRICT,
    dossier_id uuid NOT NULL REFERENCES task_dossiers(id) ON DELETE RESTRICT,
    document_id uuid NOT NULL REFERENCES documents(id) ON DELETE RESTRICT,
    revision_id uuid NOT NULL,
    created_by uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY(revision_id,document_id) REFERENCES document_revisions(id,document_id) ON DELETE RESTRICT,
    FOREIGN KEY(space_id,document_id) REFERENCES documents(space_id,id) ON DELETE RESTRICT,
    FOREIGN KEY(space_id,dossier_id) REFERENCES task_dossiers(space_id,id) ON DELETE RESTRICT,
    UNIQUE(dossier_id,document_id,revision_id)
);
CREATE FUNCTION wiki_namespace_write_guard() RETURNS trigger LANGUAGE plpgsql AS $$
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
DO $$ DECLARE table_name text; BEGIN
    FOREACH table_name IN ARRAY ARRAY['spaces','space_members','documents','document_revisions','document_drafts',
        'task_dossiers','phase_dossiers','document_task_links','document_phase_links','evidence_items','attachments','managed_task_document_links']
    LOOP
        EXECUTE format('CREATE TRIGGER namespace_write_guard BEFORE INSERT OR UPDATE OR DELETE ON %I FOR EACH ROW EXECUTE FUNCTION wiki_namespace_write_guard()',table_name);
    END LOOP;
END $$;

ALTER TABLE wiki_namespace_bindings ADD CONSTRAINT namespace_projection_consistent CHECK (COALESCE((
    command->>'schema_version'='1' AND command->'namespace'->>'registry_instance_id'=registry_instance_id::text
    AND command->'namespace'->>'namespace_id'=namespace_id::text AND command->'resource'->>'resource_id'=resource_id::text
    AND command->'resource'->>'kind'='wiki_space' AND command->>'generation'=generation::text AND command->>'state'=state
    AND command ? 'operation_id' AND command->'resource' ? 'instance_id'
    AND jsonb_typeof(command->'namespace')='object' AND jsonb_typeof(command->'resource')='object'
),false)
);

CREATE FUNCTION wiki_mark_namespace_resource() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN
 UPDATE spaces SET namespace_managed=true WHERE id=NEW.resource_id AND NOT namespace_managed;
 RETURN NEW;
END $$;
CREATE TRIGGER wiki_namespace_resource_marker AFTER INSERT ON wiki_namespace_bindings FOR EACH ROW EXECUTE FUNCTION wiki_mark_namespace_resource();

-- Legacy writes may not rely on an unreadable strict-v1 owner projection.
ALTER TABLE wiki_namespace_bindings ADD CONSTRAINT namespace_projection_shape CHECK (COALESCE((
    command - ARRAY['schema_version','namespace','resource','operation_id','generation','state','create_spec'] = '{}'::jsonb
    AND command ?& ARRAY['schema_version','namespace','resource','operation_id','generation','state']
    AND jsonb_typeof(command->'schema_version')='number' AND jsonb_typeof(command->'generation')='number'
    AND (command->'namespace') - ARRAY['registry_instance_id','namespace_id'] = '{}'::jsonb
    AND (command->'resource') - ARRAY['kind','instance_id','resource_id'] = '{}'::jsonb
    AND command->>'operation_id' ~ '^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$'
    AND command->'resource'->>'instance_id' ~ '^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$'
    AND command->>'operation_id'<>'00000000-0000-0000-0000-000000000000'
    AND command->'resource'->>'instance_id'<>'00000000-0000-0000-0000-000000000000'
    AND registry_instance_id<>'00000000-0000-0000-0000-000000000000'::uuid
    AND namespace_id<>'00000000-0000-0000-0000-000000000000'::uuid
    AND resource_id<>'00000000-0000-0000-0000-000000000000'::uuid
    AND (NOT command ? 'create_spec' OR command->'create_spec'='null'::jsonb OR jsonb_typeof(command->'create_spec')='object')
),false));
