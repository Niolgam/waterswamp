-- ==========================================================================
-- Ticket 0.1 — Versionamento OCC (DRS v3.2, seção 4.2 / RNF-01)
--
-- Toda tabela transacional crítica recebe coluna `version INTEGER NOT NULL DEFAULT 1`.
-- Toda UPDATE deve incluir: WHERE id = $id AND version = $v
-- Se rows_affected = 0 → HTTP 409 Conflict (optimistic-lock-failure).
--
-- Tabelas cobertas nesta migration:
--   vehicles  — alocação e mudanças de estado de frota
--   drivers   — credenciamento e dados de condutor
--   fuelings  — registros de abastecimento
--
-- Futuras tabelas (viagens, ordens_servico) receberão version quando
-- seus módulos forem criados.
-- ==========================================================================

ALTER TABLE vehicles ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE drivers  ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE fuelings ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
