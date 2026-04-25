-- RF-004: coluna para indicar se a unidade de medida admite quantidades fracionadas.
-- Unidades contínuas (KG, L, M, etc.) permitem frações (padrão).
-- Unidades discretas (UN, CX, PC, etc.) não permitem — o serviço rejeita qty com parte decimal.

ALTER TABLE units_of_measure
    ADD COLUMN allows_fractions BOOLEAN NOT NULL DEFAULT TRUE;

-- Marca unidades tipicamente indivisíveis como discretas.
-- Lista conservadora; o gestor pode ajustar via UPDATE direto conforme necessidade.
UPDATE units_of_measure
SET allows_fractions = FALSE
WHERE UPPER(abbreviation) IN (
    'UN',  -- unidade
    'CX',  -- caixa
    'PC',  -- peça
    'RL',  -- rolo
    'PR',  -- par
    'DZ',  -- dúzia
    'RS',  -- resma
    'AM',  -- ampola
    'FL',  -- frasco/lata
    'BD',  -- balde
    'CT'   -- cartucho
);
