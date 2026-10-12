-- Presentation snapshots do not own resources or replace historical documents/bindings.
CREATE TABLE wiki_workspace_projects (
    registry_instance_id uuid NOT NULL,
    namespace_id uuid NOT NULL,
    tracker_instance_id uuid NOT NULL,
    tracker_project_id uuid NOT NULL,
    generation bigint NOT NULL CHECK (generation > 0),
    presentation jsonb NOT NULL,
    name text NOT NULL,
    project_key text NOT NULL,
    present boolean NOT NULL DEFAULT true,
    observed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (registry_instance_id, namespace_id),
    UNIQUE (tracker_instance_id, tracker_project_id),
    CHECK (coalesce(presentation->'binding'->'namespace'->>'registry_instance_id'=registry_instance_id::text
        AND presentation->'binding'->'namespace'->>'namespace_id'=namespace_id::text
        AND presentation->'binding'->'resource'->>'instance_id'=tracker_instance_id::text
        AND presentation->'binding'->'resource'->>'resource_id'=tracker_project_id::text
        AND presentation->'binding'->'resource'->>'kind'='tracker_project'
        AND presentation->'binding'->>'generation'=generation::text
        AND presentation->>'label'=name AND presentation->>'resource_key'=project_key, false))
);
CREATE INDEX wiki_workspace_catalog_order ON wiki_workspace_projects(name, registry_instance_id, namespace_id) WHERE present;
CREATE TABLE wiki_workspace_catalog_sync (
    id boolean PRIMARY KEY DEFAULT true CHECK (id),
    observed_at timestamptz NOT NULL DEFAULT now(),
    available boolean NOT NULL
);
