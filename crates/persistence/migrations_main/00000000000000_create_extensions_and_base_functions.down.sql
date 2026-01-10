-- ============================================================================
-- SIGALM - Rollback: Extensions and Base Functions
-- ============================================================================

DROP FUNCTION IF EXISTS update_updated_at_column();
DROP EXTENSION IF EXISTS postgis CASCADE;

-- Nota: Não removemos as extensões pois podem estar em uso por outros schemas
-- DROP EXTENSION IF EXISTS "pg_trgm";
-- DROP EXTENSION IF EXISTS "uuid-ossp";
