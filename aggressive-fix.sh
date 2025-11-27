#!/bin/bash

# FIX AGRESSIVO - Força correção completa
set -e

echo "🔨 FIX AGRESSIVO - Forçando correção do casbin_rule..."
echo ""

if [ -z "$DATABASE_URL" ]; then
    if [ -f ".env" ]; then
        export $(grep -v '^#' .env | xargs)
    fi
fi

if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL não configurado"
    exit 1
fi

# 1. VERIFICAR E CORRIGIR SCHEMA
echo "1. Verificando schema..."
NULLABLE=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'casbin_rule' AND column_name IN ('v0', 'v1', 'v2', 'v3', 'v4', 'v5') AND is_nullable = 'YES'")

if [ "$NULLABLE" -gt 0 ]; then
    echo "   ⚠️  $NULLABLE colunas nullable encontradas. Corrigindo..."
    
    psql "$DATABASE_URL" -q << 'EOF'
UPDATE casbin_rule SET v0 = COALESCE(v0, ''), v1 = COALESCE(v1, ''), v2 = COALESCE(v2, ''), v3 = COALESCE(v3, ''), v4 = COALESCE(v4, ''), v5 = COALESCE(v5, '');
ALTER TABLE casbin_rule ALTER COLUMN v0 SET NOT NULL, ALTER COLUMN v0 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v1 SET NOT NULL, ALTER COLUMN v1 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v2 SET NOT NULL, ALTER COLUMN v2 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v3 SET NOT NULL, ALTER COLUMN v3 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v4 SET NOT NULL, ALTER COLUMN v4 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v5 SET NOT NULL, ALTER COLUMN v5 SET DEFAULT '';
EOF
    echo "   ✅ Schema corrigido"
else
    echo "   ✅ Schema já está correto"
fi

# 2. LIMPAR TUDO
echo ""
echo "2. Limpando caches..."
rm -rf .sqlx/ target/ 2>/dev/null || true
echo "   ✅ Caches removidos"

# 3. VERIFICAR SCHEMA FINAL
echo ""
echo "3. Schema final da tabela casbin_rule:"
psql "$DATABASE_URL" -c "\d casbin_rule" | grep -E "(v0|v1|v2|v3|v4|v5)"

# 4. REGENERAR CACHE
echo ""
echo "4. Regenerando SQLx cache..."
if cargo sqlx prepare --workspace -- --all-targets; then
    echo "   ✅ Cache regenerado"
else
    echo "   ❌ Falha ao regenerar cache"
    exit 1
fi

# 5. COMPILAR
echo ""
echo "5. Compilando..."
if cargo build --tests 2>&1 | tail -20; then
    echo ""
    echo "✅✅✅ SUCESSO! ✅✅✅"
else
    echo ""
    echo "❌ Falha na compilação"
    echo ""
    echo "Tente manualmente:"
    echo "  rm -rf .sqlx target"
    echo "  cargo sqlx prepare --workspace -- --all-targets"
    echo "  cargo build --tests"
    exit 1
fi
