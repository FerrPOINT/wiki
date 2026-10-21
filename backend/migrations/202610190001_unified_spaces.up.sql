-- Additive shared-space catalog and canonical Tracker projection.
-- Legacy IDs, timestamps, receipts and strict v1 payloads are preserved.
CREATE TABLE wiki_space_admission_state (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    active boolean NOT NULL DEFAULT false,
    cohort_revision text,
    role_migration_confirmed boolean NOT NULL DEFAULT false
);
INSERT INTO wiki_space_admission_state(singleton) VALUES (true);
CREATE TABLE wiki_space_catalog (
    registry_instance_id uuid NOT NULL CHECK (registry_instance_id <> '00000000-0000-0000-0000-000000000000'),
    space_id uuid NOT NULL CHECK (space_id <> '00000000-0000-0000-0000-000000000000'),
    local_space_id uuid NOT NULL UNIQUE REFERENCES spaces(id) ON DELETE RESTRICT,
    admin_revision bigint CHECK (admin_revision > 0),
    state text NOT NULL CHECK (state IN ('provisioning','active','archiving','archived','restoring')),
    operation_id uuid,
    PRIMARY KEY(registry_instance_id,space_id)
);
-- A legacy generation is not an Admin catalog revision.
INSERT INTO wiki_space_catalog(registry_instance_id,space_id,local_space_id,state,operation_id)
SELECT registry_instance_id,namespace_id,resource_id,state,(command->>'operation_id')::uuid FROM wiki_namespace_bindings;

ALTER TABLE wiki_space_projects ADD COLUMN id uuid NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE wiki_space_projects ADD COLUMN tracker_instance_id uuid;
ALTER TABLE wiki_space_projects ADD COLUMN project_id uuid;
ALTER TABLE wiki_space_projects DROP CONSTRAINT wiki_space_projects_pkey;
ALTER TABLE wiki_space_projects ADD PRIMARY KEY(id);
ALTER TABLE wiki_space_projects ALTER COLUMN registry_instance_id DROP NOT NULL;
ALTER TABLE wiki_space_projects ALTER COLUMN namespace_id DROP NOT NULL;
ALTER TABLE wiki_space_projects ADD CONSTRAINT project_ref_complete CHECK (
    (tracker_instance_id IS NULL) = (project_id IS NULL)
    AND (registry_instance_id IS NULL) = (namespace_id IS NULL)
    AND (tracker_instance_id IS NOT NULL OR registry_instance_id IS NOT NULL)
    AND (tracker_instance_id IS NULL OR tracker_instance_id <> '00000000-0000-0000-0000-000000000000')
    AND (project_id IS NULL OR project_id <> '00000000-0000-0000-0000-000000000000')
);
CREATE UNIQUE INDEX wiki_space_projects_canonical ON wiki_space_projects(space_id,tracker_instance_id,project_id) WHERE project_id IS NOT NULL;
CREATE UNIQUE INDEX wiki_space_projects_legacy ON wiki_space_projects(space_id,registry_instance_id,namespace_id) WHERE project_id IS NULL;
CREATE TABLE wiki_project_projection_state (
    singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton),
    tracker_owned boolean NOT NULL DEFAULT false
);
INSERT INTO wiki_project_projection_state(singleton) VALUES(true);
CREATE FUNCTION wiki_project_owner_activation_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.tracker_owned AND NOT NEW.tracker_owned THEN
        RAISE EXCEPTION 'project_link_owner_cannot_be_reverted' USING ERRCODE='42501';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER wiki_project_owner_activation_guard BEFORE UPDATE ON wiki_project_projection_state
    FOR EACH ROW EXECUTE FUNCTION wiki_project_owner_activation_guard();

