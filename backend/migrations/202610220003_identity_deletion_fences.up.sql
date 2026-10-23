-- Owner-wide fencing covers legacy SQL/repository/worker writers as well as HTTP.
CREATE TABLE wiki_identity_legacy_issuer(singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton),issuer text NOT NULL CHECK(length(issuer) BETWEEN 1 AND 2048));
CREATE TABLE wiki_identity_aliases(
 local_user_id uuid PRIMARY KEY,kind text NOT NULL DEFAULT 'central' CHECK(kind IN ('local','central')),issuer text,subject text,
 CHECK((kind='local' AND subject IS NULL AND issuer IS NULL) OR (kind='central' AND NULLIF(btrim(subject),'') IS NOT NULL)));
-- A native Wiki UUID is not evidence of a Central Auth subject.
INSERT INTO wiki_identity_aliases(local_user_id,kind,subject)
 SELECT id,CASE WHEN central_sub IS NULL THEN 'local' ELSE 'central' END,central_sub FROM users;
CREATE TABLE wiki_identity_deletion_operations(
 operation_id uuid PRIMARY KEY,issuer text NOT NULL,subject text NOT NULL,inventory_revision uuid NOT NULL,
 request jsonb NOT NULL,state text NOT NULL CHECK(state IN ('frozen','released')),fence uuid,
 revision bigint NOT NULL CHECK(revision>0),reference_count bigint NOT NULL CHECK(reference_count>=0),
 CHECK((state='frozen')=(fence IS NOT NULL)));
CREATE UNIQUE INDEX wiki_identity_active_freeze ON wiki_identity_deletion_operations(issuer,subject) WHERE state='frozen';
CREATE SEQUENCE wiki_identity_reference_revision MINVALUE 1;
CREATE TABLE wiki_identity_fence_epoch(singleton boolean PRIMARY KEY DEFAULT true CHECK(singleton),revision bigint NOT NULL DEFAULT 1 CHECK(revision>0));
INSERT INTO wiki_identity_fence_epoch(singleton) VALUES(true);
CREATE FUNCTION wiki_identity_write_lock() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 PERFORM pg_advisory_xact_lock_shared(714402216003::bigint);
 PERFORM 1 FROM wiki_identity_fence_epoch WHERE singleton FOR SHARE;
 IF NOT FOUND THEN RAISE EXCEPTION 'identity_fence_epoch_missing' USING ERRCODE='23514'; END IF;
 PERFORM nextval('wiki_identity_reference_revision');
 RETURN NULL;
END $$;
CREATE TRIGGER wiki_identity_alias_write_lock BEFORE INSERT OR UPDATE OR DELETE ON wiki_identity_aliases FOR EACH STATEMENT EXECUTE FUNCTION wiki_identity_write_lock();
CREATE FUNCTION wiki_identity_alias_insert_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE current_subject text; current_id uuid; bound text;
BEGIN
 SELECT id,central_sub INTO current_id,current_subject FROM users WHERE id=NEW.local_user_id;
 SELECT issuer INTO bound FROM wiki_identity_legacy_issuer WHERE singleton;
 IF current_id IS NOT NULL THEN
  IF (current_subject IS NULL AND (NEW.kind<>'local' OR NEW.subject IS NOT NULL OR NEW.issuer IS NOT NULL))
   OR (current_subject IS NOT NULL AND (NEW.kind<>'central' OR NEW.subject IS DISTINCT FROM current_subject OR NEW.issuer IS DISTINCT FROM bound))
  THEN RAISE EXCEPTION 'identity_alias_conflict' USING ERRCODE='23514';END IF;
 ELSIF NEW.kind='local' THEN RAISE EXCEPTION 'identity_local_profile_unknown' USING ERRCODE='23514';END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER wiki_identity_alias_insert_guard BEFORE INSERT ON wiki_identity_aliases FOR EACH ROW EXECUTE FUNCTION wiki_identity_alias_insert_guard();
