-- Add up migration script here

-- 1. Atualizar valores NULL existentes para string vazia
UPDATE casbin_rule SET v0 = '' WHERE v0 IS NULL;
UPDATE casbin_rule SET v1 = '' WHERE v1 IS NULL;
UPDATE casbin_rule SET v2 = '' WHERE v2 IS NULL;
UPDATE casbin_rule SET v3 = '' WHERE v3 IS NULL;
UPDATE casbin_rule SET v4 = '' WHERE v4 IS NULL;
UPDATE casbin_rule SET v5 = '' WHERE v5 IS NULL;

-- 2. Adicionar NOT NULL constraint
ALTER TABLE casbin_rule ALTER COLUMN v0 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v1 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v2 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v3 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v4 SET NOT NULL;
ALTER TABLE casbin_rule ALTER COLUMN v5 SET NOT NULL;

-- 3. Definir default como string vazia para novos registros
ALTER TABLE casbin_rule ALTER COLUMN v0 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v1 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v2 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v3 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v4 SET DEFAULT '';
ALTER TABLE casbin_rule ALTER COLUMN v5 SET DEFAULT '';