CREATE TABLE wiki_project_space_projection_sets (
    tracker_instance_id uuid NOT NULL CHECK (tracker_instance_id <> '00000000-0000-0000-0000-000000000000'),
    project_id uuid NOT NULL CHECK (project_id <> '00000000-0000-0000-0000-000000000000'),
    revision bigint NOT NULL CHECK (revision > 0),
    operation_id uuid NOT NULL CHECK (operation_id <> '00000000-0000-0000-0000-000000000000'),
    command jsonb NOT NULL,
    PRIMARY KEY(tracker_instance_id,project_id)
);

-- Historical operation receipts are independent of the current projection head.
CREATE TABLE wiki_project_space_projection_operations (
    operation_id uuid PRIMARY KEY CHECK(operation_id<>'00000000-0000-0000-0000-000000000000'),
    tracker_instance_id uuid NOT NULL,
    project_id uuid NOT NULL,
    revision bigint NOT NULL CHECK(revision>0),
    command jsonb NOT NULL,
    readback jsonb NOT NULL,
    owner_issuer text NOT NULL,
    owner_subject text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(tracker_instance_id,project_id,revision)
);
CREATE TABLE wiki_project_identity_mappings (
    tracker_instance_id uuid NOT NULL,
    project_id uuid NOT NULL,
    registry_instance_id uuid NOT NULL,
    namespace_id uuid NOT NULL,
    PRIMARY KEY(tracker_instance_id,project_id),
    UNIQUE(registry_instance_id,namespace_id)
);

CREATE TABLE wiki_space_owner_operations (
    operation_id uuid PRIMARY KEY CHECK(operation_id<>'00000000-0000-0000-0000-000000000000'),
    registry_instance_id uuid NOT NULL,
    space_id uuid NOT NULL,
    revision bigint NOT NULL CHECK(revision>0),
    command jsonb NOT NULL,
    readback jsonb NOT NULL,
    owner_issuer text NOT NULL,
    owner_subject text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(registry_instance_id,space_id,revision),
    FOREIGN KEY(registry_instance_id,space_id) REFERENCES wiki_space_catalog(registry_instance_id,space_id) ON DELETE RESTRICT
);

CREATE FUNCTION wiki_space_admission_activation_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.active AND (NOT NEW.active OR NEW.cohort_revision IS DISTINCT FROM OLD.cohort_revision
        OR NOT NEW.role_migration_confirmed) THEN
        RAISE EXCEPTION 'guarded_cohort_required' USING ERRCODE='42501';
    END IF;
    IF NEW.active AND NOT OLD.active THEN
        IF current_setting('transaction_isolation') <> 'read committed' THEN
            RAISE EXCEPTION 'admission_activation_requires_read_committed' USING ERRCODE='23514';
        END IF;
        LOCK TABLE spaces,wiki_space_catalog,wiki_space_projects,wiki_project_space_projection_sets IN SHARE ROW EXCLUSIVE MODE;
        IF NULLIF(btrim(NEW.cohort_revision),'') IS NULL OR NOT NEW.role_migration_confirmed
            OR EXISTS(SELECT 1 FROM spaces s LEFT JOIN wiki_space_catalog c ON c.local_space_id=s.id WHERE c.admin_revision IS NULL)
            OR EXISTS(SELECT 1 FROM wiki_space_projects WHERE project_id IS NULL)
            OR EXISTS(SELECT 1 FROM wiki_space_projects l LEFT JOIN wiki_project_space_projection_sets p USING(tracker_instance_id,project_id) WHERE p.project_id IS NULL)
        THEN RAISE EXCEPTION 'space_admission_cohort_or_projection_unconfirmed' USING ERRCODE='23514'; END IF;
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER wiki_space_admission_activation_guard BEFORE UPDATE ON wiki_space_admission_state
    FOR EACH ROW EXECUTE FUNCTION wiki_space_admission_activation_guard();

