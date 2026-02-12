CREATE TABLE fuelings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- References
    vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE RESTRICT,
    driver_id UUID NOT NULL REFERENCES drivers(id) ON DELETE RESTRICT,
    supplier_id UUID REFERENCES suppliers(id) ON DELETE SET NULL,
    fuel_type_id UUID NOT NULL REFERENCES vehicle_fuel_types(id) ON DELETE RESTRICT,
    -- Fueling data
    fueling_date TIMESTAMPTZ NOT NULL,
    odometer_km INTEGER NOT NULL,
    quantity_liters NUMERIC(10,3) NOT NULL,
    unit_price NUMERIC(10,4) NOT NULL,
    total_cost NUMERIC(12,2) NOT NULL,
    -- Optional info
    notes TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    -- Constraints
    CONSTRAINT ck_fuelings_quantity_positive CHECK (quantity_liters > 0),
    CONSTRAINT ck_fuelings_unit_price_positive CHECK (unit_price > 0),
    CONSTRAINT ck_fuelings_total_cost_positive CHECK (total_cost > 0),
    CONSTRAINT ck_fuelings_odometer_positive CHECK (odometer_km >= 0)
);

CREATE INDEX idx_fuelings_vehicle ON fuelings (vehicle_id);
CREATE INDEX idx_fuelings_driver ON fuelings (driver_id);
CREATE INDEX idx_fuelings_supplier ON fuelings (supplier_id);
CREATE INDEX idx_fuelings_fuel_type ON fuelings (fuel_type_id);
CREATE INDEX idx_fuelings_date ON fuelings (fueling_date DESC);

CREATE TRIGGER set_fuelings_updated_at
    BEFORE UPDATE ON fuelings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
