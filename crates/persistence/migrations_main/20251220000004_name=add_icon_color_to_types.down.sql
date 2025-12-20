-- Remove icon and color fields from type tables

-- Remove from site_types
ALTER TABLE site_types DROP COLUMN IF EXISTS color;
ALTER TABLE site_types DROP COLUMN IF EXISTS icon;

-- Remove from space_types
ALTER TABLE space_types DROP COLUMN IF EXISTS color;
ALTER TABLE space_types DROP COLUMN IF EXISTS icon;

-- Remove from building_types
ALTER TABLE building_types DROP COLUMN IF EXISTS color;
ALTER TABLE building_types DROP COLUMN IF EXISTS icon;
