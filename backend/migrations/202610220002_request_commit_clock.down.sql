-- Old images bypassing the fence are not valid rollback targets after activation.
DO $$ BEGIN RAISE EXCEPTION 'compatible_cohort_rollback_required'; END $$;
