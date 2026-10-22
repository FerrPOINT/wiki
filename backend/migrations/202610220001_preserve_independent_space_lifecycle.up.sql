-- Restore the M:N lifecycle rule without changing any applied migration.
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
    IF TG_TABLE_NAME='spaces' AND TG_OP='DELETE' AND EXISTS(SELECT 1 FROM spaces WHERE id=ANY(sids) AND namespace_managed) THEN
        RAISE EXCEPTION 'namespace_resource_read_only' USING ERRCODE='42501';
    END IF;
    -- Project lifecycle is provenance, not the lifecycle of this independent space.
    -- Preserve the space row lock so native writers also fence a concurrent archive.
    IF TG_TABLE_NAME<>'spaces' THEN
        PERFORM id FROM spaces WHERE id=ANY(sids) ORDER BY id FOR SHARE;
    END IF;
    IF TG_TABLE_NAME<>'spaces' AND EXISTS(SELECT 1 FROM spaces WHERE id=ANY(sids) AND archived_at IS NOT NULL) THEN
        RAISE EXCEPTION 'space_read_only' USING ERRCODE='42501';
    END IF;
    IF TG_OP='DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END $$;
