# Arquitetura de Autorização Multi-Módulo (v4)

## Sistema de Gestão Universitária — UFMT
### Waterswamp (Casbin-RS) + Angular (VAI)

---

## 1. Visão Geral do Problema

Você tem múltiplos sistemas (SIGALM, SIGFROTA, SIGEP, futuros) que compartilham:

- **Mesma base de usuários** (servidores da UFMT)
- **Mesma estrutura organizacional** (SIORG → sites → departments)
- **Mesma infraestrutura de autenticação** (Waterswamp)

Mas cada módulo tem **regras de negócio diferentes**, e um servidor pode ter **funções completamente distintas** em cada módulo. Um técnico de TI pode ser "Operador" no SIGFROTA e "Auditor" no SIGALM ao mesmo tempo.

### Princípios Fundamentais

1. **Autorização em 3 camadas**: Casbin (pode acessar?) → Filtro de dados (quais registros?) → Regra de negócio (faz sentido?)
2. **Perfis por nível de acesso**, não por funcionalidade
3. **Segregação de funções**: quem requisita não aprova, quem cadastra não audita
4. **Deny vence allow**: bloqueios individuais sempre prevalecem
5. **Sync incremental**: mudanças pontuais, não recarga total
6. **Invalidação proativa**: permissões nunca ficam obsoletas por mais de minutos
7. **Imutabilidade de registros**: soft deletes + event log, nunca DELETE físico em atribuições
8. **Simplicidade primeiro**: começar simples, extrair abstrações quando houver evidência de necessidade
9. **Alertas de segurança**: tentativas repetidas de acesso negado disparam alertas

---

## 2. Modelo Casbin: RBAC com Domínios

### 2.1. Por que RBAC com Domínios?

O modelo RBAC simples (`sub, obj, act`) que o Waterswamp usa hoje **não é suficiente**. Você precisa de **RBAC com domínio** para representar:

- **Módulo** (SIGALM, SIGFROTA, SIGEP...)
- **Escopo** (campus, almoxarifado específico, unidade...)
- **Recurso** (requisição, veículo, estoque...)
- **Ação** (criar, aprovar, visualizar...)

### 2.2. Novo `rbac_model.conf`

```ini
[request_definition]
r = sub, dom, obj, act

[policy_definition]
p = sub, dom, obj, act, eft

[role_definition]
g = _, _, _

[policy_effect]
e = some(where (p.eft == allow)) && !some(where (p.eft == deny))

[matchers]
m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && keyMatch2(r.obj, p.obj) && regexMatch(r.act, p.act)
```

**Explicação:**

| Elemento | Significado |
|----------|-------------|
| `r = sub, dom, obj, act` | Request: quem, em qual domínio, em qual recurso, qual ação |
| `p = sub, dom, obj, act, eft` | Policy: papel, domínio, recurso, ação, efeito (allow/deny) |
| `g = _, _, _` | Grouping com 3 campos: usuário, papel, domínio |
| `eft` com deny | Permite **bloquear** usuários específicos (deny sempre vence) |
| `keyMatch2` | Suporta wildcards como `/almoxarifado/:id/estoque` |

### 2.3. Conceito de "Domínio"

O domínio é composto por:

```
{modulo}:{escopo_tipo}:{escopo_id}
```

**Exemplos:**

| Domínio | Significado |
|---------|-------------|
| `sigalm:almoxarifado:uuid-almox-central-cuiaba` | Almoxarifado Central de Cuiabá |
| `sigalm:almoxarifado:uuid-almox-setorial-faet` | Almoxarifado Setorial da FAET |
| `sigalm:campus:uuid-campus-cuiaba` | SIGALM no campus Cuiabá (todas unidades) |
| `sigalm:global` | SIGALM em toda a universidade |
| `sigfrota:campus:uuid-campus-cuiaba` | Transporte no campus Cuiabá |
| `sigfrota:global` | Transporte global |
| `sigep:campus:uuid-campus-sinop` | Gestão predial campus Sinop |

---

## 3. Casbin: Design para Evolução

### 3.1. Enforcer Único por Trás de Trait

Com os volumes da UFMT (500-2.000 servidores, 3 módulos iniciais), um enforcer único com `RwLock` funciona perfeitamente. O lock de escrita durante sync incremental dura microsegundos. Multi-enforcer por módulo só se justifica com 5+ módulos ou 10.000+ policies.

A chave é esconder o enforcer atrás de um trait. Quando precisar migrar para multi-enforcer, troque a implementação sem mudar nenhum consumidor:

```rust
use async_trait::async_trait;
use uuid::Uuid;

/// Trait de autorização — todos os consumidores dependem disso,
/// não da implementação concreta do Casbin.
#[async_trait]
pub trait AuthorizationService: Send + Sync {
    async fn check_permission(
        &self,
        user_id: Uuid,
        module: &str,
        scope_id: Option<Uuid>,
        scope_type: Option<&str>,
        resource: &str,
        action: &str,
    ) -> bool;

    async fn add_assignment_policies(&self, assignment_id: Uuid) -> anyhow::Result<()>;
    async fn remove_assignment_policies(
        &self, user_id: Uuid, profile_code: &str, domain: &str,
    ) -> anyhow::Result<()>;
    async fn add_block_policy(&self, block: &UserBlock) -> anyhow::Result<()>;
    async fn full_sync(&self) -> anyhow::Result<()>;
}
```

**Fase 1 — Enforcer único:**

```rust
use casbin::{CoreApi, MgmtApi, Enforcer};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SingleEnforcerService {
    enforcer: Arc<RwLock<Enforcer>>,
    pool: PgPool,
}

#[async_trait]
impl AuthorizationService for SingleEnforcerService {
    async fn check_permission(
        &self,
        user_id: Uuid,
        module: &str,
        scope_id: Option<Uuid>,
        scope_type: Option<&str>,
        resource: &str,
        action: &str,
    ) -> bool {
        let enforcer = self.enforcer.read().await;
        let domain = build_domain(module, None, scope_id, scope_type);

        enforcer.enforce((
            user_id.to_string(),
            domain,
            resource.to_string(),
            action.to_string(),
        )).unwrap_or(false)
    }

    async fn add_assignment_policies(&self, assignment_id: Uuid) -> anyhow::Result<()> {
        let policies = sqlx::query_as!(
            AssignmentPolicy,
            r#"
            SELECT 
                upa.user_id, mp.code as profile_code, m.code as module_code,
                pp.resource, pp.action,
                upa.scope_site_id, upa.scope_entity_id, upa.scope_entity_type
            FROM user_profile_assignments upa
            JOIN module_profiles mp ON upa.profile_id = mp.id
            JOIN modules m ON mp.module_id = m.id
            JOIN profile_permissions pp ON pp.profile_id = mp.id
            WHERE upa.id = $1
            "#,
            assignment_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut enforcer = self.enforcer.write().await;

        for p in &policies {
            let domain = build_domain(
                &p.module_code, p.scope_site_id,
                p.scope_entity_id, p.scope_entity_type.as_deref(),
            );

            let _ = enforcer.add_named_grouping_policy(
                "g",
                vec![p.user_id.to_string(), p.profile_code.clone(), domain.clone()],
            ).await;

            let _ = enforcer.add_named_policy(
                "p",
                vec![
                    p.profile_code.clone(), domain,
                    p.resource.clone(), p.action.clone(), "allow".to_string(),
                ],
            ).await;
        }

        // Bump version para invalidação no frontend
        self.bump_user_permissions_version(policies[0].user_id).await?;

        Ok(())
    }

    async fn remove_assignment_policies(
        &self, user_id: Uuid, profile_code: &str, domain: &str,
    ) -> anyhow::Result<()> {
        let mut enforcer = self.enforcer.write().await;

        let _ = enforcer.remove_named_grouping_policy(
            "g",
            vec![user_id.to_string(), profile_code.to_string(), domain.to_string()],
        ).await;

        enforcer.remove_filtered_named_policy(
            "p", 0,
            vec![profile_code.to_string(), domain.to_string()],
        ).await?;

        self.bump_user_permissions_version(user_id).await?;

        Ok(())
    }

    async fn add_block_policy(&self, block: &UserBlock) -> anyhow::Result<()> {
        let domain = match block.scope_site_id {
            Some(site_id) => format!("{}:campus:{}", block.module_code, site_id),
            None => format!("{}:global", block.module_code),
        };

        let mut enforcer = self.enforcer.write().await;
        enforcer.add_named_policy(
            "p",
            vec![
                block.user_id.to_string(), domain,
                block.resource.clone(), block.action.clone(), "deny".to_string(),
            ],
        ).await?;

        self.bump_user_permissions_version(block.user_id).await?;

        Ok(())
    }

    async fn full_sync(&self) -> anyhow::Result<()> {
        let mut enforcer = self.enforcer.write().await;
        enforcer.clear_policy().await;

        // 1. Atribuições explícitas
        let assignments = sqlx::query_as!(
            AssignmentPolicy,
            r#"
            SELECT 
                upa.user_id, mp.code as profile_code, m.code as module_code,
                pp.resource, pp.action,
                upa.scope_site_id, upa.scope_entity_id, upa.scope_entity_type
            FROM user_profile_assignments upa
            JOIN module_profiles mp ON upa.profile_id = mp.id
            JOIN modules m ON mp.module_id = m.id
            JOIN profile_permissions pp ON pp.profile_id = mp.id
            WHERE upa.is_active = TRUE
              AND (upa.expires_at IS NULL OR upa.expires_at > NOW())
              AND mp.is_active = TRUE AND m.is_active = TRUE
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for a in &assignments {
            let domain = build_domain(
                &a.module_code, a.scope_site_id,
                a.scope_entity_id, a.scope_entity_type.as_deref(),
            );

            let _ = enforcer.add_named_grouping_policy(
                "g",
                vec![a.user_id.to_string(), a.profile_code.clone(), domain.clone()],
            ).await;

            let _ = enforcer.add_named_policy(
                "p",
                vec![
                    a.profile_code.clone(), domain,
                    a.resource.clone(), a.action.clone(), "allow".to_string(),
                ],
            ).await;
        }

        // 2. Perfis implícitos (requisitor_pessoal, etc.)
        let implicit = self.generate_implicit_policies().await?;
        for p in &implicit {
            let _ = enforcer.add_named_grouping_policy(
                "g",
                vec![p.user_id.to_string(), p.profile_code.clone(), p.domain.clone()],
            ).await;

            let _ = enforcer.add_named_policy(
                "p",
                vec![
                    p.profile_code.clone(), p.domain.clone(),
                    p.resource.clone(), p.action.clone(), "allow".to_string(),
                ],
            ).await;
        }

        // 3. Bloqueios (deny)
        let blocks = sqlx::query_as!(
            UserBlock,
            r#"
            SELECT user_id, module_code, resource, action, scope_site_id
            FROM user_permission_blocks
            WHERE is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for b in &blocks {
            let domain = match b.scope_site_id {
                Some(site_id) => format!("{}:campus:{}", b.module_code, site_id),
                None => format!("{}:global", b.module_code),
            };

            let _ = enforcer.add_named_policy(
                "p",
                vec![
                    b.user_id.to_string(), domain,
                    b.resource.clone(), b.action.clone(), "deny".to_string(),
                ],
            ).await;
        }

        tracing::info!(
            "Full sync: {} atribuições, {} implícitas, {} bloqueios",
            assignments.len(), implicit.len(), blocks.len()
        );

        Ok(())
    }
}

impl SingleEnforcerService {
    async fn bump_user_permissions_version(&self, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE auth_users 
            SET permissions_version = permissions_version + 1,
                permissions_changed_at = NOW()
            WHERE id = $1
            "#,
            user_id,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn generate_implicit_policies(&self) -> anyhow::Result<Vec<ImplicitPolicy>> {
        // Buscar perfis implícitos e gerar policies para todos os usuários ativos
        let result = sqlx::query_as!(
            ImplicitPolicy,
            r#"
            SELECT 
                u.id as user_id,
                mp.code as profile_code,
                m.code as module_code,
                pp.resource,
                pp.action,
                ds.site_id
            FROM auth_users u
            JOIN departments d ON u.department_id = d.id
            JOIN department_sites ds ON d.id = ds.department_id
            CROSS JOIN module_profiles mp
            JOIN modules m ON mp.module_id = m.id
            JOIN profile_permissions pp ON pp.profile_id = mp.id
            WHERE u.is_active = TRUE
              AND mp.is_implicit = TRUE
              AND mp.is_active = TRUE
              AND ds.site_id IS NOT NULL
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(result.into_iter().map(|r| ImplicitPolicy {
            user_id: r.user_id,
            profile_code: r.profile_code,
            domain: format!("{}:campus:{}", r.module_code, r.site_id),
            resource: r.resource,
            action: r.action,
        }).collect())
    }
}
```

**Futuro — Multi-enforcer (quando necessário):**

```rust
/// Quando tiver evidência de contention (5+ módulos, 10k+ policies),
/// implemente esta versão sem mudar nenhum consumidor.
pub struct MultiEnforcerService {
    enforcers: HashMap<String, Arc<RwLock<Enforcer>>>,
    pool: PgPool,
}

#[async_trait]
impl AuthorizationService for MultiEnforcerService {
    async fn check_permission(
        &self, user_id: Uuid, module: &str, /* ... */
    ) -> bool {
        // Roteia para o enforcer do módulo específico
        let enforcer = match self.enforcers.get(module) {
            Some(e) => e,
            None => return false,
        };
        let enf = enforcer.read().await;
        // ... mesmo enforce()
    }
    // ...
}
```

