pub(super) const SPACE_LIST_SQL: &str = r#"
    SELECT s.id, s.key, s.name, s.description, s.owner_id,
           CASE WHEN s.archived_at IS NULL THEN 'active' ELSE 'archived' END AS status,
           (
               SELECT COUNT(*)::bigint
               FROM documents d
               WHERE d.space_id = s.id AND d.archived_at IS NULL
           ) AS document_count,
           (
               SELECT COUNT(*)::bigint
               FROM space_members sm
               WHERE sm.space_id = s.id
           ) AS member_count,
           s.created_at, s.updated_at
    FROM spaces s
    JOIN users u ON u.id = $1 AND u.is_active = true
    LEFT JOIN space_members actor_member ON actor_member.space_id = s.id AND actor_member.user_id = u.id
    WHERE u.global_role = 'admin' OR actor_member.user_id IS NOT NULL
    ORDER BY s.key
"#;

pub(super) const SPACE_ONE_SQL: &str = r#"
    SELECT s.id, s.key, s.name, s.description, s.owner_id,
           CASE WHEN s.archived_at IS NULL THEN 'active' ELSE 'archived' END AS status,
           (
               SELECT COUNT(*)::bigint
               FROM documents d
               WHERE d.space_id = s.id AND d.archived_at IS NULL
           ) AS document_count,
           (
               SELECT COUNT(*)::bigint
               FROM space_members sm
               WHERE sm.space_id = s.id
           ) AS member_count,
           s.created_at, s.updated_at
    FROM spaces s
    WHERE s.key = $1
"#;

pub(super) const EVIDENCE_ONE_SQL: &str = r#"
    SELECT e.id, s.key AS space_key, e.document_id, td.task_key, pd.phase_key,
           e.title, e.evidence_type, e.url, e.attachment_id, e.checksum,
           e.created_by, e.created_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE e.id = $1
"#;

pub(super) const EVIDENCE_LIST_SQL: &str = r#"
    SELECT e.id, s.key AS space_key, e.document_id, td.task_key, pd.phase_key,
           e.title, e.evidence_type, e.url, e.attachment_id, e.checksum,
           e.created_by, e.created_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE ($1::text IS NULL OR s.key = $1)
      AND ($2::uuid IS NULL OR e.document_id = $2)
      AND ($3::text IS NULL OR td.task_key = $3)
      AND ($4::text IS NULL OR pd.phase_key = $4)
      AND ($5::uuid IS NULL OR EXISTS (
          SELECT 1
          FROM space_members sm
          WHERE sm.space_id = e.space_id AND sm.user_id = $5
      ))
    ORDER BY e.created_at DESC
    LIMIT $6
"#;

pub(super) const EVIDENCE_TARGET_SQL: &str = r#"
    SELECT e.id, s.key AS space_key, e.document_id, td.task_key, pd.phase_key,
           e.title, e.evidence_type, e.url, e.attachment_id, e.checksum,
           e.created_by, e.created_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE ($1::uuid IS NULL OR e.task_dossier_id = $1)
      AND ($2::uuid IS NULL OR e.phase_dossier_id = $2)
    ORDER BY e.created_at DESC
"#;

pub(super) const ATTACHMENT_ONE_SQL: &str = r#"
    SELECT id, file_name, content_type, size_bytes, checksum, uploaded_by, uploaded_at
    FROM attachments
    WHERE id = $1
"#;

