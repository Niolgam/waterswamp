-- ============================================================================
-- Migration: Create Organizational Units Enums
-- Description: Enums for organizational structure and SIORG integration
-- ============================================================================

-- Área de atuação da unidade
CREATE TYPE activity_area_enum AS ENUM (
    'SUPPORT',  -- Área meio (administrativo)
    'CORE'      -- Área fim (acadêmico/pesquisa)
);

-- Tipo interno de unidade (classificação local)
CREATE TYPE internal_unit_type_enum AS ENUM (
    'ADMINISTRATION',  -- Reitoria, Pró-reitorias
    'DEPARTMENT',      -- Departamentos acadêmicos
    'LABORATORY',      -- Laboratórios de pesquisa/ensino
    'SECTOR',          -- Setores administrativos menores
    'COUNCIL',         -- Conselhos deliberativos
    'COORDINATION',    -- Coordenações de curso
    'CENTER',          -- Centros de pesquisa/extensão
    'DIVISION'         -- Divisões administrativas
);

-- Tipo de entidade no SIORG
CREATE TYPE siorg_entity_type_enum AS ENUM (
    'ORGANIZATION',  -- Órgão raiz (ex: UFMT)
    'UNIT',          -- Unidade organizacional
    'CATEGORY',      -- Categoria de unidade
    'TYPE'           -- Tipo de unidade
);

-- Tipo de mudança no histórico SIORG
CREATE TYPE siorg_change_type_enum AS ENUM (
    'CREATION',          -- Criação de nova entidade
    'UPDATE',            -- Atualização de dados
    'EXTINCTION',        -- Extinção/desativação
    'HIERARCHY_CHANGE',  -- Mudança de subordinação
    'MERGE',             -- Fusão de unidades
    'SPLIT'              -- Desmembramento
);

-- Status de sincronização
CREATE TYPE sync_status_enum AS ENUM (
    'PENDING',      -- Aguardando processamento
    'PROCESSING',   -- Em processamento
    'COMPLETED',    -- Concluído com sucesso
    'FAILED',       -- Falhou (erro técnico)
    'CONFLICT',     -- Conflito detectado (requer revisão manual)
    'SKIPPED'       -- Ignorado (decisão manual)
);

-- Status de mapeamento SIORG
CREATE TYPE mapping_status_enum AS ENUM (
    'ACTIVE',      -- Mapeamento ativo
    'DEPRECATED',  -- Código antigo (mantido para histórico)
    'MERGED',      -- Fundido em outra unidade
    'UNMAPPED'     -- Sem correspondência no SIORG
);

COMMENT ON TYPE activity_area_enum IS 'Classifica se a unidade é área meio (suporte) ou fim (core)';
COMMENT ON TYPE internal_unit_type_enum IS 'Classificação interna de tipos de unidades organizacionais';
COMMENT ON TYPE siorg_entity_type_enum IS 'Tipos de entidades no Sistema SIORG';
COMMENT ON TYPE siorg_change_type_enum IS 'Tipos de mudanças registradas no histórico SIORG';
COMMENT ON TYPE sync_status_enum IS 'Status de processamento de sincronização';
COMMENT ON TYPE mapping_status_enum IS 'Status de mapeamento entre entidades locais e SIORG';