A troca é transparente:

```rust
// AppState usa o trait, não a implementação
pub struct AppState {
    pub auth_service: Arc<dyn AuthorizationService>,
    // ...
}

// No startup, escolha a implementação
let auth_service: Arc<dyn AuthorizationService> = Arc::new(
    SingleEnforcerService::new(pool.clone()).await?
);
```

### 3.2. Permissões em Batch (Sem Cache de Request)

Em vez de chamar `enforce()` N vezes em loop (e precisar de cache), carregue as permissões do usuário de uma vez e verifique em memória:

```rust
/// Carrega todas as permissões efetivas de um usuário para um módulo.
/// Útil em handlers que precisam verificar permissão por item (ex: can_approve por requisição).
pub async fn get_user_effective_permissions(
    pool: &PgPool,
    user_id: Uuid,
    module_code: &str,
) -> Result<UserEffectivePermissions, AppError> {
    // 1. Permissões allow
    let allows = sqlx::query_as!(
        PermissionEntry,
        r#"
        SELECT 
            pp.resource,
            pp.action,
            upa.scope_site_id,
            upa.scope_entity_id,
            upa.scope_entity_type,
            mp.scope_type
        FROM user_profile_assignments upa
        JOIN module_profiles mp ON upa.profile_id = mp.id
        JOIN modules m ON mp.module_id = m.id
        JOIN profile_permissions pp ON pp.profile_id = mp.id
        WHERE upa.user_id = $1
          AND m.code = $2
          AND upa.is_active = TRUE
          AND (upa.expires_at IS NULL OR upa.expires_at > NOW())
        "#,
        user_id,
        module_code,
    )
    .fetch_all(pool)
    .await?;

    // 2. Bloqueios deny
    let denies = sqlx::query_as!(
        BlockEntry,
        r#"
        SELECT resource, action, scope_site_id
        FROM user_permission_blocks
        WHERE user_id = $1 AND module_code = $2 AND is_active = TRUE
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        user_id,
        module_code,
    )
    .fetch_all(pool)
    .await?;

    Ok(UserEffectivePermissions { allows, denies })
}

pub struct UserEffectivePermissions {
    allows: Vec<PermissionEntry>,
    denies: Vec<BlockEntry>,
}

impl UserEffectivePermissions {
    /// Verificação em memória — O(n) mas n é pequeno (< 50 permissões por usuário)
    pub fn can(
        &self,
        resource: &str,
        action: &str,
        scope_entity_id: Option<Uuid>,
        scope_site_id: Option<Uuid>,
    ) -> bool {
        // Deny vence
        let is_blocked = self.denies.iter().any(|d|
            d.resource == resource
            && d.action == action
            && (d.scope_site_id.is_none() || d.scope_site_id == scope_site_id)
        );
        if is_blocked { return false; }

        // Verificar allow
        self.allows.iter().any(|a| {
            a.resource == resource
            && a.action == action
            && match a.scope_type.as_str() {
                "global" => true,
                "campus" => scope_site_id.is_some() && a.scope_site_id == scope_site_id,
                "unit" => scope_entity_id.is_some() && a.scope_entity_id == scope_entity_id,
                _ => false,
            }
        })
    }
}
```

**Uso em handlers com loop:**

```rust
pub async fn list_requisitions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<RequisitionDto>>, AppError> {
    // 1. Carregar permissões do usuário UMA VEZ
    let perms = get_user_effective_permissions(
        &state.pool, current_user.id, "sigalm",
    ).await?;

    // 2. Buscar requisições
    let requisitions = fetch_visible_requisitions(&state.pool, &current_user).await?;

    // 3. Enriquecer com flags de permissão (sem chamar enforce() em loop)
    let dtos: Vec<RequisitionDto> = requisitions.into_iter().map(|req| {
        RequisitionDto {
            id: req.id,
            // ... campos
            can_approve: perms.can(
                "/requisicoes", "approve",
                Some(req.warehouse_id), Some(req.site_id),
            ),
            can_edit: perms.can(
                "/requisicoes", "update",
                Some(req.warehouse_id), Some(req.site_id),
            ),
        }
    }).collect();

    Ok(Json(dtos))
}
```

Isso é mais eficiente que cache de request e mais simples de entender.

---

## 4. Perfis: Por Nível de Acesso, Não por Funcionalidade

### 4.1. Por que não criar um perfil para cada funcionalidade?

Criar perfis como "gestor_abastecimento", "gestor_licenciamento", "gestor_manutencao" gera:

- **Pesadelo operacional**: o admin precisa entender a diferença entre 9 perfis
- **Explosão combinatória**: servidor que faz "um pouco de tudo" precisa de 5 perfis
- **Manutenção difícil**: cada funcionalidade nova = perfil novo + reconfiguração de todos os usuários

### 4.2. Modelo de Níveis

Use **3-4 níveis por módulo** e controle o que cada nível pode fazer via permissões granulares:

| Nível | Descrição | Quem atribui |
|-------|-----------|--------------|
| **Requisitor** | Faz requisições/solicitações do módulo | Automático ou coordenador |
| **Operador** | Executa operações do dia-a-dia | Coordenador do escopo |
| **Gestor** | Tudo do operador + cadastros + aprovações + relatórios | Coordenador do escopo |
| **Coordenador** | Tudo do gestor + gerencia usuários do módulo no escopo | Admin global |
| **Auditor** | Somente leitura de tudo no módulo | Admin global |

### 4.3. Aplicação por Módulo

**SIGALM:**

| Nível | O que pode fazer |
|-------|-----------------|
| Requisitor | Criar requisições pessoais e departamentais |
| Operador | Registrar movimentações de estoque (entrada avulsa, saída por requisição) |
| Gestor | Tudo do operador + entrada NF + nota de fornecimento + aprovação de requisições + transferências + relatórios |
| Coordenador | Tudo do gestor + gerenciar perfis de usuários do almoxarifado |
| Auditor | Consultas e relatórios apenas |
| Diretor de Material | Perfil especial global: cadastro de materiais + saldo em qualquer almoxarifado |

**SIGFROTA:**

| Nível | O que pode fazer |
|-------|-----------------|
| Requisitor | Reservar veículos |
| Operador | Registrar abastecimentos + entrada/saída de veículos + eventos |
| Gestor | Tudo do operador + cadastros (veículos, combustíveis, modelos) + licenciamento/multas + manutenção + relatórios |
| Coordenador | Tudo do gestor + gerenciar perfis de usuários de transporte no campus |
| Auditor | Consultas e relatórios apenas |

**Quando precisar de mais granularidade:** se um servidor precisa fazer **somente** abastecimento, dê o perfil "operador" e use um **bloqueio (deny)** nas permissões que ele não deve ter. Começa simples, refina depois.

### 4.4. Tabela de Módulos

```sql
CREATE TABLE modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) UNIQUE NOT NULL,  -- 'sigalm', 'sigfrota', 'sigep'
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO modules (code, name) VALUES
    ('sigalm', 'Sistema de Gestão de Almoxarifado'),
    ('sigfrota', 'Sistema de Gestão de Frota'),
    ('sigep', 'Sistema de Gestão de Espaço Físico');
```

### 4.5. Tabela de Perfis por Módulo

```sql
CREATE TABLE module_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES modules(id),
    code VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    level INTEGER NOT NULL DEFAULT 0,        -- 0=requisitor, 10=operador, 20=gestor, 30=coordenador, 40=auditor
    scope_type VARCHAR(20) NOT NULL,         -- 'global', 'campus', 'unit'
    is_implicit BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE = atribuído automaticamente
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_module_profile UNIQUE (module_id, code)
);
```

**Dados SIGALM:**

```sql
INSERT INTO module_profiles (module_id, code, name, description, level, scope_type, is_implicit) VALUES
((SELECT id FROM modules WHERE code = 'sigalm'), 'requisitor_pessoal',
 'Requisitor Pessoal',
 'Pode criar requisições pessoais de material. Atribuído automaticamente a todo servidor.',
 0, 'campus', TRUE),

((SELECT id FROM modules WHERE code = 'sigalm'), 'requisitor_departamental',
 'Requisitor Departamental',
 'Pode criar requisições em nome da sua unidade organizacional.',
 5, 'campus', FALSE),

((SELECT id FROM modules WHERE code = 'sigalm'), 'operador_almox',
 'Operador de Almoxarifado',
 'Registra movimentações de estoque: entradas avulsas, saídas por requisição.',
 10, 'unit', FALSE),

((SELECT id FROM modules WHERE code = 'sigalm'), 'gestor_almox',
 'Gestor de Almoxarifado',
 'Todas as operações do almoxarifado: entrada NF, nota de fornecimento, aprovação de requisições, transferências, relatórios.',
 20, 'unit', FALSE),

((SELECT id FROM modules WHERE code = 'sigalm'), 'coordenador_almox',
 'Coordenador de Almoxarifado',
 'Tudo do gestor + gerencia perfis de usuários do almoxarifado no seu escopo.',
 30, 'unit', FALSE),

((SELECT id FROM modules WHERE code = 'sigalm'), 'auditor_almox',
 'Auditor do Almoxarifado',
 'Acesso somente leitura a relatórios, consultas e histórico do almoxarifado.',
 40, 'unit', FALSE),

((SELECT id FROM modules WHERE code = 'sigalm'), 'diretor_material',
 'Diretor de Material',
 'Cadastro de materiais e consulta de saldo em qualquer almoxarifado. Perfil global.',
 25, 'global', FALSE);
```

**Dados SIGFROTA:**

```sql
INSERT INTO module_profiles (module_id, code, name, description, level, scope_type, is_implicit) VALUES
((SELECT id FROM modules WHERE code = 'sigfrota'), 'requisitor_reserva',
 'Requisitor de Reserva de Veículo',
 'Pode solicitar reservas de veículos no seu campus.',
 0, 'campus', FALSE),

((SELECT id FROM modules WHERE code = 'sigfrota'), 'operador_frota',
 'Operador de Frota',
 'Registra abastecimentos, entrada/saída de veículos, eventos.',
 10, 'campus', FALSE),

((SELECT id FROM modules WHERE code = 'sigfrota'), 'gestor_frota',
 'Gestor de Frota',
 'Todas as operações: cadastros, manutenção, licenciamento, relatórios.',
 20, 'campus', FALSE),

((SELECT id FROM modules WHERE code = 'sigfrota'), 'coordenador_frota',
 'Coordenador de Frota',
 'Tudo do gestor + gerencia perfis de usuários de transporte no campus.',
 30, 'campus', FALSE),

((SELECT id FROM modules WHERE code = 'sigfrota'), 'auditor_frota',
 'Auditor de Frota',
 'Acesso somente leitura a relatórios e consultas do módulo de transporte.',
 40, 'campus', FALSE);
```

### 4.6. Tabela de Permissões por Perfil

```sql
CREATE TABLE profile_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES module_profiles(id),
    resource VARCHAR(100) NOT NULL,   -- '/requisicoes', '/estoque', '/veiculos/:id'
    action VARCHAR(50) NOT NULL,      -- 'create', 'read', 'update', 'delete', 'approve'
    
    CONSTRAINT uq_profile_permission UNIQUE (profile_id, resource, action)
);
```

**Exemplo: permissões do Gestor de Almoxarifado:**

```sql
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    -- Estoque
    ('/estoque', 'create'),
    ('/estoque', 'read'),
    ('/estoque', 'update'),
    ('/estoque/entrada-nf', 'create'),
    ('/estoque/entrada-fornecimento', 'create'),
    ('/estoque/entrada-avulsa', 'create'),
    ('/estoque/saida-requisicao', 'create'),
    ('/estoque/saida-avulsa', 'create'),
    -- Requisições
    ('/requisicoes', 'read'),
    ('/requisicoes', 'approve'),
    ('/requisicoes', 'reject'),
    -- Transferências
    ('/transferencias', 'create'),
    ('/transferencias', 'read'),
    ('/transferencias', 'approve'),
    -- Relatórios
    ('/relatorios', 'read')
) AS r(resource, action)
WHERE p.code = 'gestor_almox';

-- Operador: subconjunto do gestor
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    ('/estoque', 'read'),
    ('/estoque/entrada-avulsa', 'create'),
    ('/estoque/saida-requisicao', 'create'),
    ('/requisicoes', 'read')
) AS r(resource, action)
WHERE p.code = 'operador_almox';

-- Auditor: somente leitura
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    ('/estoque', 'read'),
    ('/requisicoes', 'read'),
    ('/transferencias', 'read'),
    ('/relatorios', 'read'),
    ('/movimentacoes', 'read')
) AS r(resource, action)
WHERE p.code = 'auditor_almox';

-- Coordenador: tudo do gestor + admin
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    -- Herda tudo do gestor (repetir ou usar view)
    ('/estoque', 'create'),
    ('/estoque', 'read'),
    ('/estoque', 'update'),
    ('/estoque/entrada-nf', 'create'),
    ('/estoque/entrada-fornecimento', 'create'),
    ('/estoque/entrada-avulsa', 'create'),
    ('/estoque/saida-requisicao', 'create'),
    ('/estoque/saida-avulsa', 'create'),
    ('/requisicoes', 'read'),
    ('/requisicoes', 'approve'),
    ('/requisicoes', 'reject'),
    ('/transferencias', 'create'),
    ('/transferencias', 'read'),
    ('/transferencias', 'approve'),
    ('/relatorios', 'read'),
    -- Admin (exclusivo do coordenador)
    ('/admin/assignments', 'create'),
    ('/admin/assignments', 'read'),
    ('/admin/assignments', 'update'),
    ('/admin/assignments', 'delete'),
    ('/admin/users', 'read')
) AS r(resource, action)
WHERE p.code = 'coordenador_almox';

-- Requisitor pessoal (implícito)
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    ('/requisicoes/pessoal', 'create'),
    ('/requisicoes/pessoal', 'read')
) AS r(resource, action)
WHERE p.code = 'requisitor_pessoal';

-- Requisitor departamental
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    ('/requisicoes/pessoal', 'create'),
    ('/requisicoes/pessoal', 'read'),
    ('/requisicoes/departamental', 'create'),
    ('/requisicoes/departamental', 'read')
) AS r(resource, action)
WHERE p.code = 'requisitor_departamental';

-- Diretor de Material (global)
INSERT INTO profile_permissions (profile_id, resource, action)
SELECT p.id, r.resource, r.action
FROM module_profiles p
CROSS JOIN (VALUES 
    ('/materiais', 'create'),
    ('/materiais', 'read'),
    ('/materiais', 'update'),
    ('/materiais', 'delete'),
    ('/estoque/saldo', 'read'),
    ('/relatorios', 'read')
) AS r(resource, action)
WHERE p.code = 'diretor_material';
```

