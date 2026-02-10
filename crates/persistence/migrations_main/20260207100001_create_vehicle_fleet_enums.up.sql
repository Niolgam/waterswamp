-- ============================
-- Vehicle Fleet Enum Types
-- ============================

CREATE TYPE vehicle_status_enum AS ENUM (
    'ACTIVE',
    'IN_MAINTENANCE',
    'RESERVED',
    'INACTIVE',
    'DECOMMISSIONING'
);

CREATE TYPE acquisition_type_enum AS ENUM (
    'PURCHASE',
    'DONATION',
    'CESSION',
    'TRANSFER'
);

CREATE TYPE decommission_reason_enum AS ENUM (
    'TOTAL_LOSS',
    'END_OF_LIFE',
    'UNECONOMICAL',
    'OTHER'
);

CREATE TYPE decommission_destination_enum AS ENUM (
    'AUCTION',
    'SCRAP',
    'DONATION',
    'OTHER'
);

CREATE TYPE document_type_enum AS ENUM (
    'CRLV',
    'INVOICE',
    'DONATION_TERM',
    'INSURANCE_POLICY',
    'TECHNICAL_REPORT',
    'PHOTO',
    'OTHER'
);
