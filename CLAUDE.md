# CLAUDE.md — Especificação Viva do Waterswamp

> **Este documento é a fonte de verdade para qualquer agente de IA ou desenvolvedor trabalhando neste repositório.**
> Atualize-o sempre que adicionar uma variável de ambiente, criar um novo crate, mudar um padrão arquitetural ou descobrir um novo "obstáculo".

---

## 1. Visão Geral da Arquitetura

Waterswamp é um sistema de **gestão de compras públicas e frotas** construído em Rust com arquitetura hexagonal (ports & adapters) em um workspace Cargo monorepo.

```
┌─────────────────────────────────────────────────────────────┐
│                        CLIENTES                             │
│              (Web UI / Mobile / API Consumers)              │
└───────────────────────────┬─────────────────────────────────┘
                            │ HTTP/REST
┌───────────────────────────▼─────────────────────────────────┐
│                    apps/api-server                          │
│   Axum · RBAC (Casbin) · JWT · Rate Limit · Audit Log      │
│   16 módulos de API · Middleware Chain · OpenAPI/Swagger    │
└───────┬───────────────────┬─────────────────────────────────┘
        │ usa               │ usa
┌───────▼───────┐   ┌───────▼───────────────────────────────┐
│ crates/       │   │ crates/application                    │
│ core-services │   │   19 services · 2 workers · metrics  │
│ JWT · AES-GCM │   │   External: SiorgClient · ComprasGov │
│ Argon2 · CORS │   └───────┬───────────────────────────────┘
└───────────────┘           │ usa (ports)
                    ┌───────▼───────────────────────────────┐
                    │ crates/domain                         │
                    │   20 models · 53 port traits          │
                    │   Value Objects · Errors · Pagination │
                    └───────┬───────────────────────────────┘
                            │ implementado por
                    ┌───────▼───────────────────────────────┐
                    │ crates/persistence                    │
                    │   20 repositórios SQLx · PgPool       │
                    │   migrations_main/ · migrations_logs/ │
                    └───────┬───────────────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              │                           │
      ┌───────▼──────┐           ┌────────▼──────┐
      │  PostgreSQL   │           │  PostgreSQL   │
      │  auth_db      │           │  logs_db      │
      │  (dados)      │           │  (auditoria)  │
      └───────────────┘           └───────────────┘

apps/siorg-worker (daemon independente)
  └── SiorgSyncWorker → SiorgClient → API gov.br → auth_db
```

**Princípio central:** toda dependência de I/O é definida como trait em `domain::ports` e implementada em `persistence`. Os services em `application` só conhecem os traits — nunca as implementações concretas.

---

## 2. Stack Tecnológico Completo

### Runtime & Web
| Componente | Crate | Versão |
|---|---|---|
| Async runtime | tokio | 1.48 (`full` + `signal`) |
| Web framework | axum | 0.8 |
| Middleware | tower / tower-http | 0.5 / 0.6 |
| Rate limiting | tower_governor + governor | 0.8 / 0.10 |
| Cookies | tower-cookies + cookie | 0.11 / 0.18 |

### Banco de Dados
| Componente | Crate | Versão |
|---|---|---|
| ORM/Query | sqlx | 0.8 (PostgreSQL, migrate, chrono, uuid) |
| Migrations | sqlx-cli | (install via cargo) |
| RBAC storage | sqlx-adapter | 1.8 |
| Cache policy | moka | 0.12 (future) |

### Segurança
| Componente | Crate | Versão |
|---|---|---|
| JWT | jsonwebtoken | 10 (aws_lc_rs) |
| RBAC | casbin + axum-casbin | 2.15 / 1.3 |
| Senha (hash) | argon2 | 0.5 |
| Força de senha | zxcvbn | 3.1 |
| MFA (TOTP) | totp-rs | 5.7 (qr, gen_secret) |
| Criptografia campo | aes-gcm | 0.10 (AES-256-GCM) |
| Blind index | hmac + sha2 | 0.12 / 0.10 |
| QR Code | qrcode | 0.14 |

### Serialização & Validação
| Componente | Crate | Versão |
|---|---|---|
| JSON | serde + serde_json | 1.0 |
| Validação | validator | 0.20 |
| Datas | chrono | 0.4 |
| UUID | uuid | 1.18 (v4) |
| Decimal financeiro | rust_decimal | 1.36 |
| Regex | regex | 1.12 |
| Base32/64 | base32 / base64 | 0.5 / 0.22 |
| Hex | hex | 0.4 |
| Rand | rand | 0.8 |

### Email
| Componente | Crate | Versão |
|---|---|---|
| SMTP client | lettre | 0.11 (tokio1, native-tls) |
| Templates | tera | 1.20 |
| HTML escape | htmlescape | 0.3 |

