CREATE INDEX task_dossiers_catalog_key_idx
    ON task_dossiers (space_id, task_key COLLATE "C");

CREATE INDEX phase_dossiers_catalog_key_idx
    ON phase_dossiers (space_id, phase_key COLLATE "C");
