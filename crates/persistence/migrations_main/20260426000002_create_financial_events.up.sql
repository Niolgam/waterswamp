CREATE TYPE financial_event_type_enum AS ENUM (
    'GLOSA_CRIADA',         -- Glosa registrada em NF lançada
    'DEVOLUCAO_CRIADA',     -- Devolução de material ao fornecedor
    'ESTORNO_LANCAMENTO',   -- Estorno de lançamento de NF (dentro de 24h)
    'EMPENHO_VALIDADO',     -- Saldo de empenho validado via Comprasnet
    'EMPENHO_INSUFICIENTE'  -- Tentativa de lançamento excedeu saldo do empenho
);

CREATE TABLE financial_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type financial_event_type_enum NOT NULL,

    -- Referências opcionais ao contexto do evento
    invoice_id UUID REFERENCES invoices(id) ON DELETE SET NULL,
    invoice_adjustment_id UUID REFERENCES invoice_adjustments(id) ON DELETE SET NULL,
    supplier_id UUID REFERENCES suppliers(id) ON DELETE SET NULL,
    warehouse_id UUID REFERENCES warehouses(id) ON DELETE SET NULL,

    -- Valores financeiros envolvidos
    amount DECIMAL(15, 2),

    -- Número de empenho (para eventos de empenho)
    commitment_number VARCHAR(100),

    -- Payload livre para metadados adicionais
    metadata JSONB,

    -- Auditoria
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_financial_events_event_type ON financial_events (event_type);
CREATE INDEX idx_financial_events_invoice ON financial_events (invoice_id) WHERE invoice_id IS NOT NULL;
CREATE INDEX idx_financial_events_supplier ON financial_events (supplier_id) WHERE supplier_id IS NOT NULL;
CREATE INDEX idx_financial_events_created_at ON financial_events (created_at DESC);
CREATE INDEX idx_financial_events_commitment ON financial_events (commitment_number) WHERE commitment_number IS NOT NULL;

COMMENT ON TABLE financial_events IS 'Log imutável de eventos financeiros (RF-028). Serve como trilha de auditoria financeira e base para integrações futuras.';
