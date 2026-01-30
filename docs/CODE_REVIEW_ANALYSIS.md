# Análise Completa de Código - Waterswamp

> **Data:** 2026-01-25
> **Escopo:** Análise completa do código buscando inconsistências, violações DRY, problemas ACID, e oportunidades de melhoria

---

## 📊 Sumário Executivo

| Categoria | Crítico | Alto | Médio | Baixo | Total |
|-----------|---------|------|-------|-------|-------|
| **Repositories (Rust)** | 3 | 8 | 5 | 2 | 18 |
| **Services (Rust)** | 2 | 6 | 8 | 4 | 20 |
| **API Handlers (Rust)** | 2 | 5 | 6 | 3 | 16 |
| **Domain Layer (Rust)** | 1 | 4 | 7 | 3 | 15 |
| **Frontend (Angular)** | 2 | 3 | 6 | 3 | 14 |
| **TOTAL** | **10** | **26** | **32** | **15** | **83** |

---

## 🔴 PROBLEMAS CRÍTICOS (Ação Imediata Necessária)

### 1. Handlers Admin Não Implementados (SEGURANÇA)
**Arquivo:** `apps/api-server/src/api/admin/users/handlers.rs`
**Linhas:** 372-407

```rust
// ban_user() e unban_user() SEMPRE retornam success: true
// sem NENHUMA validação ou persistência
pub async fn ban_user(...) -> Result<Json<UserActionResponse>, AppError> {
    Ok(Json(UserActionResponse {
        user_id,
        action: "ban".to_string(),
        success: true,  // SEMPRE true!
    }))
}
```

**Impacto:** API de segurança completamente não funcional - permite que clientes pensem que usuários foram banidos quando nada acontece.

---

### 2. Transações Faltando em Operações Multi-Step (ACID)
**Arquivos Afetados:**
- `crates/persistence/src/repositories/audit_logs_repository.rs:480-493` - cleanup_old_logs() sem transação
- `crates/persistence/src/repositories/email_verification_repository.rs:74-89` - verify_user_email() não é atômico

**Problema:** Operações que modificam múltiplas tabelas sem garantia de atomicidade podem deixar o banco em estado inconsistente.

---

### 3. Race Condition em MFA
**Arquivo:** `crates/persistence/src/repositories/mfa_repository.rs:151-179`

```rust
// Leitura dos códigos de backup
async fn get_backup_codes(&self, user_id: Uuid) -> Result<Vec<String>, RepositoryError>

// Consumo do código (transação separada!)
async fn verify_and_consume_backup_code(&self, user_id: Uuid, code_hash: &str)
```

**Problema:** Entre leitura e consumo, outra request pode usar o mesmo código.

---

### 4. N+1 Query Pattern em Updates (Performance)
**Arquivos Afetados:** 10+ repositórios fazem SELECT antes de UPDATE

```rust
// Exemplo em catalog_repository.rs:104-127
pub async fn update(&self, id: Uuid, ...) -> Result<...> {
    self.find_by_id(id).await?;  // Query 1 - DESNECESSÁRIA
    sqlx::query!("UPDATE ... WHERE id = $1", id)  // Query 2
}
```

**Impacto:** Dobra o número de queries para cada update, degradando performance significativamente.

---

### 5. Componente Angular Inexistente Referenciado
**Arquivo:** `apps/web-ui/src/app/modules/organizational/components/units-tree/units-tree.component.html:48-54`

O template referencia `<app-tree-node>` que não existe nas declarações do módulo.

**Impacto:** Erro em runtime - componente não renderiza.

---

## 🟠 VIOLAÇÕES DRY (Don't Repeat Yourself)

### Repositories Layer

#### 1. Função `map_err` Duplicada em 8+ Arquivos
**Padrão repetido:**
```rust
fn map_err(e: sqlx::Error) -> RepositoryError {
    if let Some(db_err) = e.as_database_error() {
        if let Some(code) = db_err.code() {
            if code == "23505" { return RepositoryError::Duplicate(...) }
            if code == "23503" { return RepositoryError::ForeignKey(...) }
        }
    }
    RepositoryError::Database(e.to_string())
}
```

