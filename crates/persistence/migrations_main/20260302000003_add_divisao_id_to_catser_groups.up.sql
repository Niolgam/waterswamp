-- ============================================================================
-- Migration: Adicionar divisao_id na tabela catser_groups
-- Conectando Grupo à hierarquia: Seção → Divisão → Grupo
-- ============================================================================

ALTER TABLE catser_groups
    ADD COLUMN divisao_id UUID REFERENCES catser_divisoes(id) ON DELETE RESTRICT;

CREATE INDEX idx_catser_groups_divisao ON catser_groups(divisao_id) WHERE divisao_id IS NOT NULL;