### API Docs & Observabilidade
| Componente | Crate | Versão |
|---|---|---|
| OpenAPI schema | utoipa | 5.3 |
| Swagger UI | utoipa-swagger-ui | 9 |
| RapiDoc | utoipa-rapidoc | 5 |
| ReDoc | utoipa-redoc | 5 |
| Métricas | prometheus | 0.13 |
| Tracing | tracing + tracing-subscriber | 0.1 / 0.3 |

### Testes
| Componente | Crate | Versão |
|---|---|---|
| HTTP server teste | axum-test | 18.1 |
| Mocking | mockall | 0.14 |
| .env carregamento | dotenvy | 0.15 |

### HTTP Client (externo)
| Componente | Crate | Versão |
|---|---|---|
| HTTP client | reqwest | 0.12 (json) |

### Utilitários
| Componente | Crate | Versão |
|---|---|---|
| Erros | anyhow + thiserror | 1.0 / 2 |
| Async traits | async-trait | 0.1 |
| Lazy init | lazy_static + once_cell | 1.5 / 1.19 |
| Futures | futures | 0.3 |

---

## 3. Variáveis de Ambiente

> **Convenção:** prefixo `WS_`, separador `_`, aninhamento com `__`.
> Carregadas via crate `config` com `Environment::with_prefix("WS")`.

### api-server

```bash
# ── Banco de Dados ─────────────────────────────────────────
WS_MAIN_DATABASE_URL=postgres://user:pass@localhost:5432/auth_db
WS_LOGS_DATABASE_URL=postgres://user:pass@localhost:5433/logs_db

# ── JWT ────────────────────────────────────────────────────
# Chaves EdDSA (Ed25519) em formato PEM — NÃO use HS256 em produção
WS_JWT_PRIVATE_KEY=<conteúdo do arquivo .pem>
WS_JWT_PUBLIC_KEY=<conteúdo do arquivo .pem>

# ── Criptografia de Campos ─────────────────────────────────
# Gerar: openssl rand -hex 32
# Criptografa: email, mfa_secret em repouso (AES-256-GCM)
WS_FIELD_ENCRYPTION_KEY=<64 caracteres hex>

# ── Rate Limiting ───────────────────────────────────────────
DISABLE_RATE_LIMIT=false   # true apenas em testes de integração

# ── Logging ─────────────────────────────────────────────────
RUST_LOG=info,waterswamp=debug
```

### siorg-worker

```bash
# ── Banco de Dados ─────────────────────────────────────────
DATABASE_URL=postgres://user:pass@localhost:5432/auth_db

# ── API Externa ────────────────────────────────────────────
SIORG_API_URL=https://estruturaorganizacional.dados.gov.br/doc
# SIORG_API_TOKEN — API pública, token não obrigatório

# ── Comportamento do Worker ────────────────────────────────
WORKER_BATCH_SIZE=10               # itens por ciclo de sync
WORKER_POLL_INTERVAL_SECS=5        # intervalo de polling da fila
WORKER_MAX_RETRIES=3               # tentativas máximas por item
WORKER_RETRY_BASE_DELAY_MS=1000    # atraso inicial do backoff
WORKER_RETRY_MAX_DELAY_MS=60000    # atraso máximo do backoff
WORKER_ENABLE_CLEANUP=true         # habilitar limpeza periódica
WORKER_CLEANUP_INTERVAL_SECS=3600  # intervalo de limpeza (1h)

# ── Logging ─────────────────────────────────────────────────
RUST_LOG=info,siorg_worker=debug,application::workers=debug
LOG_FORMAT=text   # ou 'json' para produção
```

### CI/CD (GitHub Actions Secrets)

```
GITHUB_TOKEN             – automático
KUBECONFIG_STAGING       – cat ~/.kube/config-staging | base64
KUBECONFIG_PRODUCTION    – cat ~/.kube/config-prod | base64
CODECOV_TOKEN            – opcional, em codecov.io
WS_FIELD_ENCRYPTION_KEY  – openssl rand -hex 32 (mesmo valor nos envs)
```

---

## 4. Estrutura de Diretórios

