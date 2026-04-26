ALTER TABLE suppliers
    ADD COLUMN quality_score DECIMAL(5, 2) NOT NULL DEFAULT 100.00;

COMMENT ON COLUMN suppliers.quality_score IS 'Score de qualidade do fornecedor [0,100]. Decrementado automaticamente por glosas e devoluções (RF-039).';

INSERT INTO system_settings (key, value, value_type, category, description)
VALUES
    ('supplier.quality_score_glosa_penalty', '10.0', 'number', 'supplier',
     'Pontos descontados do score do fornecedor por glosa registrada'),
    ('supplier.quality_score_min', '0.0', 'number', 'supplier',
     'Score mínimo do fornecedor (piso)')
ON CONFLICT (key) DO NOTHING;
