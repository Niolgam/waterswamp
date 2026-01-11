-- Drop triggers first
DROP TRIGGER IF EXISTS trg_org_unit_integrity ON organizational_units;
DROP TRIGGER IF EXISTS trg_log_org_units_siorg ON organizational_units;

-- Drop functions
DROP FUNCTION IF EXISTS fn_log_siorg_change() CASCADE;
DROP FUNCTION IF EXISTS fn_claim_sync_queue_item(siorg_entity_type_enum[]) CASCADE;
DROP FUNCTION IF EXISTS fn_update_descendant_paths(UUID) CASCADE;
DROP FUNCTION IF EXISTS fn_org_unit_integrity() CASCADE;