```
waterswamp/
├── CLAUDE.md                        # ← Este arquivo
├── Cargo.toml                       # Workspace (resolver = "2")
├── Cargo.lock
├── Dockerfile                       # Multi-stage: planner→cacher→builder→runtime
├── docker-compose.yml               # db_auth:5432 · db_logs:5433 · api:3000 · adminer:8080
├── rbac_model.conf                  # Modelo Casbin (RBAC com domínios)
├── k8s-deployment.yaml              # Template Kubernetes
├── flake.nix                        # Ambiente Nix
├── .envrc                           # Direnv: `use flake`
│
├── .github/
│   └── workflows/
│       └── ci-cd.yml                # 4 jobs: test → build → deploy-staging → deploy-production
│
├── apps/
│   ├── api-server/
│   │   ├── Cargo.toml
│   │   ├── tests/
│   │   │   ├── common.rs            # TestApp · spawn_app() · create_test_user()
│   │   │   ├── api_auth_tests.rs
│   │   │   ├── mfa_tests.rs
│   │   │   ├── security_tests.rs
│   │   │   ├── register_tests.rs
│   │   │   ├── password_reset_tests.rs
│   │   │   ├── email_verification_tests.rs
│   │   │   ├── organizational_tests.rs
│   │   │   ├── requisition_tests.rs
│   │   │   ├── supplier_tests.rs
│   │   │   ├── fleet_tests.rs
│   │   │   ├── driver_tests.rs
│   │   │   ├── fueling_tests.rs
│   │   │   ├── vehicle_fines_tests.rs (*)
│   │   │   ├── catmat_tests.rs
│   │   │   ├── catser_tests.rs
│   │   │   ├── geo_regions_tests.rs
│   │   │   ├── budget_classifications_tests.rs
│   │   │   ├── audit_log_tests.rs
│   │   │   ├── catalog_tests.rs
│   │   │   └── integration_tests.rs
│   │   └── src/
│   │       ├── main.rs              # TcpListener · serve()
│   │       ├── lib.rs               # build_application_state() · wiring completo
│   │       ├── routes/
│   │       │   ├── mod.rs           # build() — router raiz
│   │       │   ├── public.rs        # rotas sem autenticação
│   │       │   └── protected.rs     # rotas com JWT + Casbin
│   │       ├── api/
│   │       │   ├── admin/
│   │       │   │   ├── users/       # handlers · contracts
│   │       │   │   ├── policies/    # handlers · contracts
│   │       │   │   ├── audit/
│   │       │   │   └── requisitions/
│   │       │   ├── auth/            # login · register · refresh · logout · session
│   │       │   ├── budget_classifications/
│   │       │   ├── catalog/         # catmat · catser
│   │       │   ├── drivers/
│   │       │   ├── email_verification/
│   │       │   ├── fleet/
│   │       │   ├── fuelings/
│   │       │   ├── geo_regions/
│   │       │   ├── locations/       # sites · buildings · floors · spaces
│   │       │   ├── mfa/
│   │       │   ├── organizational/  # orgs · units · sync SIORG
│   │       │   ├── suppliers/
│   │       │   ├── users/           # perfil · senha
│   │       │   └── vehicle_fines/
│   │       ├── handlers/            # health · metrics · public
│   │       ├── middleware/          # auth · rbac · audit · rate_limit · security_headers
│   │       ├── extractors/          # AuthenticatedUser · PaginationParams
│   │       ├── infra/
│   │       │   ├── config.rs        # Config struct (serde + config crate)
│   │       │   ├── state.rs         # AppState (Clone)
│   │       │   ├── casbin_setup.rs  # setup_casbin() com retry
│   │       │   └── telemetry.rs     # tracing_subscriber setup
│   │       ├── openapi/             # utoipa schema registration
│   │       └── utils/
│   │
│   ├── siorg-worker/
│   │   ├── Cargo.toml
│   │   ├── .env.example
│   │   └── src/
│   │       └── main.rs              # WorkerConfig · run_forever()
│   │
│   └── web-ui/                      # Frontend (fora do workspace Rust)
│
└── crates/
    ├── domain/
    │   └── src/
    │       ├── lib.rs
    │       ├── errors.rs
    │       ├── pagination.rs
    │       ├── value_objects.rs     # Email · Username (TryFrom + validação)
    │       ├── models/              # 20 arquivos de entidades
    │       └── ports/               # 53 traits de repositório
    │
    ├── persistence/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── lib.rs
    │   │   ├── db_utils.rs
    │   │   └── repositories/        # 20 implementações SQLx
    │   ├── migrations_main/         # ~70 migrações SQL (auth_db)
    │   └── migrations_logs/         # 2 migrações SQL (logs_db)
    │
    ├── core-services/
    │   └── src/
    │       ├── lib.rs               # pub mod jwt; pub mod field_encryption; ...
    │       ├── jwt.rs               # JwtService (EdDSA)
    │       ├── field_encryption.rs  # encrypt_field · decrypt_field · blind_index · parse_key
    │       ├── security.rs          # headers · CORS
    │       └── session.rs
    │
    ├── application/
    │   └── src/
    │       ├── lib.rs
    │       ├── errors.rs
    │       ├── metrics.rs           # Prometheus counters/histograms
    │       ├── services/            # 19 implementações de use cases
    │       ├── external/
    │       │   ├── siorg_client.rs  # SSRF-protected · SIORG_ALLOWED_HOSTS
    │       │   ├── compras_gov_client.rs # SSRF-protected · COMPRAS_ALLOWED_HOSTS
    │       │   └── mod.rs
    │       └── workers/
    │           └── siorg_sync_worker.rs  # SiorgSyncWorkerCore · run_forever()
    │
    └── email-service/
        └── src/
            ├── lib.rs               # EmailService · EmailSender trait · MockEmailService
            ├── config.rs            # EmailConfig (SMTP)
            └── mock.rs              # feature-gated para testes
```