---

## 5. Escopo: Vinculação Genérica Módulo↔Unidade Responsável

### 5.1. Por que não colocar `responsible_department_id` diretamente na entidade?

Colocar `transport_department_id` no `sites` e `responsible_department_id` no `warehouses`:

- Cada módulo novo = coluna nova em tabelas existentes
- Sem histórico de mudanças
- Acoplamento forte entre schema de negócio e schema de autorização

### 5.2. Tabela de Vinculação Genérica

```sql
CREATE TABLE module_scope_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES modules(id),
    
    -- Onde: campus e/ou entidade específica
    site_id UUID REFERENCES sites(id),
    entity_id UUID,                    -- almoxarifado_id, frota_id, etc.
    entity_type VARCHAR(50),           -- 'almoxarifado', 'frota', etc.
    
    -- Quem é responsável
    responsible_department_id UUID NOT NULL REFERENCES departments(id),
    
    -- Vigência (permite histórico)
    valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
    valid_until DATE,                  -- NULL = vigente
    
    -- Metadados
    notes TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_module_scope_active UNIQUE (
        module_id, site_id, entity_id, entity_type, valid_until
    )
);

-- Índices
CREATE INDEX idx_msa_module ON module_scope_assignments(module_id);
CREATE INDEX idx_msa_site ON module_scope_assignments(site_id);
CREATE INDEX idx_msa_entity ON module_scope_assignments(entity_id, entity_type);
CREATE INDEX idx_msa_active ON module_scope_assignments(module_id, site_id) 
    WHERE valid_until IS NULL;
```

**Exemplos de dados:**

```sql
-- Almoxarifado Central de Cuiabá → Coordenadoria de Material (SIORG)
INSERT INTO module_scope_assignments 
    (module_id, site_id, entity_id, entity_type, responsible_department_id, created_by)
VALUES (
    (SELECT id FROM modules WHERE code = 'sigalm'),
    (SELECT id FROM sites WHERE name = 'Campus Cuiabá'),
    'uuid-almox-central-cba',
    'almoxarifado',
    (SELECT id FROM departments WHERE sigla = 'CMAT'),
    'uuid-admin'
);

-- Transporte do Campus Cuiabá → Coordenadoria de Transportes
INSERT INTO module_scope_assignments 
    (module_id, site_id, entity_id, entity_type, responsible_department_id, created_by)
VALUES (
    (SELECT id FROM modules WHERE code = 'sigfrota'),
    (SELECT id FROM sites WHERE name = 'Campus Cuiabá'),
    NULL,            -- sem entidade específica, é o campus inteiro
    NULL,
    (SELECT id FROM departments WHERE sigla = 'CTRANS'),
    'uuid-admin'
);

-- Futuro: Segurança Patrimonial do Campus Sinop
INSERT INTO module_scope_assignments 
    (module_id, site_id, entity_id, entity_type, responsible_department_id, created_by)
VALUES (
    (SELECT id FROM modules WHERE code = 'sigpat'),
    (SELECT id FROM sites WHERE name = 'Campus Sinop'),
    NULL,
    NULL,
    (SELECT id FROM departments WHERE sigla = 'CSEG-SINOP'),
    'uuid-admin'
);
```

### 5.3. Benefícios

- Módulo novo = INSERT na tabela, não ALTER TABLE
- Histórico natural: ao mudar responsável, seta `valid_until` no registro atual e cria novo
- Query simples para saber "quem é responsável pelo almoxarifado X hoje?"

```sql
SELECT responsible_department_id 
FROM module_scope_assignments
WHERE module_id = (SELECT id FROM modules WHERE code = 'sigalm')
  AND entity_id = 'uuid-almox-central-cba'
  AND valid_until IS NULL;
```

---

## 6. Segregação de Funções

### 6.1. Por que é obrigatório em órgão público?

Princípio básico de controle interno: quem **requisita** material não pode ser a mesma pessoa que **aprova**. Quem **cadastra** uma entrada de NF não pode ser quem **audita** o estoque. Sem isso, você abre brecha para fraude.

### 6.2. Tabela de Incompatibilidades

```sql
CREATE TABLE profile_incompatibilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_a_id UUID NOT NULL REFERENCES module_profiles(id),
    profile_b_id UUID NOT NULL REFERENCES module_profiles(id),
    scope_match BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE = incompatível somente no mesmo escopo
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_incompatibility UNIQUE (profile_a_id, profile_b_id),
    CONSTRAINT chk_different_profiles CHECK (profile_a_id != profile_b_id)
);
```

**Regras de incompatibilidade:**

```sql
-- Requisitor departamental não pode ser aprovador no mesmo escopo
INSERT INTO profile_incompatibilities (profile_a_id, profile_b_id, scope_match, reason) VALUES
(
    (SELECT id FROM module_profiles WHERE code = 'requisitor_departamental'),
    (SELECT id FROM module_profiles WHERE code = 'gestor_almox'),
    TRUE,
    'Segregação: quem requisita material não pode aprovar a própria requisição no mesmo almoxarifado'
);

-- Operador não pode ser auditor no mesmo escopo
INSERT INTO profile_incompatibilities (profile_a_id, profile_b_id, scope_match, reason) VALUES
(
    (SELECT id FROM module_profiles WHERE code = 'operador_almox'),
    (SELECT id FROM module_profiles WHERE code = 'auditor_almox'),
    TRUE,
    'Segregação: quem executa movimentações não pode auditar no mesmo almoxarifado'
);
```

**Nota:** `scope_match = TRUE` significa que João pode ser Requisitor Departamental do Almoxarifado A e Gestor do Almoxarifado B, mas **não pode ser ambos no Almoxarifado A**. Se `scope_match = FALSE`, a incompatibilidade vale para qualquer escopo.

### 6.3. Verificação na Atribuição (Rust)

```rust
async fn verify_no_incompatibility(
    pool: &PgPool,
    user_id: Uuid,
    new_profile_id: Uuid,
    scope_site_id: Option<Uuid>,
    scope_entity_id: Option<Uuid>,
) -> Result<(), AppError> {
    let conflicts = sqlx::query_as!(
        IncompatibilityConflict,
        r#"
        SELECT 
            pi.reason,
            mp_existing.name as existing_profile_name,
            mp_new.name as new_profile_name
        FROM profile_incompatibilities pi
        JOIN user_profile_assignments upa ON (
            upa.profile_id = pi.profile_a_id AND pi.profile_b_id = $2
            OR 
            upa.profile_id = pi.profile_b_id AND pi.profile_a_id = $2
        )
        JOIN module_profiles mp_existing ON upa.profile_id = mp_existing.id
        JOIN module_profiles mp_new ON mp_new.id = $2
        WHERE upa.user_id = $1
          AND upa.is_active = TRUE
          AND (upa.expires_at IS NULL OR upa.expires_at > NOW())
          AND (
              pi.scope_match = FALSE
              OR (
                  pi.scope_match = TRUE
                  AND (upa.scope_site_id = $3 OR ($3 IS NULL AND upa.scope_site_id IS NULL))
                  AND (upa.scope_entity_id = $4 OR ($4 IS NULL AND upa.scope_entity_id IS NULL))
              )
          )
        "#,
        user_id,
        new_profile_id,
        scope_site_id,
        scope_entity_id,
    )
    .fetch_all(pool)
    .await?;

    if !conflicts.is_empty() {
        let reasons: Vec<String> = conflicts.iter()
            .map(|c| format!(
                "Conflito: '{}' é incompatível com '{}'. Motivo: {}",
                c.new_profile_name, c.existing_profile_name, c.reason
            ))
            .collect();
        
        return Err(AppError::Conflict(reasons.join("; ")));
    }

    Ok(())
}
```

---

## 7. Requisições: 3 Categorias, Não "Todos Podem Tudo"

### 7.1. Categorias de Requisição

| Categoria | Quem pode | Perfil necessário | Exemplo |
|-----------|-----------|-------------------|---------|
| **Pessoal** | Qualquer servidor autenticado | `requisitor_pessoal` (implícito) | Material de escritório para mim |
| **Departamental** | Servidor designado | `requisitor_departamental` (explícito) | Material para o Laboratório X |
| **Especial** | Servidor com aprovação prévia | `requisitor_departamental` + fluxo extra | Material permanente, grande volume |

### 7.2. Perfil Implícito

O `requisitor_pessoal` tem `is_implicit = TRUE`. Isso significa que todo servidor vinculado a uma unidade organizacional **automaticamente** recebe este perfil para o campus da sua unidade. Não precisa de atribuição manual.

```rust
// No SingleEnforcerService, ao gerar policies:
async fn generate_implicit_profiles(&self) -> Result<Vec<Policy>> {
    // Buscar todos os perfis implícitos
    let implicit_profiles = sqlx::query_as!(
        ImplicitProfile,
        r#"
        SELECT mp.id as profile_id, mp.code, m.code as module_code,
               pp.resource, pp.action
        FROM module_profiles mp
        JOIN modules m ON mp.module_id = m.id
        JOIN profile_permissions pp ON pp.profile_id = mp.id
        WHERE mp.is_implicit = TRUE AND mp.is_active = TRUE
        "#
    )
    .fetch_all(&self.pool)
    .await?;

    // Para cada usuário ativo com unidade vinculada
    let users = sqlx::query_as!(
        UserWithSite,
        r#"
        SELECT u.id as user_id, d.site_id
        FROM auth_users u
        JOIN departments d ON u.department_id = d.id
        WHERE u.is_active = TRUE AND d.site_id IS NOT NULL
        "#
    )
    .fetch_all(&self.pool)
    .await?;

    // Gerar policies implícitas
    let mut policies = Vec::new();
    for user in &users {
        for profile in &implicit_profiles {
            let domain = format!("{}:campus:{}", profile.module_code, user.site_id);
            policies.push(Policy {
                user_id: user.user_id,
                profile_code: profile.code.clone(),
                domain,
                resource: profile.resource.clone(),
                action: profile.action.clone(),
            });
        }
    }

    Ok(policies)
}
```

### 7.3. Bloqueios Específicos (deny)

Para o caso "servidor X está bloqueado de fazer requisições de material permanente":

```sql
CREATE TABLE user_permission_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    module_code VARCHAR(20) NOT NULL,
    resource VARCHAR(100) NOT NULL,    -- '/requisicoes/material-permanente'
    action VARCHAR(50) NOT NULL,       -- 'create'
    scope_site_id UUID REFERENCES sites(id),
    
    -- Justificativa (OBRIGATÓRIA em órgão público)
    reason TEXT NOT NULL,                      -- descrição do motivo
    process_reference VARCHAR(100),            -- nº do processo SEI, portaria, memorando
    supporting_document_url VARCHAR(500),      -- link para documento no SEI/repositório
    
    -- Ciclo de vida (soft delete)
    blocked_by UUID NOT NULL,
    blocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    unblocked_by UUID,
    unblocked_at TIMESTAMPTZ,
    unblock_reason TEXT,
    
    CONSTRAINT uq_user_block UNIQUE (user_id, module_code, resource, action, scope_site_id)
        WHERE (is_active = TRUE),
    
    CONSTRAINT chk_block_justification CHECK (
        reason IS NOT NULL AND length(trim(reason)) > 10
    ),
    
    -- Processo SEI obrigatório apenas para bloqueios permanentes
    CONSTRAINT chk_permanent_block_needs_process CHECK (
        expires_at IS NOT NULL  -- temporário: process_reference opcional
        OR process_reference IS NOT NULL  -- permanente: process_reference obrigatório
    )
);

CREATE INDEX idx_blocks_user ON user_permission_blocks(user_id) WHERE is_active = TRUE;
CREATE INDEX idx_blocks_module ON user_permission_blocks(module_code) WHERE is_active = TRUE;
```

