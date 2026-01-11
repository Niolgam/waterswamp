-- Change cities foreign key constraint from ON DELETE RESTRICT to ON DELETE CASCADE
-- This allows automatic deletion of cities when their parent state is deleted

ALTER TABLE cities
DROP CONSTRAINT cities_state_id_fkey;

ALTER TABLE cities
ADD CONSTRAINT cities_state_id_fkey
FOREIGN KEY (state_id)
REFERENCES states(id)
ON DELETE CASCADE;
