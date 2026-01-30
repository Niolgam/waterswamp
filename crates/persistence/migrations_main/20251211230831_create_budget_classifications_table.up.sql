CREATE TABLE budget_classifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- O ID do nível superior (Ex: O 'Elemento' aponta para a 'Modalidade')
    parent_id UUID REFERENCES budget_classifications(id) ON DELETE RESTRICT,
    
    -- O código apenas deste nível (Ex: '30')
    code_part VARCHAR(10) NOT NULL,
    
    -- O código completo calculado/armazenado para busca rápida (Ex: '3.3.90.30')
    full_code VARCHAR(30) UNIQUE NOT NULL,
    
    name VARCHAR(255) NOT NULL,
    
    -- Nível da classificação (1 a 5)
    -- 1: Categoria Econômica, 2: Grupo de Despesa, 3: Modalidade, 4: Elemento, 5: Subelemento
    level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 5),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices para navegação na árvore
CREATE INDEX idx_budget_classifications_parent ON budget_classifications(parent_id);
CREATE INDEX idx_budget_classifications_level ON budget_classifications(level);
CREATE INDEX idx_budget_classifications_full_code ON budget_classifications(full_code);
CREATE INDEX idx_budget_classifications_active ON budget_classifications(is_active) WHERE is_active = TRUE;

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_budget_classifications
BEFORE UPDATE ON budget_classifications
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Função para calcular hierarquia automaticamente
CREATE OR REPLACE FUNCTION fn_calculate_budget_hierarchy()
RETURNS TRIGGER AS $$
DECLARE
    v_parent_full_code VARCHAR(30);
    v_parent_level INTEGER;
BEGIN
    -- Caso 1: É um item de nível 1 (Categoria Econômica)
    IF NEW.parent_id IS NULL THEN
        NEW.level := 1;
        NEW.full_code := NEW.code_part;
    
    -- Caso 2: É um item dependente (níveis 2 a 5)
    ELSE
        -- Busca o código completo e o nível do pai
        SELECT full_code, level 
        INTO v_parent_full_code, v_parent_level
        FROM budget_classifications 
        WHERE id = NEW.parent_id;

        -- Validação de segurança: Não permitir mais que 5 níveis
        IF v_parent_level >= 5 THEN
            RAISE EXCEPTION 'A Classificação Orçamentária não pode exceder 5 níveis.';
        END IF;

        NEW.level := v_parent_level + 1;
        NEW.full_code := v_parent_full_code || '.' || NEW.code_part;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_budget_hierarchy_auto
BEFORE INSERT OR UPDATE OF parent_id, code_part ON budget_classifications
FOR EACH ROW
EXECUTE FUNCTION fn_calculate_budget_hierarchy();

DO $$ 
DECLARE
    -- IDs de Nível 1
    v_cat_3 UUID; v_cat_4 UUID; v_cat_9 UUID;
    -- IDs de Nível 2
    v_gnd_3_1 UUID; v_gnd_3_3 UUID; v_gnd_4_4 UUID; v_gnd_4_5 UUID;
    -- IDs de Nível 3
    v_mod_3_3_90 UUID; v_mod_4_4_90 UUID;
    -- IDs de Nível 4 (Elementos específicos para capturar subelementos)
    v_elem_30 UUID; v_elem_33 UUID; v_elem_51 UUID; v_elem_52 UUID;