CREATE FUNCTION wiki_identity_alias_guard() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE bound text;
BEGIN
 IF TG_OP='DELETE' OR NEW.local_user_id IS DISTINCT FROM OLD.local_user_id
 THEN RAISE EXCEPTION 'identity_alias_immutable' USING ERRCODE='23514'; END IF;
 SELECT issuer INTO bound FROM wiki_identity_legacy_issuer WHERE singleton;
 -- Explicit NULL->central_sub linking is observed after the user row update,
 -- in the same guarded transaction. Existing history keeps its local UUID.
 IF OLD.kind='local' AND NEW.kind='central' AND NEW.subject IS NOT NULL
    AND NEW.issuer IS NOT DISTINCT FROM bound
    AND EXISTS(SELECT 1 FROM users WHERE id=NEW.local_user_id AND central_sub=NEW.subject)
 THEN RETURN NEW;END IF;
 -- Deployment assertion binds an already known central subject to its issuer.
 IF OLD.kind='central' AND NEW.kind='central' AND NEW.subject IS NOT DISTINCT FROM OLD.subject
   AND OLD.issuer IS NULL AND NEW.issuer IS NOT NULL AND NEW.issuer=bound
 THEN RETURN NEW;END IF;
 RAISE EXCEPTION 'identity_alias_immutable' USING ERRCODE='23514';
END $$;
CREATE TRIGGER wiki_identity_alias_guard BEFORE UPDATE OR DELETE ON wiki_identity_aliases FOR EACH ROW EXECUTE FUNCTION wiki_identity_alias_guard();
CREATE FUNCTION wiki_identity_bind_legacy() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'identity_legacy_issuer_immutable' USING ERRCODE='23514'; END IF;
 PERFORM pg_advisory_xact_lock(714402216003::bigint);
 UPDATE wiki_identity_fence_epoch SET revision=revision+1 WHERE singleton;
 UPDATE wiki_identity_aliases SET issuer=NEW.issuer WHERE kind='central' AND issuer IS NULL;
 RETURN NEW;
END $$;
CREATE TRIGGER wiki_identity_bind_legacy AFTER INSERT OR UPDATE OR DELETE ON wiki_identity_legacy_issuer FOR EACH ROW EXECUTE FUNCTION wiki_identity_bind_legacy();
CREATE FUNCTION wiki_identity_native_reuse_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF EXISTS(SELECT 1 FROM wiki_identity_aliases WHERE local_user_id=NEW.id AND kind='local')
   AND NOT EXISTS(SELECT 1 FROM users WHERE id=NEW.id)
 THEN RAISE EXCEPTION 'identity_local_id_reuse_denied' USING ERRCODE='23514';END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER a_wiki_identity_native_reuse_guard BEFORE INSERT ON users FOR EACH ROW EXECUTE FUNCTION wiki_identity_native_reuse_guard();
CREATE FUNCTION wiki_identity_binding_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF NEW.id IS DISTINCT FROM OLD.id OR (OLD.central_sub IS NOT NULL AND NEW.central_sub IS DISTINCT FROM OLD.central_sub)
 THEN RAISE EXCEPTION 'identity_binding_immutable' USING ERRCODE='23514'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER a_wiki_identity_binding_guard BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION wiki_identity_binding_guard();
CREATE FUNCTION wiki_identity_preserve_alias() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE bound text; expected_kind text;
BEGIN
 SELECT issuer INTO bound FROM wiki_identity_legacy_issuer WHERE singleton;
 expected_kind:=CASE WHEN NEW.central_sub IS NULL THEN 'local' ELSE 'central' END;
 -- A newly proven central binding changes the existing alias, not the local
 -- identifier on every historical foreign-key/audit reference.
 IF NEW.central_sub IS NOT NULL THEN
  UPDATE wiki_identity_aliases SET kind='central',issuer=bound,subject=NEW.central_sub
   WHERE local_user_id=NEW.id AND kind='local';
 END IF;
 INSERT INTO wiki_identity_aliases(local_user_id,kind,issuer,subject)
  VALUES(NEW.id,expected_kind,CASE WHEN expected_kind='central' THEN bound ELSE NULL END,NEW.central_sub)
  ON CONFLICT(local_user_id) DO NOTHING;
 IF EXISTS(SELECT 1 FROM wiki_identity_aliases WHERE local_user_id=NEW.id AND
    (kind<>expected_kind OR subject IS DISTINCT FROM NEW.central_sub OR issuer IS DISTINCT FROM (CASE WHEN expected_kind='central' THEN bound ELSE NULL END)))
 THEN RAISE EXCEPTION 'identity_alias_conflict' USING ERRCODE='23514'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER wiki_identity_preserve_alias AFTER INSERT OR UPDATE ON users FOR EACH ROW EXECUTE FUNCTION wiki_identity_preserve_alias();
