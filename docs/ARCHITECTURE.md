# Arquitetura do Waterswamp 🏗️

Este documento descreve a arquitetura do sistema Waterswamp, incluindo decisões de design, padrões utilizados e fluxos principais.

## 📐 Visão Geral

O Waterswamp segue uma arquitetura em camadas baseada em **Clean Architecture** e **Domain-Driven Design (DDD)**, com separação clara de responsabilidades e inversão de dependências.

```
┌─────────────────────────────────────────────────────────────┐
│                        API Layer                             │
│  (Axum Handlers, Middleware, HTTP Contracts)                 │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────┴─────────────────────────────────────┐
│                   Application Layer                          │
│  (Use Cases, Services, Business Logic)                       │
│  • AuthService  • UserService  • MfaService                  │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────┴─────────────────────────────────────┐
│                     Domain Layer                             │
│  (Entities, Value Objects, Domain Rules)                     │
│  • User  • Email  • Username  • Ports (Traits)               │
└───────────────────────┬─────────────────────────────────────┘
                        │
┌───────────────────────┴─────────────────────────────────────┐
│                 Infrastructure Layer                         │
│  (Persistence, Email, JWT, Crypto)                           │
│  • PostgreSQL  • SMTP  • Argon2  • Casbin                    │
└─────────────────────────────────────────────────────────────┘
```

## 🎯 Princípios Arquiteturais

### 1. Separation of Concerns (SoC)

Cada camada tem responsabilidades bem definidas:

- **API**: Recebe requisições HTTP, valida entrada, serializa respostas
- **Application**: Orquestra casos de uso, aplica regras de negócio
- **Domain**: Define entidades e regras de domínio
- **Infrastructure**: Implementa detalhes técnicos (banco, email, etc)

### 2. Dependency Inversion (DIP)

As camadas dependem de abstrações (Traits), não de implementações concretas:

```rust
// Domain define a interface
pub trait UserRepositoryPort {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn create(&self, username: &str, email: &str, password_hash: &str) -> Result<User>;
}

// Application usa a abstração
pub struct AuthService {
    user_repo: Arc<dyn UserRepositoryPort>,  // Trait object
}

// Infrastructure implementa
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepositoryPort for UserRepository {
    // Implementação usando SQLx
}
```

### 3. Single Responsibility (SRP)

Cada módulo tem uma única razão para mudar:

```rust
// Cada serviço tem uma responsabilidade específica
AuthService     → Autenticação e tokens
UserService     → Gerenciamento de usuários
MfaService      → Multi-factor authentication
AuditService    → Logging de auditoria
```

## 📦 Estrutura de Crates

### Workspace Cargo

```toml
[workspace]
members = [
    "apps/api-server",
    "crates/domain",
    "crates/application",
    "crates/persistence",
    "crates/core-services",
    "crates/email-service",
]
```

### Dependências entre Crates

```
api-server
    ├─> application
    │       └─> domain
    ├─> persistence
    │       └─> domain
    ├─> core-services
    └─> email-service

domain: 0 dependências internas (núcleo puro)
```

## 🔄 Fluxos Principais

### 1. Fluxo de Registro (Register)

```
┌─────────┐      ┌─────────┐      ┌──────────────┐      ┌──────────┐
│ Cliente │─────>│ Handler │─────>│ AuthService  │─────>│   Repo   │
└─────────┘      └─────────┘      └──────────────┘      └──────────┘
     │                │                   │                    │
     │ POST /register │                   │                    │
     │────────────────>                   │                    │
     │                │ validate_password │                    │
     │                │──────────────────>                     │
     │                │                   │ hash_password      │
     │                │                   │──────────────────> │
     │                │                   │ create_user        │
     │                │                   │───────────────────>│
     │                │                   │<───────────────────│
     │                │                   │ generate_tokens    │
     │                │                   │──────────────────> │
     │                │                   │ send_email         │
     │                │                   │──────────────────> │
     │                │<──────────────────│                    │
     │<───────────────│ 201 Created       │                    │
     │  { user, tokens }                  │                    │
```

**Passos:**
1. Validação de entrada (Validator)
2. Validação de força de senha (zxcvbn + common passwords)
3. Hash da senha (Argon2id, ~200-300ms)
4. Criação do usuário no banco
5. Geração de tokens (JWT + refresh token)
6. Envio de email de verificação (async, fire-and-forget)
7. Retorno de user + tokens

---

### 2. Fluxo de Login (com MFA)

