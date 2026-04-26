DELETE FROM system_settings WHERE key IN ('abc.threshold_a', 'abc.threshold_b', 'abc.enabled');
DROP TABLE IF EXISTS abc_analysis_results;
DROP TYPE IF EXISTS abc_curve_classification_enum;