BEGIN
    -- ========================================================================
    -- NÍVEL 1: CATEGORIA ECONÔMICA
    -- ========================================================================
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('3', 'DESPESA CORRENTE', NULL) RETURNING id INTO v_cat_3;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('4', 'DESPESA DE CAPITAL', NULL) RETURNING id INTO v_cat_4;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('9', 'RESERVA DE CONTINGÊNCIA', NULL) RETURNING id INTO v_cat_9;

    -- ========================================================================
    -- NÍVEL 2: GRUPO DE NATUREZA DA DESPESA (GND)
    -- ========================================================================
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('1', 'PESSOAL E ENCARGOS SOCIAIS', v_cat_3) RETURNING id INTO v_gnd_3_1;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('3', 'OUTRAS DESPESAS CORRENTES', v_cat_3) RETURNING id INTO v_gnd_3_3;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('4', 'INVESTIMENTOS', v_cat_4) RETURNING id INTO v_gnd_4_4;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('5', 'INVERSÕES FINANCEIRAS', v_cat_4) RETURNING id INTO v_gnd_4_5;

    -- ========================================================================
    -- NÍVEL 3: MODALIDADE DE APLICAÇÃO
    -- ========================================================================
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('90', 'APLICAÇÕES DIRETAS', v_gnd_3_3) RETURNING id INTO v_mod_3_3_90;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('90', 'APLICAÇÕES DIRETAS', v_gnd_4_4) RETURNING id INTO v_mod_4_4_90;

    -- ========================================================================
    -- NÍVEL 4: ELEMENTO DE DESPESA
    -- ========================================================================
    
    -- Elementos sob 3.3.90 (Custeio)
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('14', 'DIÁRIAS - CIVIL', v_mod_3_3_90);
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('30', 'MATERIAL DE CONSUMO', v_mod_3_3_90) RETURNING id INTO v_elem_30;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('33', 'PASSAGENS E DESPESAS COM LOCOMOÇÃO', v_mod_3_3_90) RETURNING id INTO v_elem_33;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('36', 'OUTROS SERVIÇOS DE TERCEIROS - PESSOA FÍSICA', v_mod_3_3_90);
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('39', 'OUTROS SERVIÇOS DE TERCEIROS - PESSOA JURÍDICA', v_mod_3_3_90);

    -- Elementos sob 4.4.90 (Capital)
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('51', 'OBRAS E INSTALAÇÕES', v_mod_4_4_90) RETURNING id INTO v_elem_51;
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES ('52', 'EQUIPAMENTOS E MATERIAL PERMANENTE', v_mod_4_4_90) RETURNING id INTO v_elem_52;

    -- ========================================================================
    -- NÍVEL 5: SUBELEMENTOS (Extraídos da Planilha 2025)
    -- ========================================================================

    -- SUBELEMENTOS DE 3.3.90.30 (MATERIAL DE CONSUMO)
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES 
    ('01', 'COMBUSTÍVEIS E LUBRIFICANTES AUTOMOTIVOS', v_elem_30),
    ('02', 'COMBUSTÍVEIS E LUBRIFICANTES DE AVIAÇÃO', v_elem_30),
    ('03', 'COMBUSTÍVEIS E LUBRIFICANTES P/ OUTRAS FINALIDADES', v_elem_30),
    ('04', 'GÁS E OUTROS MATERIAIS ENGARRAFADOS', v_elem_30),
    ('05', 'EXPLOSIVOS E MUNINÇÕES', v_elem_30),
    ('06', 'ALIMENTOS PARA ANIMAIS', v_elem_30),
    ('07', 'GÊNEROS DE ALIMENTAÇÃO', v_elem_30),
    ('08', 'ANIMAIS PARA PESQUISA E ABATE', v_elem_30),
    ('09', 'MATERIAL FARMACOLÓGICO', v_elem_30),
    ('10', 'MATERIAL ODONTOLÓGICO', v_elem_30),
    ('11', 'MATERIAL QUÍMICO', v_elem_30),
    ('12', 'MATERIAL EDUCATIVO E ESPORTIVO', v_elem_30),
    ('13', 'MATERIAL DE CULTURA E DIVULGAÇÃO', v_elem_30),
    ('14', 'MATERIAL EDUCATIVO E DE CULTURA - DISTRIBUIÇÃO GRATUITA', v_elem_30),
    ('15', 'MATERIAL PARA FESTIVIDADES E HOMENAGENS', v_elem_30),
    ('16', 'MATERIAL DE EXPEDIENTE', v_elem_30),
    ('17', 'MATERIAL DE PROCESSAMENTO DE DADOS', v_elem_30),
    ('18', 'MATERIAIS E MEDICAMENTOS P/ USO VETERINÁRIO', v_elem_30),
    ('19', 'MATERIAL DE ACONDICIONAMENTO E EMBALAGEM', v_elem_30),
    ('20', 'MATERIAL DE CAMA, MESA E BANHO', v_elem_30),
    ('21', 'MATERIAL DE COPA E COZINHA', v_elem_30),
    ('22', 'MATERIAL DE LIMPEZA E PRODUTOS DE HIGIENIZAÇÃO', v_elem_30),
    ('23', 'UNIFORMES, TECIDOS E AVIAMENTOS', v_elem_30),
    ('24', 'MATERIAL DE PROTEÇÃO E SEGURANÇA', v_elem_30),
    ('25', 'MATERIAL P/ MANUTENÇÃO DE BENS IMÓVEIS/INSTALAÇÕES', v_elem_30),
    ('26', 'MATERIAL ELÉTRICO E ELETRÔNICO', v_elem_30),
    ('27', 'MATERIAL DE MANOBRA E PATRULHAMENTO', v_elem_30),
    ('28', 'MATERIAL DE PROTEÇÃO E SOCORRO', v_elem_30),
    ('29', 'MATERIAL P/ ÁUDIO, VÍDEO E FOTO', v_elem_30),
    ('30', 'MATERIAL PARA COMUNICAÇÕES', v_elem_30),
    ('31', 'SEMENTES, MUDAS DE PLANTAS E INSUMOS', v_elem_30),
    ('32', 'SUPRIMENTO DE AVIAÇÃO', v_elem_30),
    ('33', 'MATERIAL P/ PRODUÇÃO INDUSTRIAL', v_elem_30),
    ('34', 'MATERIAL P/ PRODUÇÃO AGROPECUÁRIA', v_elem_30),
    ('35', 'MATERIAL HOSPITALAR', v_elem_30),
    ('36', 'MATERIAL LABORATORIAL', v_elem_30),
    ('37', 'MATERIAL PARA MANUTENÇÃO DE BENS MÓVEIS', v_elem_30),
    ('38', 'MATERIAL PARA MANUTENÇÃO DE AERONAVES', v_elem_30),
    ('39', 'MATERIAL PARA MANUTENÇÃO DE VEÍCULOS', v_elem_30),
    ('40', 'MATERIAL BIOLÓGICO', v_elem_30),
    ('41', 'MATERIAL PARA UTILIZAÇÃO EM CRIPTAGEM', v_elem_30),
    ('42', 'FERRAMENTAS', v_elem_30),
    ('43', 'MATERIAL PARA REABILITAÇÃO PROFISSIONAL', v_elem_30),
    ('44', 'SINALIZAÇÃO VISUAL E OUTROS', v_elem_30),
    ('45', 'MATERIAL TÉCNICO PARA SELEÇÃO E TREINAMENTO', v_elem_30),
    ('46', 'MATERIAL BIBLIOGRÁFICO NÃO IMOBILIZÁVEL', v_elem_30),
    ('47', 'AQUISIÇÃO DE SOFTWARES NÃO IMOBILIZÁVEIS', v_elem_30),
    ('48', 'BANDEIRAS, FLÂMULAS E INSÍGNIAS', v_elem_30),
    ('49', 'SUPRIMENTO DE FUNDOS', v_elem_30),
    ('50', 'BANDEIRAS, FLÂMULAS E INSÍGNIAS', v_elem_30),
    ('51', 'CDs E DVDs PARA DISTRIBUIÇÃO GRATUITA', v_elem_30),
    ('52', 'MATERIAL DE CONSUMO - PAGAMENTO ANTECIPADO', v_elem_30),
    ('53', 'MATERIAL DE CONSUMO - INTERMÉDIO DE CONSÓRCIO PÚBLICO', v_elem_30),
    ('99', 'OUTROS MATERIAIS DE CONSUMO', v_elem_30);

    -- SUBELEMENTOS DE 3.3.90.33 (PASSAGENS E DESPESAS COM LOCOMOÇÃO)
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES 
    ('01', 'PASSAGENS PARA O PAÍS', v_elem_33),
    ('02', 'PASSAGENS PARA O EXTERIOR', v_elem_33),
    ('03', 'LOCAÇÃO DE MEIOS DE TRANSPORTE', v_elem_33),
    ('04', 'MUDANÇAS EM OBJETO DE SERVIÇOS', v_elem_33),
    ('05', 'FRETES E TRANSPORTES DE ENCOMENDAS', v_elem_33),
    ('06', 'TRANSPORTE DE SERVIDORES', v_elem_33),
    ('08', 'PEDÁGIOS', v_elem_33),
    ('09', 'LOCOMOÇÃO URBANA', v_elem_33),
    ('99', 'OUTRAS DESPESAS COM LOCOMOÇÃO', v_elem_33);

    -- SUBELEMENTOS DE 4.4.90.51 (OBRAS E INSTALAÇÕES)
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES 
    ('03', 'INSTALAÇÕES', v_elem_51),
    ('04', 'BENFEITORIAS EM PROPRIEDADES DE TERCEIROS', v_elem_51),
    ('05', 'ALMOXARIFADO DE OBRAS', v_elem_51),
    ('80', 'ESTUDOS E PROJETOS', v_elem_51);

    -- SUBELEMENTOS DE 4.4.90.52 (EQUIPAMENTO E MATERIAL PERMANENTE)
    INSERT INTO budget_classifications (code_part, name, parent_id) VALUES 
    ('01', 'AERONAVES', v_elem_52),
    ('02', 'APARELHOS DE MEDIÇÃO E ORIENTAÇÃO', v_elem_52),
    ('03', 'APARELHOS E EQUIPAMENTOS DE COMUNICAÇÃO', v_elem_52),
    ('04', 'COLEÇÕES E MATERIAIS BIBLIOGRÁFICOS', v_elem_52),
    ('05', 'EQUIPAMENTO DE PROTEÇÃO, VIGILÂNCIA E SEGURANÇA', v_elem_52),
    ('06', 'EQUIPAMENTOS DE MANOBRA E PATRULHAMENTO', v_elem_52),
    ('08', 'EQUIPAMENTOS, PEÇAS E ACESSÓRIOS AERONÁUTICOS', v_elem_52),
    ('09', 'EQUIPAMENTOS, PEÇAS E ACESSÓRIOS MARÍTIMOS', v_elem_52),
    ('10', 'EQUIPAMENTOS, PEÇAS E ACESSÓRIOS DE PROTEÇÃO AO VÔO', v_elem_52),
    ('11', 'EQUIPAMENTOS E ELEMENTOS DE SINALIZAÇÃO', v_elem_52),
    ('12', 'EQUIPAMENTOS E UTENSÍLIOS HIDRÁULICOS E ELÉTRICOS', v_elem_52),
    ('13', 'EQUIPAMENTOS PARA ÁUDIO, VÍDEO E FOTO', v_elem_52),
    ('14', 'EQUIPAMENTOS PARA CINE-FOTO-SOM', v_elem_52),
    ('15', 'EQUIPAMENTOS PARA ESPORTES E DIVERSÕES', v_elem_52),
    ('16', 'EQUIPAMENTOS PARA ESCRITÓRIO', v_elem_52),
    ('17', 'MÁQUINAS E EQUIPAMENTOS AGRÍCOLAS E RODOVIÁRIOS', v_elem_52),
    ('18', 'MÁQUINAS E EQUIPAMENTOS ENERGÉTICOS', v_elem_52),
    ('19', 'MÁQUINAS E EQUIPAMENTOS INDUSTRIAIS', v_elem_52),
    ('20', 'MÁQUINAS, EQUIPAMENTOS E UTENSÍLIOS AGROPECUÁRIOS', v_elem_52),
    ('21', 'MÁQUINAS, EQUIPAMENTOS E UTENSÍLIOS DE COZINHA', v_elem_52),
    ('22', 'MÁQUINAS, EQUIPAMENTOS E UTENSÍLIOS MÉDICO-HOSPITALAR', v_elem_52),
    ('23', 'MÁQUINAS, EQUIPAMENTOS E UTENSÍLIOS ODONTOLÓGICOS', v_elem_52),
    ('24', 'MÁQUINAS, FERRAMENTAS E UTENSÍLIOS DE OFICINA', v_elem_52),
    ('25', 'MOTORES E BOMBAS', v_elem_52),
    ('26', 'MÓVEIS E UTENSÍLIOS', v_elem_52),
    ('27', 'VEÍCULOS DE TRAÇÃO MECÂNICA', v_elem_52),
    ('28', 'VEÍCULOS FERROVIÁRIOS', v_elem_52),
    ('29', 'OUTROS MATERIAIS PERMANENTES', v_elem_52),
    ('30', 'EQUIPAMENTOS PARA COMUNICAÇÕES', v_elem_52),
    ('31', 'SEMENTES, MUDAS DE PLANTAS E INSUMOS', v_elem_52),
    ('32', 'MÁQUINAS E EQUIPAMENTOS DE NATUREZA GRÁFICA', v_elem_52),
    ('33', 'EQUIPAMENTOS E MATERIAIS SIGILOSOS', v_elem_52),
    ('34', 'EQUIPAMENTOS E MATERIAIS P/ SEGURANÇA NACIONAL', v_elem_52),
    ('35', 'EQUIPAMENTOS E MATERIAIS PARA MERGULHO', v_elem_52),
    ('36', 'EQUIPAMENTOS E MATERIAIS PARA SALVAMENTO', v_elem_52),
    ('37', 'EQUIPAMENTOS E MATERIAIS PARA COMBATE A INCÊNDIO', v_elem_52),
    ('38', 'EQUIPAMENTOS E MATERIAIS PARA PROTEÇÃO CIVIL', v_elem_52),
    ('39', 'EQUIPAMENTOS E MATERIAIS PARA PREVENÇÃO DE DESASTRES', v_elem_52),
    ('40', 'EQUIPAMENTOS E MATERIAIS PARA LABORATÓRIOS', v_elem_52);

END $$;
