CREATE TABLE units_of_measure (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    description TEXT,
    is_base_unit BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_units_name UNIQUE (name),
    CONSTRAINT uq_units_symbol UNIQUE (symbol)
);

CREATE INDEX idx_units_of_measure_name ON units_of_measure(name);
CREATE INDEX idx_units_of_measure_symbol ON units_of_measure(symbol);

CREATE TRIGGER set_timestamp_units_of_measure
BEFORE UPDATE ON units_of_measure
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Dados iniciais: Unidades de medida padrão NFe
-- ============================================================================
INSERT INTO units_of_measure (symbol, name) VALUES
-- Unidades básicas
('UNID', 'UNIDADE'),
('PC', 'PEÇA'),
('KG', 'QUILOGRAMA'),
('GRAMAS', 'GRAMAS'),
('LITRO', 'LITRO'),
('ML', 'MILILITRO'),
('M', 'METRO'),
('M2', 'METRO QUADRADO'),
('M3', 'METRO CÚBICO'),
('CM', 'CENTÍMETRO'),
('CM2', 'CENTÍMETRO QUADRADO'),
('TON', 'TONELADA'),

-- Embalagens
('CX', 'CAIXA'),
('CX2', 'CAIXA COM 2 UNIDADES'),
('CX3', 'CAIXA COM 3 UNIDADES'),
('CX5', 'CAIXA COM 5 UNIDADES'),
('CX10', 'CAIXA COM 10 UNIDADES'),
('CX15', 'CAIXA COM 15 UNIDADES'),
('CX20', 'CAIXA COM 20 UNIDADES'),
('CX25', 'CAIXA COM 25 UNIDADES'),
('CX50', 'CAIXA COM 50 UNIDADES'),
('CX100', 'CAIXA COM 100 UNIDADES'),
('PACOTE', 'PACOTE'),
('FARDO', 'FARDO'),
('EMBAL', 'EMBALAGEM'),
('SACO', 'SACO'),
('SACOLA', 'SACOLA'),

-- Recipientes
('AMPOLA', 'AMPOLA'),
('BALDE', 'BALDE'),
('BANDEJ', 'BANDEJA'),
('BISNAG', 'BISNAGA'),
('BOMB', 'BOMBONA'),
('FRASCO', 'FRASCO'),
('GALAO', 'GALÃO'),
('GF', 'GARRAFA'),
('LATA', 'LATA'),
('POTE', 'POTE'),
('TAMBOR', 'TAMBOR'),
('TANQUE', 'TANQUE'),
('TUBO', 'TUBO'),
('VASIL', 'VASILHAME'),
('VIDRO', 'VIDRO'),

-- Formas
('BARRA', 'BARRA'),
('BLOCO', 'BLOCO'),
('BOBINA', 'BOBINA'),
('CAPS', 'CÁPSULA'),
('CART', 'CARTELA'),
('FOLHA', 'FOLHA'),
('ROLO', 'ROLO'),

-- Quantitativos
('CENTO', 'CENTO'),
('DUZIA', 'DÚZIA'),
('MILHEI', 'MILHEIRO'),
('PARES', 'PARES'),
('RESMA', 'RESMA'),

-- Conjuntos
('CJ', 'CONJUNTO'),
('JOGO', 'JOGO'),
('KIT', 'KIT'),
('DISP', 'DISPLAY'),
('PALETE', 'PALETE'),

-- Outros
('K', 'QUILATE'),
('MWH', 'MEGAWATT HORA');
