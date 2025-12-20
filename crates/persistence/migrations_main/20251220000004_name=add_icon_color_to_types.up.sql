-- Add icon and color fields to building_types and space_types tables
-- These fields are used for map legend and visual representation

-- Add icon and color to building_types
ALTER TABLE building_types ADD COLUMN IF NOT EXISTS icon VARCHAR(100) DEFAULT 'ki-outline ki-building';
ALTER TABLE building_types ADD COLUMN IF NOT EXISTS color VARCHAR(50) DEFAULT '#6B7280';

-- Add icon and color to space_types
ALTER TABLE space_types ADD COLUMN IF NOT EXISTS icon VARCHAR(100) DEFAULT 'ki-outline ki-element-11';
ALTER TABLE space_types ADD COLUMN IF NOT EXISTS color VARCHAR(50) DEFAULT '#6B7280';

-- Add icon and color to site_types
ALTER TABLE site_types ADD COLUMN IF NOT EXISTS icon VARCHAR(100) DEFAULT 'ki-outline ki-geolocation';
ALTER TABLE site_types ADD COLUMN IF NOT EXISTS color VARCHAR(50) DEFAULT '#6B7280';
