-- RF-ABT: estende fuelings com catálogo de combustível, vínculo de viagem e consumo calculado
ALTER TABLE fuelings
    ADD COLUMN fuel_catalog_id UUID REFERENCES fleet_fuel_catalog(id) ON DELETE SET NULL,
    ADD COLUMN trip_id         UUID REFERENCES vehicle_trips(id) ON DELETE SET NULL,
    ADD COLUMN km_anterior     INTEGER;

ALTER TABLE fuelings
    ADD COLUMN consumo_litros_100km NUMERIC(8,2) GENERATED ALWAYS AS (
        CASE
            WHEN km_anterior IS NOT NULL
             AND odometer_km IS NOT NULL
             AND odometer_km > km_anterior
             AND quantity_liters > 0
            THEN ROUND(
                (quantity_liters / NULLIF((odometer_km - km_anterior)::NUMERIC, 0)) * 100,
                2
            )
            ELSE NULL
        END
    ) STORED;

CREATE INDEX idx_fuelings_fuel_catalog ON fuelings(fuel_catalog_id) WHERE fuel_catalog_id IS NOT NULL;
CREATE INDEX idx_fuelings_trip         ON fuelings(trip_id)         WHERE trip_id IS NOT NULL;
CREATE INDEX idx_fuelings_vehicle_date ON fuelings(vehicle_id, fueling_date DESC);
