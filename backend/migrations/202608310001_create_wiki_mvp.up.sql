CREATE TABLE users (
    id uuid PRIMARY KEY,
    email text NOT NULL,
    display_name text NOT NULL,
    password_hash text NOT NULL,
    global_role text NOT NULL DEFAULT 'user',
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT users_email_not_blank CHECK (btrim(email) <> ''),
    CONSTRAINT users_display_name_not_blank CHECK (btrim(display_name) <> ''),
    CONSTRAINT users_global_role_check CHECK (global_role IN ('admin', 'user'))
);

CREATE UNIQUE INDEX users_email_idx ON users (lower(email));
CREATE INDEX users_active_idx ON users (is_active) WHERE is_active = true;

CREATE TABLE spaces (
    id uuid PRIMARY KEY,
    key text NOT NULL,
    name text NOT NULL,
    description text NOT NULL DEFAULT '',
    owner_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT spaces_key_check CHECK (key ~ '^[A-Z0-9][A-Z0-9-]{0,30}[A-Z0-9]$'),
    CONSTRAINT spaces_name_not_blank CHECK (btrim(name) <> '')
);

CREATE UNIQUE INDEX spaces_key_idx ON spaces (key);
CREATE INDEX spaces_owner_idx ON spaces (owner_id);
CREATE INDEX spaces_live_idx ON spaces (key) WHERE archived_at IS NULL;

CREATE TABLE space_members (
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role text NOT NULL,
    joined_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (space_id, user_id),
    CONSTRAINT space_members_role_check CHECK (role IN ('admin', 'editor', 'viewer'))
);

CREATE INDEX space_members_user_idx ON space_members (user_id);

CREATE TABLE documents (
    id uuid PRIMARY KEY,
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    parent_id uuid,
    slug text NOT NULL,
    title text NOT NULL,
    document_type text NOT NULL DEFAULT 'page',
    status text NOT NULL DEFAULT 'draft',
    current_revision_id uuid,
    owner_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    archived_at timestamptz,
    CONSTRAINT documents_slug_check CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND length(slug) <= 96),
    CONSTRAINT documents_title_not_blank CHECK (btrim(title) <> ''),
    CONSTRAINT documents_type_check CHECK (
        document_type IN ('page', 'requirements', 'research_note', 'implementation_note', 'test_plan', 'release_note')
    ),
    CONSTRAINT documents_status_check CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT documents_archive_status_check CHECK (
        (status = 'archived' AND archived_at IS NOT NULL)
        OR (status <> 'archived' AND archived_at IS NULL)
    ),
    CONSTRAINT documents_not_self_parent CHECK (parent_id IS NULL OR parent_id <> id),
    CONSTRAINT documents_space_id_pair UNIQUE (space_id, id),
    CONSTRAINT documents_id_space_pair UNIQUE (id, space_id)
);

ALTER TABLE documents
    ADD CONSTRAINT documents_parent_same_space_fk
    FOREIGN KEY (space_id, parent_id)
    REFERENCES documents (space_id, id)
    ON DELETE RESTRICT;

CREATE UNIQUE INDEX documents_root_slug_idx
    ON documents (space_id, slug)
    WHERE parent_id IS NULL;

CREATE UNIQUE INDEX documents_child_slug_idx
    ON documents (space_id, parent_id, slug)
    WHERE parent_id IS NOT NULL;

CREATE INDEX documents_space_parent_position_idx ON documents (space_id, parent_id, position, title);
CREATE INDEX documents_current_revision_idx ON documents (current_revision_id) WHERE current_revision_id IS NOT NULL;
CREATE INDEX documents_live_idx ON documents (space_id, updated_at DESC) WHERE archived_at IS NULL;

CREATE TABLE document_revisions (
    id uuid PRIMARY KEY,
    document_id uuid NOT NULL REFERENCES documents (id) ON DELETE CASCADE,
    version integer NOT NULL,
    title text NOT NULL,
    content_markdown text NOT NULL,
    content_html text NOT NULL,
    content_text text NOT NULL,
    content_checksum text NOT NULL,
    summary text,
    author_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    published_at timestamptz NOT NULL DEFAULT now(),
    search_vector tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(title, '')), 'A')
        || setweight(to_tsvector('simple', coalesce(content_text, '')), 'B')
    ) STORED,
    CONSTRAINT document_revisions_version_positive CHECK (version > 0),
    CONSTRAINT document_revisions_title_not_blank CHECK (btrim(title) <> ''),
    CONSTRAINT document_revisions_content_not_blank CHECK (btrim(content_markdown) <> ''),
    CONSTRAINT document_revisions_checksum_not_blank CHECK (btrim(content_checksum) <> ''),
    CONSTRAINT document_revisions_document_id_pair UNIQUE (id, document_id)
);