**Arquivos:** auth_repository, budget_classifications_repository, catalog_repository, departments_repository, geo_regions_repository, facilities_repository, mfa_repository, user_repository

**Solução:** Criar trait ou função utilitária em módulo comum.

#### 2. Lógica de Paginação Duplicada (~70 queries duplicadas)
Toda operação `list()` executa 2 queries separadas com WHERE idêntico:
```rust
// Query de dados
let items = sqlx::query_as!("SELECT * FROM ... WHERE ... LIMIT $1 OFFSET $2")
// Query de contagem (lógica WHERE duplicada!)
let total = sqlx::query_scalar!("SELECT COUNT(*) FROM ... WHERE ...")
```

**Solução:** Usar `SELECT *, COUNT(*) OVER() as total` ou abstração de paginação.

#### 3. Métodos `exists_by_*_excluding` Repetidos
**Padrão repetido em 8+ repositórios:**
```rust
async fn exists_by_X_excluding(&self, value: &str, exclude_id: Uuid) -> Result<bool, ...> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM table WHERE field = $1 AND id != $2"
    ).fetch_one(&self.pool).await?;
    Ok(count > 0)
}
```

---

### Services Layer

#### 4. Padrão de Validação de Existência Repetido (~40 vezes)
```rust
// Repetido em catalog_service, organizational_service, geo_regions_service
if self.repo.find_by_id(id).await?.is_none() {
    return Err(ServiceError::NotFound("Entidade não encontrada"));
}
```

**Solução:** Criar trait `ValidatableService` ou método helper.

#### 5. Verificação de Unicidade Repetida
```rust
if self.repo.exists_by_X(...).await? {
    return Err(ServiceError::Conflict("Já existe"));
}
```
Repetido em: catalog_service (8x), organizational_service (5x), geo_regions_service (6x)

---

### API Handlers

#### 6. Validação Manual Repetida (28+ ocorrências)
```rust
// Repetido em TODOS os handlers
if let Err(e) = payload.validate() {
    return Err(AppError::Validation(e));
}
```

**Solução:** Criar extractor Axum customizado que valida automaticamente.

---

### Domain Layer

#### 7. Value Objects com Implementação Idêntica (7 tipos)
**Arquivo:** `crates/domain/src/value_objects.rs:26-334`

Email, Username, StateCode, LocationName, MaterialCode, CatmatCode, UnitOfMeasure - todos implementam:
- `Display`
- `TryFrom<String>`
- `TryFrom<&str>`
- `AsRef<str>`

**Solução:** Macro procedural ou wrapper genérico.

#### 8. Structs Paginated* Duplicadas (11 tipos)
Cada modelo tem sua própria struct de paginação idêntica:
```rust
pub struct PaginatedX {
    pub items: Vec<XDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
```

**Solução:** `Paginated<T>` genérico.

---

### Frontend Angular

#### 9. Função loadOrganizations() Duplicada (3 componentes)
```typescript
// Idêntico em units-list, siorg-sync, units-tree
loadOrganizations(): void {
  this.organizationalService.listOrganizations({ is_active: true })
    .subscribe({
      next: (response) => { this.organizations = response.organizations; },
      error: (err) => { console.error('Error loading organizations:', err); }
    });
}
```

#### 10. Mapeamento de Labels Duplicado (3 componentes)
`getEntityTypeLabel()`, `getOperationLabel()`, `getChangeTypeLabel()` repetidos em:
- conflicts-list.component.ts
- conflict-resolver.component.ts
- stats-dashboard.component.ts

---

## 🟡 INCONSISTÊNCIAS

### Repositories

| Aspecto | Padrão A | Padrão B | Arquivos |
|---------|----------|----------|----------|
| Pool ownership | `pool: &'a PgPool` | `pool: PgPool` | audit_logs vs outros |
| Error mapping | Basic `Database(e)` | Com duplicate detection | auth vs budget |
| Update approach | Fetch-then-update | Dynamic query | catalog vs budget |