---

## 8. Atribuição de Perfis aos Usuários

### 8.1. Tabela Principal (Imutável — Soft Delete)

Em sistemas públicos, **nunca faça DELETE físico** em atribuições. Toda mudança deve ser rastreável. A abordagem é: registros desativados permanecem com `is_active = FALSE` e metadados de quem desativou.

```sql
CREATE TABLE user_profile_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    profile_id UUID NOT NULL REFERENCES module_profiles(id),
    
    -- Escopo da atribuição
    scope_site_id UUID REFERENCES sites(id),
    scope_department_id UUID REFERENCES departments(id),
    scope_entity_id UUID,
    scope_entity_type VARCHAR(50),
    
    -- Ciclo de vida (NUNCA deletar fisicamente)
    assigned_by UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,               -- para substituições temporárias
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    deactivated_by UUID,                  -- quem desativou
    deactivated_at TIMESTAMPTZ,           -- quando desativou
    deactivation_reason TEXT,             -- motivo da desativação
    
    -- Justificativa
    notes TEXT,                           -- motivo/justificativa da atribuição
    process_reference VARCHAR(100),       -- nº do processo SEI, portaria, etc.
    
    CONSTRAINT uq_user_profile_scope_active UNIQUE (
        user_id, profile_id, scope_site_id, 
        scope_department_id, scope_entity_id
    ) WHERE (is_active = TRUE),
    
    CONSTRAINT chk_deactivation_consistency CHECK (
        (is_active = TRUE AND deactivated_by IS NULL AND deactivated_at IS NULL)
        OR
        (is_active = FALSE AND deactivated_by IS NOT NULL AND deactivated_at IS NOT NULL)
    )
);

CREATE INDEX idx_upa_user ON user_profile_assignments(user_id) WHERE is_active = TRUE;
CREATE INDEX idx_upa_user_history ON user_profile_assignments(user_id, assigned_at DESC);
CREATE INDEX idx_upa_profile ON user_profile_assignments(profile_id);
CREATE INDEX idx_upa_scope_site ON user_profile_assignments(scope_site_id);
CREATE INDEX idx_upa_scope_entity ON user_profile_assignments(scope_entity_id, scope_entity_type);
CREATE INDEX idx_upa_expiring ON user_profile_assignments(expires_at) 
    WHERE is_active = TRUE AND expires_at IS NOT NULL;
```

**Desativação em vez de DELETE (transacional):**

```rust
pub async fn revoke_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
    revoked_by: Uuid,
    reason: &str,
    process_reference: Option<&str>,
) -> Result<(), AppError> {
    // NUNCA: DELETE FROM user_profile_assignments WHERE id = $1
    // SEMPRE: soft delete + evento na MESMA transação
    
    let mut tx = pool.begin().await?;

    // 1. Soft delete
    let result = sqlx::query!(
        r#"
        UPDATE user_profile_assignments 
        SET is_active = FALSE,
            deactivated_by = $2,
            deactivated_at = NOW(),
            deactivation_reason = $3
        WHERE id = $1 AND is_active = TRUE
        RETURNING user_id, profile_id
        "#,
        assignment_id,
        revoked_by,
        reason,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(record) = result else {
        return Err(AppError::NotFound("Atribuição não encontrada ou já desativada".into()));
    };

    // 2. Registrar evento (MESMA transação)
    sqlx::query!(
        r#"
        INSERT INTO assignment_events 
            (assignment_id, event_type, actor_id, reason, process_reference)
        VALUES ($1, 'revoked', $2, $3, $4)
        "#,
        assignment_id, revoked_by, reason, process_reference,
    )
    .execute(&mut *tx)
    .await?;

    // 3. Commit — ambos acontecem ou nenhum
    tx.commit().await?;

    // 4. Sync incremental + bump version (fora da transação — se falhar, o full_sync corrige)
    // ...

    Ok(())
}
```

### 8.2. Eventos de Ciclo de Vida (Event Sourcing Light)

O soft delete mostra o estado final, mas não captura o ciclo completo. Se o admin atribuiu → suspendeu → reativou → revogou, o soft delete só mostra "revogado". A tabela de eventos registra tudo:

```sql
CREATE TABLE assignment_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES user_profile_assignments(id),
    event_type VARCHAR(20) NOT NULL,  -- 'created', 'suspended', 'reactivated', 'revoked', 'expired'
    actor_id UUID NOT NULL,
    reason TEXT,
    process_reference VARCHAR(100),
    metadata JSONB DEFAULT '{}',       -- dados extras do contexto
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assignment_events_assignment ON assignment_events(assignment_id, created_at);
CREATE INDEX idx_assignment_events_actor ON assignment_events(actor_id, created_at DESC);
```

Uso:

```rust
async fn record_assignment_event(
    pool: &PgPool,
    assignment_id: Uuid,
    event_type: &str,
    actor_id: Uuid,
    reason: Option<&str>,
    process_reference: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        INSERT INTO assignment_events 
            (assignment_id, event_type, actor_id, reason, process_reference)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        assignment_id, event_type, actor_id, reason, process_reference,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Ao criar uma atribuição — também transacional
pub async fn create_assignment_with_event(
    pool: &PgPool,
    payload: &CreateAssignmentRequest,
    assigned_by: Uuid,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;

    // 1. Inserir atribuição
    let assignment_id = sqlx::query_scalar!(
        r#"
        INSERT INTO user_profile_assignments 
            (user_id, profile_id, scope_site_id, scope_department_id, 
             scope_entity_id, scope_entity_type, assigned_by, expires_at, 
             notes, process_reference)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        payload.user_id, payload.profile_id, payload.scope_site_id,
        payload.scope_department_id, payload.scope_entity_id,
        payload.scope_entity_type, assigned_by, payload.expires_at,
        payload.notes, payload.process_reference,
    )
    .fetch_one(&mut *tx)
    .await?;

    // 2. Registrar evento (MESMA transação)
    sqlx::query!(
        r#"
        INSERT INTO assignment_events 
            (assignment_id, event_type, actor_id, reason, process_reference)
        VALUES ($1, 'created', $2, $3, $4)
        "#,
        assignment_id, assigned_by, payload.notes, payload.process_reference,
    )
    .execute(&mut *tx)
    .await?;

    // 3. Commit
    tx.commit().await?;

    Ok(assignment_id)
}
```

**Regra:** toda operação que modifica `user_profile_assignments` E insere em `assignment_events` deve estar na mesma transação SQL. Se o INSERT do evento falhar, o UPDATE/INSERT da atribuição é revertido automaticamente.

### 8.3. Atribuições Temporárias

O campo `expires_at` resolve substituições (férias, licença, designação temporária). Um job periódico desativa atribuições expiradas:

```rust
// Cron job: a cada hora
pub async fn deactivate_expired_assignments(pool: &PgPool) -> Result<u64> {
    // Soft delete com motivo automático
    let result = sqlx::query!(
        r#"
        UPDATE user_profile_assignments 
        SET is_active = FALSE,
            deactivated_by = assigned_by,
            deactivated_at = NOW(),
            deactivation_reason = 'Expiração automática (expires_at atingido)'
        WHERE is_active = TRUE 
          AND expires_at IS NOT NULL 
          AND expires_at < NOW()
        RETURNING id, user_id
        "#
    )
    .fetch_all(pool)
    .await?;

    if !result.is_empty() {
        tracing::info!("{} atribuições temporárias expiradas", result.len());
        // Disparar resync incremental para cada usuário afetado
    }

    Ok(result.len() as u64)
}
```

### 8.4. Validação de Negócio na Atribuição

Antes de atribuir um perfil, o sistema pode validar pré-requisitos específicos do módulo. Com 2-3 módulos, um `match` simples é mais legível que traits abstratas. Quando tiver 10+ módulos, extraia para traits:

```rust
/// Valida pré-requisitos para atribuição de perfil.
/// Simples e direto — extrair para traits quando tiver 10+ módulos.
async fn validate_assignment_prerequisites(
    pool: &PgPool,
    module_code: &str,
    profile_code: &str,
    user_id: Uuid,
) -> Result<(), AppError> {
    match (module_code, profile_code) {
        // SIGALM: operador e gestor precisam de treinamento
        ("sigalm", "operador_almox" | "gestor_almox") => {
            let has_training = sqlx::query_scalar!(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM user_trainings
                    WHERE user_id = $1 
                      AND training_code = 'ALMOX_OPERACIONAL'
                      AND completed_at IS NOT NULL
                      AND (expires_at IS NULL OR expires_at > NOW())
                )
                "#,
                user_id,
            )
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Erro ao verificar treinamento: {}", e)))?;

            if !has_training.unwrap_or(false) {
                return Err(AppError::ValidationFailed(
                    "Servidor precisa completar o treinamento 'Operação de Almoxarifado' \
                     antes de receber este perfil. Verifique no SIGCAP."
                        .to_string()
                ));
            }
        }

        // SIGFROTA: operador e gestor precisam de CNH válida
        ("sigfrota", "operador_frota" | "gestor_frota") => {
            let has_cnh = sqlx::query_scalar!(
                r#"
                SELECT EXISTS(
                    SELECT 1 FROM user_documents
                    WHERE user_id = $1 
                      AND document_type = 'CNH'
                      AND expiry_date > NOW()
                )
                "#,
                user_id,
            )
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Erro ao verificar CNH: {}", e)))?;

            if !has_cnh.unwrap_or(false) {
                return Err(AppError::ValidationFailed(
                    "Servidor precisa ter CNH válida cadastrada no sistema para \
                     receber perfil operacional de frota."
                        .to_string()
                ));
            }
        }

        // Demais: sem pré-requisitos
        _ => {}
    }

    Ok(())
}
```

Uso no handler de atribuição:

```rust
pub async fn create_assignment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateAssignmentRequest>,
) -> Result<StatusCode, AppError> {
    // 1. Verificar se pode atribuir (nível, escopo)
    verify_can_assign(&state, &current_user, &payload).await?;

    // 2. Verificar incompatibilidades
    verify_no_incompatibility(
        &state.pool, payload.user_id, payload.profile_id,
        payload.scope_site_id, payload.scope_entity_id,
    ).await?;

    // 3. Validar pré-requisitos do módulo
    validate_assignment_prerequisites(
        &state.pool,
        &payload.module_code,
        &payload.profile_code,
        payload.user_id,
    ).await?;

    // 4. Criar atribuição
    let assignment_id = sqlx::query_scalar!(/* ... */)
        .fetch_one(&state.pool)
        .await?;

    // 5. Sync incremental
    state.auth_service
        .add_assignment_policies(assignment_id)
        .await?;

    Ok(StatusCode::CREATED)
}
```

---

## 9. Hierarquia de Autorização ≠ Hierarquia Organizacional

### 9.1. O Problema

A hierarquia SIORG (via `caminho_hierarquico`) pode não refletir a hierarquia real de autoridade para fins de um módulo específico. Uma coordenação pode estar hierarquicamente abaixo de uma pró-reitoria no SIORG, mas operacionalmente responder a outra estrutura para fins de almoxarifado.

### 9.2. Hierarquia por Módulo

```sql
CREATE TABLE module_department_hierarchy (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id UUID NOT NULL REFERENCES modules(id),
    department_id UUID NOT NULL REFERENCES departments(id),
    parent_department_id UUID REFERENCES departments(id),
    
    -- NULL = usa a hierarquia SIORG padrão
    -- Se preenchido, sobrescreve a hierarquia SIORG para este módulo
    
    valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
    valid_until DATE,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_module_dept_hierarchy UNIQUE (module_id, department_id, valid_until)
);
```

### 9.3. Query de Visibilidade com Hierarquia

```rust
/// Retorna IDs de departamentos visíveis para um módulo.
/// Usa hierarquia específica do módulo se existir, senão usa SIORG.
async fn get_visible_departments(
    pool: &PgPool,
    module_code: &str,
    user_department_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let departments = sqlx::query_scalar!(
        r#"
        WITH RECURSIVE dept_tree AS (
            -- Base: departamento do usuário
            SELECT d.id, d.id as root_id
            FROM departments d
            WHERE d.id = $2
            
            UNION ALL
            
            -- Recursivo: sub-departamentos
            SELECT child.id, dt.root_id
            FROM departments child
            JOIN dept_tree dt ON (
                -- Primeiro tenta hierarquia específica do módulo
                EXISTS (
                    SELECT 1 FROM module_department_hierarchy mdh
                    JOIN modules m ON mdh.module_id = m.id
                    WHERE m.code = $1
                      AND mdh.department_id = child.id
                      AND mdh.parent_department_id = dt.id
                      AND mdh.valid_until IS NULL
                )
                OR (
                    -- Se não tem hierarquia específica, usa SIORG
                    NOT EXISTS (
                        SELECT 1 FROM module_department_hierarchy mdh
                        JOIN modules m ON mdh.module_id = m.id
                        WHERE m.code = $1
                          AND mdh.department_id = child.id
                          AND mdh.valid_until IS NULL
                    )
                    AND child.parent_id = dt.id
                )
            )
        )
        SELECT id FROM dept_tree
        "#,
        module_code,
        user_department_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(departments)
}
```

---

## 10. Sincronização com o Casbin

A sincronização é implementada pelo `SingleEnforcerService` (seção 3.1):

