CREATE TYPE import_job_status_enum AS ENUM (
    'PENDING',
    'RUNNING',
    'COMPLETED',
    'FAILED',
    'PARTIAL'
);

CREATE TYPE import_entity_type_enum AS ENUM (
    'SUPPLIER',
    'CATALOG_ITEM',
    'INITIAL_STOCK'
);

CREATE TABLE legacy_import_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    entity_type import_entity_type_enum NOT NULL,
    status import_job_status_enum NOT NULL DEFAULT 'PENDING',
    submitted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    total_records INTEGER NOT NULL DEFAULT 0,
    processed_records INTEGER NOT NULL DEFAULT 0,
    success_records INTEGER NOT NULL DEFAULT 0,
    failed_records INTEGER NOT NULL DEFAULT 0,
    error_log JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_import_jobs_status ON legacy_import_jobs(status);
CREATE INDEX idx_import_jobs_entity_type ON legacy_import_jobs(entity_type, created_at DESC);
CREATE INDEX idx_import_jobs_submitted_by ON legacy_import_jobs(submitted_by) WHERE submitted_by IS NOT NULL;

CREATE TRIGGER set_timestamp_legacy_import_jobs
BEFORE UPDATE ON legacy_import_jobs
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