### Services

| Aspecto | Padrão A | Padrão B | Arquivos |
|---------|----------|----------|----------|
| Delete return | `Result<bool>` | `Result<()>` | catalog vs organizational |
| Field visibility | `pub repository` | private | organizational vs catalog |
| Error language | Português | English | catalog vs budget |

### API Handlers

| Aspecto | Padrão A | Padrão B | Arquivos |
|---------|----------|----------|----------|
| POST status | 201 CREATED | 200 OK | auth vs admin/users |
| Error format | `AppError` | `(StatusCode, String)` | auth vs catalog |
| API version | `/api/v1/` | `/api/` (sem versão) | auth vs catalog |

### Domain

| Aspecto | Padrão A | Padrão B | Arquivos |
|---------|----------|----------|----------|
| Port error type | `RepositoryError` | `String` | todos vs email.rs |
| Update payload | `Option<T>` | `Option<Option<T>>` | budget vs catalog |
| Tree structure | Campos duplicados | `#[serde(flatten)]` | budget vs organizational |

---

## 🔵 PROBLEMAS DE ARQUITETURA

### 1. Lógica de Negócio no Service Layer (Deveria estar no Domain)

**Exemplo:** `crates/application/src/services/catalog_service.rs:147-152`
```rust
// Regra de negócio: grupo não pode ter subgrupos se tiver itens
if self.group_repo.has_items(parent_id).await? {
    return Err(ServiceError::BadRequest("Grupo pai já possui itens"));
}
```

Esta é uma regra de domínio que deveria estar na entidade `CatalogGroup`.

### 2. Acoplamento Excessivo de Repositórios

**OrganizationalUnitService** depende de 5 repositórios:
- `unit_repository`
- `org_repository`
- `category_repository`
- `type_repository`
- `settings_repository`

Dificulta testes e manutenção.

### 3. Configuração como Dados Runtime

```rust
// organizational_service.rs:531-535
let allow_custom: bool = if let Some(setting) =
    self.settings_repository.get("units.allow_custom_units").await? {
    serde_json::from_value(setting.value).unwrap_or(true)
} else { true };
```

**Problemas:**
- String literal para key
- Desserialização silenciosa com default
- Sem validação na startup

### 4. Modelos Anêmicos

Todos os DTOs são containers de dados sem comportamento:
```rust
pub struct UserDto {
    pub id: Uuid,
    pub username: Username,
    pub email: Email,
    // ... apenas campos, nenhum método
}
```

---

## 🟣 PROBLEMAS DE SEGURANÇA

### 1. Input Validation Faltando em Policies
**Arquivo:** `apps/api-server/src/api/admin/policies/handlers.rs:23-58`

```rust
// obj e act são passados diretamente ao Casbin sem validação
let policy_exists = enforcer.has_policy(vec![
    payload.sub.clone(),  // Apenas este é validado
    payload.obj.clone(),  // NÃO validado!
    payload.act.clone(),  // NÃO validado!
]);
```

### 2. Métodos HTTP Incorretos para Operações de Estado
**Arquivo:** `apps/api-server/src/api/organizational/mod.rs:75-76`

```rust
// ERRADO: GET não deveria modificar estado
.route("/{id}/deactivate", get(handlers::deactivate_organizational_unit))
.route("/{id}/activate", get(handlers::activate_organizational_unit))
```

**Correção:** Usar POST ou PATCH.

### 3. Rate Limiting Hardcoded
```rust
// email_verification/handlers.rs:21-22
const MAX_VERIFICATION_REQUESTS_PER_HOUR: i64 = 3;  // Deveria ser configurável
```

### 4. Senhas Sem Validação de Complexidade
Nenhum service valida:
- Comprimento mínimo de senha
- Caracteres especiais obrigatórios
- Histórico de senhas

---

## 🟢 PROBLEMAS ANGULAR (Frontend)