- **Startup**: `full_sync()` carrega todas as policies, perfis implícitos e bloqueios
- **Mudança pontual**: `add_assignment_policies()` e `remove_assignment_policies()` fazem sync incremental
- **Bloqueio**: `add_block_policy()` adiciona deny imediatamente
- **Expiração**: cron job horário desativa atribuições vencidas e dispara sync incremental
- **Rebuild forçado**: endpoint `POST /api/admin/policies/rebuild` (admin global) executa `full_sync()`

O `full_sync()` completo fica reservado para startup e situações excepcionais. Todas as operações do dia-a-dia usam sync incremental.

---

## 11. Middleware de Autorização (Axum)

```rust
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn authorization_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let current_user = request
        .extensions()
        .get::<CurrentUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?
        .clone();

    let path = request.uri().path().to_string();
    let method = request.method().clone();

    // Parse da rota: /api/{module}/...
    let route_info = parse_route(&path)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let action = method_to_action(method.as_str());

    let allowed = state.auth_service.check_permission(
        current_user.id,
        &route_info.module,
        route_info.scope_id,
        route_info.scope_type.as_deref(),
        &route_info.resource,
        &action,
    ).await;

    if !allowed {
        return Err(StatusCode::FORBIDDEN);
    }

    // Adicionar header de invalidação se permissões mudaram
    let mut response = next.run(request).await;
    
    if let Some(stale) = check_permissions_stale(
        &state.pool, 
        current_user.id, 
        current_user.permissions_version,
    ).await {
        if stale {
            response.headers_mut().insert(
                "X-Permissions-Stale",
                "true".parse().unwrap(),
            );
        }
    }

    Ok(response)
}

fn method_to_action(method: &str) -> String {
    match method {
        "GET" => "read".to_string(),
        "POST" => "create".to_string(),
        "PUT" | "PATCH" => "update".to_string(),
        "DELETE" => "delete".to_string(),
        _ => "read".to_string(),
    }
}

struct RouteInfo {
    module: String,
    scope_id: Option<Uuid>,
    scope_type: Option<String>,
    resource: String,
}

fn parse_route(path: &str) -> Result<RouteInfo, ()> {
    // /api/sigalm/almoxarifado/{almox_id}/estoque → module=sigalm, scope=almoxarifado:almox_id, resource=/estoque
    // /api/sigfrota/campus/{campus_id}/reservas → module=sigfrota, scope=campus:campus_id, resource=/reservas
    // /api/sigalm/materiais → module=sigalm, scope=global, resource=/materiais
    
    let parts: Vec<&str> = path.trim_start_matches("/api/").split('/').collect();
    
    if parts.is_empty() {
        return Err(());
    }

    let module = parts[0].to_string();

    // Detectar escopo baseado no padrão da rota
    let (scope_type, scope_id, resource_start) = if parts.len() >= 3 {
        match parts[1] {
            "almoxarifado" | "frota" | "campus" => {
                let scope_type = parts[1].to_string();
                let scope_id = Uuid::parse_str(parts[2]).ok();
                (Some(scope_type), scope_id, 3)
            }
            _ => (None, None, 1),
        }
    } else {
        (None, None, 1)
    };

    let resource = format!("/{}", parts[resource_start..].join("/"));

    Ok(RouteInfo {
        module,
        scope_id,
        scope_type,
        resource,
    })
}
```

---

## 12. API de Permissões para o Frontend

### 11.1. Endpoint Principal

```rust
// GET /api/auth/me/permissions
pub async fn get_my_permissions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<(StatusCode, impl IntoResponse), AppError> {
    let assignments = sqlx::query_as!(
        AssignmentDto,
        r#"
        SELECT 
            m.code as module,
            mp.code as profile,
            mp.name as profile_name,
            mp.scope_type,
            mp.level as profile_level,
            upa.scope_site_id,
            upa.scope_entity_id,
            upa.scope_entity_type,
            s.name as site_name,
            upa.expires_at,
            array_agg(DISTINCT pp.resource || ':' || pp.action) as "permissions!"
        FROM user_profile_assignments upa
        JOIN module_profiles mp ON upa.profile_id = mp.id
        JOIN modules m ON mp.module_id = m.id
        JOIN profile_permissions pp ON pp.profile_id = mp.id
        LEFT JOIN sites s ON upa.scope_site_id = s.id
        WHERE upa.user_id = $1
          AND upa.is_active = TRUE
          AND (upa.expires_at IS NULL OR upa.expires_at > NOW())
          AND mp.is_active = TRUE
          AND m.is_active = TRUE
        GROUP BY m.code, mp.code, mp.name, mp.scope_type, mp.level,
                 upa.scope_site_id, upa.scope_entity_id, 
                 upa.scope_entity_type, s.name, upa.expires_at
        "#,
        current_user.id,
    )
    .fetch_all(&state.pool)
    .await?;

    // Carregar perfis implícitos
    let implicit_assignments = get_implicit_assignments(&state.pool, current_user.id).await?;

    // Carregar bloqueios
    let blocks = sqlx::query_as!(
        BlockDto,
        r#"
        SELECT module_code as module, resource, action, scope_site_id, reason
        FROM user_permission_blocks
        WHERE user_id = $1 AND is_active = TRUE
          AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        current_user.id,
    )
    .fetch_all(&state.pool)
    .await?;

    // Buscar version para ETag
    let version = sqlx::query_scalar!(
        "SELECT permissions_version FROM auth_users WHERE id = $1",
        current_user.id,
    )
    .fetch_one(&state.pool)
    .await?;

    let response = UserPermissionsResponse {
        user_id: current_user.id,
        version,
        assignments: [assignments, implicit_assignments].concat(),
        blocks,
    };

    // ETag baseado no version
    let etag = format!("\"perm-v{}\"", version);
    
    Ok((
        StatusCode::OK,
        [("ETag", etag)],
        Json(response),
    ))
}
```

### 11.2. Resposta JSON

```json
{
  "user_id": "uuid-do-joao",
  "version": 42,
  "assignments": [
    {
      "module": "sigalm",
      "profile": "gestor_almox",
      "profileName": "Gestor de Almoxarifado",
      "scopeType": "unit",
      "profileLevel": 20,
      "scopeSiteId": "uuid-campus-cuiaba",
      "scopeEntityId": "uuid-almox-central-cba",
      "scopeEntityType": "almoxarifado",
      "siteName": "Campus Cuiabá",
      "expiresAt": null,
      "permissions": [
        "/estoque:create", "/estoque:read", "/estoque:update",
        "/estoque/entrada-nf:create", "/requisicoes:read", 
        "/requisicoes:approve", "/transferencias:create"
      ]
    },
    {
      "module": "sigalm",
      "profile": "requisitor_pessoal",
      "profileName": "Requisitor Pessoal",
      "scopeType": "campus",
      "profileLevel": 0,
      "scopeSiteId": "uuid-campus-cuiaba",
      "scopeEntityId": null,
      "scopeEntityType": null,
      "siteName": "Campus Cuiabá",
      "expiresAt": null,
      "permissions": [
        "/requisicoes/pessoal:create", "/requisicoes/pessoal:read"
      ]
    },
    {
      "module": "sigfrota",
      "profile": "requisitor_reserva",
      "profileName": "Requisitor de Reserva de Veículo",
      "scopeType": "campus",
      "profileLevel": 0,
      "scopeSiteId": "uuid-campus-cuiaba",
      "scopeEntityId": null,
      "scopeEntityType": null,
      "siteName": "Campus Cuiabá",
      "expiresAt": null,
      "permissions": [
        "/reservas:create", "/reservas:read"
      ]
    }
  ],
  "blocks": [
    {
      "module": "sigalm",
      "resource": "/requisicoes/material-permanente",
      "action": "create",
      "scopeSiteId": null,
      "reason": "Pendência de prestação de contas"
    }
  ]
}
```

---

## 13. Frontend Angular — Arquitetura Completa

### 13.1. Permission Store com TTL e Invalidação

```typescript
// libs/shared/features/auth/data-access/permission.store.ts

import { computed, inject } from '@angular/core';
import {
  signalStore, withState, withComputed, withMethods, 
  withHooks, patchState
} from '@ngrx/signals';
import { HttpClient } from '@angular/common/http';
import { firstValueFrom } from 'rxjs';

export interface AssignmentInfo {
  module: string;
  profile: string;
  profileName: string;
  scopeType: 'global' | 'campus' | 'unit';
  profileLevel: number;
  scopeSiteId: string | null;
  scopeEntityId: string | null;
  scopeEntityType: string | null;
  siteName: string | null;
  expiresAt: string | null;
  permissions: string[];
}

export interface BlockInfo {
  module: string;
  resource: string;
  action: string;
  scopeSiteId: string | null;
  reason: string;
}

interface PermissionState {
  assignments: AssignmentInfo[];
  blocks: BlockInfo[];
  version: number;
  loaded: boolean;
  lastLoadedAt: number | null;  // timestamp
}

const TTL_MS = 5 * 60 * 1000; // 5 minutos

export const PermissionStore = signalStore(
  { providedIn: 'root' },

  withState<PermissionState>({
    assignments: [],
    blocks: [],
    version: 0,
    loaded: false,
    lastLoadedAt: null,
  }),

  withComputed((store) => ({
    availableModules: computed(() => {
      const modules = new Set(store.assignments().map(a => a.module));
      return [...modules];
    }),

    profilesByModule: computed(() => {
      const map = new Map<string, AssignmentInfo[]>();
      for (const a of store.assignments()) {
        const list = map.get(a.module) || [];
        list.push(a);
        map.set(a.module, list);
      }
      return map;
    }),

    isStale: computed(() => {
      const lastLoaded = store.lastLoadedAt();
      if (!lastLoaded) return true;
      return Date.now() - lastLoaded > TTL_MS;
    }),
  })),

  withMethods((store, http = inject(HttpClient)) => ({
    // ---- Carregamento ----

    async loadPermissions(): Promise<void> {
      try {
        const response = await firstValueFrom(
          http.get<{
            assignments: AssignmentInfo[];
            blocks: BlockInfo[];
            version: number;
          }>('/api/auth/me/permissions')
        );

        patchState(store, {
          assignments: response.assignments,
          blocks: response.blocks,
          version: response.version,
          loaded: true,
          lastLoadedAt: Date.now(),
        });
      } catch (error) {
        console.error('Falha ao carregar permissões', error);
      }
    },

    async refreshIfStale(): Promise<void> {
      if (store.isStale()) {
        await this.loadPermissions();
      }
    },

    /** 
     * Chamado pelo interceptor HTTP quando detecta header X-Permissions-Stale.
     * Recarrega permissões em background.
     */
    async handleStaleNotification(): Promise<void> {
      await this.loadPermissions();
    },

    clearPermissions(): void {
      patchState(store, {
        assignments: [], blocks: [], version: 0,
        loaded: false, lastLoadedAt: null,
      });
    },

    // ---- Verificações ----

    hasPermission(
      module: string,
      resource: string,
      action: string,
      scopeEntityId?: string | null,
      scopeSiteId?: string | null,
    ): boolean {
      // 1. Bloqueios vencem (deny wins)
      const isBlocked = store.blocks().some(b =>
        b.module === module &&
        b.resource === resource &&
        b.action === action &&
        (!b.scopeSiteId || b.scopeSiteId === scopeSiteId)
      );
      if (isBlocked) return false;

      // 2. Verificar permissão
      const permKey = `${resource}:${action}`;
      return store.assignments().some(a => {
        if (a.module !== module) return false;
        if (!a.permissions.includes(permKey)) return false;

        // Verificar escopo
        if (a.scopeType === 'global') return true;
        if (scopeEntityId && a.scopeEntityId === scopeEntityId) return true;
        if (scopeSiteId && a.scopeSiteId === scopeSiteId) return true;

        // Campus match: se o assignment é do campus e não precisa de entity específica
        if (a.scopeType === 'campus' && !scopeEntityId && a.scopeSiteId === scopeSiteId) return true;

        return false;
      });
    },

    hasProfile(module: string, profile: string, scopeEntityId?: string): boolean {
      return store.assignments().some(a =>
        a.module === module &&
        a.profile === profile &&
        (!scopeEntityId || a.scopeEntityId === scopeEntityId)
      );
    },

    hasModuleAccess(module: string): boolean {
      return store.assignments().some(a => a.module === module);
    },

    getModuleSites(module: string): { id: string; name: string }[] {
      const sites = new Map<string, string>();
      for (const a of store.assignments()) {
        if (a.module === module && a.scopeSiteId && a.siteName) {
          sites.set(a.scopeSiteId, a.siteName);
        }
      }
      return [...sites.entries()].map(([id, name]) => ({ id, name }));
    },

    /**
     * Retorna versão das permissões para incluir no JWT claims
     * ou verificação de staleness.
     */
    getVersion(): number {
      return store.version();
    },
  })),
);
```

### 13.2. Interceptor de Invalidação

```typescript
// libs/shared/features/auth/util/interceptors/permissions-stale.interceptor.ts

import { HttpInterceptorFn } from '@angular/common/http';
import { inject } from '@angular/core';
import { PermissionStore } from '../../data-access/permission.store';

/**
 * Interceptor que detecta o header X-Permissions-Stale 
 * e recarrega permissões em background.
 */
export const permissionsStaleInterceptor: HttpInterceptorFn = (req, next) => {
  const permissionStore = inject(PermissionStore);

  return next(req).pipe(
    tap(event => {
      if (event instanceof HttpResponse) {
        const staleHeader = event.headers.get('X-Permissions-Stale');
        if (staleHeader === 'true') {
          // Recarrega em background, sem bloquear a resposta atual
          permissionStore.handleStaleNotification();
        }
      }
    }),
  );
};
```