CREATE UNIQUE INDEX document_revisions_document_version_idx ON document_revisions (document_id, version);
CREATE INDEX document_revisions_document_published_idx ON document_revisions (document_id, published_at DESC);
CREATE INDEX document_revisions_search_idx ON document_revisions USING GIN (search_vector);

ALTER TABLE documents
    ADD CONSTRAINT documents_current_revision_same_document_fk
    FOREIGN KEY (current_revision_id, id)
    REFERENCES document_revisions (id, document_id)
    ON DELETE RESTRICT;

CREATE TABLE document_drafts (
    document_id uuid PRIMARY KEY REFERENCES documents (id) ON DELETE CASCADE,
    author_id uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    content_markdown text NOT NULL DEFAULT '',
    base_revision_id uuid,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT document_drafts_base_revision_fk
        FOREIGN KEY (base_revision_id, document_id)
        REFERENCES document_revisions (id, document_id)
        ON DELETE RESTRICT
);

CREATE INDEX document_drafts_author_idx ON document_drafts (author_id, updated_at DESC);

CREATE TABLE task_dossiers (
    id uuid PRIMARY KEY,
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    task_key text NOT NULL,
    title_snapshot text,
    external_url text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT task_dossiers_key_not_blank CHECK (btrim(task_key) <> ''),
    CONSTRAINT task_dossiers_space_id_pair UNIQUE (space_id, id),
    CONSTRAINT task_dossiers_id_space_pair UNIQUE (id, space_id)
);

CREATE UNIQUE INDEX task_dossiers_space_key_idx ON task_dossiers (space_id, task_key);
CREATE INDEX task_dossiers_space_idx ON task_dossiers (space_id, updated_at DESC);

CREATE TABLE phase_dossiers (
    id uuid PRIMARY KEY,
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    phase_key text NOT NULL,
    phase_name text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT phase_dossiers_key_check CHECK (
        phase_key ~ '^[a-z0-9][a-z0-9_-]{0,62}[a-z0-9]$'
        OR phase_key ~ '^[a-z0-9]$'
    ),
    CONSTRAINT phase_dossiers_space_id_pair UNIQUE (space_id, id),
    CONSTRAINT phase_dossiers_id_space_pair UNIQUE (id, space_id)
);

CREATE UNIQUE INDEX phase_dossiers_space_key_idx ON phase_dossiers (space_id, phase_key);
CREATE INDEX phase_dossiers_space_idx ON phase_dossiers (space_id, updated_at DESC);

CREATE TABLE document_task_links (
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    document_id uuid NOT NULL,
    task_dossier_id uuid NOT NULL,
    created_by uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (document_id, task_dossier_id),
    CONSTRAINT document_task_links_document_same_space_fk
        FOREIGN KEY (document_id, space_id)
        REFERENCES documents (id, space_id)
        ON DELETE CASCADE,
    CONSTRAINT document_task_links_task_same_space_fk
        FOREIGN KEY (task_dossier_id, space_id)
        REFERENCES task_dossiers (id, space_id)
        ON DELETE CASCADE
);

CREATE INDEX document_task_links_task_idx ON document_task_links (task_dossier_id, created_at DESC);

CREATE TABLE document_phase_links (
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    document_id uuid NOT NULL,
    phase_dossier_id uuid NOT NULL,
    created_by uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (document_id, phase_dossier_id),
    CONSTRAINT document_phase_links_document_same_space_fk
        FOREIGN KEY (document_id, space_id)
        REFERENCES documents (id, space_id)
        ON DELETE CASCADE,
    CONSTRAINT document_phase_links_phase_same_space_fk
        FOREIGN KEY (phase_dossier_id, space_id)
        REFERENCES phase_dossiers (id, space_id)
        ON DELETE CASCADE
);

CREATE INDEX document_phase_links_phase_idx ON document_phase_links (phase_dossier_id, created_at DESC);

CREATE TABLE attachments (
    id uuid PRIMARY KEY,
    space_id uuid REFERENCES spaces (id) ON DELETE CASCADE,
    owner_entity_type text,
    owner_entity_id uuid,
    file_name text NOT NULL,
    content_type text NOT NULL,
    size_bytes bigint NOT NULL,
    storage_key text NOT NULL,
    checksum text NOT NULL,
    uploaded_by uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    uploaded_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT attachments_owner_shape_check CHECK (
        (space_id IS NULL AND owner_entity_type IS NULL AND owner_entity_id IS NULL)
        OR (
            space_id IS NOT NULL
            AND owner_entity_type IN ('document', 'revision', 'evidence')
            AND owner_entity_id IS NOT NULL
        )
    ),
    CONSTRAINT attachments_file_name_not_blank CHECK (btrim(file_name) <> ''),
    CONSTRAINT attachments_content_type_not_blank CHECK (btrim(content_type) <> ''),
    CONSTRAINT attachments_size_positive CHECK (size_bytes > 0),
    CONSTRAINT attachments_storage_key_not_blank CHECK (btrim(storage_key) <> ''),
    CONSTRAINT attachments_checksum_not_blank CHECK (btrim(checksum) <> ''),
    CONSTRAINT attachments_id_space_pair UNIQUE (id, space_id)
);