CREATE FUNCTION wiki_project_projection_write_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE owned boolean;
BEGIN
    PERFORM singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE;
    -- A real row transition fences stale REPEATABLE READ writers as well as RC.
    SELECT tracker_owned INTO owned FROM wiki_project_projection_state WHERE singleton FOR SHARE;
    IF ((SELECT active FROM wiki_space_admission_state WHERE singleton) OR owned)
        AND current_setting('pdlc.wiki.projection_owner',true) IS DISTINCT FROM 'tracker_v2'
    THEN RAISE EXCEPTION 'project_links_owned_by_tracker' USING ERRCODE='42501'; END IF;
    IF TG_OP='DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER wiki_project_projection_write_guard BEFORE INSERT OR UPDATE OR DELETE ON wiki_space_projects
    FOR EACH ROW EXECUTE FUNCTION wiki_project_projection_write_guard();


-- The legacy function is replaced only by this new additive migration. Its old
-- checksum/receipt is unchanged, and its exact old rule remains before activation.
CREATE OR REPLACE FUNCTION wiki_namespace_write_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE data jsonb; sid uuid; sids uuid[] := '{}'; lifecycle text; catalog_count integer := 0; wanted integer;
BEGIN
    -- Same lock order as activation and owner-v2: admission -> sorted catalog rows.
    -- A cached HTTP proof cannot bypass an archive committed while a writer waits.
    PERFORM singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE;
    IF TG_TABLE_NAME='wiki_space_projects' AND current_setting('pdlc.wiki.projection_owner',true)='tracker_v2' THEN
        -- A confirmed metadata projection may retain/remove archived associations.
        -- It never updates content, membership or the space lifecycle.
        IF TG_OP='DELETE' THEN RETURN OLD; END IF;
        RETURN NEW;
    END IF;
    IF current_setting('pdlc.wiki.lifecycle_owner',true)='admin_v2'
        AND TG_TABLE_NAME IN ('spaces','space_members') THEN
        IF TG_OP='DELETE' THEN RAISE EXCEPTION 'space_lifecycle_preserves_data' USING ERRCODE='42501'; END IF;
        RETURN NEW;
    END IF;
    IF TG_TABLE_NAME='spaces' AND EXISTS(SELECT 1 FROM wiki_space_catalog c JOIN wiki_space_owner_operations o USING(registry_instance_id,space_id) WHERE c.local_space_id=COALESCE((to_jsonb(NEW)->>'id')::uuid,(to_jsonb(OLD)->>'id')::uuid)) THEN
        RAISE EXCEPTION 'space_metadata_owned_by_admin' USING ERRCODE='42501';
    END IF;
        FOR data IN SELECT value FROM jsonb_array_elements(CASE TG_OP WHEN 'INSERT' THEN jsonb_build_array(to_jsonb(NEW)) WHEN 'DELETE' THEN jsonb_build_array(to_jsonb(OLD)) ELSE jsonb_build_array(to_jsonb(OLD),to_jsonb(NEW)) END)
        LOOP
            IF TG_TABLE_NAME='spaces' THEN sid := (data->>'id')::uuid;
            ELSIF data ? 'space_id' THEN sid := (data->>'space_id')::uuid;
            ELSE SELECT space_id INTO sid FROM documents WHERE id=(data->>'document_id')::uuid; END IF;
            sids := array_append(sids,sid);
        END LOOP;
        SELECT array_agg(DISTINCT value ORDER BY value) INTO sids FROM unnest(sids) value WHERE value IS NOT NULL;
    -- Lock even before selecting legacy/v2 behavior: concurrent adoption/archive
    -- must not leave an old writer running under the previous metadata owner.
    PERFORM local_space_id FROM wiki_space_catalog WHERE local_space_id=ANY(sids) ORDER BY local_space_id FOR SHARE;
    IF (SELECT active FROM wiki_space_admission_state WHERE singleton)
        OR EXISTS(SELECT 1 FROM wiki_space_catalog c JOIN wiki_space_owner_operations o USING(registry_instance_id,space_id) WHERE c.local_space_id=ANY(sids)) THEN
        IF TG_TABLE_NAME='spaces' THEN
            RAISE EXCEPTION 'space_metadata_owned_by_admin' USING ERRCODE='42501';
        END IF;
        wanted := coalesce(cardinality(sids),0);
        FOR lifecycle IN SELECT state FROM wiki_space_catalog WHERE local_space_id=ANY(sids) AND admin_revision IS NOT NULL ORDER BY local_space_id FOR SHARE
        LOOP
            catalog_count := catalog_count+1;
            IF lifecycle <> 'active' THEN RAISE EXCEPTION 'shared_space_read_only' USING ERRCODE='42501'; END IF;
        END LOOP;
        IF catalog_count <> wanted THEN RAISE EXCEPTION 'shared_space_projection_unconfirmed' USING ERRCODE='42501'; END IF;
        IF TG_TABLE_NAME='document_revisions' AND TG_OP<>'INSERT' AND wanted>0 THEN
            RAISE EXCEPTION 'published_revision_immutable' USING ERRCODE='42501';
        END IF;
        IF TG_OP='DELETE' THEN RETURN OLD; END IF;
        RETURN NEW;
    END IF;
    IF TG_TABLE_NAME='spaces' AND TG_OP='UPDATE' THEN
        IF OLD.namespace_managed AND NOT NEW.namespace_managed THEN RAISE EXCEPTION 'namespace_marker_immutable' USING ERRCODE='42501'; END IF;
        IF NOT OLD.namespace_managed AND NEW.namespace_managed AND to_jsonb(OLD)-'namespace_managed'=to_jsonb(NEW)-'namespace_managed' AND EXISTS(SELECT 1 FROM wiki_namespace_bindings WHERE resource_id=NEW.id) THEN RETURN NEW; END IF;
    END IF;
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