---

## 5. Serviços, Jobs e Models

### 5.1 Services (crates/application/src/services/)

| Service | Responsabilidade Principal |
|---|---|
| `AuditService` | Grava logs de auditoria assíncronos em `logs_db` |
| `AuthService` | Login, registro, refresh token, logout, rotação de família |
| `BudgetClassificationsService` | CRUD de classificações orçamentárias |
| `CatalogService` | Catálogo de materiais (CATMAT) e serviços (CATSER) |
| `DriverService` | CRUD de motoristas/operadores |
| `FuelingService` | Registros de abastecimento de veículos |
| `GeoRegionsService` | Países, estados, municípios |
| `MfaService` | Setup TOTP, verificação, backup codes, regeneração |
| `OrganizationService` | CRUD de organizações |
| `OrganizationalUnitCategoryService` | Categorias de unidades organizacionais |
| `OrganizationalUnitTypeService` | Tipos de unidades organizacionais |
| `OrganizationalUnitService` | Unidades, hierarquia, sync SIORG |
| `RequisitionService` | Requisições de compra (fluxo completo) |
| `SiorgSyncService` | Orquestra sync do SIORG para a base de dados |
| `SupplierService` | CRUD de fornecedores |
| `SystemSettingsService` | Configurações do sistema (chave-valor) |
| `UserService` | CRUD de usuários, roles, banimento, reset de senha |
| `VehicleService` | Gestão de frota (veículos, documentos, histórico) |
| `VehicleFineService` | Infrações de trânsito e histórico de status |

### 5.2 Background Jobs

| Job | Localização | Gatilho | Frequência |
|---|---|---|---|
| `SiorgSyncWorker` | `application::workers::siorg_sync_worker` | Daemon (`run_forever`) | `WORKER_POLL_INTERVAL_SECS` (padrão 5s) |
| Limpeza de sync antigos | Dentro do `SiorgSyncWorker` | Timer interno | `WORKER_CLEANUP_INTERVAL_SECS` (padrão 1h) |
| Envio de email | Fire-and-forget nos services | Eventos de negócio | Imediato (async) |
| Gravação de auditoria | `AuditService` middleware | Cada request autenticado | Imediato (async) |

### 5.3 Domain Models (crates/domain/src/models/)

| Arquivo | Entidades Principais |
|---|---|
| `audit.rs` | `AuditLog`, `AuditEntry` |
| `auth.rs` | `Claims`, `TokenType`, `RefreshToken` |
| `budget_classifications.rs` | `BudgetClassification` |
| `catalog.rs` | `ComprasGovGrupoMaterial`, `ComprasGovItemMaterial`, `ComprasGovResponse`, etc. |
| `driver.rs` | `Driver`, `DriverStatus` |
| `email.rs` | `EmailVerificationToken` |
| `facilities.rs` | `Site`, `Building`, `Floor`, `Space`, `BuildingType`, `SpaceType` |
| `fueling.rs` | `Fueling`, `FuelingRecord` |
| `geo_regions.rs` | `Country`, `State`, `City` |
| `mfa.rs` | `MfaSetupToken`, `BackupCode` |
| `organizational.rs` | `Organization`, `OrganizationalUnit`, `SiorgSyncQueue`, `SiorgHistory` |
| `policy.rs` | `Policy`, `CasbinRule` |
| `requisition.rs` | `Requisition`, `RequisitionItem`, `RequisitionStatus` |
| `session.rs` | `Session` |
| `supplier.rs` | `Supplier` |
| `user.rs` | `User`, `UserExtended`, `Role`, `UserStatus` |
| `vehicle.rs` | `Vehicle`, `VehicleDocument`, `VehicleStatusHistory`, `VehicleMake`, `VehicleModel` |
| `vehicle_fine.rs` | `VehicleFine`, `VehicleFineType`, `VehicleFineStatusHistory` |

### 5.4 Repository Ports (domain::ports) — 53 traits

Agrupados por domínio:

- **Auth/User:** `UserRepositoryPort`, `AuthRepositoryPort`, `MfaRepositoryPort`, `SessionRepositoryPort`
- **Geo:** `CountryRepositoryPort`, `StateRepositoryPort`, `CityRepositoryPort`
- **Org:** `OrganizationRepositoryPort`, `OrganizationalUnitRepositoryPort`, `OrganizationalUnitCategoryRepositoryPort`, `OrganizationalUnitTypeRepositoryPort`, `SystemSettingsRepositoryPort`, `SiorgSyncQueueRepositoryPort`, `SiorgHistoryRepositoryPort`
- **Catalog:** `BudgetClassificationRepositoryPort`, `UnitOfMeasureRepositoryPort`, `UnitConversionRepositoryPort`, `CatmatGroupRepositoryPort`, `CatmatClassRepositoryPort`, `CatmatPdmRepositoryPort`, `CatmatItemRepositoryPort`, `CatserSectionRepositoryPort`, `CatserDivisionRepositoryPort`, `CatserGroupRepositoryPort`, `CatserClassRepositoryPort`, `CatserItemRepositoryPort`
- **Facilities:** `SiteRepositoryPort`, `BuildingRepositoryPort`, `BuildingTypeRepositoryPort`, `FloorRepositoryPort`, `SpaceRepositoryPort`, `SpaceTypeRepositoryPort`
- **Procurement:** `SupplierRepositoryPort`, `RequisitionRepositoryPort`, `RequisitionItemRepositoryPort`
- **Fleet:** `DriverRepositoryPort`, `VehicleRepositoryPort`, `VehicleDocumentRepositoryPort`, `VehicleStatusHistoryRepositoryPort`, `VehicleCategoryRepositoryPort`, `VehicleMakeRepositoryPort`, `VehicleModelRepositoryPort`, `VehicleColorRepositoryPort`, `VehicleFuelTypeRepositoryPort`, `VehicleTransmissionTypeRepositoryPort`, `FuelingRepositoryPort`, `VehicleFineRepositoryPort`, `VehicleFineTypeRepositoryPort`, `VehicleFineStatusHistoryRepositoryPort`
- **External:** `EmailServicePort`

---

## 6. Os 12 Obstáculos Comuns (Common Hurdles)

### H-01: Ambiguidade de trait `new_from_slice` no HMAC

**Sintoma:** `error[E0034]: multiple applicable items in scope` em `HmacSha256::new_from_slice(key)`

**Causa:** Tanto `KeyInit` quanto `hmac::Mac` têm `new_from_slice`.

**Solução:**
```rust
// ❌ Ambíguo
let mut mac = HmacSha256::new_from_slice(key).unwrap();

// ✅ Correto
type HmacSha256 = Hmac<Sha256>;
let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(key).unwrap();
```

---

### H-02: `Email` value object rejeita ciphertext criptografado

**Sintoma:** `TryFrom<String>` falha ao tentar construir `Email` a partir de ciphertext base64url (não passa validação de formato de email).

**Causa:** `Email` valida formato RFC via regex antes de aceitar o valor.

**Solução:** Usar structs de raw row (`RawUserRow`) com `email: String` no `FromRow`, descriptografar e construir `Email` manualmente depois:

```rust
#[derive(sqlx::FromRow)]
struct RawUserRow {
    id: Uuid,
    email: String, // ciphertext — não tente construir Email aqui
    // ...
}

// Depois de SELECT:
let raw = sqlx::query_as::<_, RawUserRow>("SELECT ...").fetch_one(&self.pool).await?;
let plaintext_email = self.decrypt_email(&raw.email)?;
let user = UserDto { email: Email::try_from(plaintext_email)?, ... };
```

---

### H-03: Busca por email requer `email_index`, não `email`

**Sintoma:** `find_by_email`, `exists_by_email` retornam zero resultados mesmo com usuário existente.

**Causa:** O campo `email` contém ciphertext AES-GCM (não determinístico). Buscas por igualdade precisam do índice HMAC-SHA256.

**Solução:**
```rust
// ❌ Nunca busque por email direto
WHERE email = $1

// ✅ Sempre use email_index
let idx = blind_index(&email.to_lowercase(), &self.encryption_key);
WHERE email_index = $1  -- bind idx
```

---

### H-04: `create_test_user` precisa da chave de criptografia

**Sintoma:** Usuário inserido nos testes não é encontrado por `find_by_email` ou `find_for_login`.

**Causa:** O helper antigo não criptografava o email nem criava `email_index`.

**Solução:** Sempre passar `enc_key` e usar a assinatura atualizada:
```rust
// ✅
let (username, email, password) =
    common::create_test_user(&pool, &app.field_encryption_key).await?;
```

---

### H-05: `mfa_secret` em testes precisa ser criptografado antes do INSERT direto via SQL

**Sintoma:** `get_mfa_secret()` falha ou retorna erro de decriptação em testes que inserem segredo diretamente no banco.

**Causa:** `get_mfa_secret` tenta decriptografar o valor — se for plaintext, falha.

**Solução:**
```rust
// ❌
sqlx::query("UPDATE users SET mfa_secret = 'MYSECRET' WHERE id = $1").bind(id)...

// ✅
let encrypted = app.encrypt_field("MYSECRET");
sqlx::query("UPDATE users SET mfa_secret = $2 WHERE id = $1").bind(id).bind(&encrypted)...
```