### 13.3. Pre-fetching com Resolver (evita flicker de UI)

O guard verifica permissão, mas o componente pode renderizar antes dos dados estarem prontos — causando "flicker" onde botões aparecem e desaparecem. O resolver garante que as permissões estão carregadas **antes** da renderização:

```typescript
// libs/shared/features/auth/util/resolvers/permissions.resolver.ts

import { inject } from '@angular/core';
import { ResolveFn, Router } from '@angular/router';
import { PermissionStore } from '../../data-access/permission.store';

const MAX_RETRIES = 2;
const RETRY_DELAY_MS = 1000;

/**
 * Resolver que garante que as permissões estão carregadas e atualizadas
 * ANTES do componente ser renderizado. Evita o "flicker" de UI.
 * 
 * Inclui retry estratégico: se a rede falhar, tenta até 2x com 1s de intervalo.
 * Se todas as tentativas falharem, redireciona para /service-unavailable.
 * 
 * Uso:
 * {
 *   path: 'almoxarifado',
 *   resolve: { permissions: permissionsResolver },
 *   canActivate: [modulePermissionGuard],
 *   data: { module: 'sigalm' }
 * }
 */
export const permissionsResolver: ResolveFn<boolean> = async () => {
  const permissionStore = inject(PermissionStore);
  const router = inject(Router);

  // Já carregado e dentro do TTL — nada a fazer
  if (permissionStore.loaded() && !permissionStore.isStale()) {
    return true;
  }

  // Tentar carregar com retry
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    try {
      await permissionStore.loadPermissions();
      return true;
    } catch (error) {
      console.warn(
        `permissionsResolver: tentativa ${attempt + 1}/${MAX_RETRIES + 1} falhou`,
        error,
      );

      if (attempt < MAX_RETRIES) {
        await new Promise(resolve => setTimeout(resolve, RETRY_DELAY_MS));
      }
    }
  }

  // Todas as tentativas falharam
  console.error('permissionsResolver: falha ao carregar permissões após retries');

  // Se já tinha dados (mesmo stale), permite navegação com dados antigos
  // em vez de bloquear o usuário — melhor UX
  if (permissionStore.loaded()) {
    console.warn('permissionsResolver: usando permissões em cache (stale)');
    return true;
  }

  // Sem dados nenhum: redirecionar para página de erro
  router.navigate(['/service-unavailable']);
  return false;
};

/**
 * Resolver específico para módulo — pode pré-carregar dados
 * adicionais do módulo (ex: lista de almoxarifados do usuário).
 */
export const moduleContextResolver: ResolveFn<ModuleContext | null> = async (route) => {
  const permissionStore = inject(PermissionStore);

  // Garantir permissões carregadas (o permissionsResolver já fez isso,
  // mas por segurança verificamos)
  if (!permissionStore.loaded()) {
    return null;
  }

  const module = route.data['module'] as string;
  if (!module) return null;

  const sites = permissionStore.getModuleSites(module);

  return {
    module,
    sites,
    assignments: permissionStore.profilesByModule().get(module) || [],
  };
};

interface ModuleContext {
  module: string;
  sites: { id: string; name: string }[];
  assignments: AssignmentInfo[];
}
```

**Fine-grained Reactivity com Computed Signals:**

Em vez de chamar `permissionStore.hasPermission(...)` repetidamente nos templates, crie signals derivados nos componentes:

```typescript
// Exemplo: componente de estoque que precisa verificar várias permissões

@Component({
  selector: 'app-estoque',
  standalone: true,
  imports: [HasPermissionDirective],
  template: `
    <h2>Estoque</h2>
    
    @if (canCreateEntry()) {
      <button (click)="novaEntrada()">Nova Entrada</button>
    }
    
    @if (canApproveRequisitions()) {
      <button (click)="aprovarRequisicoes()">Aprovar Requisições</button>
    }
    
    @if (isAuditor()) {
      <div class="audit-banner">Modo Auditoria — Somente Leitura</div>
    }
    
    <!-- ... -->
  `,
})
export class EstoqueComponent {
  private permissionStore = inject(PermissionStore);
  
  // Estes signals derivados recalculam automaticamente quando 
  // as permissões mudam (ex: após handleStaleNotification)
  
  canCreateEntry = computed(() => 
    this.permissionStore.hasPermission('sigalm', '/estoque/entrada-nf', 'create')
  );
  
  canApproveRequisitions = computed(() => 
    this.permissionStore.hasPermission('sigalm', '/requisicoes', 'approve')
  );
  
  isAuditor = computed(() => 
    this.permissionStore.hasProfile('sigalm', 'auditor_almox')
  );
  
  isManager = computed(() => 
    this.permissionStore.hasProfile('sigalm', 'gestor_almox') ||
    this.permissionStore.hasProfile('sigalm', 'coordenador_almox')
  );
  
  // Sites onde o usuário tem acesso ao SIGALM
  availableSites = computed(() => 
    this.permissionStore.getModuleSites('sigalm')
  );
}
```

**Quando usar `computed` vs `*hasPermission`:**

| Situação | Use |
|----------|-----|
| Mostrar/ocultar bloco inteiro de UI | `*hasPermission` (diretiva) |
| Habilitar/desabilitar botão | `computed` signal |
| Lógica condicional no componente | `computed` signal |
| Menu de navegação | `*hasPermission` (diretiva) |
| Múltiplas verificações no mesmo componente | `computed` signals (legibilidade) |

### 13.4. Permission Guard (Síncrono — Resolver Garante Dados)

O guard **não faz await**. O resolver (seção 13.3) já garantiu que as permissões estão carregadas antes da renderização. Isso elimina a duplicação de carregamento:

```typescript
// libs/shared/features/auth/util/guards/module-permission.guard.ts

import { inject } from '@angular/core';
import { CanActivateFn, Router, ActivatedRouteSnapshot } from '@angular/router';
import { PermissionStore } from '../../data-access/permission.store';

/**
 * Guard SÍNCRONO de permissão por módulo/recurso/perfil.
 * 
 * IMPORTANTE: usar sempre junto com o permissionsResolver na rota.
 * O resolver carrega as permissões (async), o guard só consulta (sync).
 * 
 * Uso nas rotas:
 * {
 *   path: 'almoxarifado',
 *   resolve: { permissions: permissionsResolver },  // ← carrega dados
 *   canActivate: [modulePermissionGuard],            // ← só consulta
 *   data: { 
 *     module: 'sigalm',
 *     // Opcionais:
 *     permission: '/estoque',
 *     action: 'read',
 *     profile: 'gestor_almox'
 *   }
 * }
 */
export const modulePermissionGuard: CanActivateFn = (
  route: ActivatedRouteSnapshot,
) => {
  const permissionStore = inject(PermissionStore);
  const router = inject(Router);

  // Sem await — o resolver já carregou as permissões.
  // Se o store tem dados stale (rede caiu durante refresh), ainda permite navegação.
  // Melhor mostrar UI com permissões 5min desatualizadas do que tela branca.
  if (!permissionStore.loaded()) {
    console.warn('modulePermissionGuard: permissions not loaded. Missing permissionsResolver?');
    return router.createUrlTree(['/service-unavailable']);
  }

  const module = route.data['module'] as string;
  const permission = route.data['permission'] as string | undefined;
  const action = route.data['action'] as string | undefined;
  const profile = route.data['profile'] as string | undefined;

  if (!module) {
    console.warn('modulePermissionGuard: module not specified in route data');
    return router.createUrlTree(['/access-denied']);
  }

  // Verificação por perfil
  if (profile) {
    return permissionStore.hasProfile(module, profile)
      ? true
      : router.createUrlTree(['/access-denied']);
  }

  // Verificação por permissão específica
  if (permission && action) {
    return permissionStore.hasPermission(module, permission, action)
      ? true
      : router.createUrlTree(['/access-denied']);
  }

  // Verificação básica: tem acesso ao módulo?
  return permissionStore.hasModuleAccess(module)
    ? true
    : router.createUrlTree(['/access-denied']);
};
```

### 13.5. Diretiva `*hasPermission`

```typescript
// libs/shared/features/auth/util/directives/has-permission.directive.ts

import {
  Directive, inject, input, effect,
  TemplateRef, ViewContainerRef, signal,
} from '@angular/core';
import { PermissionStore } from '../../data-access/permission.store';

export interface PermissionCheck {
  module: string;
  resource?: string;
  action?: string;
  profile?: string;
  scopeEntityId?: string;
  scopeSiteId?: string;
}

/**
 * Diretiva estrutural para mostrar/ocultar elementos baseado em permissão.
 * 
 * Exemplos:
 * 
 * <!-- Por permissão específica -->
 * <button *hasPermission="{ module: 'sigalm', resource: '/estoque', action: 'create' }">
 *   Nova Entrada
 * </button>
 * 
 * <!-- Por perfil -->
 * <div *hasPermission="{ module: 'sigalm', profile: 'gestor_almox' }">
 *   Painel do Gestor
 * </div>
 * 
 * <!-- Por módulo -->
 * <nav-item *hasPermission="{ module: 'sigfrota' }">Transporte</nav-item>
 * 
 * <!-- Com escopo -->
 * <button *hasPermission="{ 
 *     module: 'sigalm', resource: '/requisicoes', action: 'approve', 
 *     scopeEntityId: almoxarifado.id 
 * }">
 *   Aprovar
 * </button>
 */
@Directive({
  selector: '[hasPermission]',
  standalone: true,
})
export class HasPermissionDirective {
  hasPermission = input.required<PermissionCheck>();

  private store = inject(PermissionStore);
  private templateRef = inject(TemplateRef<unknown>);
  private viewContainer = inject(ViewContainerRef);
  private hasView = signal(false);

  constructor() {
    effect(() => {
      const check = this.hasPermission();
      const allowed = this.evaluate(check);

      if (allowed && !this.hasView()) {
        this.viewContainer.createEmbeddedView(this.templateRef);
        this.hasView.set(true);
      } else if (!allowed && this.hasView()) {
        this.viewContainer.clear();
        this.hasView.set(false);
      }
    });
  }

  private evaluate(check: PermissionCheck): boolean {
    if (!check.module) return false;

    if (check.profile) {
      return this.store.hasProfile(check.module, check.profile, check.scopeEntityId);
    }

    if (check.resource && check.action) {
      return this.store.hasPermission(
        check.module, check.resource, check.action,
        check.scopeEntityId, check.scopeSiteId
      );
    }

    return this.store.hasModuleAccess(check.module);
  }
}
```

### 13.6. Menu Dinâmico

```typescript
// libs/shared/features/shell/ui/navigation/navigation.component.ts

interface NavModule {
  code: string;
  label: string;
  icon: string;
  route: string;
  children: NavItem[];
}

interface NavItem {
  label: string;
  route: string;
  permission?: PermissionCheck;
}

@Component({
  selector: 'app-navigation',
  standalone: true,
  imports: [RouterLink, HasPermissionDirective],
  template: `
    @for (mod of visibleModules(); track mod.code) {
      <div class="nav-module">
        <span class="material-icons">{{ mod.icon }}</span>
        <h3>{{ mod.label }}</h3>
        
        @for (item of mod.children; track item.route) {
          @if (item.permission) {
            <a *hasPermission="item.permission" [routerLink]="item.route">
              {{ item.label }}
            </a>
          } @else {
            <a [routerLink]="item.route">{{ item.label }}</a>
          }
        }
      </div>
    }
  `,
})
export class NavigationComponent {
  private permissionStore = inject(PermissionStore);

  private allModules: NavModule[] = [
    {
      code: 'sigalm',
      label: 'Almoxarifado',
      icon: 'warehouse',
      route: '/almoxarifado',
      children: [
        {
          label: 'Minhas Requisições',
          route: '/almoxarifado/minhas-requisicoes',
          permission: { module: 'sigalm', resource: '/requisicoes/pessoal', action: 'read' },
        },
        {
          label: 'Requisições da Unidade',
          route: '/almoxarifado/requisicoes',
          permission: { module: 'sigalm', resource: '/requisicoes', action: 'read' },
        },
        {
          label: 'Estoque',
          route: '/almoxarifado/estoque',
          permission: { module: 'sigalm', resource: '/estoque', action: 'read' },
        },
        {
          label: 'Entrada por NF',
          route: '/almoxarifado/entrada-nf',
          permission: { module: 'sigalm', resource: '/estoque/entrada-nf', action: 'create' },
        },
        {
          label: 'Transferências',
          route: '/almoxarifado/transferencias',
          permission: { module: 'sigalm', resource: '/transferencias', action: 'read' },
        },
        {
          label: 'Relatórios',
          route: '/almoxarifado/relatorios',
          permission: { module: 'sigalm', resource: '/relatorios', action: 'read' },
        },
        {
          label: 'Cadastro de Materiais',
          route: '/almoxarifado/materiais',
          permission: { module: 'sigalm', profile: 'diretor_material' },
        },
      ],
    },
    {
      code: 'sigfrota',
      label: 'Transporte',
      icon: 'directions_car',
      route: '/transporte',
      children: [
        {
          label: 'Reservar Veículo',
          route: '/transporte/reservas',
          permission: { module: 'sigfrota', resource: '/reservas', action: 'create' },
        },
        {
          label: 'Veículos',
          route: '/transporte/veiculos',
          permission: { module: 'sigfrota', resource: '/veiculos', action: 'read' },
        },
        {
          label: 'Abastecimento',
          route: '/transporte/abastecimento',
          permission: { module: 'sigfrota', resource: '/abastecimento', action: 'read' },
        },
        {
          label: 'Manutenção',
          route: '/transporte/manutencao',
          permission: { module: 'sigfrota', resource: '/manutencao', action: 'read' },
        },
        {
          label: 'Licenciamento / Multas',
          route: '/transporte/licenciamento',
          permission: { module: 'sigfrota', resource: '/licenciamento', action: 'read' },
        },
      ],
    },
  ];

  visibleModules = computed(() =>
    this.allModules.filter(mod => this.permissionStore.hasModuleAccess(mod.code))
  );
}
```