```
┌─────────┐      ┌─────────┐      ┌──────────────┐      ┌──────────┐
│ Cliente │─────>│ Handler │─────>│ AuthService  │─────>│   Repo   │
└─────────┘      └─────────┘      └──────────────┘      └──────────┘
     │                │                   │                    │
     │ POST /login    │                   │                    │
     │────────────────>                   │                    │
     │                │                   │ find_user          │
     │                │                   │───────────────────>│
     │                │                   │<───────────────────│
     │                │                   │ verify_password    │
     │                │                   │──────────────────> │
     │                │                   │ check_mfa_enabled  │
     │                │                   │───────────────────>│
     │                │ MFA Required?     │                    │
     │                │<──────────────────│                    │
     │<───────────────│ 200 + mfa_token   │                    │
     │                │                   │                    │
     │ POST /mfa/verify                   │                    │
     │────────────────>                   │                    │
     │                │                   │ verify_totp        │
     │                │                   │───────────────────>│
     │                │                   │ generate_tokens    │
     │                │                   │───────────────────>│
     │<───────────────│ 200 + tokens      │                    │
```

**Passos:**
1. Buscar usuário por username/email
2. Verificar senha (Argon2 verify, constant-time)
3. **Se MFA habilitado:**
   - Gerar token de desafio MFA (JWT, 5min)
   - Retornar `{ mfa_required: true, mfa_token }`
4. **Se MFA não habilitado:**
   - Gerar tokens (access + refresh)
   - Retornar tokens

---

### 3. Fluxo de Refresh Token (com Token Theft Detection)

```
┌─────────┐      ┌─────────┐      ┌──────────────┐      ┌──────────┐
│ Cliente │─────>│ Handler │─────>│ AuthService  │─────>│ AuthRepo │
└─────────┘      └─────────┘      └──────────────┘      └──────────┘
     │                │                   │                    │
     │ POST /refresh  │                   │                    │
     │────────────────>                   │                    │
     │                │                   │ find_token (hash)  │
     │                │                   │───────────────────>│
     │                │                   │<───────────────────│
     │                │                   │ check_revoked      │
     │                │                   │───────────────────>│
     │                │   TOKEN REUSED?   │                    │
     │                │<──────────────────│ THEFT DETECTED!    │
     │                │   invalidate_     │                    │
     │                │   family()        │                    │
     │                │                   │───────────────────>│
     │<───────────────│ 401 Unauthorized  │                    │
     │                │                   │                    │
     │         (ou se válido)             │                    │
     │                │                   │ rotate_token       │
     │                │                   │───────────────────>│
     │                │                   │ revoke_old         │
     │                │                   │───────────────────>│
     │                │                   │ generate_new       │
     │                │                   │───────────────────>│
     │<───────────────│ 200 + new_tokens  │                    │
```

**Token Family Pattern:**
- Cada refresh token tem um `family_id` e `parent_token_hash`
- Quando rotaciona: novo token aponta para o anterior
- Se token reutilizado: **toda a família é invalidada**
- Métrica: `token_theft_detected_total{detection_type="reuse"}`

---

### 4. Fluxo de Autorização (RBAC com Casbin)

```
┌─────────┐      ┌──────────────┐      ┌─────────┐      ┌───────┐
│ Cliente │─────>│ Middleware   │─────>│ Casbin  │─────>│ Cache │
└─────────┘      └──────────────┘      └─────────┘      └───────┘
     │                  │                    │               │
     │ GET /admin/...   │                    │               │
     │─────────────────>│ extract JWT        │               │
     │                  │──────────────────> │               │
     │                  │ check_permission   │               │
     │                  │───────────────────>│ cache_get     │
     │                  │                    │──────────────>│
     │                  │                    │<──────────────│
     │                  │                    │ (miss)        │
     │                  │                    │ enforce()     │
     │                  │                    │──────────────>│
     │                  │                    │ cache_set     │
     │                  │                    │──────────────>│
     │                  │<───────────────────│ allowed       │
     │                  │ next()             │               │
     │                  │───────────────────>│               │
     │<─────────────────│ 200 OK             │               │
```

**Modelo RBAC:**
```ini
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
```

**Cache:**
- Implementado com Moka (in-memory, LRU)
- TTL: 5 minutos
- Métrica: `policy_cache_hits_total{result="hit|miss"}`

---

## 🔒 Segurança

### Password Hashing (Argon2id)

```rust
const ARGON2_M_COST: u32 = 65536;  // 64 MiB memory
const ARGON2_T_COST: u32 = 3;      // 3 iterations
const ARGON2_P_COST: u32 = 4;      // 4 parallel threads

// Formato PHC string:
// $argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>
```

**Performance:**
- Hash: ~200-300ms
- Verify: ~200-300ms (constant-time)
- Métrica: `password_hash_duration_seconds{operation="hash|verify"}`

### JWT Structure

**Access Token (1h):**
```json
{
  "sub": "user_id",
  "username": "johndoe",
  "type": "access",
  "exp": 1234567890,
  "iat": 1234564290
}
```

**Refresh Token (7 dias):**
- UUID opaco armazenado como SHA-256 hash
- Familia-based rotation para detecção de roubo

### Rate Limiting

```rust
// Por categoria de endpoint
LOGIN: 5 requisições / 10 segundos
ADMIN: 10 requisições / 2 segundos
API: 50 requisições / 200 milissegundos
```

