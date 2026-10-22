-- Restore the guarded cohort and its data together; do not reinstall a known bad guard.
DO $$ BEGIN
    RAISE EXCEPTION 'unified_space_schema_requires_preserving_guarded_cohort_rollback';
END $$;
