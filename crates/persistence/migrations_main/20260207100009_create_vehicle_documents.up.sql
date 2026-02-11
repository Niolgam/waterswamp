CREATE TABLE vehicle_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    document_type document_type_enum NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    description TEXT,
    uploaded_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_documents_vehicle ON vehicle_documents (vehicle_id);
CREATE INDEX idx_vehicle_documents_type ON vehicle_documents (document_type);

CREATE TRIGGER set_vehicle_documents_updated_at
    BEFORE UPDATE ON vehicle_documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
