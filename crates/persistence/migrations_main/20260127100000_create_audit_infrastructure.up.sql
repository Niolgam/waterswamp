-- Migration: Infraestrutura de Auditoria Avançada
-- ============================================================================
-- Este migration cria a base para auditoria detalhada e rollbacks controlados
-- ============================================================================

-- ============================================================================
-- 1. CONFIGURAÇÃO DE CONTEXTO DE SESSÃO
-- ============================================================================
-- Permite que a aplicação defina o usuário atual para os triggers de auditoria

CREATE OR REPLACE FUNCTION fn_set_audit_context(
    p_user_id UUID,
    p_ip_address TEXT DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
    PERFORM set_config('audit.current_user_id', p_user_id::TEXT, TRUE);
    PERFORM set_config('audit.ip_address', COALESCE(p_ip_address, ''), TRUE);
    PERFORM set_config('audit.user_agent', COALESCE(p_user_agent, ''), TRUE);
END;
$$ LANGUAGE plpgsql;

-- Função helper para recuperar contexto de forma segura
CREATE OR REPLACE FUNCTION fn_get_audit_user_id() 
RETURNS UUID AS $$
DECLARE
    v_user_id TEXT;
BEGIN
    v_user_id := current_setting('audit.current_user_id', TRUE);
    IF v_user_id IS NULL OR v_user_id = '' THEN
        RETURN NULL;
    END IF;
    RETURN v_user_id::UUID;
EXCEPTION WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION fn_get_audit_ip() 
RETURNS INET AS $$
DECLARE
    v_ip TEXT;
BEGIN
    v_ip := current_setting('audit.ip_address', TRUE);
    IF v_ip IS NULL OR v_ip = '' THEN
        RETURN NULL;
    END IF;
    RETURN v_ip::INET;
EXCEPTION WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- 2. ENUM PARA TIPOS DE OPERAÇÃO
-- ============================================================================

DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'audit_operation_enum') THEN
        CREATE TYPE audit_operation_enum AS ENUM (
            'INSERT',
            'UPDATE', 
            'DELETE',
            'SOFT_DELETE',
            'RESTORE',
            'ROLLBACK',
            'STATUS_CHANGE',
            'APPROVAL',
            'REJECTION',
            'CANCELLATION'
        );
    END IF;
END $$;

-- ============================================================================
-- 3. TABELA GENÉRICA DE CHANGELOG (para entidades menos críticas)
-- ============================================================================

CREATE TABLE IF NOT EXISTS entity_changelog (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Identificação da entidade
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    
    -- Operação
    operation audit_operation_enum NOT NULL,
    
    -- Dados (snapshots)
    data_before JSONB,
    data_after JSONB,
    
    -- Campos alterados (para facilitar consultas)
    changed_fields TEXT[],
    
    -- Contexto da operação
    performed_by UUID,
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT,
    
    -- Justificativa (obrigatória para algumas operações)
    reason TEXT,
    
    -- Agrupamento de operações (mesma transação)
    transaction_id UUID DEFAULT gen_random_uuid(),
    
    -- Controle de rollback
    is_rollback BOOLEAN NOT NULL DEFAULT FALSE,
    rollback_source_id UUID REFERENCES entity_changelog(id)
);

-- Índices para entity_changelog
CREATE INDEX idx_changelog_entity 
    ON entity_changelog(entity_type, entity_id, performed_at DESC);
CREATE INDEX idx_changelog_operation 
    ON entity_changelog(operation);
CREATE INDEX idx_changelog_user 
    ON entity_changelog(performed_by) WHERE performed_by IS NOT NULL;
CREATE INDEX idx_changelog_date 
    ON entity_changelog(performed_at DESC);
CREATE INDEX idx_changelog_transaction 
    ON entity_changelog(transaction_id);
CREATE INDEX idx_changelog_rollbacks 
    ON entity_changelog(entity_type, entity_id) WHERE is_rollback = TRUE;

-- Índice GIN para busca em campos alterados
CREATE INDEX idx_changelog_changed_fields 
    ON entity_changelog USING GIN(changed_fields);

-- ============================================================================
-- 4. FUNÇÃO UTILITÁRIA: EXTRAIR CAMPOS ALTERADOS
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_get_changed_fields(
    p_old JSONB,
    p_new JSONB
) RETURNS TEXT[] AS $$
DECLARE
    v_changed TEXT[] := '{}';
    v_key TEXT;
BEGIN
    -- Campos removidos ou alterados
    FOR v_key IN SELECT key FROM jsonb_each(p_old)
    LOOP
        IF NOT p_new ? v_key OR p_old->v_key IS DISTINCT FROM p_new->v_key THEN
            v_changed := array_append(v_changed, v_key);
        END IF;
    END LOOP;
    
    -- Campos adicionados
    FOR v_key IN SELECT key FROM jsonb_each(p_new)
    LOOP
        IF NOT p_old ? v_key THEN
            v_changed := array_append(v_changed, v_key);
        END IF;
    END LOOP;
    
    RETURN v_changed;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- 5. FUNÇÃO PARA COMPARAR JSONB E GERAR DIFF LEGÍVEL
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_generate_diff(
    p_old JSONB,
    p_new JSONB
) RETURNS JSONB AS $$
DECLARE
    v_diff JSONB := '{}';
    v_key TEXT;
    v_old_val JSONB;
    v_new_val JSONB;
BEGIN
    -- Percorrer todos os campos
    FOR v_key IN 
        SELECT DISTINCT key 
        FROM (
            SELECT key FROM jsonb_each(COALESCE(p_old, '{}'))
            UNION
            SELECT key FROM jsonb_each(COALESCE(p_new, '{}'))
        ) keys
    LOOP
        v_old_val := p_old->v_key;
        v_new_val := p_new->v_key;
        
        IF v_old_val IS DISTINCT FROM v_new_val THEN
            v_diff := v_diff || jsonb_build_object(
                v_key, jsonb_build_object(
                    'old', v_old_val,
                    'new', v_new_val
                )
            );
        END IF;
    END LOOP;
    
    RETURN v_diff;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- COMENTÁRIOS
-- ============================================================================

COMMENT ON TABLE entity_changelog IS 'Log genérico de alterações em entidades do sistema';
COMMENT ON FUNCTION fn_set_audit_context IS 'Define contexto de auditoria (usuário, IP) para a sessão atual';
COMMENT ON FUNCTION fn_get_changed_fields IS 'Extrai lista de campos alterados entre dois JSONBs';
COMMENT ON FUNCTION fn_generate_diff IS 'Gera diff detalhado entre dois estados JSONB';
