-- Reverte a migration odometer_series

DROP INDEX IF EXISTS idx_leituras_hodometro_quarentena;
DROP INDEX IF EXISTS idx_leituras_hodometro_veiculo_status;
DROP INDEX IF EXISTS idx_leituras_hodometro_request_id;
DROP TABLE IF EXISTS leituras_hodometro;
DROP TYPE IF EXISTS leituras_hodometro_status_enum;
DROP TYPE IF EXISTS leituras_hodometro_fonte_enum;