### 1. Memory Leaks - Subscriptions Não Canceladas
**5 componentes** subscrevem observables sem implementar `OnDestroy`:
- units-list.component.ts (4 subscriptions)
- siorg-sync.component.ts (5 subscriptions)
- conflict-resolver.component.ts (2 subscriptions)
- units-tree.component.ts (2 subscriptions)
- conflicts-list.component.ts (3 subscriptions)

### 2. Change Detection Strategy Faltando
**Todos os 7 componentes** usam default change detection ao invés de `OnPush`, impactando performance.

### 3. Uso de `any` Type (12 instâncias)
```typescript
// sync.models.ts
payload: any;
detected_changes?: any;
local_value?: any;
siorg_value?: any;
```

### 4. APIs do Browser ao Invés de Angular (27 instâncias)
```typescript
// Deveria usar MatDialog/NgbModal
alert('Mensagem');
confirm('Confirma?');
```

### 5. `.toPromise()` Depreciado
```typescript
// stats-dashboard.component.ts:39-40
this.syncService.getDetailedStats().toPromise()  // DEPRECIADO
// Usar: firstValueFrom() ou forkJoin()
```

---

## 📋 RECOMENDAÇÕES PRIORITÁRIAS

### 🔴 Prioridade Crítica (Fazer Agora)

1. **Implementar ban_user/unban_user** corretamente
2. **Adicionar transações** em email_verification e audit_logs
3. **Corrigir race condition** no MFA backup codes
4. **Criar componente** app-tree-node no Angular

### 🟠 Prioridade Alta (Esta Sprint)

5. **Criar abstrações DRY:**
   - `map_db_error()` função comum
   - `Paginated<T>` genérico
   - `validate_exists()` helper

6. **Unificar padrões:**
   - Todos os deletes retornam `Result<()>`
   - Todos os creates retornam status 201
   - Todas mensagens em um idioma (preferencialmente inglês)

7. **Corrigir HTTP methods** para activate/deactivate

8. **Implementar OnDestroy** em todos componentes Angular

### 🟡 Prioridade Média (Próximas Sprints)

9. **Otimizar queries:**
   - Remover N+1 em updates
   - Usar window functions para paginação

10. **Melhorar Domain:**
    - Mover regras de negócio para entidades
    - Criar value objects faltantes (CNPJ, PostalCode)

11. **Padronizar Angular:**
    - Usar OnPush em todos componentes
    - Substituir browser alerts por modais
    - Eliminar uso de `any`

### 🟢 Prioridade Baixa (Backlog)

12. Adicionar HATEOAS links nas respostas API
13. Implementar lazy loading no Angular
14. Criar service de logging ao invés de console.error
15. Documentar estratégia de validação

---

## 📈 ESTIMATIVA DE DÉBITO TÉCNICO

| Categoria | Linhas de Código Afetadas | Esforço Estimado |
|-----------|---------------------------|------------------|
| Abstrações DRY | ~2,500 linhas duplicadas | 3-4 dias |
| Transações ACID | ~200 linhas | 1 dia |
| Segurança | ~150 linhas | 1 dia |
| Consistência API | ~500 linhas | 2 dias |
| Otimização queries | ~400 linhas | 2 dias |
| Angular best practices | ~800 linhas | 2-3 dias |
| **TOTAL** | **~4,550 linhas** | **11-15 dias** |

---

## 🎯 CONCLUSÃO

O codebase do Waterswamp demonstra boa arquitetura geral (Clean Architecture, DDD), mas acumulou débito técnico significativo em:

1. **Repetição de código** - ~60% poderia ser reduzido com abstrações apropriadas
2. **Inconsistências** - Diferentes padrões para o mesmo problema entre módulos
3. **Gaps de segurança** - Handlers stub e validações faltando
4. **Performance** - N+1 queries e lack of OnPush no Angular

**Recomendação:** Dedicar 2 sprints focadas em refatoração antes de adicionar novas features, priorizando os itens críticos de segurança e ACID.