CREATE FUNCTION wiki_identity_local_ref(local_id text) RETURNS TABLE(issuer text,subject text,resolved boolean) LANGUAGE plpgsql STABLE AS $$
DECLARE known_kind text;
BEGIN
 IF local_id IS NULL OR local_id='' THEN RETURN; END IF;
 IF local_id !~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$' THEN issuer:=NULL;subject:=local_id;resolved:=false;RETURN NEXT;RETURN;END IF;
 SELECT a.kind,a.issuer,a.subject INTO known_kind,issuer,subject FROM wiki_identity_aliases a WHERE a.local_user_id=local_id::uuid;
 IF NOT FOUND THEN issuer:=NULL;subject:=local_id;resolved:=false;RETURN NEXT;
 ELSIF known_kind='central' THEN resolved:=true;RETURN NEXT;
 END IF;
 -- An explicitly native profile is not a guessed central identity.
END $$;
-- Visit only typed lifecycle commands/receipts, never document content or free-form evidence.
CREATE FUNCTION wiki_identity_json_refs(document jsonb,inherited_issuer text,depth integer DEFAULT 0)
 RETURNS TABLE(issuer text,subject text,resolved boolean) LANGUAGE plpgsql STABLE AS $$
DECLARE item record; nested text; issuer_key text;
BEGIN
 IF document IS NULL OR document='null'::jsonb THEN RETURN;END IF;
 IF depth>32 THEN issuer:=NULL;subject:=NULL;resolved:=false;RETURN NEXT;RETURN;END IF;
 IF jsonb_typeof(document)='object' THEN
  FOREACH issuer_key IN ARRAY ARRAY['issuer','actor_issuer','owner_issuer','responsible_issuer'] LOOP
   IF document ? issuer_key AND (jsonb_typeof(document->issuer_key) IS DISTINCT FROM 'string'
     OR NULLIF(btrim(document->>issuer_key),'') IS NULL) THEN
    issuer:=NULL;subject:=NULL;resolved:=false;RETURN NEXT;RETURN;
   END IF;
  END LOOP;
  nested:=COALESCE(document->>'issuer',document->>'actor_issuer',inherited_issuer);
  FOR item IN SELECT * FROM jsonb_each(document) LOOP
   IF item.key IN ('subject','owner_subject','responsible_subject','actor_subject','central_subject') THEN
    IF item.key='subject' AND item.value='null'::jsonb THEN issuer:=NULL;subject:=NULL;resolved:=false;RETURN NEXT;END IF;
    IF item.value<>'null'::jsonb THEN
     issuer:=COALESCE(CASE item.key WHEN 'owner_subject' THEN document->>'owner_issuer'
       WHEN 'actor_subject' THEN document->>'actor_issuer' WHEN 'responsible_subject' THEN document->>'responsible_issuer'
       ELSE NULL END,nested);
     subject:=item.value#>>'{}';resolved:=(jsonb_typeof(item.value)='string' AND NULLIF(btrim(subject),'') IS NOT NULL);RETURN NEXT;
    END IF;
   ELSIF item.key IN ('owner_id','actor_id','user_id','created_by') THEN
    IF item.value<>'null'::jsonb THEN RETURN QUERY SELECT * FROM wiki_identity_local_ref(item.value#>>'{}');END IF;
   ELSIF jsonb_typeof(item.value) IN ('object','array') THEN
    RETURN QUERY SELECT * FROM wiki_identity_json_refs(item.value,nested,depth+1);
   END IF;
  END LOOP;
 ELSIF jsonb_typeof(document)='array' THEN
  FOR item IN SELECT value FROM jsonb_array_elements(document) LOOP RETURN QUERY SELECT * FROM wiki_identity_json_refs(item.value,inherited_issuer,depth+1);END LOOP;
 END IF;
END $$;
CREATE FUNCTION wiki_identity_row_refs(source text,document jsonb)
 RETURNS TABLE(issuer text,subject text,resolved boolean) LANGUAGE plpgsql STABLE AS $$
DECLARE key text;keys text[];payloads text[];bound text;
BEGIN
 SELECT l.issuer INTO bound FROM wiki_identity_legacy_issuer l WHERE singleton;
 CASE source
  WHEN 'users' THEN
   IF document->>'central_sub' IS NOT NULL THEN issuer:=bound;subject:=document->>'central_sub';resolved:=(NULLIF(btrim(subject),'') IS NOT NULL);RETURN NEXT;END IF;
  WHEN 'wiki_identity_aliases' THEN
   IF document->>'kind'='central' THEN issuer:=document->>'issuer';subject:=document->>'subject';resolved:=(NULLIF(btrim(subject),'') IS NOT NULL);RETURN NEXT;END IF;
  WHEN 'spaces','documents' THEN keys:=ARRAY['owner_id'];
  WHEN 'space_members','auth_sessions' THEN keys:=ARRAY['user_id'];
  WHEN 'document_revisions','document_drafts' THEN keys:=ARRAY['author_id'];
  WHEN 'document_task_links','document_phase_links','evidence_items','managed_task_document_links','wiki_space_projects' THEN keys:=ARRAY['created_by'];
  WHEN 'attachments' THEN keys:=ARRAY['uploaded_by'];
  WHEN 'audit_log' THEN
   keys:=ARRAY['actor_id'];
   IF document->>'entity_type'='user' THEN RETURN QUERY SELECT * FROM wiki_identity_local_ref(document->>'entity_id');END IF;
  WHEN 'idempotency_records' THEN keys:=ARRAY['actor_id'];
  WHEN 'wiki_confirmed_global_roles' THEN keys:=ARRAY['user_id','granted_by'];
  WHEN 'wiki_namespace_bindings','wiki_project_space_projection_sets' THEN payloads:=ARRAY['command'];
  WHEN 'wiki_space_owner_operations','wiki_project_space_projection_operations' THEN
   issuer:=document->>'owner_issuer';subject:=document->>'owner_subject';resolved:=(NULLIF(issuer,'') IS NOT NULL AND NULLIF(subject,'') IS NOT NULL);RETURN NEXT;
   payloads:=ARRAY['command','readback'];
  ELSE RAISE EXCEPTION 'identity_reference_source_unknown';
 END CASE;
 FOREACH key IN ARRAY COALESCE(keys,ARRAY[]::text[]) LOOP RETURN QUERY SELECT * FROM wiki_identity_local_ref(document->>key);END LOOP;
 FOREACH key IN ARRAY COALESCE(payloads,ARRAY[]::text[]) LOOP RETURN QUERY SELECT * FROM wiki_identity_json_refs(document->key,bound);END LOOP;
END $$;
CREATE FUNCTION wiki_identity_write_guard() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 IF EXISTS(SELECT 1 FROM wiki_identity_row_refs(TG_TABLE_NAME,to_jsonb(NEW)) r WHERE NOT r.resolved OR (COALESCE(length(btrim(r.issuer)),0)=0 AND EXISTS(SELECT 1 FROM wiki_identity_legacy_issuer WHERE singleton)))
 THEN RAISE EXCEPTION 'identity_reference_unknown' USING ERRCODE='23514',CONSTRAINT='wiki_identity_reference_unknown';END IF;
 IF EXISTS(SELECT 1 FROM wiki_identity_row_refs(TG_TABLE_NAME,to_jsonb(NEW)) r JOIN wiki_identity_deletion_operations f ON f.issuer=r.issuer AND f.subject=r.subject WHERE f.state='frozen')
 THEN RAISE EXCEPTION 'identity_deletion_fenced' USING ERRCODE='23514',CONSTRAINT='wiki_identity_deletion_fenced';END IF;
 RETURN NEW;
END $$;
DO $$ DECLARE source text; BEGIN
 FOREACH source IN ARRAY ARRAY['users','spaces','space_members','documents','document_revisions','document_drafts','document_task_links','document_phase_links','attachments','evidence_items','audit_log','auth_sessions','idempotency_records','managed_task_document_links','wiki_space_projects','wiki_confirmed_global_roles','wiki_namespace_bindings','wiki_project_space_projection_sets','wiki_space_owner_operations','wiki_project_space_projection_operations'] LOOP
 EXECUTE format('CREATE TRIGGER wiki_identity_reference_lock BEFORE INSERT OR UPDATE OR DELETE ON %I FOR EACH STATEMENT EXECUTE FUNCTION wiki_identity_write_lock()',source);
 EXECUTE format('CREATE TRIGGER z_wiki_identity_reference_guard BEFORE INSERT OR UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION wiki_identity_write_guard()',source);
 END LOOP;
END $$;

CREATE TRIGGER z_wiki_identity_alias_reference_guard BEFORE INSERT OR UPDATE ON wiki_identity_aliases FOR EACH ROW EXECUTE FUNCTION wiki_identity_write_guard();