---

### H-06: Clientes externos obrigatoriamente exigem validação SSRF

**Sintoma:** Compilação passa, mas qualquer novo cliente HTTP que aceite URL configurável viola a política de segurança.

**Regra:** Todo cliente que aceita URL dinâmica (ex: do banco de dados, de env var ou de request) **deve** chamar `ssrf_validate()` antes de usar a URL.

**Solução (padrão já existente em `siorg_client.rs` e `compras_gov_client.rs`):**
```rust
const MY_ALLOWED_HOSTS: &[&str] = &["api.exemplo.gov.br"];

impl MyClient {
    pub fn new(base_url: String) -> Result<Self> {
        ssrf_validate(&base_url, MY_ALLOWED_HOSTS)?;
        // ...
    }

    pub fn update_url(&mut self, url: String) -> Result<()> {
        ssrf_validate(&url, MY_ALLOWED_HOSTS)?;
        self.base_url = url;
        Ok(())
    }
}
```

---

### H-07: `update_urls()` retorna `Result<()>`, não `()`

**Sintoma:** `error[E0308]: mismatched types` ao chamar `client.update_urls(...)` sem tratar o `Result`.

**Causa:** A assinatura foi alterada de `-> ()` para `-> Result<()>` para forçar validação SSRF.

**Solução:** Sempre use `?` ou `.unwrap()` ao chamar `update_urls`.

---

### H-08: Setup do Casbin precisa de retry por concorrência

**Sintoma:** `setup_casbin` falha esporadicamente em testes paralelos com erro de lock de tabela.

**Causa:** Múltiplos testes tentam inicializar Casbin simultaneamente.

**Solução (já implementada em `common.rs`):**
```rust
let enforcer = match setup_casbin(pool.clone()).await {
    Ok(e) => e,
    Err(_) => setup_casbin(pool.clone()).await.expect("Casbin falhou no retry"),
};
```

---

### H-09: Dois bancos de dados — sempre use o pool correto

| Pool | Variável | Uso |
|---|---|---|
| `db_pool_auth` (main) | `WS_MAIN_DATABASE_URL` | Todos os dados de negócio |
| `db_pool_logs` | `WS_LOGS_DATABASE_URL` | Apenas `AuditService` |

**Sintoma:** `relation "audit_logs" does not exist` — você usou `pool_auth` no lugar de `pool_logs`.

---

### H-10: Migrações precisam de `.up.sql` e `.down.sql`

Toda migração em `migrations_main/` tem dois arquivos:
```
20260315000001_minha_migration.up.sql   # aplicar
20260315000001_minha_migration.down.sql # reverter
```

Nomenclatura: `YYYYMMDDHHmmSS_descricao_snake_case.{up,down}.sql`

---

### H-11: Prefixo `WS_` com `__` para aninhamento no config

**Sintoma:** Variável de ambiente não é carregada — campo fica `None` ou usa valor padrão.

**Causa:** O crate `config` usa `prefix("WS").separator("__")`. Variáveis simples usam `WS_NOME_CAMPO`. Campos aninhados (structs dentro de structs) precisam de `__`.

```bash
# ✅ Campo simples
WS_FIELD_ENCRYPTION_KEY=abc123...

# ✅ Campo aninhado (ex: Config { db: DbConfig { url: ... } })
WS_DB__URL=postgres://...
```

---

### H-12: `cargo check --workspace` antes de qualquer commit

O repositório usa features que só são verificadas com `--workspace`. Um `cargo check` em um único crate pode não revelar quebras em dependentes.

```bash
# Sempre antes de commitar:
cargo check --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

---

## 7. Os 14 Design Patterns do Projeto

### P-01: Arquitetura Hexagonal (Ports & Adapters)

Toda dependência de I/O é abstraída como trait em `domain::ports`. Os services em `application` só conhecem traits. A camada `persistence` fornece as implementações concretas. Isso permite trocar o banco ou mockar repositórios sem mudar a lógica de negócio.

### P-02: Repository Pattern

Cada aggregate root tem um port trait (ex: `UserRepositoryPort`) e uma implementação SQLx (ex: `UserRepository`). A implementação recebe apenas `PgPool` (e `enc_key` se necessário). Nunca exponha SQL fora de `persistence::repositories`.

### P-03: Raw Row Structs para Campos Criptografados

Quando um campo criptografado está em uma coluna que seria mapeada a um value object com validação (ex: `Email`), usar uma struct intermediária com `String` cru:

```rust
#[derive(sqlx::FromRow)]
struct RawUserRow { email: String, ... }  // ciphertext OK aqui