CREATE UNIQUE INDEX attachments_storage_key_idx ON attachments (storage_key);
CREATE INDEX attachments_owner_idx ON attachments (owner_entity_type, owner_entity_id) WHERE owner_entity_id IS NOT NULL;
CREATE INDEX attachments_checksum_idx ON attachments (checksum);
CREATE INDEX attachments_staged_idx ON attachments (uploaded_by, uploaded_at DESC) WHERE owner_entity_id IS NULL;

CREATE TABLE evidence_items (
    id uuid PRIMARY KEY,
    space_id uuid NOT NULL REFERENCES spaces (id) ON DELETE CASCADE,
    document_id uuid,
    task_dossier_id uuid,
    phase_dossier_id uuid,
    evidence_type text NOT NULL,
    title text NOT NULL,
    url text,
    attachment_id uuid REFERENCES attachments (id) ON DELETE RESTRICT,
    checksum text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_by uuid NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT evidence_title_not_blank CHECK (btrim(title) <> ''),
    CONSTRAINT evidence_has_target_check CHECK (
        document_id IS NOT NULL OR task_dossier_id IS NOT NULL OR phase_dossier_id IS NOT NULL
    ),
    CONSTRAINT evidence_payload_shape_check CHECK (
        (evidence_type = 'external_url' AND url IS NOT NULL AND attachment_id IS NULL)
        OR (evidence_type = 'uploaded_file' AND url IS NULL AND attachment_id IS NOT NULL)
    ),
    CONSTRAINT evidence_document_same_space_fk
        FOREIGN KEY (document_id, space_id)
        REFERENCES documents (id, space_id)
        ON DELETE CASCADE,
    CONSTRAINT evidence_task_same_space_fk
        FOREIGN KEY (task_dossier_id, space_id)
        REFERENCES task_dossiers (id, space_id)
        ON DELETE CASCADE,
    CONSTRAINT evidence_phase_same_space_fk
        FOREIGN KEY (phase_dossier_id, space_id)
        REFERENCES phase_dossiers (id, space_id)
        ON DELETE CASCADE,
    CONSTRAINT evidence_attachment_same_space_fk
        FOREIGN KEY (attachment_id, space_id)
        REFERENCES attachments (id, space_id)
        ON DELETE RESTRICT
);

CREATE INDEX evidence_document_idx ON evidence_items (document_id, created_at DESC) WHERE document_id IS NOT NULL;
CREATE INDEX evidence_task_idx ON evidence_items (task_dossier_id, created_at DESC) WHERE task_dossier_id IS NOT NULL;
CREATE INDEX evidence_phase_idx ON evidence_items (phase_dossier_id, created_at DESC) WHERE phase_dossier_id IS NOT NULL;
CREATE INDEX evidence_space_idx ON evidence_items (space_id, created_at DESC);
CREATE UNIQUE INDEX evidence_attachment_idx ON evidence_items (attachment_id) WHERE attachment_id IS NOT NULL;

CREATE TABLE document_templates (
    id uuid PRIMARY KEY,
    space_id uuid REFERENCES spaces (id) ON DELETE CASCADE,
    name text NOT NULL,
    document_type text NOT NULL,
    content_markdown text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT document_templates_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT document_templates_type_check CHECK (
        document_type IN ('requirements', 'research_note', 'implementation_note', 'test_plan', 'release_note')
    ),
    CONSTRAINT document_templates_content_not_blank CHECK (btrim(content_markdown) <> '')
);

CREATE UNIQUE INDEX document_templates_global_name_idx
    ON document_templates (lower(name))
    WHERE space_id IS NULL;

CREATE UNIQUE INDEX document_templates_space_name_idx
    ON document_templates (space_id, lower(name))
    WHERE space_id IS NOT NULL;

CREATE INDEX document_templates_active_idx ON document_templates (is_active) WHERE is_active = true;

CREATE TABLE audit_log (
    id uuid PRIMARY KEY,
    actor_id uuid REFERENCES users (id) ON DELETE SET NULL,
    action text NOT NULL,
    entity_type text NOT NULL,
    entity_id uuid NOT NULL,
    diff jsonb,
    request_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT audit_action_not_blank CHECK (btrim(action) <> ''),
    CONSTRAINT audit_entity_type_not_blank CHECK (btrim(entity_type) <> ''),
    CONSTRAINT audit_request_id_not_blank CHECK (btrim(request_id) <> '')
);

CREATE INDEX audit_entity_idx ON audit_log (entity_type, entity_id, created_at DESC);
CREATE INDEX audit_actor_time_idx ON audit_log (actor_id, created_at DESC) WHERE actor_id IS NOT NULL;
CREATE INDEX audit_time_idx ON audit_log (created_at DESC);
