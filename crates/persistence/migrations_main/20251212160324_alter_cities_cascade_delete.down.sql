-- Revert cities foreign key constraint back to ON DELETE RESTRICT

ALTER TABLE cities
DROP CONSTRAINT cities_state_id_fkey;

ALTER TABLE cities
ADD CONSTRAINT cities_state_id_fkey
FOREIGN KEY (state_id)
REFERENCES states(id)
ON DELETE RESTRICT;