fn to_user_dto(raw: RawUserRow, key: &[u8; 32]) -> Result<UserDto> {
    let email = Email::try_from(decrypt_field(&raw.email, key)?)?;
    Ok(UserDto { email, ... })
}
```

### P-04: Blind Index para Campos Criptografados Pesquisáveis

AES-GCM é não-determinístico (nonce aleatório por chamada). Para buscas exatas (ex: email), armazene um HMAC-SHA256 do valor em lowercase em uma coluna separada (`email_index`). Busque sempre pelo índice, nunca pelo ciphertext.

### P-05: Fallback de Migração para Decriptação

Durante períodos de migração (ex: antes de re-encriptografar todos os emails), `decrypt_email()` tenta descriptografar e, se falhar, retorna o valor cru (plaintext legado). Remover este fallback após migração completa.

```rust
fn decrypt_email(&self, value: &str) -> String {
    decrypt_field(value, &self.encryption_key)
        .unwrap_or_else(|_| value.to_string()) // fallback legado
}
```

### P-06: SSRF Allow-list em Clientes Externos

Todo cliente HTTP que aceita URL configurável valida contra uma lista de hosts permitidos antes de usar a URL. Hosts privados/loopback são sempre bloqueados. `ssrf_validate()` é chamado no construtor e em qualquer método que altere a URL.

### P-07: Exponential Backoff com Jitter no Worker

O `SiorgSyncWorker` usa backoff exponencial para retries com delay máximo configurável:
`delay = min(base * 2^attempt, max_delay)`. Implementado sem crates externos — apenas `tokio::time::sleep`.

### P-08: Service Layer como Orquestrador

Services em `application` orquestram múltiplos repositórios e nunca acessam SQL diretamente. Cada método de service é um use case completo: validação → acesso a dados → lógica de negócio → efeitos colaterais (email, auditoria).

### P-09: Value Objects com Validação no Construtor

`Email`, `Username` e similares em `domain::value_objects` validam no `TryFrom<String>`. Erros de domínio são levantados na fronteira de entrada, não nos services.

### P-10: Middleware Chain Composável (Tower)

```
Request
  → SecurityHeaders
  → RateLimit
  → Auth (JWT decode)
  → RBAC (Casbin enforce)
  → Audit (log request)
  → Handler
```
Cada middleware é independente e testável isoladamente. Novos middlewares são inseridos na chain sem modificar handlers.

### P-11: Feature-Gated Mock para Testes

`email-service` exporta `MockEmailService` (implementação de `mockall`) apenas com feature `mock`. Em produção, o mock nunca é compilado. Em testes de integração, `MockEmailService` pode capturar e inspecionar emails enviados.

### P-12: AppState como Injeção de Dependência

`AppState` é um struct `Clone` que agrega todos os services, repositórios e configurações. Passado via `axum::extract::State<AppState>`. Evita global state e facilita testes com states customizados.

### P-13: Policy Cache com TTL (Moka)

Decisões de RBAC do Casbin são cacheadas em `moka::future::Cache<String, bool>` para evitar hits no banco a cada request. Cache tem TTL e é invalidado quando policies mudam.

### P-14: Multi-Stage Docker Build com cargo-chef

```dockerfile
# Stage 1: planner — gera recipe.json
# Stage 2: cacher  — compila dependências (camada Docker cacheada)
# Stage 3: builder — compila o binário
# Stage 4: runtime — debian:bookworm-slim, não-root, ~50MB
```
Sem esse padrão, qualquer mudança de código recompila todas as dependências (~10 min). Com ele, apenas o código do projeto é recompilado (~1-2 min).

---

## 8. Pipeline Semanal Completo

### CI/CD Automático (GitHub Actions — `.github/workflows/ci-cd.yml`)

| Gatilho | Jobs Executados |
|---|---|
| `push` para `main`, `develop`, `staging` | `test` → `build` → (deploy se `develop`) |
| `pull_request` para `main`, `develop` | `test` |
| Release publicada | `test` → `build` → `deploy-production` |

**Job `test` (Rust Test Suite):**
```
1. Setup PostgreSQL x2 (auth:5432, logs:5433) via services
2. Setup Rust toolchain (stable) + clippy + rustfmt
3. Cache dependências (Swatinem/rust-cache)
4. Instalar dependências de sistema (pkg-config, libssl-dev, cmake, clang)
5. Instalar sqlx-cli
6. Rodar migrações (auth_db + logs_db)
7. cargo fmt --all -- --check
8. cargo clippy --all-targets --all-features -- -D warnings
9. cargo test --lib (unit tests)
10. cargo test --test '*' -- --nocapture (integration tests) [WS_FIELD_ENCRYPTION_KEY obrigatório]
11. cargo audit (CVE scan)
12. cargo tarpaulin (cobertura — apenas push para main)
13. Upload para Codecov
```

**Job `build` (Docker Image):**
```
1. Setup QEMU (multi-platform)
2. Setup Docker Buildx
3. Login no GHCR
4. docker/metadata-action (tags: branch, semver, sha, latest)
5. Build + push linux/amd64 e linux/arm64
6. Cache GHA (gha mode=max)
```

**Job `deploy-staging` (automático em develop):**
```
1. kubectl set image deployment/waterswamp-api ...develop
2. kubectl rollout status --timeout=120s
3. kubectl get pods -n waterswamp-staging
```

**Job `deploy-production` (em release publicada):**
```
1. kubectl set image deployment/waterswamp-api ...<tag>
2. kubectl rollout status --timeout=180s
3. kubectl get pods + logs --tail=50
```

### Manutenção Manual Recomendada (Semanal)

| Dia | Tarefa | Comando |
|---|---|---|
| Segunda | Atualizar dependências e verificar CVEs | `cargo update && cargo audit` |
| Terça | Verificar migrações pendentes em staging | `sqlx migrate info --database-url $STAGING_URL` |
| Quarta | Revisar logs de auditoria (acessos anômalos) | Query em `logs_db.audit_logs` |
| Quinta | Verificar tamanho da fila SIORG e erros de sync | Query em `auth_db.siorg_sync_queue WHERE status = 'failed'` |
| Sexta | Verificar cobertura de testes e lint | `cargo tarpaulin --out Html && cargo clippy` |
| Sábado | Limpeza de imagens Docker antigas no GHCR | GitHub UI ou `gh api` |
| Domingo | Verificar backup automático dos bancos PostgreSQL | Confirmar snapshot no provedor de infra |

### Verificações Mensais

```bash
# Atualizar rbac_model.conf se novas regras de autorização foram adicionadas
# Rodar EXPLAIN ANALYZE nas queries críticas (find_by_email, list_users, etc.)
# Revisar tokens JWT expirados/não-revogados em auth.refresh_tokens
# Verificar se WS_FIELD_ENCRYPTION_KEY foi rotacionado (re-encriptografar se necessário)
# Limpar registros antigos em siorg_history (mais de 90 dias)
```

---

## 9. Checklist Pós-Implementação

Use este checklist após qualquer nova feature, bugfix ou refactoring antes de criar um PR.

### Código

- [ ] `cargo check --workspace` passa sem erros
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` sem warnings
- [ ] `cargo fmt --all` aplicado (sem diff)
- [ ] Novos handlers têm contratos de request/response em `contracts.rs`
- [ ] Lógica de negócio está no service, não no handler
- [ ] SQL está no repositório, não no service ou handler
- [ ] Novos campos sensíveis (PII, segredos) usam `encrypt_field` + `blind_index` se pesquisável

