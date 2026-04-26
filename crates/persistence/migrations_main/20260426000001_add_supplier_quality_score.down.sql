ALTER TABLE suppliers DROP COLUMN IF EXISTS quality_score;

DELETE FROM system_settings WHERE key IN (
    'supplier.quality_score_glosa_penalty',
    'supplier.quality_score_min'
);
