CREATE TYPE abc_curve_classification_enum AS ENUM ('A', 'B', 'C');

CREATE TABLE abc_analysis_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    warehouse_id UUID REFERENCES warehouses(id) ON DELETE CASCADE,
    catalog_item_id UUID NOT NULL REFERENCES catmat_items(id) ON DELETE CASCADE,
    classification abc_curve_classification_enum NOT NULL,
    total_value DECIMAL(15, 4) NOT NULL DEFAULT 0,
    cumulative_percentage DECIMAL(8, 6) NOT NULL,
    rank_position INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_abc_results_run_warehouse ON abc_analysis_results(run_at DESC, warehouse_id);
CREATE INDEX idx_abc_results_classification ON abc_analysis_results(run_at DESC, classification);
CREATE INDEX idx_abc_results_catalog_item ON abc_analysis_results(catalog_item_id);

INSERT INTO system_settings (key, value, description) VALUES
    ('abc.threshold_a', '0.80', 'ABC curve: cumulative value % threshold for class A (top 80%)'),
    ('abc.threshold_b', '0.95', 'ABC curve: cumulative value % threshold for class B (next 15%, C is remainder)'),
    ('abc.enabled',     'true', 'Enable ABC analysis feature')
ON CONFLICT (key) DO NOTHING;
