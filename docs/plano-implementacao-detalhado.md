# Plano de Implementação - Sistema de Autenticação e Autorização

## Guia de Execução Detalhado

**Baseado em:** Guia Arquitetural Híbrido On-Premise  
**Stack:** Rust (Axum, SQLx, Argon2, Casbin-rs) + PostgreSQL + Angular 17+  
**Metodologia:** Iterativa e incremental, MVP-first

---

## 📋 Índice

1. [Visão Geral do Plano](#1-visão-geral-do-plano)
2. [Fase 0: Setup e Preparação](#fase-0-setup-e-preparação)
3. [Fase 1: MVP Core (Semanas 1-6)](#fase-1-mvp-core)
4. [Fase 2: Security Hardening (Semanas 7-10)](#fase-2-security-hardening)
5. [Fase 3: Advanced Features (Semanas 11-14)](#fase-3-advanced-features)
6. [Fase 4: Produção e Otimização (Semanas 15-16)](#fase-4-produção-e-otimização)
7. [Cronograma Visual](#cronograma-visual)
8. [Critérios de Qualidade](#critérios-de-qualidade)
9. [Riscos e Mitigações](#riscos-e-mitigações)

---

## 1. Visão Geral do Plano

### 1.1. Objetivos

**Objetivo Principal:**  
Implementar sistema de autenticação e autorização on-premise, seguro, escalável e manutenível.

**Objetivos Secundários:**
- Compliance com OWASP Top 10
- Zero Trust Architecture implementada
- Defense in Depth em todas as camadas
- Código testado e documentado
- Sistema auditável e monitorável

### 1.2. Premissas

- Time: 2-3 desenvolvedores (1 backend, 1 frontend, 1 full-stack)
- Dedicação: tempo integral
- Ambiente de desenvolvimento configurado
- Acesso a servidor on-premise para staging/produção
- Conhecimento intermediário de Rust e Angular

### 1.3. Entregas por Fase

| Fase | Duração | Entregas Principais |
|------|---------|---------------------|
| **Fase 0** | 3-5 dias | Ambiente configurado, estrutura de projetos |
| **Fase 1** | 6 semanas | MVP funcional com auth básica, CRUD, admin básico |
| **Fase 2** | 4 semanas | MFA, token rotation, rate limiting, security hardening |
| **Fase 3** | 4 semanas | Session management, analytics, advanced admin |
| **Fase 4** | 2 semanas | Deploy produção, monitoring, documentação final |

**Total:** 16 semanas (~4 meses)

### 1.4. Definição de Pronto (DoD)

Uma tarefa está "pronta" quando:
- [ ] Código implementado e revisado
- [ ] Testes unitários escritos e passando (cobertura > 80%)
- [ ] Testes de integração escritos e passando
- [ ] Documentação atualizada
- [ ] Code review aprovado
- [ ] Merge para branch principal

---

## Fase 0: Setup e Preparação

**Duração:** 3-5 dias  
**Objetivo:** Configurar ambiente de desenvolvimento e estrutura inicial dos projetos

### Sprint 0.1: Configuração de Infraestrutura (Dia 1)

#### Tarefa 0.1.1: Setup PostgreSQL
**Responsável:** Backend Dev  
**Prioridade:** P0 (Crítica)  
**Estimativa:** 2 horas

**Descrição:**
Instalar e configurar PostgreSQL para desenvolvimento

**Passos:**
1. Instalar PostgreSQL 14+ localmente
2. Criar usuário para aplicação
3. Criar database `auth_system_dev`
4. Configurar connection string
5. Habilitar extensões necessárias (uuid-ossp, pgcrypto)

**Critérios de Aceitação:**
- [ ] PostgreSQL rodando localmente
- [ ] Database criado e acessível
- [ ] Usuário criado com permissões apropriadas
- [ ] Extensões instaladas
- [ ] Connection string documentada

**Artefatos:**
- `docs/setup-database.md` com instruções

---

#### Tarefa 0.1.2: Setup Git e Estrutura de Repositórios
**Responsável:** Full-stack Dev  
**Prioridade:** P0  
**Estimativa:** 1 hora

**Descrição:**
Criar repositórios e estrutura de versionamento

**Passos:**
1. Criar repositório Git (pode ser mono-repo ou multi-repo)
2. Configurar `.gitignore` para Rust e Angular
3. Criar branches principais: `main`, `develop`, `staging`
4. Configurar branch protection rules
5. Setup conventional commits

**Estrutura Recomendada (mono-repo):**
```
auth-system/
├── .git/
├── .gitignore
├── README.md
├── backend/
├── frontend/
└── docs/
```

**Critérios de Aceitação:**
- [ ] Repositório criado e configurado
- [ ] Branches principais criadas
- [ ] `.gitignore` configurado
- [ ] README.md inicial
- [ ] Política de commits definida

---

#### Tarefa 0.1.3: Configurar Ambiente Docker (Opcional)
**Responsável:** Full-stack Dev  
**Prioridade:** P1 (Alta)  
**Estimativa:** 2 horas

**Descrição:**
Configurar Docker para facilitar setup de desenvolvimento

**Passos:**
1. Criar `docker-compose.yml` para PostgreSQL
2. Criar `docker-compose.yml` para ambiente completo
3. Documentar uso do Docker

**Critérios de Aceitação:**
- [ ] `docker-compose up` sobe PostgreSQL
- [ ] Volumes persistentes configurados
- [ ] Documentação de uso

---

### Sprint 0.2: Setup Backend Rust (Dia 2)

#### Tarefa 0.2.1: Criar Workspace Rust
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Descrição:**
Criar estrutura de workspace modular conforme arquitetura

**Estrutura:**
```
backend/
├── Cargo.toml (workspace)
├── .env.example
├── .cargo/
│   └── config.toml
├── crates/
│   ├── domain/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── user.rs
│   │       ├── role.rs
│   │       └── audit.rs
│   │
│   ├── auth-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── jwt.rs
│   │       ├── password.rs
│   │       └── tokens.rs
│   │
│   ├── authz-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── casbin.rs
│   │       └── policies.rs
│   │
│   ├── infra-database/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs
│   │       └── repositories/
│   │
│   └── infra-email/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── smtp.rs
│
├── apps/
│   └── api-server/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── config/
│           ├── modules/
│           │   ├── auth/
│           │   ├── users/
│           │   └── admin/
│           └── shared/
│
├── migrations/
└── config/
    └── casbin/
        ├── model.conf
        └── policy.csv
```

**Cargo.toml do Workspace:**
```toml
[workspace]
members = [
    "crates/domain",
    "crates/auth-core",
    "crates/authz-core",
    "crates/infra-database",
    "crates/infra-email",
    "apps/api-server",
]
resolver = "2"

[workspace.dependencies]
# Core
tokio = { version = "1", features = ["full"] }
axum = { version = "0.7", features = ["macros"] }
tower = { version = "0.4", features = ["limit", "timeout"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }

# Crypto
argon2 = "0.5"
jsonwebtoken = "9"
sha2 = "0.10"
rand = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Validation
validator = { version = "0.16", features = ["derive"] }

# Auth
casbin = { version = "2", features = ["runtime-tokio"] }

# Utils
uuid = { version = "1", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config
config = "0.13"
dotenvy = "0.15"

# Email
lettre = { version = "0.11", features = ["tokio1-native-tls"] }
```

**Critérios de Aceitação:**
- [ ] Workspace compila sem erros
- [ ] Todos os crates criados
- [ ] Dependências básicas adicionadas
- [ ] Estrutura de pastas completa
- [ ] README.md em cada crate explicando seu propósito

---

#### Tarefa 0.2.2: Configurar SQLx e Migrations
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Descrição:**
Configurar SQLx CLI e sistema de migrations

**Passos:**
1. Instalar SQLx CLI: `cargo install sqlx-cli`
2. Criar `.env` com DATABASE_URL
3. Inicializar SQLx: `sqlx database create`
4. Criar estrutura de migrations
5. Configurar offline mode

**Critérios de Aceitação:**
- [ ] SQLx CLI instalado
- [ ] Database criado via SQLx
- [ ] Pasta `migrations/` estruturada
- [ ] `.env.example` documentado
- [ ] `sqlx prepare` funcional para offline mode

---

#### Tarefa 0.2.3: Setup Logging e Error Handling
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 2 horas

**Descrição:**
Configurar sistema de logging estruturado e error handling

**Arquivos a Criar:**
- `apps/api-server/src/shared/errors.rs`
- `apps/api-server/src/shared/logging.rs`

**Estrutura de Error:**
```rust
// Usar thiserror para erros customizados
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    // etc...
}

// Implementar IntoResponse para Axum
```

**Critérios de Aceitação:**
- [ ] Error types definidos
- [ ] Logging configurado com tracing
- [ ] JSON structured logs
- [ ] Error responses padronizados

---

### Sprint 0.3: Setup Frontend Angular (Dia 3)

#### Tarefa 0.3.1: Criar Projeto Angular
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Descrição:**
Criar projeto Angular com SSR

**Passos:**
1. Instalar Angular CLI: `npm install -g @angular/cli@17`
2. Criar projeto: `ng new frontend --ssr --routing --style=scss`
3. Configurar standalone components
4. Configurar path aliases

**Estrutura:**
```
frontend/
├── src/
│   ├── app/
│   │   ├── core/
│   │   │   ├── services/
│   │   │   ├── guards/
│   │   │   ├── interceptors/
│   │   │   └── models/
│   │   │
│   │   ├── shared/
│   │   │   ├── components/
│   │   │   ├── directives/
│   │   │   └── pipes/
│   │   │
│   │   ├── features/
│   │   │   ├── auth/
│   │   │   ├── dashboard/
│   │   │   ├── profile/
│   │   │   └── admin/
│   │   │
│   │   └── layouts/
│   │
│   ├── assets/
│   ├── environments/
│   └── styles/
│
├── angular.json
├── tsconfig.json
├── package.json
└── server.ts
```

**Critérios de Aceitação:**
- [ ] Projeto criado e rodando
- [ ] SSR configurado e funcional
- [ ] Estrutura de pastas criada
- [ ] Path aliases configurados
- [ ] `ng serve` e `ng serve:ssr` funcionais

---

#### Tarefa 0.3.2: Instalar Dependências Frontend
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 1 hora

**Descrição:**
Instalar bibliotecas necessárias

**Dependências:**
```json
{
  "dependencies": {
    "@angular/material": "^17.0.0",
    "@ngrx/signals": "^17.0.0",
    "rxjs": "^7.8.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0"
  }
}
```

**Critérios de Aceitação:**
- [ ] Todas dependências instaladas
- [ ] Angular Material configurado
- [ ] Theme selecionado
- [ ] Sem vulnerabilidades críticas

---

#### Tarefa 0.3.3: Configurar Environment e API Base
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 1 hora

**Descrição:**
Configurar variáveis de ambiente e base da API

**Arquivos:**
```typescript
// src/environments/environment.ts
export const environment = {
  production: false,
  apiUrl: 'http://localhost:3000/api/v1',
  apiTimeout: 30000
};

// src/environments/environment.prod.ts
export const environment = {
  production: true,
  apiUrl: '/api/v1',
  apiTimeout: 30000
};
```

**Critérios de Aceitação:**
- [ ] Environments configurados
- [ ] API URL configurável
- [ ] Build produção usa environment correto

---

### Sprint 0.4: Documentação Inicial (Dia 4)

#### Tarefa 0.4.1: Documentação de Setup
**Responsável:** Full-stack Dev  
**Prioridade:** P1  
**Estimativa:** 2 horas

**Descrição:**
Documentar processo de setup completo

**Documentos a Criar:**
- `README.md` (raiz)
- `backend/README.md`
- `frontend/README.md`
- `docs/SETUP.md`
- `docs/CONTRIBUTING.md`

**Conteúdo Mínimo:**
- Pré-requisitos
- Instalação passo a passo
- Como rodar localmente
- Como rodar testes
- Estrutura do projeto
- Convenções de código

**Critérios de Aceitação:**
- [ ] Qualquer dev consegue seguir e configurar ambiente
- [ ] Todos os comandos documentados
- [ ] Screenshots/exemplos onde apropriado

---

#### Tarefa 0.4.2: Setup CI/CD Básico
**Responsável:** Full-stack Dev  
**Prioridade:** P2 (Média)  
**Estimativa:** 3 horas

**Descrição:**
Configurar pipeline básico de CI

**GitHub Actions / GitLab CI:**
- Lint backend (rustfmt, clippy)
- Testes backend
- Build backend
- Lint frontend (eslint)
- Testes frontend
- Build frontend

**Critérios de Aceitação:**
- [ ] Pipeline roda em cada PR
- [ ] Falha se lint ou testes falharem
- [ ] Caching configurado para velocidade

---

## Fase 1: MVP Core

**Duração:** 6 semanas  
**Objetivo:** Sistema básico funcional de autenticação e autorização

---

## Sprint 1.1: Database Schema e Models (Semana 1)

### Tarefa 1.1.1: Criar Migration - Tabela Users
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Descrição:**
Criar primeira migration para tabela de usuários

**Comando:**
```bash
sqlx migrate add create_users_table
```

**SQL:**
```sql
-- migrations/001_create_users_table.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    password_hash VARCHAR(255) NOT NULL,
    
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    phone VARCHAR(20),
    
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
    role VARCHAR(50) DEFAULT 'user' NOT NULL,
    
    last_login_at TIMESTAMPTZ,
    last_login_ip INET,
    failed_login_attempts INTEGER DEFAULT 0,
    locked_until TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    
    CONSTRAINT email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}$')
);

CREATE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_status ON users(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role ON users(role);

-- Trigger para updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

**Critérios de Aceitação:**
- [ ] Migration criada
- [ ] `sqlx migrate run` executa sem erros
- [ ] Tabela criada com constraints corretas
- [ ] Índices criados
- [ ] Trigger funcional

---

### Tarefa 1.1.2: Criar Migrations - Tabelas de Tokens
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Descrição:**
Criar migrations para refresh tokens, email verification e password reset

**Comandos:**
```bash
sqlx migrate add create_refresh_tokens_table
sqlx migrate add create_email_verification_tokens_table
sqlx migrate add create_password_reset_tokens_table
```

**Critérios de Aceitação:**
- [ ] 3 migrations criadas
- [ ] Todas executam sem erros
- [ ] Foreign keys corretas
- [ ] Índices apropriados

---

### Tarefa 1.1.3: Criar Migration - Tabela Audit Logs
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 1 hora

**SQL Exemplo:**
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    event_type VARCHAR(50) NOT NULL,
    event_category VARCHAR(50) NOT NULL,
    
    description TEXT,
    metadata JSONB,
    
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_metadata ON audit_logs USING gin(metadata);
```

**Critérios de Aceitação:**
- [ ] Migration criada e executada
- [ ] Índices GIN para JSONB

---

### Tarefa 1.1.4: Criar Migration - Casbin Rules
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 1 hora

**SQL:**
```sql
CREATE TABLE casbin_rule (
    id SERIAL PRIMARY KEY,
    ptype VARCHAR(100) NOT NULL,
    v0 VARCHAR(100),
    v1 VARCHAR(100),
    v2 VARCHAR(100),
    v3 VARCHAR(100),
    v4 VARCHAR(100),
    v5 VARCHAR(100),
    
    CONSTRAINT unique_key UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
);

-- Policies iniciais
INSERT INTO casbin_rule (ptype, v0, v1, v2) VALUES
    ('p', 'admin', 'users', 'read'),
    ('p', 'admin', 'users', 'write'),
    ('p', 'admin', 'users', 'delete'),
    ('p', 'admin', 'audit_logs', 'read'),
    ('p', 'user', 'profile', 'read'),
    ('p', 'user', 'profile', 'write');
```

**Critérios de Aceitação:**
- [ ] Tabela criada
- [ ] Policies básicas inseridas
- [ ] Constraint de unicidade funcional

---

### Tarefa 1.1.5: Criar Domain Models (Rust)
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 4 horas

**Descrição:**
Implementar structs de domínio no crate `domain`

**Arquivos:**
- `crates/domain/src/user.rs`
- `crates/domain/src/role.rs`
- `crates/domain/src/audit.rs`
- `crates/domain/src/token.rs`

**Estrutura Exemplo:**
```rust
// crates/domain/src/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    #[serde(skip_serializing)]
    pub password_hash: String,
    
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    
    pub status: UserStatus,
    pub role: String,
    
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
}

// DTOs
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            first_name: user.first_name,
            last_name: user.last_name,
            role: user.role,
            status: user.status,
            created_at: user.created_at,
        }
    }
}
```

**Critérios de Aceitação:**
- [ ] Todos models definidos
- [ ] DTOs para request/response
- [ ] Conversões (From/Into) implementadas
- [ ] Serialization/deserialization testada
- [ ] Documentação inline

---

## Sprint 1.2: Auth Core - Password e JWT (Semana 1-2)

### Tarefa 1.2.1: Implementar Password Service (Argon2)
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Arquivo:** `crates/auth-core/src/password.rs`

**Funcionalidades:**
- Hash de senha com Argon2id
- Verificação de senha
- Validação de força de senha
- Configuração de parâmetros

**Estrutura:**
```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Version, Params,
};

pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    pub fn new() -> Self {
        let params = Params::new(
            19456,  // m_cost: 19 MiB
            2,      // t_cost: 2 iterações
            1,      // p_cost: paralelismo
            None,
        ).expect("Invalid Argon2 parameters");
        
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            params,
        );
        
        Self { argon2 }
    }
    
    pub fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        // Implementação
    }
    
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        // Implementação
    }
}

pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    // Implementação
}
```

**Critérios de Aceitação:**
- [ ] Hash funcional
- [ ] Verificação funcional
- [ ] Parâmetros configuráveis
- [ ] Testes unitários (hash, verify, casos edge)
- [ ] Validação de força implementada
- [ ] Benchmark de performance (<500ms por hash)

**Testes Obrigatórios:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_hash_and_verify() { }
    
    #[test]
    fn test_verify_invalid_password() { }
    
    #[test]
    fn test_password_strength_validation() { }
    
    #[test]
    fn test_reject_common_passwords() { }
}
```

---

### Tarefa 1.2.2: Implementar JWT Service (EdDSA)
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 4 horas

**Arquivo:** `crates/auth-core/src/jwt.rs`

**Funcionalidades:**
- Geração de JWT com EdDSA
- Validação de JWT
- Claims customizados
- Key management

**Estrutura:**
```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user_id
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: i64,
}

impl JwtService {
    pub fn new(private_key_pem: &[u8], public_key_pem: &[u8], access_token_expiry: i64) -> Self {
        // Implementação
    }
    
    pub fn generate_access_token(&self, user: &User) -> Result<String, JwtError> {
        // Implementação
    }
    
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        // Implementação
    }
}
```

**Critérios de Aceitação:**
- [ ] Geração de token funcional
- [ ] Validação funcional
- [ ] EdDSA implementado (não HMAC)
- [ ] Expiração respeitada
- [ ] Testes unitários completos
- [ ] Key rotation preparado (aceitar múltiplas chaves)

---

### Tarefa 1.2.3: Implementar Refresh Token Service
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 4 horas

**Arquivo:** `crates/auth-core/src/tokens.rs`

**Funcionalidades:**
- Geração de refresh token opaco
- Armazenamento com hash
- Validação
- Rotação básica (será melhorada na Fase 2)

**Estrutura:**
```rust
use sha2::{Sha256, Digest};
use rand::{thread_rng, Rng};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub struct RefreshTokenService {
    db: PgPool,
}

impl RefreshTokenService {
    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<String, TokenError> {
        // Implementação
    }
    
    pub async fn verify_refresh_token(&self, token: &str) -> Result<Uuid, TokenError> {
        // Implementação
    }
    
    pub async fn revoke_token(&self, token: &str) -> Result<(), TokenError> {
        // Implementação
    }
    
    fn hash_token(&self, token: &str) -> String {
        // SHA-256 hash
    }
}
```

**Critérios de Aceitação:**
- [ ] Token gerado com 256 bits de entropia
- [ ] Hash SHA-256 no storage
- [ ] Validação funcional
- [ ] Revogação funcional
- [ ] Testes unitários

---

## Sprint 1.3: Database Repositories (Semana 2)

### Tarefa 1.3.1: Implementar User Repository
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 5 horas

**Arquivo:** `crates/infra-database/src/repositories/user_repository.rs`

**Operações:**
- create
- find_by_id
- find_by_email
- update
- delete (soft)
- list (com paginação)

**Estrutura:**
```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::User;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn create(&self, user: CreateUserDto) -> Result<User, DbError> {
        // Implementação com sqlx::query_as!
    }
    
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DbError> {
        // Implementação
    }
    
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        // Implementação
    }
    
    // ... outras operações
}
```

**Critérios de Aceitação:**
- [ ] Todas operações CRUD implementadas
- [ ] Queries type-safe com sqlx macros
- [ ] Paginação implementada
- [ ] Testes de integração (usando testcontainers)
- [ ] Error handling apropriado

---

### Tarefa 1.3.2: Implementar Token Repositories
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Arquivos:**
- `refresh_token_repository.rs`
- `email_verification_repository.rs`
- `password_reset_repository.rs`

**Critérios de Aceitação:**
- [ ] Operações CRUD para cada tipo de token
- [ ] Cleanup de tokens expirados
- [ ] Testes de integração

---

### Tarefa 1.3.3: Implementar Audit Log Repository
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 2 horas

**Arquivo:** `audit_log_repository.rs`

**Operações:**
- create (append-only)
- search (com filtros)
- count

**Critérios de Aceitação:**
- [ ] Inserção funcional
- [ ] Busca com filtros complexos
- [ ] Paginação
- [ ] Testes

---

## Sprint 1.4: Auth Endpoints (Semana 2-3)

### Tarefa 1.4.1: Implementar Register Endpoint
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 5 horas

**Arquivo:** `apps/api-server/src/modules/auth/handlers.rs`

**Endpoint:** `POST /api/v1/auth/register`

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Doe",
  "phone": "+5511999999999"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Registro realizado com sucesso. Verifique seu email.",
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "email_verified": false
    }
  }
}
```

**Fluxo:**
1. Validar input
2. Verificar email não duplicado
3. Hash senha
4. Criar usuário
5. Gerar token de verificação
6. Enviar email (mock na Fase 1)
7. Retornar resposta
8. Logar evento

**Critérios de Aceitação:**
- [ ] Endpoint funcional
- [ ] Validação de input
- [ ] Error handling
- [ ] Testes de integração
- [ ] Rate limiting (básico)

---

### Tarefa 1.4.2: Implementar Login Endpoint
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 6 horas

**Endpoint:** `POST /api/v1/auth/login`

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJ...",
    "refresh_token": "opaque_token",
    "token_type": "Bearer",
    "expires_in": 900,
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "role": "user"
    }
  }
}
```

**Fluxo:**
1. Validar input
2. Rate limiting check
3. Buscar usuário
4. Verificar conta ativa
5. Verificar senha
6. Incrementar falhas se inválida
7. Se válida: gerar tokens
8. Atualizar last_login
9. Logar evento
10. Retornar tokens

**Critérios de Aceitação:**
- [ ] Endpoint funcional
- [ ] Account lockout implementado (5 tentativas)
- [ ] Tokens gerados corretamente
- [ ] Cookies configurados (HttpOnly, Secure)
- [ ] Audit log
- [ ] Testes extensivos (casos de sucesso e falha)

---

### Tarefa 1.4.3: Implementar Logout Endpoint
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Endpoint:** `POST /api/v1/auth/logout`

**Headers:**
```
Authorization: Bearer {access_token}
```

**Request:**
```json
{
  "refresh_token": "opaque_token"
}
```

**Fluxo:**
1. Extrair user do JWT
2. Revogar refresh token
3. Limpar cookie
4. Logar evento

**Critérios de Aceitação:**
- [ ] Token revogado
- [ ] Cookie limpo
- [ ] Audit log
- [ ] Testes

---

### Tarefa 1.4.4: Implementar Refresh Token Endpoint
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Endpoint:** `POST /api/v1/auth/refresh`

**Cookie:** refresh_token (enviado automaticamente)

**Response:**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJ...",
    "refresh_token": "new_opaque_token",
    "expires_in": 900
  }
}
```

**Fluxo:**
1. Ler refresh token do cookie
2. Validar token
3. Buscar usuário
4. Gerar novo access token
5. Rotacionar refresh token (básico, será melhorado na Fase 2)
6. Retornar novos tokens

**Critérios de Aceitação:**
- [ ] Refresh funcional
- [ ] Rotação básica implementada
- [ ] Novos tokens válidos
- [ ] Testes

---

### Tarefa 1.4.5: Implementar Auth Middleware
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Arquivo:** `apps/api-server/src/modules/auth/middleware.rs`

**Funcionalidade:**
- Extrair JWT do header Authorization
- Validar JWT
- Verificar usuário ativo
- Injetar Claims e User no request

**Estrutura:**
```rust
use axum::{
    middleware::Next,
    http::{Request, StatusCode},
    response::Response,
    Extension,
};

pub async fn auth_middleware<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Implementação
}
```

**Critérios de Aceitação:**
- [ ] Middleware funcional
- [ ] JWT validado
- [ ] User injetado no request
- [ ] Error handling apropriado
- [ ] Testes

---

## Sprint 1.5: Casbin Integration (Semana 3)

### Tarefa 1.5.1: Configurar Casbin
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Arquivo:** `crates/authz-core/src/casbin.rs`

**Model File:** `config/casbin/model.conf`
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

**Critérios de Aceitação:**
- [ ] Casbin inicializado
- [ ] PostgreSQL adapter configurado
- [ ] Model carregado
- [ ] Policies básicas carregadas
- [ ] Testes de enforcement

---

### Tarefa 1.5.2: Implementar Authorization Middleware
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 4 horas

**Arquivo:** `apps/api-server/src/modules/authorization/middleware.rs`

**Funcionalidade:**
- Extrair user, resource, action do request
- Enforcement via Casbin
- Negar acesso se não autorizado

**Uso:**
```rust
Router::new()
    .route("/users", get(list_users))
    .route_layer(from_fn_with_state(
        state.clone(),
        require_permission("users", "read")
    ))
```

**Critérios de Aceitação:**
- [ ] Middleware funcional
- [ ] Enforcement correto
- [ ] Audit log de access denied
- [ ] Testes

---

### Tarefa 1.5.3: Implementar Policy Management
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Endpoints (Admin Only):**
- `POST /api/v1/admin/policies` - Adicionar policy
- `DELETE /api/v1/admin/policies` - Remover policy
- `GET /api/v1/admin/policies` - Listar policies
- `GET /api/v1/admin/roles/:role/permissions` - Listar permissões de role

**Critérios de Aceitação:**
- [ ] CRUD de policies
- [ ] Listagem funcional
- [ ] Apenas admin pode acessar
- [ ] Audit log
- [ ] Testes

---

## Sprint 1.6: User Management (Semana 4)

### Tarefa 1.6.1: Implementar User CRUD Endpoints
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 6 horas

**Endpoints:**
- `GET /api/v1/users` - Listar (com paginação, filtros)
- `GET /api/v1/users/:id` - Buscar por ID
- `PUT /api/v1/users/:id` - Atualizar
- `DELETE /api/v1/users/:id` - Deletar (soft)

**Permissões:**
- Listar: admin
- Buscar próprio perfil: user
- Buscar qualquer: admin
- Atualizar próprio: user
- Atualizar qualquer: admin
- Deletar: admin

**Critérios de Aceitação:**
- [ ] Todos endpoints funcionais
- [ ] Paginação implementada
- [ ] Filtros por role, status
- [ ] Autorização via Casbin
- [ ] Validação de input
- [ ] Testes

---

### Tarefa 1.6.2: Implementar Profile Endpoints
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Endpoints:**
- `GET /api/v1/profile` - Perfil do usuário autenticado
- `PUT /api/v1/profile` - Atualizar perfil
- `POST /api/v1/profile/change-password` - Mudar senha

**Critérios de Aceitação:**
- [ ] Endpoints funcionais
- [ ] Change password requer senha atual
- [ ] Audit log
- [ ] Testes

---

## Sprint 1.7: Frontend Auth UI (Semana 4-5)

### Tarefa 1.7.1: Criar Auth Service
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 4 horas

**Arquivo:** `src/app/core/services/auth.service.ts`

**Funcionalidades:**
- login()
- register()
- logout()
- refreshToken()
- getCurrentUser()
- isAuthenticated()
- hasRole()

**Usando Signals:**
```typescript
@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly router = inject(Router);
  
  private readonly currentUserSignal = signal<User | null>(null);
  readonly currentUser = this.currentUserSignal.asReadonly();
  
  login(credentials: LoginRequest): Observable<LoginResponse> {
    // Implementação
  }
  
  // ... outras funções
}
```

**Critérios de Aceitação:**
- [ ] Todas operações implementadas
- [ ] Tokens gerenciados corretamente
- [ ] Signals para reatividade
- [ ] Error handling
- [ ] Testes unitários

---

### Tarefa 1.7.2: Criar HTTP Interceptors
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 3 horas

**Interceptors:**
- AuthInterceptor: adiciona token
- RefreshInterceptor: refresh automático em 401
- ErrorInterceptor: tratamento global de erros

**Arquivo:** `src/app/core/interceptors/auth.interceptor.ts`

**Critérios de Aceitação:**
- [ ] Token adicionado automaticamente
- [ ] Refresh funcional
- [ ] Evita loop de refresh
- [ ] Error handling global
- [ ] Testes

---

### Tarefa 1.7.3: Criar Route Guards
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Guards:**
- AuthGuard: requer autenticação
- RoleGuard: requer role específica

**Arquivos:**
- `src/app/core/guards/auth.guard.ts`
- `src/app/core/guards/role.guard.ts`

**Critérios de Aceitação:**
- [ ] Guards funcionais
- [ ] Redirect para login
- [ ] returnUrl preservado
- [ ] Testes

---

### Tarefa 1.7.4: Criar Login Component
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 5 horas

**Arquivo:** `src/app/features/auth/login/login.component.ts`

**Features:**
- Formulário reativo
- Validação client-side
- Loading state
- Error messages
- "Lembrar-me" (opcional)
- Link para registro e forgot password

**Critérios de Aceitação:**
- [ ] UI funcional e responsiva
- [ ] Validação sincronizada com backend
- [ ] Loading indicators
- [ ] Error handling
- [ ] Acessibilidade (ARIA)
- [ ] Testes

---

### Tarefa 1.7.5: Criar Register Component
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 5 horas

**Arquivo:** `src/app/features/auth/register/register.component.ts`

**Features:**
- Formulário com todos os campos
- Validação de força de senha (visual)
- Confirmação de senha
- Termos de uso (checkbox)

**Critérios de Aceitação:**
- [ ] UI funcional
- [ ] Validação completa
- [ ] Password strength meter
- [ ] Error handling
- [ ] Testes

---

### Tarefa 1.7.6: Criar Dashboard Component
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Arquivo:** `src/app/features/dashboard/dashboard.component.ts`

**Features:**
- Mensagem de boas-vindas
- Informações do usuário
- Links para perfil, logout

**Critérios de Aceitação:**
- [ ] UI básica funcional
- [ ] Protegido por AuthGuard
- [ ] Mostra dados do usuário
- [ ] Testes

---

## Sprint 1.8: Admin Dashboard Básico (Semana 5-6)

### Tarefa 1.8.1: Criar Admin Layout
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 4 horas

**Arquivo:** `src/app/layouts/admin-layout/admin-layout.component.ts`

**Features:**
- Sidebar com navegação
- Header com user menu
- Responsivo

**Critérios de Aceitação:**
- [ ] Layout funcional
- [ ] Navegação entre páginas admin
- [ ] Responsivo
- [ ] Testes

---

### Tarefa 1.8.2: Criar User List Component (Admin)
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 6 horas

**Arquivo:** `src/app/features/admin/users/user-list/user-list.component.ts`

**Features:**
- Tabela com paginação
- Busca por email
- Filtros por role e status
- Ações: editar, suspender, deletar

**Critérios de Aceitação:**
- [ ] Listagem funcional
- [ ] Paginação
- [ ] Filtros funcionais
- [ ] Ações implementadas
- [ ] Confirmação para deleção
- [ ] Testes

---

### Tarefa 1.8.3: Criar User Edit Component (Admin)
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 4 horas

**Features:**
- Editar informações do usuário
- Mudar role
- Suspender/reativar

**Critérios de Aceitação:**
- [ ] Formulário funcional
- [ ] Validação
- [ ] Save funcional
- [ ] Testes

---

## Sprint 1.9: Testing e Bug Fixes (Semana 6)

### Tarefa 1.9.1: Testes de Integração Backend
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 8 horas

**Escopo:**
- Testes end-to-end de todos os fluxos de auth
- Testes de autorização
- Testes de edge cases

**Setup:**
```rust
// Usar testcontainers para PostgreSQL
use testcontainers::{clients, images};

#[tokio::test]
async fn test_full_registration_flow() {
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());
    
    // Setup app com test database
    // Executar testes
}
```

**Critérios de Aceitação:**
- [ ] Cobertura > 80%
- [ ] Todos fluxos críticos testados
- [ ] Casos de erro testados
- [ ] Testes passando no CI

---

### Tarefa 1.9.2: Testes E2E Frontend
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 6 horas

**Ferramenta:** Playwright ou Cypress

**Cenários:**
- Registro completo
- Login e navegação
- Logout
- Admin: listar e editar usuários

**Critérios de Aceitação:**
- [ ] Testes E2E implementados
- [ ] Rodando no CI
- [ ] Flaky tests resolvidos

---

### Tarefa 1.9.3: Bug Bash e Fixes
**Responsável:** Todo o time  
**Prioridade:** P0  
**Estimativa:** 8 horas

**Atividades:**
- Testar manualmente todos os fluxos
- Documentar bugs encontrados
- Priorizar e corrigir

**Critérios de Aceitação:**
- [ ] Bugs críticos corrigidos
- [ ] Bugs conhecidos documentados
- [ ] Sistema estável

---

## Fase 2: Security Hardening

**Duração:** 4 semanas  
**Objetivo:** Fortalecer segurança com MFA, token rotation, rate limiting avançado

---

## Sprint 2.1: Token Family e Detecção de Roubo (Semana 7)

### Tarefa 2.1.1: Atualizar Schema - Token Family
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Migration:**
```sql
ALTER TABLE refresh_tokens
ADD COLUMN token_family UUID NOT NULL DEFAULT uuid_generate_v4(),
ADD COLUMN parent_token_id UUID REFERENCES refresh_tokens(id);

CREATE INDEX idx_refresh_tokens_family 
ON refresh_tokens(token_family) WHERE NOT revoked;
```

**Critérios de Aceitação:**
- [ ] Migration executada
- [ ] Índice criado
- [ ] Dados existentes migrados

---

### Tarefa 2.1.2: Implementar Token Rotation com Detecção
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 6 horas

**Atualizar:** `crates/auth-core/src/tokens.rs`

**Nova funcionalidade:**
- Detectar reuso de token revogado
- Revogar família inteira
- Notificar usuário via email
- Logar alerta de segurança

**Critérios de Aceitação:**
- [ ] Rotação implementada
- [ ] Detecção de roubo funcional
- [ ] Toda família revogada em caso de suspeita
- [ ] Email enviado
- [ ] Audit log detalhado
- [ ] Testes extensivos

---

## Sprint 2.2: Multi-Factor Authentication (Semana 7-8)

### Tarefa 2.2.1: Adicionar Schema MFA
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 2 horas

**Migration:**
```sql
ALTER TABLE users
ADD COLUMN mfa_enabled BOOLEAN DEFAULT FALSE,
ADD COLUMN mfa_secret VARCHAR(255),
ADD COLUMN backup_codes TEXT[];

CREATE TABLE mfa_recovery_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash VARCHAR(255) NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMPTZ
);
```

**Critérios de Aceitação:**
- [ ] Colunas adicionadas
- [ ] Tabela de recovery codes criada

---

### Tarefa 2.2.2: Implementar TOTP Service
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 5 horas

**Dependência:** Adicionar crate `totp-rs`

**Arquivo:** `crates/auth-core/src/mfa.rs`

**Funcionalidades:**
- Gerar secret TOTP
- Gerar QR code URL
- Verificar código TOTP
- Gerar backup codes

**Critérios de Aceitação:**
- [ ] Secret gerado corretamente
- [ ] QR code URL funcional
- [ ] Validação de código TOTP
- [ ] Backup codes gerados
- [ ] Testes

---

### Tarefa 2.2.3: Implementar MFA Endpoints
**Responsável:** Backend Dev  
**Prioridade:** P0  
**Estimativa:** 6 horas

**Endpoints:**
- `POST /api/v1/profile/mfa/setup` - Iniciar setup
- `POST /api/v1/profile/mfa/verify` - Verificar e ativar
- `POST /api/v1/profile/mfa/disable` - Desativar
- `POST /api/v1/auth/mfa/verify` - Verificar no login

**Fluxo de Setup:**
1. User solicita setup
2. Backend gera secret
3. Retorna QR code e backup codes
4. User escaneia QR code no app (Google Authenticator)
5. User submete código de verificação
6. Se válido: ativar MFA

**Fluxo de Login com MFA:**
1. Login com email/senha bem-sucedido
2. Se MFA ativado: retornar `mfa_required: true`
3. Frontend solicita código TOTP
4. User submete código
5. Backend valida
6. Se válido: emitir tokens

**Critérios de Aceitação:**
- [ ] Setup completo funcional
- [ ] Login com MFA funcional
- [ ] Backup codes funcionais
- [ ] Desabilitar MFA funcional
- [ ] Testes

---

### Tarefa 2.2.4: UI de MFA (Frontend)
**Responsável:** Frontend Dev  
**Prioridade:** P0  
**Estimativa:** 8 horas

**Components:**
- MFA Setup Component
- MFA Verify Component (no login)
- MFA Management Component (profile)

**Critérios de Aceitação:**
- [ ] Setup UI funcional
- [ ] QR code exibido
- [ ] Backup codes exibidos e downloadable
- [ ] Login flow com MFA
- [ ] Disable MFA funcional
- [ ] Testes

---

## Sprint 2.3: Rate Limiting Avançado (Semana 8)

### Tarefa 2.3.1: Implementar Rate Limiter Service
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 5 horas

**Opções:**
- Usar `tower-governor` crate
- Ou implementar custom com Redis (opcional) ou PostgreSQL

**Estratégia:** Sliding Window

**Configuração:**
```rust
pub struct RateLimitConfig {
    pub login_per_ip: (u32, Duration),      // 5 req/min
    pub register_per_ip: (u32, Duration),   // 3 req/hora
    pub api_per_user: (u32, Duration),      // 1000 req/hora
}
```

**Critérios de Aceitação:**
- [ ] Rate limiter implementado
- [ ] Sliding window funcional
- [ ] Configurável por endpoint
- [ ] Retorna 429 com Retry-After
- [ ] Testes

---

### Tarefa 2.3.2: Aplicar Rate Limiting nos Endpoints
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Aplicar em:**
- `/auth/login`
- `/auth/register`
- `/auth/refresh`
- `/auth/password-reset`
- Endpoints de API (geral)

**Critérios de Aceitação:**
- [ ] Rate limiting aplicado
- [ ] Testes verificam enforcement
- [ ] Logs de rate limit hits

---

## Sprint 2.4: CAPTCHA Integration (Semana 9)

### Tarefa 2.4.1: Escolher e Configurar CAPTCHA
**Responsável:** Full-stack Dev  
**Prioridade:** P2  
**Estimativa:** 4 horas

**Opções On-Premise:**
- hCaptcha (self-hosted)
- Custom challenge-response
- Implementação simplificada

**Decisão:** Documentar escolha e justificativa

**Critérios de Aceitação:**
- [ ] CAPTCHA escolhido e configurado
- [ ] Integração backend
- [ ] Integração frontend
- [ ] Testes

---

### Tarefa 2.4.2: Aplicar CAPTCHA em Endpoints Críticos
**Responsável:** Full-stack Dev  
**Prioridade:** P2  
**Estimativa:** 3 horas

**Aplicar em:**
- Register (sempre ou após X tentativas?)
- Login (após 3 falhas)
- Password reset

**Critérios de Aceitação:**
- [ ] CAPTCHA funcional
- [ ] UX não degradada
- [ ] Testes

---

## Sprint 2.5: Security Headers e CSRF (Semana 9)

### Tarefa 2.5.1: Implementar Security Headers
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 2 horas

**Headers:**
- Content-Security-Policy
- X-Content-Type-Options: nosniff
- X-Frame-Options: DENY
- X-XSS-Protection: 1; mode=block
- Strict-Transport-Security (HSTS)

**Implementação:** Middleware Axum

**Critérios de Aceitação:**
- [ ] Todos headers configurados
- [ ] Verificar com securityheaders.com
- [ ] Testes

---

### Tarefa 2.5.2: Implementar CSRF Protection
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 4 horas

**Estratégia:** Double Submit Cookie

**Implementação:**
- Gerar CSRF token no login
- Enviar via cookie
- Exigir em header X-CSRF-Token
- Validar em endpoints que modificam estado

**Critérios de Aceitação:**
- [ ] CSRF tokens gerados
- [ ] Validação funcional
- [ ] Frontend integrado
- [ ] Testes

---

## Sprint 2.6: Session Management UI (Semana 10)

### Tarefa 2.6.1: Endpoint para Listar Sessões Ativas
**Responsável:** Backend Dev  
**Prioridade:** P2  
**Estimativa:** 3 horas

**Endpoint:** `GET /api/v1/profile/sessions`

**Response:**
```json
{
  "sessions": [
    {
      "id": "uuid",
      "device": "Chrome on Windows",
      "location": "São Paulo, BR",
      "ip_address": "192.168.1.1",
      "last_active": "2024-01-01T12:00:00Z",
      "current": true
    }
  ]
}
```

**Critérios de Aceitação:**
- [ ] Listagem funcional
- [ ] Sessão atual marcada
- [ ] Device fingerprinting básico
- [ ] Testes

---

### Tarefa 2.6.2: Endpoint para Revogar Sessão
**Responsável:** Backend Dev  
**Prioridade:** P2  
**Estimativa:** 2 horas

**Endpoint:** `DELETE /api/v1/profile/sessions/:id`

**Funcionalidade:**
- Revogar refresh token específico
- Não permitir revogar sessão atual

**Critérios de Aceitação:**
- [ ] Revogação funcional
- [ ] Validações
- [ ] Testes

---

### Tarefa 2.6.3: UI de Gerenciamento de Sessões
**Responsável:** Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 5 horas

**Component:** Session Management

**Features:**
- Listar sessões ativas
- Mostrar device, location, last active
- Botão "Revogar" por sessão
- Botão "Revogar todas as outras"

**Critérios de Aceitação:**
- [ ] UI funcional
- [ ] Listagem de sessões
- [ ] Revogação funcional
- [ ] Confirmações apropriadas
- [ ] Testes

---

## Sprint 2.7: Security Testing e Audit (Semana 10)

### Tarefa 2.7.1: Security Testing
**Responsável:** Full-stack Dev  
**Prioridade:** P0  
**Estimativa:** 8 horas

**Atividades:**
- Rodar OWASP ZAP scan
- Testar manualmente vulnerabilidades conhecidas
- SQL injection attempts
- XSS attempts
- CSRF attempts
- Brute force attempts (verificar rate limiting)

**Critérios de Aceitação:**
- [ ] Scan completo executado
- [ ] Vulnerabilidades críticas corrigidas
- [ ] Relatório de segurança documentado

---

### Tarefa 2.7.2: Dependency Audit
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 2 horas

**Comandos:**
```bash
cargo audit
npm audit
```

**Atividades:**
- Identificar vulnerabilidades
- Atualizar dependências vulneráveis
- Documentar dependências que não podem ser atualizadas

**Critérios de Aceitação:**
- [ ] Audit executado
- [ ] Vulnerabilidades críticas resolvidas
- [ ] Relatório documentado

---

## Fase 3: Advanced Features

**Duração:** 4 semanas  
**Objetivo:** Features avançadas e melhorias de UX

---

## Sprint 3.1: Email Service (Semana 11)

### Tarefa 3.1.1: Configurar SMTP Service
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Arquivo:** `crates/infra-email/src/smtp.rs`

**Configuração:**
- SMTP host, port, credentials
- Templates de email
- Retry logic

**Critérios de Aceitação:**
- [ ] SMTP configurado
- [ ] Envio funcional
- [ ] Templates básicos
- [ ] Error handling
- [ ] Testes (mock SMTP)

---

### Tarefa 3.1.2: Templates de Email
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 4 horas

**Templates:**
- Verificação de email
- Password reset
- Notificação de login incomum
- MFA habilitado
- Senha alterada

**Formato:** HTML responsivo

**Critérios de Aceitação:**
- [ ] Templates criados
- [ ] Responsivos
- [ ] Branded (logo, cores)
- [ ] Testados em clientes de email

---

### Tarefa 3.1.3: Integrar Email nos Fluxos
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Integrar em:**
- Registro (verificação)
- Password reset
- Login incomum (detecção básica)
- MFA ativado
- Password changed

**Critérios de Aceitação:**
- [ ] Emails enviados nos fluxos corretos
- [ ] Async (não bloqueia request)
- [ ] Testes

---

## Sprint 3.2: Password Reset e Email Verification (Semana 11-12)

### Tarefa 3.2.1: Implementar Forgot Password Endpoint
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Endpoint:** `POST /api/v1/auth/forgot-password`

**Fluxo:**
1. User submete email
2. Se email existe: gerar token, enviar email
3. Sempre retornar mensagem genérica

**Critérios de Aceitação:**
- [ ] Endpoint funcional
- [ ] Token gerado e enviado
- [ ] Não revela se email existe
- [ ] Rate limiting
- [ ] Testes

---

### Tarefa 3.2.2: Implementar Reset Password Endpoint
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Endpoint:** `POST /api/v1/auth/reset-password`

**Request:**
```json
{
  "token": "reset_token",
  "new_password": "NewSecurePass123!"
}
```

**Fluxo:**
1. Validar token
2. Validar nova senha
3. Hash nova senha
4. Atualizar no banco
5. Revogar todos refresh tokens
6. Enviar email de confirmação

**Critérios de Aceitação:**
- [ ] Endpoint funcional
- [ ] Token validado
- [ ] Tokens revogados
- [ ] Email enviado
- [ ] Testes

---

### Tarefa 3.2.3: UI de Password Reset
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 5 horas

**Components:**
- Forgot Password Component
- Reset Password Component

**Critérios de Aceitação:**
- [ ] UI funcional
- [ ] Validação de senha
- [ ] Feedback claro
- [ ] Testes

---

### Tarefa 3.2.4: Implementar Email Verification
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Endpoint:** `GET /api/v1/auth/verify-email/:token`

**Fluxo:**
1. Validar token
2. Marcar email como verificado
3. Marcar token como usado
4. Redirecionar para login ou dashboard

**Critérios de Aceitação:**
- [ ] Verificação funcional
- [ ] One-time use
- [ ] Expiração respeitada
- [ ] Testes

---

### Tarefa 3.2.5: UI de Email Verification
**Responsável:** Frontend Dev  
**Prioridade:** P1  
**Estimativa:** 3 horas

**Components:**
- Verify Email Component
- Resend Verification Component

**Critérios de Aceitação:**
- [ ] UI funcional
- [ ] Feedback claro
- [ ] Reenvio funcional
- [ ] Testes

---

## Sprint 3.3: Advanced Admin Features (Semana 12-13)

### Tarefa 3.3.1: Dashboard com Estatísticas
**Responsável:** Backend Dev + Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 6 horas

**Endpoint:** `GET /api/v1/admin/stats`

**Estatísticas:**
- Total users
- Active users
- New users (today, week, month)
- Total logins today
- Failed logins today
- Accounts locked

**UI:**
- Cards com números
- Gráficos simples (Chart.js ou similar)

**Critérios de Aceitação:**
- [ ] Endpoint funcional
- [ ] UI com estatísticas
- [ ] Gráficos básicos
- [ ] Atualização em tempo real (opcional)
- [ ] Testes

---

### Tarefa 3.3.2: Audit Log Viewer (Admin)
**Responsável:** Backend Dev + Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 6 horas

**Endpoint:** `GET /api/v1/admin/audit-logs`

**Filtros:**
- Por usuário
- Por tipo de evento
- Por data range
- Por sucesso/falha

**UI:**
- Tabela com paginação
- Filtros
- Export para CSV

**Critérios de Aceitação:**
- [ ] Endpoint funcional
- [ ] UI com filtros
- [ ] Paginação
- [ ] Export funcional
- [ ] Testes

---

### Tarefa 3.3.3: Role Management (Admin)
**Responsável:** Backend Dev + Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 6 horas

**Endpoints:**
- `GET /api/v1/admin/roles` - Listar roles
- `POST /api/v1/admin/roles` - Criar role
- `PUT /api/v1/admin/roles/:id` - Atualizar role
- `DELETE /api/v1/admin/roles/:id` - Deletar role
- `GET /api/v1/admin/roles/:id/permissions` - Listar permissões
- `PUT /api/v1/admin/roles/:id/permissions` - Atualizar permissões

**UI:**
- Listagem de roles
- Criar/editar role
- Gerenciar permissões (checkboxes)

**Critérios de Aceitação:**
- [ ] CRUD de roles funcional
- [ ] Gestão de permissões funcional
- [ ] UI intuitiva
- [ ] Validações
- [ ] Testes

---

## Sprint 3.4: Security Notifications (Semana 13)

### Tarefa 3.4.1: Implementar Detecção de Anomalias
**Responsável:** Backend Dev  
**Prioridade:** P2  
**Estimativa:** 6 horas

**Anomalias:**
- Login de IP diferente do usual
- Login de país diferente
- Múltiplas falhas seguidas de sucesso
- Velocity check (login em locais distantes em curto espaço)

**Ação:**
- Logar evento
- Enviar email de notificação
- Opcionalmente: exigir MFA adicional

**Critérios de Aceitação:**
- [ ] Detecção implementada
- [ ] Email enviado
- [ ] Testes com cenários anômalos

---

### Tarefa 3.4.2: UI de Notificações de Segurança
**Responsável:** Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 4 horas

**Component:** Security Notifications

**Features:**
- Listar notificações recentes
- Marcar como lida
- Detalhes da notificação

**Critérios de Aceitação:**
- [ ] UI funcional
- [ ] Notificações exibidas
- [ ] Badge de não lidas
- [ ] Testes

---

## Sprint 3.5: Performance Optimization (Semana 14)

### Tarefa 3.5.1: Database Query Optimization
**Responsável:** Backend Dev  
**Prioridade:** P2  
**Estimativa:** 4 horas

**Atividades:**
- Analisar query plans (EXPLAIN ANALYZE)
- Adicionar índices necessários
- Otimizar queries N+1
- Implementar caching onde apropriado (Redis opcional)

**Critérios de Aceitação:**
- [ ] Queries críticas analisadas
- [ ] Índices adicionados
- [ ] Melhoria mensurável de performance

---

### Tarefa 3.5.2: Frontend Performance
**Responsável:** Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 4 horas

**Atividades:**
- Lazy loading de rotas
- Image optimization
- Bundle size analysis
- Code splitting
- Memoization onde apropriado

**Critérios de Aceitação:**
- [ ] Lighthouse score > 90
- [ ] Bundle size reduzido
- [ ] Lazy loading implementado

---

### Tarefa 3.5.3: Load Testing
**Responsável:** Full-stack Dev  
**Prioridade:** P2  
**Estimativa:** 4 horas

**Ferramenta:** k6 ou locust

**Cenários:**
- 100 usuários simultâneos fazendo login
- 1000 requests/sec em endpoints protegidos
- Stress test até encontrar limites

**Critérios de Aceitação:**
- [ ] Testes executados
- [ ] Bottlenecks identificados
- [ ] Limites documentados
- [ ] Melhorias implementadas (se possível)

---

## Fase 4: Produção e Otimização

**Duração:** 2 semanas  
**Objetivo:** Deploy em produção e documentação final

---

## Sprint 4.1: Deployment Preparation (Semana 15)

### Tarefa 4.1.1: Configurar Ambiente de Staging
**Responsável:** Full-stack Dev  
**Prioridade:** P0  
**Estimativa:** 6 horas

**Atividades:**
- Provisionar servidor (ou VMs)
- Instalar PostgreSQL
- Configurar TLS/SSL
- Deploy da aplicação
- Configurar variáveis de ambiente

**Critérios de Aceitação:**
- [ ] Staging rodando
- [ ] Acessível via HTTPS
- [ ] Database configurado
- [ ] Teste manual completo

---

### Tarefa 4.1.2: Configurar CI/CD para Deploy
**Responsável:** Full-stack Dev  
**Prioridade:** P0  
**Estimativa:** 6 horas

**Pipeline:**
1. Testes (já existente)
2. Build de produção
3. Deploy para staging (automático em push para `develop`)
4. Deploy para produção (manual com approval)

**Critérios de Aceitação:**
- [ ] Pipeline completo
- [ ] Deploy automático para staging
- [ ] Deploy manual para produção
- [ ] Rollback funcional

---

### Tarefa 4.1.3: Setup de Monitoring
**Responsável:** Full-stack Dev  
**Prioridade:** P0  
**Estimativa:** 8 horas

**Stack:**
- Prometheus para métricas
- Grafana para visualização
- Loki para logs (ou ELK)
- Alertmanager para alertas

**Métricas:**
- Request rate, latency, errors
- Database connections, query time
- CPU, memória, disco
- Failed logins, account lockouts
- Token generation time

**Alertas:**
- Error rate > 5%
- Response time > 1s
- Failed login spike
- Database connection pool exhausted
- Certificate expiring soon

**Critérios de Aceitação:**
- [ ] Prometheus rodando
- [ ] Grafana com dashboards
- [ ] Logs centralizados
- [ ] Alertas configurados
- [ ] On-call definido

---

## Sprint 4.2: Documentation (Semana 15-16)

### Tarefa 4.2.1: Documentação de API
**Responsável:** Backend Dev  
**Prioridade:** P1  
**Estimativa:** 6 horas

**Formato:** OpenAPI/Swagger

**Conteúdo:**
- Todos endpoints documentados
- Request/response examples
- Authentication
- Error codes

**Ferramenta:** Usar `utoipa` crate para gerar OpenAPI

**Critérios de Aceitação:**
- [ ] Swagger UI acessível
- [ ] Todos endpoints documentados
- [ ] Examples completos
- [ ] Try-it-out funcional

---

### Tarefa 4.2.2: Documentação de Arquitetura
**Responsável:** Full-stack Dev  
**Prioridade:** P1  
**Estimativa:** 8 horas

**Documentos:**
- `docs/ARCHITECTURE.md` - Visão geral
- `docs/SECURITY.md` - Decisões de segurança
- `docs/DATABASE.md` - Schema e migrations
- `docs/DEPLOYMENT.md` - Processo de deploy
- `docs/MONITORING.md` - Setup de monitoring
- `docs/TROUBLESHOOTING.md` - Problemas comuns

**Critérios de Aceitação:**
- [ ] Todos documentos criados
- [ ] Diagramas incluídos
- [ ] Atualizado com implementação real

---

### Tarefa 4.2.3: User Documentation
**Responsável:** Frontend Dev  
**Prioridade:** P2  
**Estimativa:** 4 horas

**Documentos:**
- User guide para funcionalidades
- Admin guide
- FAQ

**Critérios de Aceitação:**
- [ ] Documentação clara
- [ ] Screenshots
- [ ] Casos de uso comuns

---

## Sprint 4.3: Production Deploy (Semana 16)

### Tarefa 4.3.1: Production Deployment
**Responsável:** Full-stack Dev  
**Prioridade:** P0  
**Estimativa:** 4 horas

**Checklist:**
- [ ] Backup de produção (se existir)
- [ ] Deploy via pipeline
- [ ] Smoke tests
- [ ] Verificar logs
- [ ] Verificar métricas
- [ ] Notificar stakeholders

**Critérios de Aceitação:**
- [ ] Sistema rodando em produção
- [ ] Sem erros críticos
- [ ] Monitoring funcional
- [ ] Rollback plan testado

---

### Tarefa 4.3.2: Post-Deploy Monitoring
**Responsável:** Todo o time  
**Prioridade:** P0  
**Estimativa:** 8 horas (distribuído)

**Atividades:**
- Monitorar métricas por 24h
- Responder a alertas
- Corrigir issues críticos
- Documentar problemas

**Critérios de Aceitação:**
- [ ] 24h de monitoring completo
- [ ] Issues críticos resolvidos
- [ ] Sistema estável

---

### Tarefa 4.3.3: Retrospectiva e Lessons Learned
**Responsável:** Todo o time  
**Prioridade:** P1  
**Estimativa:** 2 horas

**Atividades:**
- Reunião de retrospectiva
- Documentar o que funcionou bem
- Documentar o que pode melhorar
- Action items para próximos projetos

**Critérios de Aceitação:**
- [ ] Retrospectiva realizada
- [ ] Lessons learned documentadas
- [ ] Action items definidos

---

## Cronograma Visual

```
Fase 0: Setup (Semana 0)
███ 3-5 dias

Fase 1: MVP Core (Semanas 1-6)
████████████████████████ 6 semanas
├─ Sprint 1.1: Database & Models (Semana 1)
├─ Sprint 1.2: Auth Core (Semana 1-2)
├─ Sprint 1.3: Repositories (Semana 2)
├─ Sprint 1.4: Auth Endpoints (Semana 2-3)
├─ Sprint 1.5: Casbin (Semana 3)
├─ Sprint 1.6: User Management (Semana 4)
├─ Sprint 1.7: Frontend Auth (Semana 4-5)
├─ Sprint 1.8: Admin Dashboard (Semana 5-6)
└─ Sprint 1.9: Testing (Semana 6)

Fase 2: Security Hardening (Semanas 7-10)
████████████████ 4 semanas
├─ Sprint 2.1: Token Family (Semana 7)
├─ Sprint 2.2: MFA (Semana 7-8)
├─ Sprint 2.3: Rate Limiting (Semana 8)
├─ Sprint 2.4: CAPTCHA (Semana 9)
├─ Sprint 2.5: Security Headers (Semana 9)
├─ Sprint 2.6: Session Management (Semana 10)
└─ Sprint 2.7: Security Testing (Semana 10)

Fase 3: Advanced Features (Semanas 11-14)
████████████████ 4 semanas
├─ Sprint 3.1: Email Service (Semana 11)
├─ Sprint 3.2: Password Reset (Semana 11-12)
├─ Sprint 3.3: Advanced Admin (Semana 12-13)
├─ Sprint 3.4: Security Notifications (Semana 13)
└─ Sprint 3.5: Performance (Semana 14)

Fase 4: Produção (Semanas 15-16)
████████ 2 semanas
├─ Sprint 4.1: Deployment Prep (Semana 15)
├─ Sprint 4.2: Documentation (Semana 15-16)
└─ Sprint 4.3: Production Deploy (Semana 16)

Total: 16 semanas (~4 meses)
```

---

## Critérios de Qualidade

### Code Quality

**Backend:**
- [ ] Código segue convenções Rust (rustfmt, clippy)
- [ ] Nenhum warning de clippy
- [ ] Documentação inline para funções públicas
- [ ] Error handling apropriado (não usar .unwrap() em prod)
- [ ] Cobertura de testes > 80%

**Frontend:**
- [ ] Código segue convenções Angular
- [ ] ESLint sem warnings
- [ ] TypeScript strict mode
- [ ] Componentes reutilizáveis
- [ ] Cobertura de testes > 70%

### Security Checklist

- [ ] Argon2id para senhas
- [ ] JWT com EdDSA
- [ ] Refresh token rotation com detecção de roubo
- [ ] MFA implementado
- [ ] Rate limiting em endpoints críticos
- [ ] CAPTCHA em endpoints sensíveis
- [ ] CSRF protection
- [ ] Security headers configurados
- [ ] TLS/HTTPS obrigatório
- [ ] Input validation em todos endpoints
- [ ] SQL injection prevention (prepared statements)
- [ ] XSS prevention
- [ ] Audit logging completo
- [ ] No secrets hardcoded
- [ ] Dependencies auditadas

### Performance Benchmarks

- [ ] Login < 500ms (p95)
- [ ] Register < 1s (p95)
- [ ] API endpoints < 200ms (p95)
- [ ] Password hash < 500ms
- [ ] Database queries < 100ms (p95)
- [ ] Frontend First Contentful Paint < 1.5s
- [ ] Frontend Time to Interactive < 3s
- [ ] Lighthouse score > 90

---

## Riscos e Mitigações

### Riscos Técnicos

**R1: Performance do Argon2 em produção**
- **Probabilidade:** Média
- **Impacto:** Médio
- **Mitigação:** Benchmark early, ajustar parâmetros, considerar async processing

**R2: Token rotation complexity**
- **Probabilidade:** Alta
- **Impacto:** Alto
- **Mitigação:** Testes extensivos, documentação detalhada, monitoring

**R3: Database migrations em produção**
- **Probabilidade:** Média
- **Impacto:** Alto
- **Mitigação:** Testar em staging, backup antes de aplicar, plano de rollback

**R4: Casbin performance em escala**
- **Probabilidade:** Baixa
- **Impacto:** Alto
- **Mitigação:** Caching de policies, benchmarks, considerar alternativas se necessário

### Riscos de Cronograma

**R5: Underestimate de complexidade**
- **Probabilidade:** Alta
- **Impacto:** Médio
- **Mitigação:** Buffer de 20% no cronograma, priorizar MVP, features avançadas podem ser postergadas

**R6: Dependências de terceiros**
- **Probabilidade:** Média
- **Impacto:** Médio
- **Mitigação:** Avaliar dependências early, ter alternativas identificadas

### Riscos de Segurança

**R7: Vulnerabilidade em dependência**
- **Probabilidade:** Média
- **Impacto:** Alto
- **Mitigação:** Cargo audit regular, atualização frequente, monitoring de CVEs

**R8: Configuração incorreta de segurança**
- **Probabilidade:** Média
- **Impacto:** Alto
- **Mitigação:** Security checklist, peer review, penetration testing

---

## Próximos Passos

1. **Review deste plano** com o time
2. **Refinar estimativas** baseado em conhecimento do time
3. **Setup de repositórios** e ferramentas
4. **Kickoff da Fase 0**
5. **Reuniões diárias** (15 min stand-up)
6. **Sprint reviews** ao final de cada sprint
7. **Retrospectivas** ao final de cada fase

---

## Conclusão

Este plano fornece um roadmap detalhado e executável para implementar o sistema de autenticação e autorização. Cada tarefa possui:

- ✅ Descrição clara
- ✅ Critérios de aceitação objetivos
- ✅ Estimativas de tempo
- ✅ Priorização
- ✅ Dependências identificadas

O plano é **iterativo e incremental**, permitindo entregar valor continuamente enquanto evolui para o sistema completo e seguro definido no guia arquitetural.

**Lembre-se:** Este é um documento vivo. Ajuste conforme necessário baseado em feedback, descobertas técnicas e mudanças de requisitos.