### 13.7. Rotas com Resolver + Guard

```typescript
// apps/main-app/src/app/app.routes.ts

export const appRoutes: Routes = [
  {
    path: 'almoxarifado',
    resolve: { permissions: permissionsResolver },   // ← carrega primeiro
    canActivate: [modulePermissionGuard],             // ← consulta depois
    data: { module: 'sigalm' },
    loadChildren: () => import('./modules/sigalm/sigalm.routes').then(m => m.SIGALM_ROUTES),
  },
  {
    path: 'transporte',
    resolve: { permissions: permissionsResolver },
    canActivate: [modulePermissionGuard],
    data: { module: 'sigfrota' },
    loadChildren: () => import('./modules/sigfrota/sigfrota.routes').then(m => m.SIGFROTA_ROUTES),
  },
  {
    path: 'admin',
    resolve: { permissions: permissionsResolver },
    canActivate: [modulePermissionGuard],
    data: { module: 'admin' },
    loadChildren: () => import('./modules/admin/admin.routes').then(m => m.ADMIN_ROUTES),
  },
  {
    path: 'access-denied',
    loadComponent: () => import('./shared/pages/access-denied.component')
      .then(c => c.AccessDeniedComponent),
  },
  {
    path: 'service-unavailable',
    loadComponent: () => import('./shared/pages/service-unavailable.component')
      .then(c => c.ServiceUnavailableComponent),
  },
];

// apps/main-app/src/app/modules/sigalm/sigalm.routes.ts
export const SIGALM_ROUTES: Routes = [
  {
    path: '',
    component: SigalmLayoutComponent,
    children: [
      {
        path: 'minhas-requisicoes',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', permission: '/requisicoes/pessoal', action: 'read' },
        loadComponent: () => import('./features/requisicoes/minhas-requisicoes.component'),
      },
      {
        path: 'requisicoes',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', permission: '/requisicoes', action: 'read' },
        loadComponent: () => import('./features/requisicoes/requisicoes-list.component'),
      },
      {
        path: 'estoque',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', permission: '/estoque', action: 'read' },
        loadComponent: () => import('./features/estoque/estoque-list.component'),
      },
      {
        path: 'entrada-nf',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', permission: '/estoque/entrada-nf', action: 'create' },
        loadComponent: () => import('./features/entrada-nf/entrada-nf.component'),
      },
      {
        path: 'transferencias',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', permission: '/transferencias', action: 'read' },
        loadComponent: () => import('./features/transferencias/transferencias.component'),
      },
      {
        path: 'relatorios',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', permission: '/relatorios', action: 'read' },
        loadComponent: () => import('./features/relatorios/relatorios.component'),
      },
      {
        path: 'materiais',
        canActivate: [modulePermissionGuard],
        data: { module: 'sigalm', profile: 'diretor_material' },
        loadComponent: () => import('./features/materiais/materiais-cadastro.component'),
      },
    ],
  },
];
```

---

## 14. Visibilidade de Dados (Data-Level Security)

A autorização Casbin decide "pode acessar /requisicoes?". O filtro de dados decide "**quais** requisições aparecem?". São camadas separadas.

### 13.1. Regra de Visibilidade

- Requisitor vê **suas próprias** requisições
- Gestor/Coordenador vê requisições da **sua unidade e sub-unidades**
- Auditor vê **todas** no seu escopo
- Diretor de Material vê **saldos** de qualquer almoxarifado

### 13.2. Implementação

```rust
pub async fn list_requisitions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<RequisitionFilters>,
) -> Result<Json<Vec<Requisition>>, AppError> {
    // O middleware já verificou: pode acessar /requisicoes (read)

    // Determinar escopo de visibilidade baseado no perfil mais alto
    let visible_department_ids = get_visible_departments(
        &state.pool,
        "sigalm",
        current_user.department_id,
    ).await?;

    let requisitions = sqlx::query_as!(
        Requisition,
        r#"
        SELECT r.* FROM requisitions r
        WHERE 
            r.requester_id = $1                    -- próprias
            OR r.department_id = ANY($2)           -- da unidade e sub-unidades
        ORDER BY r.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        current_user.id,
        &visible_department_ids,
        params.limit.unwrap_or(50),
        params.offset.unwrap_or(0),
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(requisitions))
}
```

### 13.3. Restrição de Campus

```rust
/// Verificar se destino é do mesmo campus (para requisições)
async fn verify_same_campus(
    pool: &PgPool,
    user_site_id: Uuid,
    target_department_id: Uuid,
) -> Result<bool, AppError> {
    let same = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM departments d
            JOIN department_sites ds ON d.id = ds.department_id
            WHERE d.id = $2 AND ds.site_id = $1
        )
        "#,
        user_site_id,
        target_department_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(same.unwrap_or(false))
}
```

---

## 15. Regras Específicas de Negócio

### 14.1. Almoxarifados Central vs Setorial

```rust
// No handler de entrada de NF
pub async fn create_entrada_nf(
    State(state): State<AppState>,
    Path(almox_id): Path<Uuid>,
    /* ... */
) -> Result<(), AppError> {
    let warehouse = get_warehouse(&state.pool, almox_id).await?;
    
    if warehouse.warehouse_type != "central" {
        return Err(AppError::BadRequest(
            "Somente almoxarifados centrais podem receber notas fiscais. \
             Almoxarifados setoriais devem solicitar transferência."
                .into()
        ));
    }
    // ...
}
```

### 14.2. Transferências entre Campi

Transferências entre almoxarifados de campi diferentes exigem que o usuário tenha permissão em **ambos** ou tenha perfil global:

```rust
pub async fn create_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateTransferRequest>,
) -> Result<(), AppError> {
    let origin = get_warehouse(&state.pool, payload.origin_warehouse_id).await?;
    let destination = get_warehouse(&state.pool, payload.destination_warehouse_id).await?;

    // Verificar permissão no almoxarifado de origem
    let can_origin = state.auth_service.check_permission(
        current_user.id, "sigalm",
        Some(origin.id), Some("almoxarifado"),
        "/transferencias", "create",
    ).await;

    if !can_origin {
        return Err(AppError::Forbidden("Sem permissão no almoxarifado de origem".into()));
    }

    // Se inter-campus, verificar se tem perfil global ou permissão no destino
    if origin.site_id != destination.site_id {
        let can_destination = state.auth_service.check_permission(
            current_user.id, "sigalm",
            Some(destination.id), Some("almoxarifado"),
            "/transferencias", "approve",
        ).await;

        let is_global = state.auth_service.check_permission(
            current_user.id, "sigalm", None, None,
            "/transferencias", "create",
        ).await;

        if !can_destination && !is_global {
            return Err(AppError::Forbidden(
                "Transferência entre campi requer permissão em ambos almoxarifados ou perfil global"
                    .into()
            ));
        }
    }

    // Criar transferência...
    Ok(())
}
```

---

## 16. App de Administração de Permissões

### 15.1. Quem Pode Gerenciar Quem?

| Papel | Pode gerenciar | Restrição de escopo |
|-------|---------------|---------------------|
| Admin Global | Todos os perfis | Todos os módulos e escopos |
| Coordenador do Módulo (global) | Perfis do seu módulo | Todos os escopos do módulo |
| Coordenador do Módulo (campus) | Perfis do seu módulo | Apenas no seu campus |
| Coordenador do Módulo (unit) | Perfis do seu módulo | Apenas na sua unidade (ex: almoxarifado) |

### 15.2. Restrição de Nível

Um coordenador **não pode** atribuir perfis de nível igual ou superior ao seu. Isso impede escalada de privilégios:

```rust
async fn verify_can_assign(
    state: &AppState,
    current_user: &CurrentUser,
    target_user_id: Uuid,
    target_profile: &ModuleProfile,
    scope: &AssignmentScope,
) -> Result<(), AppError> {
    // 1. Admin global pode tudo
    if current_user.is_admin {
        return Ok(());
    }

    // 2. Verificar se tem permissão de admin no escopo
    let can_admin = state.auth_service.check_permission(
        current_user.id,
        &target_profile.module_code,
        scope.entity_id,
        scope.entity_type.as_deref(),
        "/admin/assignments",
        "create",
    ).await;

    if !can_admin {
        return Err(AppError::Forbidden(
            "Sem permissão para gerenciar atribuições neste escopo".into()
        ));
    }

    // 3. Verificar nível COM ESCOPO: o nível do usuário depende de onde ele está atuando.
    //    João pode ser Coordenador (30) no Campus A e Operador (10) no Campus B.
    //    Ele só pode atribuir Gestores (20) no Campus A, não no B.
    let my_max_level = get_user_max_profile_level_in_scope(
        &state.pool,
        current_user.id,
        &target_profile.module_code,
        scope.site_id,
        scope.entity_id,
        scope.entity_type.as_deref(),
    ).await?;

    if target_profile.level >= my_max_level {
        return Err(AppError::Forbidden(format!(
            "Não é possível atribuir perfil de nível {} neste escopo. \
             Seu nível máximo aqui é {}.",
            target_profile.level, my_max_level
        )));
    }

    // 4. Verificar incompatibilidades
    verify_no_incompatibility(
        &state.pool,
        target_user_id,
        target_profile.id,
        scope.site_id,
        scope.entity_id,
    ).await?;

    Ok(())
}

/// Retorna o nível máximo do usuário para um módulo DENTRO de um escopo específico.
/// Considera tanto perfis do escopo exato quanto perfis globais (que valem em qualquer escopo).
async fn get_user_max_profile_level_in_scope(
    pool: &PgPool,
    user_id: Uuid,
    module_code: &str,
    scope_site_id: Option<Uuid>,
    scope_entity_id: Option<Uuid>,
    scope_entity_type: Option<&str>,
) -> Result<i32, AppError> {
    let max_level = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(MAX(mp.level), 0) as "level!"
        FROM user_profile_assignments upa
        JOIN module_profiles mp ON upa.profile_id = mp.id
        JOIN modules m ON mp.module_id = m.id
        WHERE upa.user_id = $1
          AND m.code = $2
          AND upa.is_active = TRUE
          AND (upa.expires_at IS NULL OR upa.expires_at > NOW())
          AND (
              -- Perfil global: vale em qualquer escopo
              mp.scope_type = 'global'
              -- Perfil de campus: vale se o campus bate
              OR (mp.scope_type = 'campus' AND upa.scope_site_id = $3)
              -- Perfil de unidade: vale se a entidade bate
              OR (mp.scope_type = 'unit' 
                  AND upa.scope_entity_id = $4 
                  AND upa.scope_entity_type = $5)
          )
        "#,
        user_id,
        module_code,
        scope_site_id,
        scope_entity_id,
        scope_entity_type,
    )
    .fetch_one(pool)
    .await?;

    Ok(max_level)
}
```

---

## 17. Auditoria de Permissões e Segurança

### 17.1. Log de Auditoria

```sql
CREATE TABLE permission_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(20) NOT NULL,      -- 'assign', 'revoke', 'block', 'unblock', 'expire'
    actor_id UUID NOT NULL,           -- quem fez a ação
    target_user_id UUID NOT NULL,     -- quem foi afetado
    
    -- Detalhes
    profile_id UUID REFERENCES module_profiles(id),
    module_code VARCHAR(20),
    scope_site_id UUID,
    scope_entity_id UUID,
    scope_entity_type VARCHAR(50),
    
    -- Justificativa
    process_reference VARCHAR(100),   -- nº do processo SEI
    
    -- Contexto completo (para auditoria histórica)
    details JSONB NOT NULL DEFAULT '{}',
    
    -- Metadados
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_target ON permission_audit_log(target_user_id, created_at DESC);
CREATE INDEX idx_audit_actor ON permission_audit_log(actor_id, created_at DESC);
CREATE INDEX idx_audit_module ON permission_audit_log(module_code, created_at DESC);
```

### 17.2. Detecção de Tentativas de Escalada de Privilégios

Quando o Casbin retorna `deny`, o sistema deve registrar e monitorar. Tentativas repetidas de acesso negado ao mesmo recurso indicam potencial escalada de privilégios:

