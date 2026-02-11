CREATE TABLE vehicle_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
    old_status vehicle_status_enum,
    new_status vehicle_status_enum NOT NULL,
    reason TEXT,
    changed_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vehicle_status_history_vehicle ON vehicle_status_history (vehicle_id);
CREATE INDEX idx_vehicle_status_history_date ON vehicle_status_history (created_at);