CREATE FUNCTION wiki_shared_catalog_owner_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM singleton FROM wiki_space_admission_state WHERE singleton FOR SHARE;
    PERFORM local_space_id FROM wiki_space_catalog
        WHERE registry_instance_id=COALESCE((to_jsonb(NEW)->>'registry_instance_id')::uuid,(to_jsonb(OLD)->>'registry_instance_id')::uuid)
        AND space_id=COALESCE((to_jsonb(NEW)->>'space_id')::uuid,(to_jsonb(NEW)->>'namespace_id')::uuid,(to_jsonb(OLD)->>'space_id')::uuid,(to_jsonb(OLD)->>'namespace_id')::uuid)
        FOR SHARE;
    IF ((SELECT active FROM wiki_space_admission_state WHERE singleton)
        OR EXISTS(SELECT 1 FROM wiki_space_owner_operations o WHERE o.registry_instance_id=COALESCE((to_jsonb(NEW)->>'registry_instance_id')::uuid,(to_jsonb(OLD)->>'registry_instance_id')::uuid) AND o.space_id=COALESCE((to_jsonb(NEW)->>'space_id')::uuid,(to_jsonb(NEW)->>'namespace_id')::uuid,(to_jsonb(OLD)->>'space_id')::uuid,(to_jsonb(OLD)->>'namespace_id')::uuid)))
        AND current_setting('pdlc.wiki.lifecycle_owner',true) IS DISTINCT FROM 'admin_v2'
    THEN RAISE EXCEPTION 'space_metadata_owned_by_admin' USING ERRCODE='42501'; END IF;
    IF TG_OP='DELETE' THEN RAISE EXCEPTION 'space_lifecycle_preserves_data' USING ERRCODE='42501'; END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER wiki_shared_catalog_owner_guard BEFORE INSERT OR UPDATE OR DELETE ON wiki_space_catalog
    FOR EACH ROW EXECUTE FUNCTION wiki_shared_catalog_owner_guard();
CREATE TRIGGER wiki_legacy_binding_owner_guard BEFORE INSERT OR UPDATE OR DELETE ON wiki_namespace_bindings
    FOR EACH ROW EXECUTE FUNCTION wiki_shared_catalog_owner_guard();
