#!/bin/bash

# Script para verificar schema e regenerar SQLx cache completamente
set -e

echo "🔍 Diagnóstico completo do problema casbin_rule..."
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Verificar DATABASE_URL
if [ -z "$DATABASE_URL" ]; then
    if [ -f ".env" ]; then
        export $(grep -v '^#' .env | xargs)
    fi
fi

if [ -z "$DATABASE_URL" ]; then
    echo -e "${RED}✗${NC} DATABASE_URL não configurado"
    exit 1
fi

echo "DATABASE_URL: $DATABASE_URL"
echo ""

# ============================================================================
# 1. VERIFICAR SCHEMA ATUAL
# ============================================================================
echo -e "${BLUE}1️⃣  Verificando schema da tabela casbin_rule...${NC}"
echo ""

echo "Schema atual:"
psql "$DATABASE_URL" -c "\d casbin_rule"
echo ""

# Contar colunas nullable
NULLABLE_COUNT=$(psql "$DATABASE_URL" -t -c "
    SELECT COUNT(*) 
    FROM information_schema.columns 
    WHERE table_name = 'casbin_rule' 
    AND column_name IN ('v0', 'v1', 'v2', 'v3', 'v4', 'v5')
    AND is_nullable = 'YES'
")

if [ "$NULLABLE_COUNT" -gt 0 ]; then
    echo -e "${RED}✗${NC} Problema encontrado: $NULLABLE_COUNT colunas ainda são NULLABLE"
    echo ""
    echo "Colunas nullable:"
    psql "$DATABASE_URL" -c "
        SELECT column_name, is_nullable, column_default
        FROM information_schema.columns 
        WHERE table_name = 'casbin_rule' 
        AND column_name IN ('v0', 'v1', 'v2', 'v3', 'v4', 'v5')
        AND is_nullable = 'YES'
    "
    echo ""
    
    read -p "Deseja corrigir agora? (S/n): " RESPONSE
    if [[ ! "$RESPONSE" =~ ^[Nn]$ ]]; then
        echo ""
        echo "Corrigindo schema..."
        
        psql "$DATABASE_URL" << 'EOF'
-- Converter NULL para string vazia
UPDATE casbin_rule SET v0 = '' WHERE v0 IS NULL;
UPDATE casbin_rule SET v1 = '' WHERE v1 IS NULL;
UPDATE casbin_rule SET v2 = '' WHERE v2 IS NULL;
UPDATE casbin_rule SET v3 = '' WHERE v3 IS NULL;
UPDATE casbin_rule SET v4 = '' WHERE v4 IS NULL;
UPDATE casbin_rule SET v5 = '' WHERE v5 IS NULL;

-- Forçar NOT NULL
ALTER TABLE casbin_rule ALTER COLUMN v0 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v1 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v2 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v3 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v4 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v5 SET NOT NULL;

-- Defaults
ALTER TABLE casbin_rule ALTER COLUMN v0 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v1 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v2 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v3 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v4 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v5 SET DEFAULT '';
EOF
        
        echo -e "${GREEN}✓${NC} Schema corrigido!"
        echo ""
    fi
else
    echo -e "${GREEN}✓${NC} Schema está correto: todas as colunas são NOT NULL"
    echo ""
fi

# ============================================================================
# 2. LIMPAR CACHE DO SQLx
# ============================================================================
echo -e "${BLUE}2️⃣  Limpando cache do SQLx...${NC}"

if [ -d ".sqlx" ]; then
    echo "Removendo diretório .sqlx/..."
    rm -rf .sqlx/
    echo -e "${GREEN}✓${NC} Cache removido"
else
    echo "Nenhum cache encontrado"
fi

if [ -d "target" ]; then
    echo "Limpando build artifacts..."
    cargo clean
    echo -e "${GREEN}✓${NC} Build limpo"
fi

echo ""

# ============================================================================
# 3. REGENERAR SQLx OFFLINE DATA
# ============================================================================
echo -e "${BLUE}3️⃣  Regenerando SQLx offline data...${NC}"
echo ""

echo "Executando: cargo sqlx prepare --workspace -- --all-targets"
echo ""

if cargo sqlx prepare --workspace -- --all-targets 2>&1 | tee /tmp/sqlx-prepare.log; then
    echo ""
    echo -e "${GREEN}✓${NC} SQLx offline data regenerado com sucesso!"
    echo ""
    
    echo "Arquivos gerados:"
    ls -lh .sqlx/ 2>/dev/null || echo "Nenhum arquivo .sqlx gerado"
else
    echo ""
    echo -e "${RED}✗${NC} Falha ao regenerar offline data"
    echo "Veja log completo em: /tmp/sqlx-prepare.log"
    echo ""
    exit 1
fi

echo ""

# ============================================================================
# 4. VERIFICAR CACHE GERADO
# ============================================================================
echo -e "${BLUE}4️⃣  Verificando cache gerado...${NC}"

if [ -d ".sqlx" ]; then
    echo "Arquivos no cache:"
    find .sqlx -type f -name "*.json" | head -10
    echo ""
    
    # Verificar se tem queries do casbin_rule
    if grep -r "casbin_rule" .sqlx/ 2>/dev/null | head -5; then
        echo ""
        echo -e "${GREEN}✓${NC} Cache contém queries do casbin_rule"
    else
        echo -e "${YELLOW}⚠${NC} Cache não parece conter queries do casbin_rule"
    fi
else
    echo -e "${YELLOW}⚠${NC} Diretório .sqlx não foi criado"
fi

echo ""

# ============================================================================
# 5. TESTAR COMPILAÇÃO
# ============================================================================
echo -e "${BLUE}5️⃣  Testando compilação...${NC}"
echo ""

echo "Executando: cargo build --tests"
echo ""

if cargo build --tests 2>&1 | tee /tmp/build-test.log; then
    echo ""
    echo -e "${GREEN}✅✅✅ SUCESSO! Compilação funcionou! ✅✅✅${NC}"
    echo ""
else
    echo ""
    echo -e "${RED}✗✗✗ FALHA NA COMPILAÇÃO ✗✗✗${NC}"
    echo ""
    echo "Últimas 30 linhas do erro:"
    tail -30 /tmp/build-test.log
    echo ""
    echo "Log completo em: /tmp/build-test.log"
    echo ""
    
    # Verificar se ainda é o mesmo erro
    if grep -q "trait bound.*String.*From.*Option.*String" /tmp/build-test.log; then
        echo -e "${YELLOW}⚠${NC} Ainda é o mesmo erro de Option<String>"
        echo ""
        echo "Possíveis causas:"
        echo "  1. SQLx cache não foi regenerado corretamente"
        echo "  2. Schema do database ainda tem colunas nullable"
        echo "  3. Versão incompatível do sqlx-adapter"
        echo ""
        echo "Tentativas adicionais:"
        echo "  - Verificar schema: psql \$DATABASE_URL -c '\\d casbin_rule'"
        echo "  - Remover Cargo.lock: rm Cargo.lock && cargo build"
        echo "  - Atualizar sqlx-adapter no Cargo.toml"
    fi
    
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Diagnóstico e correção completos!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Próximos passos:"
echo "  1. Executar testes: cargo test"
echo "  2. Commit do cache: git add .sqlx && git commit -m 'Update SQLx cache'"
echo ""