### Segurança

- [ ] Novos clientes HTTP externos têm `ssrf_validate()` no construtor e em `update_url()`
- [ ] Novos endpoints protegidos têm middleware de RBAC (Casbin)
- [ ] Inputs de usuário são validados com `validator` antes de processar
- [ ] Nenhum segredo, chave ou token está hard-coded ou em comentários
- [ ] Nenhuma PII é logada em nível `info` ou `debug`

### Banco de Dados

- [ ] Nova migração tem arquivo `.up.sql` e `.down.sql`
- [ ] Migração testada com `sqlx migrate run` e `sqlx migrate revert` localmente
- [ ] Novos índices adicionados para colunas usadas em `WHERE` de alta frequência
- [ ] `email_index` atualizado junto com `email` em todos os UPDATE relevantes

### Testes

- [ ] Testes de integração adicionados para o novo fluxo (happy path + erro)
- [ ] `create_test_user` chamado com `(pool, &app.field_encryption_key)`
- [ ] Inserts diretos de `mfa_secret` via SQL usam `app.encrypt_field()`
- [ ] `cleanup_test_users` cobre todos os dados criados no teste
- [ ] `cargo test --test '*'` passa localmente com banco de dados real

### Infraestrutura

- [ ] Novas variáveis de ambiente documentadas na seção 3 deste CLAUDE.md
- [ ] `WS_FIELD_ENCRYPTION_KEY` adicionado ao secret do GitHub Actions se necessário
- [ ] `AppState` atualizado se novo service ou repositório foi adicionado
- [ ] `build_application_state()` em `lib.rs` inicializa e injeta o novo componente
- [ ] `openapi/` atualizado com novos schemas se a API pública mudou

### Documentação

- [ ] Este `CLAUDE.md` atualizado se:
  - Nova variável de ambiente foi adicionada
  - Novo crate foi criado
  - Novo padrão arquitetural foi introduzido
  - Novo obstáculo comum foi descoberto e resolvido
- [ ] Comentários de código adicionados em lógica não-óbvia (criptografia, RBAC, sync)

---

> **Última atualização:** 2026-03-15
> **Versão Rust:** stable (testado com 1.85+)
> **Versão PostgreSQL:** 16 (mínimo: 14)