```sql
CREATE TABLE access_denial_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    module_code VARCHAR(20) NOT NULL,
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    scope_domain VARCHAR(200),         -- domínio Casbin completo
    
    -- Contexto do request
    ip_address INET,
    user_agent TEXT,
    request_path VARCHAR(500),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_denial_user_time ON access_denial_log(user_id, created_at DESC);
CREATE INDEX idx_denial_resource ON access_denial_log(module_code, resource, created_at DESC);

-- View para detectar padrões suspeitos (threshold 10+ e filtra retries)
CREATE VIEW suspicious_access_patterns AS
SELECT 
    user_id,
    module_code,
    resource,
    action,
    COUNT(*) as denial_count,
    MIN(created_at) as first_attempt,
    MAX(created_at) as last_attempt,
    COUNT(DISTINCT ip_address) as distinct_ips,
    COUNT(DISTINCT request_path) as distinct_paths
FROM access_denial_log
WHERE created_at > NOW() - INTERVAL '1 hour'
GROUP BY user_id, module_code, resource, action
HAVING COUNT(*) >= 10                        -- threshold alto para evitar falsos positivos
   AND COUNT(DISTINCT request_path) >= 3;    -- não contar retries no mesmo endpoint
```

```rust
/// Registra deny e verifica padrão suspeito
pub async fn log_access_denial(
    pool: &PgPool,
    user_id: Uuid,
    module: &str,
    resource: &str,
    action: &str,
    domain: &str,
    ip: Option<IpAddr>,
    user_agent: Option<&str>,
    request_path: &str,
) -> Result<(), AppError> {
    // 1. Registrar negação
    sqlx::query!(
        r#"
        INSERT INTO access_denial_log 
            (user_id, module_code, resource, action, scope_domain, 
             ip_address, user_agent, request_path)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        user_id, module, resource, action, domain,
        ip.map(|ip| ip.to_string()).as_deref(),
        user_agent, request_path,
    )
    .execute(pool)
    .await?;

    // 2. Verificar padrão suspeito
    let denial_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM access_denial_log
        WHERE user_id = $1 
          AND module_code = $2
          AND resource = $3
          AND created_at > NOW() - INTERVAL '1 hour'
        "#,
        user_id, module, resource,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    if denial_count >= 10 {
        // Disparar alerta
        tracing::warn!(
            "⚠️ ALERTA DE SEGURANÇA: Usuário {} tentou acessar {}:{} {} vezes na última hora (negado). \
             Possível tentativa de escalada de privilégios.",
            user_id, module, resource, denial_count
        );

        // Registrar alerta na auditoria
        sqlx::query!(
            r#"
            INSERT INTO permission_audit_log 
                (action, actor_id, target_user_id, module_code, details)
            VALUES ('security_alert', $1, $1, $2, $3)
            "#,
            user_id,
            module,
            serde_json::json!({
                "type": "repeated_denial",
                "resource": resource,
                "action": action,
                "denial_count": denial_count,
                "window": "1h",
            }),
        )
        .execute(pool)
        .await?;

        // Futuramente: enviar notificação por email ao admin de segurança
        // notify_security_admin(user_id, module, resource, denial_count).await;
    }

    Ok(())
}
```

Integração com o middleware de autorização:

```rust
pub async fn authorization_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let current_user = request.extensions().get::<CurrentUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?.clone();

    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let route_info = parse_route(&path).map_err(|_| StatusCode::BAD_REQUEST)?;
    let action = method_to_action(method.as_str());

    let domain = build_domain(
        &route_info.module, route_info.scope_site_id,
        route_info.scope_id, route_info.scope_type.as_deref(),
    );

    let allowed = state.auth_service.check_permission(
        current_user.id, &route_info.module,
        route_info.scope_id, route_info.scope_type.as_deref(),
        &route_info.resource, &action,
    ).await;

    if !allowed {
        // Registrar deny com detecção de padrão
        let ip = request.extensions().get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip());
        let user_agent = request.headers().get("user-agent")
            .and_then(|v| v.to_str().ok());

        let _ = log_access_denial(
            &state.pool, current_user.id,
            &route_info.module, &route_info.resource, &action,
            &domain, ip, user_agent, &path,
        ).await;

        return Err(StatusCode::FORBIDDEN);
    }

    // ... continua com X-Permissions-Stale check
    let mut response = next.run(request).await;
    
    if let Ok(stale) = check_permissions_stale(
        &state.pool, current_user.id, current_user.permissions_version,
    ).await {
        if stale {
            response.headers_mut().insert(
                "X-Permissions-Stale", "true".parse().unwrap(),
            );
        }
    }

    Ok(response)
}
```

### 17.3. Limpeza Periódica do Log de Denials

```rust
// Cron job: diário
pub async fn cleanup_old_denial_logs(pool: &PgPool) -> Result<u64> {
    // Manter apenas 30 dias de logs de negação
    // (alertas de segurança ficam no permission_audit_log permanentemente)
    let result = sqlx::query!(
        "DELETE FROM access_denial_log WHERE created_at < NOW() - INTERVAL '30 days'"
    )
    .execute(pool)
    .await?;

    tracing::info!("Limpeza: {} registros de denial removidos", result.rows_affected());
    Ok(result.rows_affected())
}
```

---

## 18. Resumo da Arquitetura

```
┌──────────────────────────────────────────────────────────────┐
│                      FRONTEND (Angular)                      │
│                                                              │
│  ┌──────────────────┐  ┌────────────┐  ┌────────────────┐   │
│  │ PermissionStore  │  │ Guard      │  │ *hasPermission │   │
│  │ (signal + TTL)   │──│ (sync)     │  │ (directive)    │   │
│  │ + ETag refresh   │  └────────────┘  └────────────────┘   │
│  │ + computed()     │  ┌────────────┐  ┌────────────────┐   │
│  │   signals        │  │ Resolver   │  │ Stale          │   │
│  └────────┬─────────┘  │ (prefetch) │  │ Interceptor    │   │
│           │             └────────────┘  └────────────────┘   │
│           │ GET /api/auth/me/permissions (ETag)              │
│           │ ◄── X-Permissions-Stale header                   │
│           ▼                                                  │
├──────────────────────────────────────────────────────────────┤
│                      BACKEND (Rust/Axum)                     │
│                                                              │
│  ┌──────────┐  ┌───────────────┐  ┌──────────┐             │
│  │ Auth     │  │ Authorization │  │ Data     │             │
│  │ (JWT)    │──│ Middleware    │──│ Filter   │             │
│  │          │  │ + deny log    │  │ (SQL)    │             │
│  └──────────┘  └───────┬───────┘  └──────────┘             │
│                        │                                     │
│     ┌──────────────────┼──────────────────┐                  │
│     │ dyn AuthorizationService            │                  │
│     │ ┌─────────────────────────────────┐ │                  │
│     │ │ SingleEnforcerService (fase 1)  │ │                  │
│     │ │ Arc<RwLock<Enforcer>>           │ │                  │
│     │ └─────────────────────────────────┘ │                  │
│     │ + batch: get_user_effective_perms() │                  │
│     └──────────────────┬──────────────────┘                  │
│                        │                                     │
│     ┌──────────────────┼──────────────────┐                  │
│     │ validate_assignment_prerequisites() │                  │
│     │ • SIGALM: treinamento              │                  │
│     │ • SIGFROTA: CNH                    │                  │
│     └─────────────────────────────────────┘                  │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                        DATABASE                              │
│                                                              │
│  ┌──────────────┐  ┌──────────────────────────┐             │
│  │ casbin_rule   │  │ user_profile_assignments │             │
│  │ (gerada)      │  │ (imutável / soft delete) │             │
│  └──────────────┘  │ + assignment_events       │             │
│                     └──────────────────────────┘             │
│                                                              │
│  ┌──────────────┐  ┌──────────────────────────┐             │
│  │ modules      │  │ module_scope_assignments  │             │
│  │ profiles     │  │ (vinculação genérica)     │             │
│  │ permissions  │  └──────────────────────────┘             │
│  └──────────────┘                                            │
│                                                              │
│  ┌─────────────────────────┐  ┌──────────────────────────┐  │
│  │ user_permission_blocks  │  │ profile_incompatibilities │  │
│  │ (deny + SEI permanente) │  │ (segregação de funções)   │  │
│  └─────────────────────────┘  └──────────────────────────┘  │
│                                                              │
│  ┌─────────────────────────┐  ┌──────────────────────────┐  │
│  │ module_dept_hierarchy   │  │ permission_audit_log      │  │
│  └─────────────────────────┘  │ + access_denial_log       │  │
│                                │ + suspicious_patterns     │  │
│                                └──────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## 19. Checklist de Implementação

### Fase 1: Infraestrutura de Autorização (3-4 semanas)
- [ ] Migrar `rbac_model.conf` para RBAC com domínios (4 campos + eft)
- [ ] Criar tabelas: `modules`, `module_profiles`, `profile_permissions`
- [ ] Criar tabela: `user_profile_assignments` (com soft delete, `deactivated_by`, `process_reference`)
- [ ] Criar tabela: `assignment_events` (event sourcing light)
- [ ] Criar tabela: `user_permission_blocks` (SEI obrigatório só para permanentes)
- [ ] Criar tabela: `profile_incompatibilities`
- [ ] Criar tabela: `module_scope_assignments` (vinculação genérica)
- [ ] Criar tabela: `module_department_hierarchy` (hierarquia por módulo)
- [ ] Criar tabela: `permission_audit_log`
- [ ] Criar tabela: `access_denial_log` + view `suspicious_access_patterns` (threshold 10+)
- [ ] Adicionar `permissions_version` e `permissions_changed_at` ao `auth_users`
- [ ] Implementar trait `AuthorizationService`
- [ ] Implementar `SingleEnforcerService` (enforcer único atrás do trait)
- [ ] Implementar `get_user_effective_permissions()` (batch em vez de cache)
- [ ] Implementar `validate_assignment_prerequisites()` (match simples)
- [ ] Implementar middleware de autorização com deny logging
- [ ] Implementar endpoint `GET /api/auth/me/permissions` com ETag
- [ ] Implementar header `X-Permissions-Stale` no middleware
- [ ] Implementar verificação de incompatibilidade na atribuição
- [ ] Implementar soft delete (nunca DELETE físico) + `record_assignment_event()`
- [ ] Implementar cron job de expiração de atribuições temporárias
- [ ] Implementar cron job de limpeza de denial logs (30 dias)

### Fase 2: Frontend Base (2-3 semanas)
- [ ] Implementar `PermissionStore` com TTL e invalidação por ETag
- [ ] Implementar `permissionsStaleInterceptor`
- [ ] Implementar `permissionsResolver` (prefetch antes de render)
- [ ] Implementar `modulePermissionGuard` (síncrono — resolver garante dados)
- [ ] Implementar diretiva `*hasPermission`
- [ ] Padrão de computed signals para verificações frequentes
- [ ] Menu dinâmico baseado em permissões
- [ ] Tela de Access Denied

### Fase 3: Admin de Permissões (2-3 semanas)
- [ ] Tela de busca e listagem de usuários
- [ ] Tela de atribuição de perfis (com verificação de nível, incompatibilidade e validação)
- [ ] Tela de bloqueios (razão obrigatória + SEI obrigatório só para permanentes)
- [ ] Filtro por escopo (admin só vê o que pode gerenciar)
- [ ] Tela de auditoria de mudanças de permissão
- [ ] Tela de alertas de segurança (padrões suspeitos, threshold 10+)
- [ ] Tela de configuração de `module_scope_assignments`
- [ ] Tela de histórico de atribuições (soft deletes + eventos visíveis)

### Fase 4: Perfis dos Módulos (1-2 semanas por módulo)
- [ ] SIGALM: cadastrar perfis (requisitor, operador, gestor, coordenador, auditor, diretor material)
- [ ] SIGALM: cadastrar permissões de cada perfil
- [ ] SIGALM: cadastrar incompatibilidades (requisitor ≠ aprovador, operador ≠ auditor)
- [ ] SIGALM: validação de treinamento no match de `validate_assignment_prerequisites()`
- [ ] SIGALM: regras específicas (central vs setorial, mesmo campus)
- [ ] SIGFROTA: cadastrar perfis
- [ ] SIGFROTA: cadastrar permissões
- [ ] SIGFROTA: validação de CNH no match de `validate_assignment_prerequisites()`
- [ ] SIGFROTA: regras específicas
- [ ] SIGEP: cadastrar perfis
- [ ] SIGEP: cadastrar permissões

### Fase 5: Testes e Ajustes (1-2 semanas)
- [ ] Testes de integração: atribuição → validação → sync → enforce → frontend
- [ ] Testes de incompatibilidade
- [ ] Testes de bloqueio (deny wins)
- [ ] Testes de transferência entre campi
- [ ] Testes de perfil implícito
- [ ] Testes de expiração de atribuição temporária
- [ ] Testes de soft delete, assignment_events e auditoria
- [ ] Testes de detecção de escalada de privilégios (threshold 10+, filtro retries)
- [ ] Testes de batch `get_user_effective_permissions()` em handlers com loop
- [ ] Testes de resolver + guard síncrono (sem flicker)
- [ ] Load test com volume realista de policies
- [ ] Verificar que DELETE físico nunca ocorre em atribuições/bloqueios

### Marco Futuro: Quando Migrar para Multi-Enforcer
- [ ] Monitorar latência do `enforce()` com Prometheus
- [ ] Se p99 > 5ms ou contention no RwLock evidente → trocar `SingleEnforcerService` por `MultiEnforcerService`
- [ ] Se > 10 módulos → extrair `validate_assignment_prerequisites()` para trait `AssignmentValidator`
