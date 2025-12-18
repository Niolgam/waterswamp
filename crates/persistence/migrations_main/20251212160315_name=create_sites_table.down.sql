-- Drop sites table

DROP TRIGGER IF EXISTS set_sites_updated_at ON sites;
DROP INDEX IF EXISTS idx_sites_name;
DROP INDEX IF EXISTS idx_sites_site_type_id;
DROP INDEX IF EXISTS idx_sites_city_id;
DROP TABLE IF EXISTS sites;