---

## 📊 Observabilidade

### Structured Logging

```rust
tracing::info!(
    event_type = "user_login",
    user_id = %user.id,
    ip_address = %client_ip,
    "Usuário fez login com sucesso"
);
```

**Formatos:**
- Desenvolvimento: Text (colorido, legível)
- Produção: JSON (para Datadog, ELK, Loki)

### Métricas Prometheus

Veja [METRICS_GUIDE.md](./METRICS_GUIDE.md) para detalhes completos.

**Categorias:**
- HTTP (requests, latency)
- Auth (login, token refresh, theft detection)
- Passwords (hash duration)
- Casbin (cache hits, enforcement time)
- MFA (operations)

---

## 🗄️ Banco de Dados

### Schema Principal (auth database)

```sql
-- Usuários
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Extensões de usuário (MFA, etc)
CREATE TABLE user_extensions (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    mfa_enabled BOOLEAN DEFAULT FALSE,
    totp_secret TEXT,
    backup_codes TEXT[],
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Refresh Tokens (família-based)
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    family_id UUID NOT NULL,
    parent_token_hash TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    INDEX idx_token_hash (token_hash),
    INDEX idx_family_id (family_id),
    INDEX idx_user_id (user_id)
);

-- Políticas Casbin
CREATE TABLE casbin_rule (
    id SERIAL PRIMARY KEY,
    ptype VARCHAR(100),
    v0 VARCHAR(100),
    v1 VARCHAR(100),
    v2 VARCHAR(100),
    v3 VARCHAR(100),
    v4 VARCHAR(100),
    v5 VARCHAR(100)
);
```

### Schema de Audit Logs (logs database)

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(255),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    timestamp TIMESTAMPTZ DEFAULT NOW(),

    INDEX idx_user_id (user_id),
    INDEX idx_action (action),
    INDEX idx_timestamp (timestamp)
);
```

---

## 🚀 Performance

### Otimizações

1. **Connection Pooling**: SQLx com pool de 20 conexões
2. **Casbin Cache**: Moka in-memory, 5min TTL
3. **Async I/O**: Tokio runtime, non-blocking
4. **Prepared Statements**: SQLx compile-time checked queries
5. **Password Hashing**: `spawn_blocking` para não bloquear runtime

### Benchmarks

| Operação | Latência (P50) | Latência (P95) |
|----------|---------------|----------------|
| Login (sem MFA) | 220ms | 280ms |
| Login (com MFA) | 15ms | 25ms |
| Token Refresh | 5ms | 12ms |
| RBAC Check (cache hit) | 0.5ms | 1ms |
| RBAC Check (cache miss) | 2ms | 5ms |

---

## 🔄 Deployment

### Containerização (Docker)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/api-server /usr/local/bin/
CMD ["api-server"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: waterswamp
spec:
  replicas: 3
  selector:
    matchLabels:
      app: waterswamp
  template:
    spec:
      containers:
      - name: api
        image: waterswamp:latest
        env:
        - name: WS_AUTH_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-secrets
              key: auth-url
        ports:
        - containerPort: 3000
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
```

---

## 📚 Decisões de Arquitetura (ADRs)

### ADR-001: Clean Architecture

**Contexto**: Necessidade de manter código testável e desacoplado.

**Decisão**: Adotar Clean Architecture com camadas bem definidas.

**Consequências**:
- ✅ Código testável (dependency injection)
- ✅ Flexibilidade para trocar infra (ex: Postgres → DynamoDB)
- ❌ Mais boilerplate (traits, implementações)

---

### ADR-002: Argon2id para Password Hashing

**Contexto**: Necessidade de proteger senhas contra ataques modernos.

**Decisão**: Usar Argon2id com parâmetros OWASP (64 MiB, 3 iter, 4 threads).

**Consequências**:
- ✅ Resistente a GPU/ASIC attacks
- ✅ Proteção contra side-channel attacks
- ❌ ~250ms por operação (trade-off segurança vs UX)

---

### ADR-003: Token Family Pattern

**Contexto**: Detecção de roubo de refresh tokens.

**Decisão**: Implementar refresh token rotation com família.

**Consequências**:
- ✅ Detecção automática de token reuse
- ✅ Invalidação proativa de tokens comprometidos
- ❌ Mais complexidade no código de refresh

---

## 🤝 Contribuindo

Ao contribuir, siga estes princípios:

1. **Camadas**: Respeite as fronteiras de camadas
2. **Dependency Rule**: Dependências sempre apontam para dentro
3. **Tests**: Adicione testes unitários e de integração
4. **Docs**: Atualize esta documentação se mudanças arquiteturais

---

## 📖 Referências

- [Clean Architecture (Uncle Bob)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design (Eric Evans)](https://www.domainlanguage.com/ddd/)
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Casbin RBAC Documentation](https://casbin.org/docs/rbac)
