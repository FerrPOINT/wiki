-- Rollback keeps additive links and policy-compatible schema; use a qualified cohort.
DO $$ BEGIN RAISE EXCEPTION 'unified_space_schema_requires_preserving_guarded_cohort_rollback'; END $$;
