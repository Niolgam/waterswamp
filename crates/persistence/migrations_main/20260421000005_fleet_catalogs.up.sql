-- ==========================================================================
-- Ticket 1.2 — RF-ADM-07/08: Catálogos Operacionais da Frota
--
-- RF-ADM-07: Catálogo de Combustíveis — vinculado a CATMAT.
-- RF-ADM-08: Catálogo de Serviços de Manutenção — vinculado a CATSER.
-- ==========================================================================

CREATE TABLE fleet_fuel_catalog (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    nome            TEXT        NOT NULL,
    catmat_item_id  UUID        REFERENCES catmat_items(id) ON DELETE SET NULL,
    unidade         TEXT        NOT NULL DEFAULT 'LITRO',
    ativo           BOOLEAN     NOT NULL DEFAULT TRUE,
    notes           TEXT,
    created_by      UUID,
    updated_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE fleet_maintenance_services (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    nome            TEXT        NOT NULL,
    catser_item_id  UUID        REFERENCES catser_items(id) ON DELETE SET NULL,
    ativo           BOOLEAN     NOT NULL DEFAULT TRUE,
    notes           TEXT,
    created_by      UUID,
    updated_by      UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fleet_fuel_catmat    ON fleet_fuel_catalog(catmat_item_id) WHERE catmat_item_id IS NOT NULL;
CREATE INDEX idx_fleet_maint_catser   ON fleet_maintenance_services(catser_item_id) WHERE catser_item_id IS NOT NULL;
CREATE INDEX idx_fleet_fuel_ativo     ON fleet_fuel_catalog(ativo) WHERE ativo = TRUE;
CREATE INDEX idx_fleet_maint_ativo    ON fleet_maintenance_services(ativo) WHERE ativo = TRUE;
