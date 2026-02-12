-- Add last odometer tracking columns to vehicles
ALTER TABLE vehicles
    ADD COLUMN last_odometer_km INTEGER,
    ADD COLUMN last_odometer_date TIMESTAMPTZ,
    ADD COLUMN last_fueling_id UUID REFERENCES fuelings(id) ON DELETE SET NULL;

-- Index for efficient lookups by the trigger
CREATE INDEX idx_fuelings_vehicle_date_km
    ON fuelings (vehicle_id, fueling_date DESC, odometer_km DESC);

-- Trigger function to keep vehicles.last_odometer_* in sync with fuelings
CREATE OR REPLACE FUNCTION update_vehicle_last_odometer()
RETURNS TRIGGER AS $$
DECLARE
    v_vehicle_id UUID;
    v_current_last_fueling_id UUID;
    v_current_date TIMESTAMPTZ;
    v_current_km INTEGER;
    v_best RECORD;
BEGIN
    -- =============================================
    -- HANDLE DELETE
    -- =============================================
    IF TG_OP = 'DELETE' THEN
        SELECT last_fueling_id INTO v_current_last_fueling_id
        FROM vehicles WHERE id = OLD.vehicle_id;

        IF v_current_last_fueling_id = OLD.id THEN
            -- Deleted fueling was the last one: recalculate from remaining fuelings
            SELECT f.id, f.odometer_km, f.fueling_date INTO v_best
            FROM fuelings f
            WHERE f.vehicle_id = OLD.vehicle_id
            ORDER BY f.fueling_date DESC, f.odometer_km DESC
            LIMIT 1;

            UPDATE vehicles
            SET last_odometer_km = v_best.odometer_km,
                last_odometer_date = v_best.fueling_date,
                last_fueling_id = v_best.id
            WHERE id = OLD.vehicle_id;
        END IF;

        RETURN OLD;
    END IF;

    -- =============================================
    -- HANDLE UPDATE: if vehicle_id changed, fix the OLD vehicle first
    -- =============================================
    IF TG_OP = 'UPDATE' AND OLD.vehicle_id IS DISTINCT FROM NEW.vehicle_id THEN
        SELECT last_fueling_id INTO v_current_last_fueling_id
        FROM vehicles WHERE id = OLD.vehicle_id;

        IF v_current_last_fueling_id = OLD.id THEN
            -- This fueling was the last for the old vehicle: recalculate
            SELECT f.id, f.odometer_km, f.fueling_date INTO v_best
            FROM fuelings f
            WHERE f.vehicle_id = OLD.vehicle_id
            ORDER BY f.fueling_date DESC, f.odometer_km DESC
            LIMIT 1;

            UPDATE vehicles
            SET last_odometer_km = v_best.odometer_km,
                last_odometer_date = v_best.fueling_date,
                last_fueling_id = v_best.id
            WHERE id = OLD.vehicle_id;
        END IF;
    END IF;

    -- =============================================
    -- HANDLE INSERT or UPDATE: process the target vehicle
    -- =============================================
    v_vehicle_id := NEW.vehicle_id;

    SELECT last_fueling_id, last_odometer_date, last_odometer_km
    INTO v_current_last_fueling_id, v_current_date, v_current_km
    FROM vehicles WHERE id = v_vehicle_id;

    -- If updating the fueling that IS the current last, recalculate from scratch
    -- (its date or km may have decreased)
    IF TG_OP = 'UPDATE' AND OLD.vehicle_id = NEW.vehicle_id AND v_current_last_fueling_id = NEW.id THEN
        SELECT f.id, f.odometer_km, f.fueling_date INTO v_best
        FROM fuelings f
        WHERE f.vehicle_id = v_vehicle_id
        ORDER BY f.fueling_date DESC, f.odometer_km DESC
        LIMIT 1;

        UPDATE vehicles
        SET last_odometer_km = v_best.odometer_km,
            last_odometer_date = v_best.fueling_date,
            last_fueling_id = v_best.id
        WHERE id = v_vehicle_id;
    ELSE
        -- INSERT or UPDATE of a non-last fueling: compare with current values
        IF v_current_date IS NULL THEN
            -- No previous odometer data: set it
            UPDATE vehicles
            SET last_odometer_km = NEW.odometer_km,
                last_odometer_date = NEW.fueling_date,
                last_fueling_id = NEW.id
            WHERE id = v_vehicle_id;
        ELSIF NEW.fueling_date > v_current_date THEN
            -- Newer date: always update
            UPDATE vehicles
            SET last_odometer_km = NEW.odometer_km,
                last_odometer_date = NEW.fueling_date,
                last_fueling_id = NEW.id
            WHERE id = v_vehicle_id;
        ELSIF NEW.fueling_date = v_current_date AND NEW.odometer_km > v_current_km THEN
            -- Same date, higher km: update
            UPDATE vehicles
            SET last_odometer_km = NEW.odometer_km,
                last_odometer_date = NEW.fueling_date,
                last_fueling_id = NEW.id
            WHERE id = v_vehicle_id;
        END IF;
        -- Else: older date, or same date with lower/equal km — do nothing
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to fuelings table (AFTER so the row is already persisted)
CREATE TRIGGER trg_update_vehicle_last_odometer
    AFTER INSERT OR UPDATE OR DELETE ON fuelings
    FOR EACH ROW
    EXECUTE FUNCTION update_vehicle_last_odometer();

-- Backfill existing data: set last_odometer from the most recent fueling per vehicle
UPDATE vehicles v
SET last_odometer_km = sub.odometer_km,
    last_odometer_date = sub.fueling_date,
    last_fueling_id = sub.id
FROM (
    SELECT DISTINCT ON (vehicle_id)
        vehicle_id, id, odometer_km, fueling_date
    FROM fuelings
    ORDER BY vehicle_id, fueling_date DESC, odometer_km DESC
) sub
WHERE v.id = sub.vehicle_id;