pub(super) const SEARCH_DOCUMENTS_SQL: &str = r#"
    WITH search_query AS (
        SELECT CASE
            WHEN NULLIF(btrim($1::text), '') IS NULL THEN NULL
            ELSE websearch_to_tsquery('simple', $1::text)
        END AS query
    ),
    matching_revisions AS MATERIALIZED (
        SELECT cr.id,
               ts_rank_cd(cr.search_vector, sq.query) AS search_rank
        FROM search_query sq
        JOIN document_revisions cr ON sq.query IS NOT NULL
        WHERE cr.search_vector @@ sq.query
    )
    SELECT d.id,
           'document' AS result_type,
           COALESCE(cr.title, d.title) AS title,
           s.key AS space_key,
           '/documents/' || d.slug AS url,
           CASE
               WHEN cr.id IS NOT NULL THEN COALESCE(NULLIF(cr.content_text, ''), cr.title)
               ELSE COALESCE(NULLIF(dd.content_markdown, ''), d.title)
           END AS snippet,
           d.updated_at,
           CASE
               WHEN sq.query IS NULL THEN 0::real
               WHEN mr.id IS NOT NULL THEN mr.search_rank
               ELSE ts_rank_cd(
                   setweight(to_tsvector('simple', coalesce(d.title, '')), 'A')
                   || setweight(to_tsvector('simple', coalesce(dd.content_markdown, '')), 'B'),
                   sq.query
               )
           END AS search_rank
    FROM search_query sq
    CROSS JOIN documents d
    JOIN spaces s ON s.id = d.space_id
    LEFT JOIN document_drafts dd ON dd.document_id = d.id
    LEFT JOIN document_revisions cr ON cr.id = d.current_revision_id
    LEFT JOIN matching_revisions mr ON mr.id = d.current_revision_id
    WHERE (
        sq.query IS NULL
        OR mr.id IS NOT NULL
        OR (
            cr.id IS NULL
            AND (
                setweight(to_tsvector('simple', coalesce(d.title, '')), 'A')
                || setweight(to_tsvector('simple', coalesce(dd.content_markdown, '')), 'B')
            ) @@ sq.query
        )
    )
      AND ($2::text IS NULL OR s.key = $2)
      AND ($3::text IS NULL OR EXISTS (
          SELECT 1
          FROM document_task_links dtl
          JOIN task_dossiers td ON td.id = dtl.task_dossier_id
          WHERE dtl.document_id = d.id AND td.task_key = $3
      ))
      AND ($4::text IS NULL OR EXISTS (
          SELECT 1
          FROM document_phase_links dpl
          JOIN phase_dossiers pd ON pd.id = dpl.phase_dossier_id
          WHERE dpl.document_id = d.id AND pd.phase_key = $4
      ))
      AND ($5::text IS NULL OR d.document_type = $5)
      AND ($6::boolean OR d.archived_at IS NULL)
      AND ($7::uuid IS NULL OR EXISTS (
          SELECT 1
          FROM space_members sm
          WHERE sm.space_id = d.space_id AND sm.user_id = $7
      ))
    ORDER BY search_rank DESC, d.updated_at DESC
    LIMIT $8
"#;

pub(super) const SEARCH_EVIDENCE_SQL: &str = r#"
    SELECT e.id,
           'evidence' AS result_type,
           e.title,
           s.key AS space_key,
           '/evidence?id=' || e.id::text AS url,
           COALESCE(e.url, e.evidence_type) AS snippet,
           e.created_at AS updated_at
    FROM evidence_items e
    JOIN spaces s ON s.id = e.space_id
    LEFT JOIN task_dossiers td ON td.id = e.task_dossier_id
    LEFT JOIN phase_dossiers pd ON pd.id = e.phase_dossier_id
    WHERE (
        $1 = '%%'
        OR lower(e.title) LIKE $1 ESCAPE E'\\'
        OR lower(COALESCE(e.url, '')) LIKE $1 ESCAPE E'\\'
    )
      AND ($2::text IS NULL OR s.key = $2)
      AND ($3::text IS NULL OR td.task_key = $3)
      AND ($4::text IS NULL OR pd.phase_key = $4)
      AND ($5::uuid IS NULL OR EXISTS (
          SELECT 1
          FROM space_members sm
          WHERE sm.space_id = e.space_id AND sm.user_id = $5
      ))
    ORDER BY e.created_at DESC
    LIMIT $6
"#;
