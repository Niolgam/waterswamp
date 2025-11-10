# Arquitetura Completa de Autenticação e Autorização On-Premise

## Guia de Referência Arquitetural Híbrido

**Stack Core:** Rust (Axum, SQLx, Argon2, Casbin-rs, PostgreSQL) + Angular 17+

**Filosofia:** Defense in Depth, Zero Trust Architecture, Security by Design

---

## 📖 Índice

1. [Filosofia e Princípios Fundamentais](#1-filosofia-e-princípios-fundamentais)
2. [Decisões Arquiteturais Estruturais](#2-decisões-arquiteturais-estruturais)
3. [Stack Tecnológica e Justificativas](#3-stack-tecnológica-e-justificativas)
4. [Arquitetura de Segurança](#4-arquitetura-de-segurança)
5. [Modelo de Dados e Persistência](#5-modelo-de-dados-e-persistência)
6. [Gestão de Autenticação](#6-gestão-de-autenticação)
7. [Gestão de Autorização](#7-gestão-de-autorização)
8. [Gestão de Sessões e Tokens](#8-gestão-de-sessões-e-tokens)
9. [Fluxos de Segurança Críticos](#9-fluxos-de-segurança-críticos)
10. [User Management e Administração](#10-user-management-e-administração)
11. [Logging, Auditoria e Monitoramento](#11-logging-auditoria-e-monitoramento)
12. [Arquitetura Frontend](#12-arquitetura-frontend)
13. [Comunicação Backend-Frontend](#13-comunicação-backend-frontend)
14. [Segurança de Infraestrutura](#14-segurança-de-infraestrutura)
15. [Estratégias de Deployment](#15-estratégias-de-deployment)
16. [Compliance e Standards](#16-compliance-e-standards)
17. [Roadmap de Evolução](#17-roadmap-de-evolução)

---

## 1. Filosofia e Princípios Fundamentais

### 1.1. Zero Trust Architecture

**Princípio Central:** "Nunca confie, sempre verifique"

**Premissas:**
- Assuma que a rede interna está comprometida
- Não existe "perímetro seguro" - cada requisição deve ser validada
- Privilégio mínimo em todas as camadas
- Verificação contínua da identidade e contexto
- Segmentação micro-perimetral

**Aplicações Práticas:**
- Toda requisição deve incluir token válido, mesmo entre serviços internos
- Validação de autorização em cada endpoint, não apenas no gateway
- Logs de auditoria para todas as ações, incluindo as permitidas
- Revogação imediata de credenciais comprometidas
- Monitoramento comportamental para detectar anomalias

### 1.2. Defense in Depth

**Princípio Central:** Múltiplas camadas de segurança independentes

**Camadas de Defesa:**

**Camada 1 - Perímetro:**
- TLS 1.3 obrigatório (mesmo internamente)
- Firewall de aplicação (rate limiting, IP whitelisting)
- Proteção DDoS

**Camada 2 - Autenticação:**
- Argon2id para hashing de senhas
- Multi-factor authentication (quando aplicável)
- Account lockout após tentativas falhadas
- Detecção de credential stuffing

**Camada 3 - Autorização:**
- RBAC/ABAC com Casbin
- Policies como código (versionadas)
- Separation of duties
- Least privilege enforcement

**Camada 4 - Aplicação:**
- Input validation rigorosa
- Output encoding
- Proteção contra CSRF, XSS, SQLi
- Secrets management (nunca hardcode)

**Camada 5 - Dados:**
- Encryption at rest
- Encryption in transit
- Database-level permissions
- Backup encryption

**Camada 6 - Monitoramento:**
- Logging estruturado de eventos de segurança
- Alertas para comportamentos anômalos
- Audit trail imutável
- Incident response procedures

### 1.3. Security by Design

**Princípios:**
- Segurança não é feature, é requisito fundamental
- Fail securely: em caso de erro, negar acesso
- Default deny: tudo negado até explicitamente permitido
- Complete mediation: verificar cada acesso, sem cache de decisões críticas
- Psychological acceptability: segurança não deve ser obstáculo à usabilidade

---

## 2. Decisões Arquiteturais Estruturais

### 2.1. Monolito Modular vs Microserviços

**Decisão: Monolito Modular Inicialmente**

**Justificativa Detalhada:**

**Por que NÃO microserviços agora:**
- Ambiente on-premise não tem elasticidade cloud nativa
- Latência de rede interna adiciona overhead desnecessário
- Complexidade operacional: service discovery, mTLS, distributed tracing
- Consistência transacional é crítica em operações de auth
- Time pequeno: overhead de coordenação entre serviços
- Debugging distribuído é exponencialmente mais complexo

**Por que monolito modular:**
- Latência interna sub-milissegundo (function calls vs network calls)
- Transações ACID garantidas nativamente
- Single point of deployment simplifica rollbacks
- Stack traces completas facilitam debugging
- Um processo = menos recursos de infra
- Permite evolução futura para microserviços quando necessário

**Estrutura Modular (Workspace Rust):**

O monolito será organizado em crates separados dentro de um workspace:

**Crates de Domínio (Domain Layer):**
- `domain` - Entidades de negócio puras, sem dependências externas
- Define: User, Role, Permission, Session, AuditEvent
- Regras de negócio validadas via métodos de domínio

**Crates de Infraestrutura (Infrastructure Layer):**
- `infra-database` - Conexão PostgreSQL, migrations, repositories
- `infra-email` - SMTP client, templates de email
- `infra-cache` - Redis client (opcional, para cache de policies)

**Crates de Aplicação (Application Layer):**
- `auth-core` - Lógica de autenticação reutilizável
  - JWT generation/validation
  - Password hashing/verification
  - Token rotation logic
- `authz-core` - Lógica de autorização
  - Casbin enforcer wrapper
  - Policy management
  - Permission checking

**Aplicação Principal:**
- `api-server` - Binário executável Axum
  - Orquestra todos os crates
  - Define rotas HTTP
  - Configura middlewares
  - Gerencia lifecycle da aplicação

**Benefícios desta Estrutura:**
- Separação clara de responsabilidades
- Crates testáveis isoladamente
- Compilação paralela de crates independentes
- Reutilização em outros projetos (ex: CLI admin tool)
- Facilita extração futura para microserviço se necessário
- Dependency inversion: domínio não depende de infra

### 2.2. Quando Migrar para Microserviços

**Sinais que justificam separação:**

**Indicadores Técnicos:**
- Latência de rede interna < 1ms consistente
- Database sharding já implementado
- Time > 50 desenvolvedores
- Módulos com ciclos de release muito diferentes
- Necessidade de tecnologias diferentes por módulo

**Indicadores de Negócio:**
- Múltiplas aplicações consumindo auth (web, mobile apps, APIs B2B)
- Requisitos regulatórios de isolamento
- SLA diferenciados por módulo
- Escalabilidade horizontal é gargalo comprovado

**Estratégia de Extração:**
1. `auth-core` crate vira `identity-provider` service
2. Comunicação via gRPC (melhor que REST para serviços internos)
3. mTLS obrigatório entre serviços
4. Distributed tracing com OpenTelemetry
5. Circuit breakers para resiliência

---

## 3. Stack Tecnológica e Justificativas

### 3.1. Backend: Rust

**Por que Rust:**

**Segurança de Memória:**
- Memory safety garantida em compile-time
- Elimina classes inteiras de vulnerabilidades (buffer overflow, use-after-free, data races)
- Critical para sistema de autenticação onde exploits de memória são vetores de ataque comuns

**Performance Previsível:**
- Sem garbage collection pauses
- Zero-cost abstractions
- Performance comparável a C/C++ mas com segurança
- Essencial para operações criptográficas intensivas

**Concorrência Segura:**
- Ownership model previne data races
- Async/await nativo para I/O bound operations
- Threads seguras para CPU-bound (hashing Argon2)

**Ecossistema Moderno:**
- Cargo para dependency management
- Crates.io com auditoria de segurança
- Tooling excelente (rustfmt, clippy)
- Strong typing previne bugs em compile-time

### 3.2. Web Framework: Axum

**Por que Axum:**

**Performance:**
- Baseado em Hyper (HTTP client/server mais rápido em Rust)
- Overhead mínimo sobre raw TCP
- Async nativo (Tokio runtime)

**Ergonomia:**
- Type-safe extractors
- Composição de middlewares via Tower
- Error handling ergonômico
- Integração natural com ecossistema Tower

**Extensibilidade:**
- Middleware system baseado em Tower Layers
- Fácil adicionar CORS, rate limiting, compression, etc.
- Custom extractors para lógica de negócio

### 3.3. Database: PostgreSQL

**Por que PostgreSQL:**

**ACID Compliance:**
- Transações confiáveis para operações críticas
- Isolamento previne race conditions
- Rollback automático em erros

**Features Avançadas:**
- JSONB para metadados flexíveis
- Row-level security (RLS) adicional
- Full-text search
- Extensões (pgcrypto, uuid-ossp)

**Maturidade:**
- 25+ anos de desenvolvimento
- Track record comprovado em ambientes enterprise
- Documentação extensa
- Comunidade ativa

**Performance:**
- Índices sofisticados (B-tree, Hash, GiST, GIN)
- Query planner otimizado
- Connection pooling nativo
- Particionamento de tabelas para escalabilidade

### 3.4. SQL Layer: SQLx

**Por que SQLx:**

**Compile-time Verification:**
- Queries verificadas contra schema real do banco
- Previne typos e incompatibilidades de tipo
- Refactoring seguro: mudanças no schema quebram compilação

**Async Nativo:**
- Non-blocking I/O
- Múltiplas queries concorrentes sem threads
- Integração perfeita com Tokio

**Prepared Statements Automáticos:**
- Proteção contra SQL injection por design
- Performance: query parsing uma vez, execução múltiplas

**Type Safety:**
- Mapeamento automático de tipos SQL <-> Rust
- Compile-time errors para conversões inválidas

### 3.5. Password Hashing: Argon2

**Por que Argon2id:**

**Padrão Ouro Atual:**
- Vencedor do Password Hashing Competition 2015
- Recomendado por OWASP, NIST, IETF

**Memory-Hard:**
- Resistente a ataques com GPUs e ASICs
- Custo de memória configurável
- Dificulta brute-force massivo

**Configurável:**
- Time cost: número de iterações
- Memory cost: quantidade de RAM necessária
- Parallelism: número de threads
- Permite ajuste para hardware específico

**Variantes:**
- Argon2d: data-dependent, melhor contra GPUs
- Argon2i: data-independent, resistente a side-channel
- **Argon2id: híbrido, recomendado (combina benefícios)**

### 3.6. Authorization: Casbin-rs

**Por que Casbin:**

**Policy as Code:**
- Policies em arquivos texto versionados
- Review de mudanças via pull requests
- Rollback trivial de policies problemáticas

**Flexibilidade:**
- Suporta múltiplos modelos: RBAC, ABAC, ACL, RESTful
- Predicados complexos em policies
- Herança de roles

**Separação de Concerns:**
- Lógica de autorização desacoplada do código de negócio
- Mudanças em permissions sem redeploy
- Testável isoladamente

**Adapter PostgreSQL:**
- Policies persistidas no banco
- Auditoria de mudanças
- Distribuição automática entre instâncias

### 3.7. Frontend: Angular 17+

**Por que Angular:**

**Framework Completo:**
- Routing, HTTP client, forms, testing - tudo built-in
- Não precisa montar stack de bibliotecas
- Decisões arquiteturais já tomadas

**TypeScript Nativo:**
- Type safety end-to-end
- Refactoring seguro
- Intellisense poderoso

**Enterprise Ready:**
- Suporte a aplicações grandes e complexas
- Padrões estabelecidos (módulos, services, guards)
- CLI poderoso para scaffolding

**SSR e Hydration:**
- Angular Universal para renderização servidor
- Hydration incremental/híbrida nativa (v16+)
- Performance otimizada

**Longevidade:**
- Mantido pelo Google
- Release schedule previsível
- Longo suporte para versões LTS

---

## 4. Arquitetura de Segurança

### 4.1. Modelo de Ameaças

**Ameaças Consideradas:**

**Externas:**
- Brute force attacks em endpoints de login
- Credential stuffing (senhas vazadas de outros sites)
- Token theft (XSS, MITM, malware)
- SQL injection
- CSRF attacks
- DDoS

**Internas:**
- Insider threats (funcionário malicioso)
- Privilege escalation
- Lateral movement após comprometimento
- Data exfiltration

**Supply Chain:**
- Dependências comprometidas (crates, npm packages)
- Backdoors em bibliotecas
- Malicious updates

### 4.2. Mitigações por Camada

**Camada de Rede:**
- TLS 1.3 obrigatório em todas as conexões (interno e externo)
- Certificados válidos com rotação regular
- HSTS headers
- Certificate pinning (opcional, para mobile)

**Camada de Aplicação:**
- Input validation em todos os endpoints
- Output encoding para prevenir XSS
- Prepared statements para prevenir SQLi
- Rate limiting por IP e por usuário
- CORS restritivo
- Content Security Policy headers

**Camada de Autenticação:**
- Argon2id com parâmetros robustos
- Account lockout progressivo
- CAPTCHA após N tentativas falhadas
- Monitoring de padrões de login anômalos
- Session binding (IP, User-Agent)

**Camada de Autorização:**
- Default deny em todas as policies
- Least privilege enforcement
- Separation of duties para operações críticas
- Approval workflows para mudanças de permissão
- Audit log de todas as decisões de autorização

**Camada de Dados:**
- Encryption at rest para campos sensíveis
- Database-level permissions adicionais
- Backup encryption
- Audit triggers no banco

### 4.3. Princípios de Criptografia

**Algoritmos Aprovados:**

**Hashing:**
- Senhas: Argon2id exclusivamente
- Tokens: SHA-256 para storage (tokens opacos)
- Integridade: BLAKE3 para checksums

**Assinatura de JWT:**
- **Preferido: EdDSA (Ed25519)**
  - Chaves menores (32 bytes)
  - Mais rápido que RSA
  - Resistente a timing attacks
  - Padrão moderno
- Alternativa: ECDSA (P-256) se EdDSA não disponível
- Evitar: HMAC (chave simétrica = menos seguro para distribuição)

**Encryption (se necessário para PII):**
- AES-256-GCM para dados em repouso
- ChaCha20-Poly1305 alternativa performática

**Key Management:**
- Chaves nunca hardcoded
- Armazenadas em variáveis de ambiente ou secrets manager
- Rotação de chaves JWT a cada 90 dias
- Múltiplas chaves ativas para graceful rotation
- Backup de chaves em local seguro (hardware security module idealmente)

**Randomness:**
- Usar sempre `OsRng` (gerador criptograficamente seguro do OS)
- Nunca usar `rand::thread_rng()` para tokens de segurança
- Mínimo 256 bits de entropia para tokens críticos

---

## 5. Modelo de Dados e Persistência

### 5.1. Princípios de Design do Schema

**Normalização:**
- Terceira forma normal (3NF) para consistência
- Desnormalização estratégica apenas para performance crítica
- Audit tables separadas (append-only)

**Integridade Referencial:**
- Foreign keys sempre declaradas
- Cascade deletes apenas onde apropriado
- Soft deletes para entidades de negócio

**Indexação:**
- Índices em todas as foreign keys
- Índices compostos para queries comuns
- Índices parciais (filtered) para reduzir tamanho
- Análise regular de query plans

**Auditoria:**
- Timestamps: created_at, updated_at, deleted_at
- Tracking: created_by, updated_by
- Histórico: tabelas de audit log separadas

### 5.2. Entidades Principais

**Users:**
- Identificação: ID (UUID), email (único)
- Autenticação: password_hash, salt (se não embutido no hash)
- Estado: is_active, is_email_verified, is_locked
- Metadados: created_at, updated_at, deleted_at
- Security: failed_login_attempts, locked_until, last_password_change
- Profile: first_name, last_name, phone (opcional)

**Refresh Tokens:**
- Identificação: token_hash (primary key)
- Relacionamento: user_id (foreign key)
- Lifecycle: expires_at, created_at
- Revogação: revoked (boolean), revoked_at, revoked_reason
- Security tracking: user_agent, ip_address, device_id
- **Token family**: token_family (UUID), parent_token_id (para detecção de roubo)

**Email Verification Tokens:**
- Identificação: token_hash
- Relacionamento: user_id
- Lifecycle: expires_at (curta, ex: 24h), created_at, used_at
- One-time use enforcement

**Password Reset Tokens:**
- Identificação: token_hash
- Relacionamento: user_id
- Lifecycle: expires_at (muito curta, ex: 1h), created_at, used_at
- Security: ip_address do solicitante
- Revogação: invalidar ao usar ou ao expirar

**Audit Logs:**
- Identificação: ID (serial/UUID)
- Relacionamento: user_id (nullable, para eventos de sistema)
- Classificação: event_type, event_category, severity
- Conteúdo: description, metadata (JSONB)
- Contexto: ip_address, user_agent, request_id
- Resultado: success (boolean), error_message
- Timestamp: created_at (immutable)

**Casbin Rules:**
- Tabela gerenciada pelo Casbin adapter
- Colunas: ptype, v0, v1, v2, v3, v4, v5
- Índices customizados para queries de enforcement

### 5.3. Estratégias de Particionamento

**Audit Logs:**
- Particionar por data (monthly)
- Arquivar partições antigas para cold storage
- Reter online apenas últimos 6-12 meses

**Refresh Tokens:**
- Limpeza automática de tokens expirados
- Job periódico (ex: diariamente) para deletar tokens > 30 dias após expiração

### 5.4. Backup e Recovery

**Backup Strategy:**
- Full backup diário
- Incremental backup a cada 6 horas
- Transaction logs para point-in-time recovery
- Retenção: 30 dias online, 1 ano archived

**Encryption:**
- Backups sempre encriptados (AES-256)
- Chaves de backup separadas das chaves de aplicação
- Teste regular de restore procedures

**Disaster Recovery:**
- RTO (Recovery Time Objective): < 4 horas
- RPO (Recovery Point Objective): < 15 minutos
- Documentação detalhada de procedures
- Drill de recovery trimestral

---

## 6. Gestão de Autenticação

### 6.1. Fluxo de Registro (Sign-up)

**Etapas:**
1. Usuário submete formulário com email, senha, dados pessoais
2. Validação de input no backend (formato, força da senha)
3. Verificar email não duplicado
4. Hash da senha com Argon2id
5. Criar usuário no banco (status: pending_verification)
6. Gerar token de verificação de email
7. Enviar email com link de verificação
8. Retornar sucesso (sem revelar se email já existe - prevenir enumeração)
9. Logar evento de registro em audit log

**Validação de Senha:**
- Mínimo 8 caracteres (recomendado 12+)
- Pelo menos: 1 maiúscula, 1 minúscula, 1 número, 1 caractere especial
- Não permitir senhas comuns (checklist de senhas vazadas)
- Não permitir senhas que contenham o email
- Feedback claro sobre requisitos

**Rate Limiting:**
- Máximo 5 tentativas de registro por IP por hora
- CAPTCHA após 3 tentativas

### 6.2. Fluxo de Login

**Etapas:**
1. Usuário submete email e senha
2. Rate limiting check (prevenir brute force)
3. Buscar usuário por email
4. Verificar se conta está ativa e não bloqueada
5. Verificar senha com Argon2
6. Se inválida:
   - Incrementar contador de falhas
   - Se atingir threshold: bloquear conta temporariamente
   - Logar tentativa falhada
   - Retornar erro genérico (não revelar se email existe)
7. Se válida:
   - Resetar contador de falhas
   - Verificar se email foi verificado
   - Gerar access token (JWT curto)
   - Gerar refresh token (opaco, longo)
   - Salvar refresh token no banco (hash)
   - Atualizar last_login_at, last_login_ip
   - Logar login bem-sucedido
   - Retornar tokens

**Account Lockout:**
- Bloquear após 5 tentativas falhadas
- Lockout progressivo: 5min, 15min, 1h, 24h
- Notificar usuário via email sobre bloqueio
- Permitir unlock via link no email ou contato com suporte

**Security Monitoring:**
- Alertar sobre logins de IPs/localizações incomuns
- Detectar padrões de credential stuffing
- Notificar usuário sobre novo dispositivo

### 6.3. Fluxo de Logout

**Single Device Logout:**
1. Usuário clica em logout
2. Frontend envia refresh token ao backend
3. Backend marca token como revogado
4. Frontend limpa access token da memória
5. Frontend limpa cookie de refresh token
6. Redirecionar para página de login

**Logout de Todos os Dispositivos:**
1. Usuário solicita (geralmente após suspeita de comprometimento)
2. Backend revoga todos os refresh tokens do usuário
3. Invalida todas as sessões ativas
4. Notifica usuário via email
5. Força re-autenticação em todos os dispositivos

### 6.4. Verificação de Email

**Geração de Token:**
- Token criptograficamente seguro (32+ bytes)
- Hash SHA-256 para storage
- Expiração curta (24 horas)
- One-time use enforcement

**Link de Verificação:**
- Formato: `https://app.domain.com/verify-email/{token}`
- Token na URL (não em query string para evitar leaks em logs)

**Processo:**
1. Usuário clica no link no email
2. Frontend extrai token e chama API
3. Backend valida token (não expirado, não usado)
4. Marca email como verificado
5. Marca token como usado
6. Logar evento
7. Redirecionar para dashboard ou login

**Reenvio de Email:**
- Permitir reenvio após 1 minuto
- Invalidar token anterior ao gerar novo
- Limitar a 3 reenvios por hora

### 6.5. Password Reset

**Solicitação de Reset:**
1. Usuário informa email
2. Se email existe:
   - Gerar token de reset (expiração muito curta: 1h)
   - Enviar email com link
   - Logar solicitação
3. Sempre retornar mensagem genérica (prevenir enumeração de emails)

**Reset de Senha:**
1. Usuário clica no link com token
2. Frontend apresenta formulário de nova senha
3. Usuário submete nova senha
4. Backend valida token
5. Valida força da nova senha
6. Hash da nova senha
7. Atualizar password_hash no banco
8. Marcar token como usado
9. **Revogar todos os refresh tokens** (força logout em todos dispositivos)
10. Logar reset de senha
11. Enviar email de confirmação
12. Redirecionar para login

**Security Considerations:**
- Nunca revelar se email existe no sistema
- Tokens de reset extremamente curtos (1h máximo)
- Invalidar token após uso
- Forçar logout global após mudança de senha
- Notificar usuário via email sobre mudança

---

## 7. Gestão de Autorização

### 7.1. Modelo de Autorização

**Escolha: RBAC (Role-Based Access Control) com suporte a ABAC**

**Justificativa:**
- RBAC é suficiente para 90% dos casos
- Simples de entender e manter
- Escalável para centenas de roles
- Casbin permite evoluir para ABAC quando necessário

**Hierarquia:**
```
User → Roles → Permissions → Resources
```

**Exemplo de Roles:**
- superadmin: acesso total ao sistema
- admin: gerenciamento de usuários e configurações
- moderator: moderação de conteúdo
- user: acesso básico
- guest: acesso read-only limitado

### 7.2. Casbin Configuration

**Model File (model.conf):**
- Define a estrutura de políticas
- Request definition: subject, object, action
- Policy definition: regras de permissão
- Role definition: herança de roles
- Effect: allow ou deny
- Matchers: lógica de matching (wildcards, regex)

**Policy File/Database:**
- Policies armazenadas no PostgreSQL
- Formato: `p, role, resource, action`
- Grouping: `g, user, role`
- Versionamento via migrations

**Enforcement:**
- Middleware Axum para checar permissões
- Cache de decisões para performance (cuidado com invalidação)
- Fail-secure: em caso de erro, negar acesso

### 7.3. Granularidade de Permissões

**Níveis:**

**Resource-level:**
- `/users` → read, write, delete
- `/admin/settings` → read, write
- `/reports` → read, export

**Object-level (quando necessário):**
- `user:{id}` → próprio usuário pode editar
- `post:{id}` → apenas autor pode deletar

**Attribute-based (casos especiais):**
- Permitir acesso apenas durante horário comercial
- Permitir apenas de IPs específicos
- Permitir apenas se MFA ativado

### 7.4. Gestão de Policies

**Princípios:**
- Policies como código (versionadas no Git)
- Code review obrigatório para mudanças
- Testes automatizados de policies
- Rollback fácil em caso de problemas

**Deployment de Policies:**
- Migrations do banco para policies iniciais
- API administrativa para mudanças dinâmicas
- Sincronização automática entre instâncias
- Validação de policies antes de aplicar

**Auditoria:**
- Logar todas as mudanças de policies
- Quem mudou, quando, o que mudou
- Histórico completo de policies

---

## 8. Gestão de Sessões e Tokens

### 8.1. Modelo Híbrido de Tokens

**Decisão Arquitetural: Token Pair Pattern**

**Access Token (JWT):**
- **Propósito:** Carregar claims do usuário (id, role, permissions)
- **Duração:** Muito curta (15 minutos)
- **Formato:** JWT assinado com EdDSA
- **Armazenamento Cliente:** 
  - Opção A (SPA): Memória (variável JavaScript)
  - Opção B (SSR): Cookie HttpOnly + Secure + SameSite=Strict
- **Armazenamento Servidor:** Nenhum (stateless)
- **Claims Incluídos:**
  - sub: user_id
  - email: email do usuário
  - role: role principal
  - permissions: lista de permissões (opcional, se não muito grande)
  - iat: issued at
  - exp: expiration
  - jti: JWT ID (para revogação futura)

**Refresh Token:**
- **Propósito:** Renovar access token sem re-autenticação
- **Duração:** Longa (7-30 dias)
- **Formato:** String opaca criptograficamente aleatória (256 bits)
- **Armazenamento Cliente:** Exclusivamente cookie HttpOnly + Secure + SameSite=Strict
- **Armazenamento Servidor:** Hash SHA-256 do token no PostgreSQL
- **Security Feature:** Token family para detecção de roubo

### 8.2. Refresh Token Rotation

**Princípio:** Cada uso do refresh token gera um novo par de tokens

**Fluxo:**
1. Cliente envia refresh token (via cookie)
2. Backend valida token:
   - Existe no banco
   - Não está revogado
   - Não expirou
   - **Verifica se não foi usado** (já revogado = possível roubo)
3. Se válido:
   - Revoga token antigo
   - Gera novo access token
   - Gera novo refresh token (mesma família)
   - Salva novo refresh token no banco
   - Retorna novo access token + seta novo refresh token em cookie
4. Se inválido ou já usado:
   - **Revoga toda a família de tokens**
   - Loga alerta de segurança
   - Notifica usuário via email
   - Força logout em todos dispositivos

### 8.3. Token Family e Detecção de Roubo

**Conceito:**
- Todos os refresh tokens gerados a partir de um login formam uma "família"
- Família identificada por UUID único
- Cada token conhece seu "pai" (token que o gerou)

**Detecção de Roubo:**
- Se um token já revogado for apresentado, significa:
  - Atacante roubou token e está usando
  - OU usuário legítimo tentou usar token antigo
- Ação: invalidar toda a família por precaução
- Usuário legítimo apenas precisa fazer login novamente

**Estrutura:**
```
Login (Token Family: abc-123)
  → Refresh Token 1
      → Refresh Token 2 (revoga 1)
          → Refresh Token 3 (revoga 2)
              → Refresh Token 4 (revoga 3)

Se Token 2 for reutilizado = ALERTA
```

### 8.4. Cookie Configuration

**Flags Obrigatórias:**
- `HttpOnly`: JavaScript não pode acessar (previne XSS)
- `Secure`: Apenas transmitido via HTTPS
- `SameSite=Strict`: Previne CSRF
- `Path=/api/auth`: Limita scope do cookie
- `Max-Age`: Tempo de expiração

**Domain Configuration:**
- Se API e frontend em subdomínios diferentes:
  - `Domain=.example.com` para compartilhar cookie
- Se API e frontend no mesmo domínio:
  - Omitir Domain (mais seguro)

### 8.5. Revogação de Tokens

**Cenários de Revogação:**

**Logout Voluntário:**
- Revoga apenas o refresh token atual
- Mantém outros dispositivos ativos

**Logout de Todos Dispositivos:**
- Revoga todos refresh tokens do usuário
- Após mudança de senha (obrigatório)
- A pedido do usuário (suspeita de comprometimento)

**Revogação Administrativa:**
- Admin pode revogar tokens de qualquer usuário
- Útil para suspensão de conta
- Logar motivo da revogação

**Revogação Automática:**
- Token expirado (cleanup job)
- Detecção de roubo (família inteira)
- Padrão de uso suspeito (geo-anomalia, velocity check)

### 8.6. Stateless vs Stateful Trade-offs

**Access Token Stateless:**
- ✅ Não requer lookup no banco a cada request
- ✅ Escala horizontalmente sem compartilhar estado
- ✅ Performance: sub-milissegundo para verificar assinatura
- ❌ Não pode ser revogado antes da expiração
- ❌ Claims desatualizados até expirar

**Solução Híbrida (Recomendada):**
- Access token stateless para performance
- Refresh token stateful para controle
- Access token curto minimiza janela de claims desatualizados
- Blacklist opcional para JTIs críticos (cache Redis)

---

## 9. Fluxos de Segurança Críticos

### 9.1. Multi-Factor Authentication (MFA)

**Quando Implementar:**
- Obrigatório para administradores
- Opcional para usuários regulares
- Obrigatório após detecção de login suspeito

**Métodos Recomendados:**

**TOTP (Time-based One-Time Password):**
- Compatível com Google Authenticator, Authy
- Secret armazenado encriptado no banco
- Janela de tempo: 30 segundos
- Permitir 1-2 time windows de tolerância (atraso/adiantamento)

**Backup Codes:**
- Gerar 10 códigos de uso único no setup do MFA
- Usuário deve salvar em local seguro
- Cada código usado é invalidado
- Permitir regeneração (com autenticação forte)

**Fluxo de MFA:**
1. Usuário faz login com credenciais
2. Se MFA habilitado: não emitir tokens ainda
3. Solicitar código TOTP
4. Validar código
5. Se válido: emitir tokens normalmente
6. Se inválido: limitar tentativas, bloquear após 3 falhas

### 9.2. Detecção de Anomalias

**Sinais de Alerta:**

**Login de Localização Incomum:**
- IP de país diferente do usual
- Velocity check: login em locais distantes em curto espaço de tempo
- Ação: Exigir MFA adicional ou bloquear e notificar

**Padrão de Uso Anômalo:**
- Múltiplos logins falhados seguidos de sucesso (possível brute force)
- Acesso a recursos nunca acessados antes
- Volume anormal de requisições
- Ação: Rate limit mais agressivo, challenge adicional

**Device Fingerprinting:**
- Trackear combinação de User-Agent, screen resolution, timezone
- Novo device: notificar usuário e exigir confirmação
- Device conhecido: login mais suave

### 9.3. Session Management

**Session Binding:**
- Associar sessão a IP e User-Agent
- Se mudança detectada: invalidar e re-autenticar
- Tolerância: alguns proxies podem mudar IP

**Session Timeout:**
- Absolute timeout: 8 horas (configurável)
- Idle timeout: 30 minutos (configurável)
- Warning antes do timeout (1 minuto antes)

**Concurrent Sessions:**
- Limitar número de sessões simultâneas por usuário
- Default: 5 dispositivos
- Permitir usuário gerenciar sessões ativas
- Mostrar: device, location, last active

### 9.4. Rate Limiting Strategy

**Níveis de Rate Limiting:**

**Global:**
- Limite por IP: 100 req/min
- Protege contra DDoS básico

**Por Endpoint:**
- Login: 5 tentativas/min por IP
- Register: 3 tentativas/hora por IP
- Password reset: 3 tentativas/hora por email
- Refresh token: 10 tentativas/min por usuário

**Por Usuário:**
- API calls: 1000 req/hora após autenticação
- Diferenciado por role (admin = mais permissivo)

**Implementação:**
- Sliding window algorithm (mais justo que fixed window)
- Storage: Redis (performance) ou PostgreSQL (simplicidade)
- Response: HTTP 429 com Retry-After header

### 9.5. CAPTCHA Integration

**Quando Usar:**
- Após 3 tentativas de login falhadas
- Em registro de novo usuário
- Em solicitação de password reset

**Implementação On-Premise:**
- Não usar serviços cloud (reCAPTCHA é Google)
- Alternativas: hCaptcha (self-hosted), custom challenge-response
- Balancear segurança com usabilidade

---

## 10. User Management e Administração

### 10.1. CRUD de Usuários

**Operações:**

**Create (Registro):**
- Self-service via formulário público
- Ou criação administrativa (admin dashboard)
- Criação admin: permite setar role inicial, pular verificação de email

**Read:**
- Usuário pode visualizar próprio perfil
- Admin pode listar e buscar todos usuários
- Filtros: por role, status, data de criação
- Paginação obrigatória (nunca retornar todos de uma vez)

**Update:**
- Usuário pode atualizar próprio perfil (nome, telefone, avatar)
- Mudança de email requer re-verificação
- Mudança de senha requer senha atual
- Admin pode atualizar qualquer usuário
- Admin pode mudar role (requer approval de outro admin)

**Delete:**
- Soft delete preferível (marcar como deleted_at)
- Hard delete apenas para compliance (GDPR - direito ao esquecimento)
- Antes de deletar: verificar dependências (posts, comentários, etc.)
- Anonimizar em vez de deletar quando possível

### 10.2. Painel Administrativo

**Funcionalidades:**

**Dashboard:**
- Estatísticas: total usuários, novos hoje/semana/mês
- Gráficos: crescimento de usuários, distribuição de roles
- Alertas: logins falhados, contas bloqueadas
- Métricas de segurança: tentativas de brute force, tokens revogados

**Gestão de Usuários:**
- Listagem com filtros e busca
- Visualizar detalhes de usuário
- Editar perfil e roles
- Suspender/reativar conta
- Resetar senha (gera link de reset)
- Visualizar sessões ativas
- Forçar logout

**Gestão de Roles e Permissions:**
- Listar roles e suas permissões
- Criar/editar/deletar roles
- Atribuir permissões a roles
- Visualizar quais usuários têm cada role
- Dry-run de mudanças de policy (testar antes de aplicar)

**Audit Logs Viewer:**
- Busca por usuário, tipo de evento, data
- Filtros avançados
- Export para CSV/JSON
- Visualização de eventos relacionados (trail de ações)

**Segurança:**
- Todas as ações administrativas requerem MFA
- Approval workflow para mudanças críticas (mudar role de outro admin)
- Audit log de todas as ações administrativas
- Session timeout mais curto para admins (15 min idle)

### 10.3. Gestão de Roles

**Role Hierarchy:**
- Definir se roles têm hierarquia (admin > moderator > user)
- Herança de permissões (role filho herda permissões do pai)

**Built-in Roles:**
- superadmin: não pode ser deletado, acesso total
- admin: gerenciamento de sistema
- user: role padrão para novos usuários
- guest: acesso read-only

**Custom Roles:**
- Permitir criação de roles customizadas
- Nomeação clara e descritiva
- Documentação do propósito da role

**Permission Assignment:**
- Permissões sempre atribuídas a roles, não diretamente a usuários
- Exceção: permissões especiais temporárias (edge cases)

### 10.4. User Suspension e Reativação

**Motivos para Suspensão:**
- Violação de termos de uso
- Atividade suspeita
- Solicitação do próprio usuário
- Ordem legal

**Processo de Suspensão:**
1. Admin marca usuário como suspended
2. Revogar todos os tokens imediatamente
3. Bloquear novos logins
4. Logar motivo da suspensão
5. Notificar usuário via email (se apropriado)
6. Permitir período de recurso

**Reativação:**
1. Admin marca usuário como active
2. Usuário deve fazer novo login (tokens foram revogados)
3. Logar reativação
4. Notificar usuário

---

## 11. Logging, Auditoria e Monitoramento

### 11.1. Structured Logging

**Princípios:**
- Logs estruturados (JSON) para parsing fácil
- Níveis apropriados: ERROR, WARN, INFO, DEBUG, TRACE
- Context: sempre incluir request_id para correlação
- Async logging para não bloquear requests

**Informações a Logar:**

**Por Request:**
- Request ID (UUID)
- Method e Path
- User ID (se autenticado)
- IP Address
- User-Agent
- Timestamp
- Response status code
- Response time
- Erro (stack trace se aplicável)

**Eventos de Segurança:**
- Login sucesso/falha
- Logout
- Token refresh
- Password change
- Email verification
- Role change
- Permission grant/revoke
- Account lockout/unlock
- MFA setup/disable

**Informações Proibidas:**
- Senhas (hash ou plaintext)
- Tokens completos (logar apenas prefixo)
- Números de cartão de crédito
- Dados sensíveis de PII (exceto se necessário e encriptado)

### 11.2. Audit Trail

**Propósito:**
- Compliance e investigação de incidentes
- Responder: quem fez o quê, quando, onde, por quê

**Tabela de Audit Logs:**
- Append-only (nunca deletar ou modificar)
- Particionada por data
- Retenção: mínimo 1 ano, idealmente 7 anos para compliance

**Eventos Auditáveis:**

**Autenticação:**
- Login attempt (success/failure)
- Logout
- Password change
- Password reset request
- MFA enable/disable

**Autorização:**
- Access denied (403)
- Permission check failure
- Role assignment
- Policy change

**User Management:**
- User creation
- User update
- User deletion
- Account suspension
- Email verification

**Administrative:**
- Todas as ações no painel admin
- Configuration changes
- Policy deployments
- User impersonation (se existir)

**Metadados por Evento:**
- User ID (ator)
- Target (usuário ou recurso afetado)
- Action
- Timestamp
- IP Address
- User-Agent
- Request ID
- Result (success/failure)
- Error message (se falha)
- Additional context (JSON)

### 11.3. Monitoramento e Alertas

**Métricas a Monitorar:**

**Performance:**
- Request latency (p50, p95, p99)
- Throughput (requests/sec)
- Error rate
- Database query time
- Token generation time (Argon2 pode ser lento)

**Segurança:**
- Failed login rate
- Account lockouts
- Token revocations
- CSRF violations
- Rate limit hits
- Anomalous login patterns

**Saúde do Sistema:**
- CPU e memória
- Database connections
- Disk space
- Database replication lag (se aplicável)

**Alertas Críticos:**
- Error rate > 5%
- Failed login rate spike
- Database connection pool exhausted
- Disk space < 10%
- Certificate expiring < 7 days

**Ferramentas On-Premise:**
- Prometheus para métricas
- Grafana para visualização
- Loki para logs (alternativa ao ELK)
- Alertmanager para alertas
- On-call rotation para resposta a incidentes

### 11.4. Log Rotation e Retenção

**Estratégia:**
- Rotação diária ou quando atingir tamanho máximo
- Compressão de logs antigos (gzip)
- Arquivamento para cold storage após 90 dias
- Deleção após período de retenção

**Retenção por Tipo:**
- Application logs: 90 dias online, 1 ano archive
- Security logs: 1 ano online, 7 anos archive
- Audit logs: permanente (ou conforme compliance)
- Access logs: 30 dias online, 1 ano archive

---

## 12. Arquitetura Frontend

### 12.1. Single Page Application (SPA) vs Server-Side Rendering (SSR)

**Decisão: Hybrid Approach**

**SPA para Aplicação Autenticada:**
- Melhor experiência após login
- Interatividade sem page reloads
- Estado gerenciado client-side

**SSR para Páginas Públicas:**
- Landing page, login, register
- Melhor SEO
- Faster first contentful paint
- Funciona sem JavaScript

**Angular Universal:**
- Renderiza inicial HTML no servidor
- Hidrata componentes no cliente
- Lazy loading de módulos não críticos

### 12.2. Estrutura Modular

**Core Module:**
- Services singleton (AuthService, HttpClient configurado)
- Guards (AuthGuard, RoleGuard)
- Interceptors (AuthInterceptor, ErrorInterceptor)
- Modelos de dados (User, Role, etc.)

**Shared Module:**
- Componentes reutilizáveis (Header, Footer, Loading, Alert)
- Diretivas customizadas
- Pipes
- Sem serviços (apenas declarações)

**Feature Modules:**
- Auth Module: login, register, forgot-password, verify-email
- User Profile Module: view/edit profile, change password
- Admin Module: user management, audit logs, settings
- Lazy loaded para performance

**Layout Modules:**
- Public Layout: para páginas não autenticadas
- Authenticated Layout: com sidebar, header
- Admin Layout: layout específico para admin

### 12.3. State Management

**Estratégia:**
- NgRx Signal Store para estado global reativo
- Signals para estado local de componentes
- Evitar over-engineering: nem tudo precisa estar no store

**Estado Global:**
- Current user
- Authentication status
- User permissions/roles
- Global notifications/alerts

**Estado Local:**
- Form state
- UI state (modals, tabs)
- Loading states

### 12.4. Routing e Guards

**Route Structure:**
```
/
├── /auth
│   ├── /login
│   ├── /register
│   ├── /verify-email/:token
│   ├── /forgot-password
│   └── /reset-password/:token
├── /dashboard (protected)
├── /profile (protected)
└── /admin (protected, role: admin)
    ├── /users
    ├── /roles
    └── /audit-logs
```

**Guards:**

**AuthGuard:**
- Verifica se usuário está autenticado
- Redireciona para /login se não
- Salva returnUrl para redirecionar após login

**RoleGuard:**
- Verifica se usuário tem role necessária
- Redireciona para /unauthorized se não
- Aceita array de roles permitidas

**Lazy Loading:**
- Admin module só carrega se usuário for admin
- Reduz bundle inicial
- Melhora performance

### 12.5. Forms e Validação

**Reactive Forms:**
- Type-safe
- Validação declarativa
- Fácil testar

**Validação Client-side:**
- Mesmas regras do backend
- Feedback imediato ao usuário
- Não substitui validação do backend

**Custom Validators:**
- Password strength
- Email format
- Async validators (verificar email disponível)

**Error Handling:**
- Mensagens de erro claras e específicas
- Internacionalizadas
- Acessíveis (ARIA labels)

---

## 13. Comunicação Backend-Frontend

### 13.1. API Design

**Princípios REST:**
- Recursos como substantivos: `/users`, `/roles`
- Verbos HTTP: GET, POST, PUT, DELETE, PATCH
- Status codes apropriados: 200, 201, 400, 401, 403, 404, 500
- HATEOAS opcional (links para recursos relacionados)

**Versionamento:**
- Versão na URL: `/api/v1/users`
- Ou header: `Accept: application/vnd.api.v1+json`
- Manter v1 até todos clientes migrarem

**Pagination:**
- Obrigatória para listas
- Query params: `?page=1&page_size=20`
- Response incluir metadata: total, pages, current_page
- Default page_size: 20, max: 100

**Filtering e Sorting:**
- Query params: `?status=active&sort=-created_at`
- Prefixo `-` para ordem descendente
- Múltiplos filtros com operadores: `?age_gte=18&age_lte=65`

### 13.2. Request/Response Format

**Request Headers:**
- `Authorization: Bearer {access_token}`
- `Content-Type: application/json`
- `X-Request-ID: {uuid}` (para rastreamento)
- `Accept-Language: pt-BR` (i18n)

**Response Format Padrão:**
```json
{
  "success": true,
  "data": {...},
  "message": "Operação bem-sucedida",
  "timestamp": "2024-01-01T12:00:00Z",
  "request_id": "uuid"
}
```

**Error Response Format:**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_CREDENTIALS",
    "message": "Email ou senha inválidos",
    "details": [...],
    "timestamp": "2024-01-01T12:00:00Z",
    "request_id": "uuid"
  }
}
```

### 13.3. Error Handling

**Status Codes:**
- 400 Bad Request: validação falhou
- 401 Unauthorized: não autenticado
- 403 Forbidden: autenticado mas sem permissão
- 404 Not Found: recurso não existe
- 409 Conflict: conflito (email duplicado)
- 422 Unprocessable Entity: validação de negócio falhou
- 429 Too Many Requests: rate limit
- 500 Internal Server Error: erro inesperado

**Error Codes Customizados:**
- `EMAIL_ALREADY_EXISTS`
- `INVALID_CREDENTIALS`
- `ACCOUNT_LOCKED`
- `EMAIL_NOT_VERIFIED`
- `TOKEN_EXPIRED`
- `INSUFFICIENT_PERMISSIONS`

**Frontend Error Handling:**
- Interceptor captura erros globalmente
- Mostra toast/notification para erros
- Log errors para análise
- Retry automático para erros transientes (500, 503)

### 13.4. CORS Configuration

**Produção:**
- Whitelist específico de origins: `https://app.example.com`
- Não usar `*` wildcard
- Credentials allowed: true (para cookies)

**Desenvolvimento:**
- Permitir `http://localhost:4200` (Angular dev server)
- Considerar proxy reverso para evitar CORS

**Headers:**
- `Access-Control-Allow-Origin`
- `Access-Control-Allow-Methods`
- `Access-Control-Allow-Headers`
- `Access-Control-Allow-Credentials`
- `Access-Control-Max-Age` (cache de preflight)

### 13.5. HTTP Interceptors

**AuthInterceptor:**
- Adiciona Authorization header em toda requisição
- Lê access token do storage
- Ignora requisições para endpoints públicos

**RefreshInterceptor:**
- Captura 401 responses
- Pausa requisição original
- Tenta refresh token
- Se sucesso: retry requisição original com novo token
- Se falha: logout e redireciona para login
- Evita múltiplos refresh simultâneos (lock)

**ErrorInterceptor:**
- Captura erros HTTP
- Formata mensagens de erro
- Mostra notificações
- Loga erros

**LoadingInterceptor:**
- Mostra loading spinner global
- Contador de requisições pendentes
- Hide spinner quando todas completarem

---

## 14. Segurança de Infraestrutura

### 14.1. TLS Configuration

**Certificados:**
- TLS 1.3 preferencial
- Fallback: TLS 1.2 (mínimo aceitável)
- Proibir: TLS 1.1, 1.0, SSLv3
- Certificados de CA confiável (Let's Encrypt, empresa)
- Wildcard certificate se múltiplos subdomínios

**Cipher Suites:**
- Preferir AEAD ciphers (AES-GCM, ChaCha20-Poly1305)
- Forward secrecy (ECDHE)
- Desabilitar ciphers fracos (RC4, 3DES, MD5, SHA1)

**Certificate Management:**
- Rotação automática antes da expiração
- Monitorar expiração (alertar 30 dias antes)
- Backup de certificados e chaves privadas
- Chaves privadas protegidas (permissões restritas)

**HSTS (HTTP Strict Transport Security):**
- `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`
- Força HTTPS para todos os acessos futuros

### 14.2. Firewall e Network Segmentation

**Firewall Rules:**
- Default deny all
- Whitelist apenas portas necessárias:
  - 443 (HTTPS)
  - 80 (HTTP, apenas para redirect para HTTPS)
- Database port (5432 PostgreSQL) acessível apenas por app server
- SSH port (22) apenas de IPs administrativos

**Network Segmentation:**
- DMZ para frontend/API
- Internal network para database
- Admin network para acesso administrativo
- VLAN separation

**Intrusion Detection:**
- IDS/IPS para detectar ataques
- Alertas para scans de porta, brute force
- Automatic IP blocking para comportamento malicioso

### 14.3. Database Security

**Authentication:**
- Não usar usuário postgres default
- Criar usuário específico para aplicação
- Password forte, rotacionado regularmente
- Autenticação por certificado para conexões remotas

**Authorization:**
- Princípio do mínimo privilégio
- Usuário da aplicação tem apenas: SELECT, INSERT, UPDATE, DELETE
- Sem DROP, CREATE, ALTER permissions
- Schemas separados se múltiplas apps

**Encryption:**
- Encryption at rest (filesystem encryption ou PostgreSQL TDE)
- Encryption in transit (SSL/TLS para conexões)
- Backup encryption

**Network:**
- Database não exposta publicamente
- Apenas app server pode conectar
- Conexões SSL obrigatórias

**Auditing:**
- Habilitar logging de queries
- Log connections e disconnections
- Detectar queries anômalas (muitos JOINs, SELECTs sem WHERE)

### 14.4. Secrets Management

**Nunca:**
- Hardcode secrets no código
- Commit secrets no Git
- Logar secrets

**Storage:**
- Variáveis de ambiente (para desenvolvimento)
- Secrets manager (HashiCorp Vault, self-hosted)
- Encrypted configuration files

**Rotação:**
- Database passwords: a cada 90 dias
- JWT signing keys: a cada 90 dias (com overlap)
- API keys: a cada 180 dias

**Access Control:**
- Secrets acessíveis apenas por processos que precisam
- Audit log de acesso a secrets
- Encrypt secrets at rest

---

## 15. Estratégias de Deployment

### 15.1. Ambiente de Deployment

**Ambientes:**
- Development: máquina local
- Staging: réplica de produção para testes
- Production: ambiente final

**Infrastructure as Code:**
- Docker para containerização
- Docker Compose para orquestração local/staging
- Scripts de deployment automatizados
- Idempotência: deployment pode ser executado múltiplas vezes

### 15.2. Build e Release

**Backend Build:**
- Compilação otimizada: `cargo build --release`
- Testes automatizados obrigatórios
- Linting e formatação verificados
- Binary versionado (Git tag)

**Frontend Build:**
- Compilação para produção: `ng build --configuration production`
- Minificação e tree-shaking
- Lazy loading verificado
- Output versionado

**Artifacts:**
- Backend: binary único
- Frontend: pasta dist/ com assets
- Database migrations: scripts SQL versionados
- Configuration files separados por ambiente

### 15.3. Database Migrations

**Strategy:**
- Migrations versionadas sequencialmente
- Nunca editar migration já aplicada
- Forward-only (evitar rollback se possível)
- Testar migration em staging antes de produção

**Execution:**
- Automatizada no deployment
- Idempotente (pode executar múltiplas vezes)
- Logging detalhado
- Backup antes de aplicar

**Rollback:**
- Ter plano de rollback para cada migration
- Preferencialmente evitar (planejar mudanças compatíveis)
- Se necessário: migration reversa ou restore de backup

### 15.4. Blue-Green Deployment

**Conceito:**
- Dois ambientes idênticos: Blue (atual) e Green (novo)
- Deploy para Green enquanto Blue serve tráfego
- Teste Green
- Switch tráfego para Green
- Blue fica standby para rollback rápido

**Benefícios:**
- Zero downtime
- Rollback instantâneo
- Testes em ambiente de produção antes de switch

**Considerations:**
- Requer recursos duplicados temporariamente
- Database migration deve ser compatível com ambas versões
- Shared database requer cuidado

### 15.5. Monitoring Deployment

**Health Checks:**
- Endpoint `/health` que verifica:
  - Aplicação está respondendo
  - Database está acessível
  - Dependencies estão disponíveis
- Usado por load balancer para detectar instâncias unhealthy

**Smoke Tests:**
- Após deployment, executar testes críticos
- Login, acesso a recursos protegidos, etc.
- Se falhar: rollback automático

**Gradual Rollout:**
- Não deployar para todas instâncias simultaneamente
- Canary deployment: 1 instância primeiro
- Se estável: gradualmente para as demais
- Monitorar error rate durante rollout

---

## 16. Compliance e Standards

### 16.1. OWASP Top 10 Coverage

**A01: Broken Access Control**
- Mitigation: Casbin enforcement em cada endpoint, princípio do mínimo privilégio

**A02: Cryptographic Failures**
- Mitigation: Argon2id para senhas, EdDSA para JWTs, TLS 1.3, encryption at rest

**A03: Injection**
- Mitigation: SQLx prepared statements, input validation, output encoding

**A04: Insecure Design**
- Mitigation: Defense in depth, threat modeling, secure defaults

**A05: Security Misconfiguration**
- Mitigation: Configurações revisadas, defaults seguros, regular security audits

**A06: Vulnerable and Outdated Components**
- Mitigation: Dependências atualizadas, cargo audit, npm audit

**A07: Identification and Authentication Failures**
- Mitigation: MFA, account lockout, strong password policy, secure session management

**A08: Software and Data Integrity Failures**
- Mitigation: JWT signatures, token rotation, immutable audit logs

**A09: Security Logging and Monitoring Failures**
- Mitigation: Structured logging, audit trail, security monitoring, alertas

**A10: Server-Side Request Forgery (SSRF)**
- Mitigation: Input validation, whitelist de URLs, sem user-controlled redirects

### 16.2. GDPR Considerations

**Data Minimization:**
- Coletar apenas dados necessários
- Não armazenar dados sensíveis desnecessariamente

**Right to Access:**
- Usuário pode solicitar cópia de seus dados
- Export em formato legível por máquina (JSON)

**Right to Erasure (Right to be Forgotten):**
- Usuário pode solicitar deleção de dados
- Hard delete ou anonimização
- Considerar obrigações legais de retenção

**Consent Management:**
- Opt-in explícito para coleta de dados
- Granularidade (permitir/negar por tipo de dado)
- Fácil revogação de consentimento

**Data Breach Notification:**
- Detectar breaches rapidamente (monitoring)
- Notificar autoridades em 72h
- Notificar usuários afetados

### 16.3. PCI DSS (se aplicável)

**Se processar pagamentos:**
- Nunca armazenar dados completos de cartão
- Usar gateway de pagamento que assume PCI compliance
- Minimizar scope de compliance

### 16.4. Security Audits

**Frequency:**
- Security review trimestral
- Penetration test anual
- Code review de mudanças sensíveis

**Scope:**
- Application security
- Infrastructure security
- Social engineering resistance
- Physical security (on-premise)

**Remediation:**
- Priorizar vulnerabilidades por severidade
- Definir SLA para correção (críticas: 24h, altas: 1 semana, médias: 1 mês)
- Re-test após correção

---

## 17. Roadmap de Evolução

### 17.1. Fase 1: MVP (Meses 1-3)

**Core Features:**
- Registro e login com email/senha
- Verificação de email
- Password reset
- CRUD básico de usuários
- RBAC com Casbin
- Admin dashboard básico
- Audit logging
- JWT + Refresh token

**Infraestrutura:**
- PostgreSQL setup
- Ambiente de staging
- CI/CD básico
- Monitoring básico

### 17.2. Fase 2: Security Hardening (Meses 4-6)

**Security Features:**
- MFA (TOTP)
- Account lockout robusto
- Refresh token rotation e detecção de roubo
- Rate limiting granular
- CAPTCHA integration
- Session management UI para usuários
- Security notifications (login incomum, etc.)

**Infraestrutura:**
- TLS interno
- Network segmentation
- IDS/IPS setup
- Enhanced monitoring e alertas

### 17.3. Fase 3: Advanced Features (Meses 7-12)

**Features:**
- OAuth2 provider (se necessário para outras apps)
- SSO (Single Sign-On) para múltiplas aplicações
- API keys para integrações
- Webhooks para eventos de auth
- Advanced analytics (login patterns, security insights)
- Machine learning para detecção de anomalias

**Infraestrutura:**
- High availability setup
- Database replication
- Disaster recovery procedures
- Performance optimization

### 17.4. Fase 4: Escala (Ano 2+)

**Se Necessário:**
- Migração para microserviços (identity provider separado)
- Sharding de database
- Multi-region deployment
- Cache distribuído (Redis cluster)
- Message queue para operações assíncronas
- GraphQL API (se requisitado)

---

## 📋 Checklist de Segurança Final

### Autenticação
- [ ] Argon2id com parâmetros robustos (time=3, mem=64MB)
- [ ] Salt único por usuário
- [ ] Validação forte de senha (comprimento, complexidade, senhas comuns)
- [ ] Account lockout após tentativas falhadas
- [ ] MFA para admins
- [ ] Email verification obrigatória

### Autorização
- [ ] Casbin RBAC implementado
- [ ] Default deny em policies
- [ ] Least privilege enforcement
- [ ] Audit de mudanças de permissions

### Tokens
- [ ] JWT assinado com EdDSA
- [ ] Access token curto (15 min)
- [ ] Refresh token longo (30 dias) em cookie HttpOnly
- [ ] Token rotation implementado
- [ ] Token family para detecção de roubo
- [ ] Revogação funcional

### Rede
- [ ] TLS 1.3 obrigatório (interno e externo)
- [ ] HSTS habilitado
- [ ] Certificados válidos e monitorados

### Cookies
- [ ] HttpOnly flag
- [ ] Secure flag
- [ ] SameSite=Strict
- [ ] Scope apropriado

### Headers de Segurança
- [ ] Content-Security-Policy
- [ ] X-Content-Type-Options: nosniff
- [ ] X-Frame-Options: DENY
- [ ] X-XSS-Protection

### Rate Limiting
- [ ] Login endpoint: 5/min
- [ ] Registration: 3/hora
- [ ] Password reset: 3/hora
- [ ] APIs: 1000/hora por usuário

### Proteção CSRF
- [ ] CSRF tokens implementados
- [ ] Double submit cookie ou synchronizer token

### Input Validation
- [ ] Validação no backend (nunca confiar no frontend)
- [ ] Prepared statements (SQLx)
- [ ] Output encoding

### Logging e Auditoria
- [ ] Structured logging
- [ ] Audit trail de eventos críticos
- [ ] Logs imutáveis
- [ ] Monitoramento de eventos de segurança
- [ ] Alertas configurados

### Database
- [ ] Princípio do mínimo privilégio
- [ ] Encryption at rest
- [ ] SSL/TLS para conexões
- [ ] Backups encriptados
- [ ] Disaster recovery testado

### Secrets
- [ ] Nenhum secret hardcoded
- [ ] Secrets em environment variables ou secrets manager
- [ ] Rotação regular de secrets

### Deployment
- [ ] Blue-green deployment ou similar
- [ ] Rollback plan
- [ ] Health checks
- [ ] Smoke tests pós-deployment

### Monitoring
- [ ] Métricas de performance
- [ ] Métricas de segurança
- [ ] Alertas críticos configurados
- [ ] On-call rotation definido

### Compliance
- [ ] OWASP Top 10 addressed
- [ ] GDPR considerations (se aplicável)
- [ ] Security audit agendado
- [ ] Incident response plan documentado

---

## 🎯 Conclusão

Este guia fornece uma arquitetura completa, segura e escalável para um sistema de autenticação e autorização on-premise. As decisões arquiteturais são baseadas em:

1. **Princípios de Segurança Sólidos:** Zero Trust e Defense in Depth
2. **Stack Moderno e Confiável:** Rust, PostgreSQL, Angular
3. **Práticas Comprovadas:** OWASP, NIST, industry standards
4. **Pragmatismo:** Monolito modular antes de microserviços
5. **Manutenibilidade:** Código limpo, testável, documentado
6. **Performance:** Decisões conscientes de trade-offs

O próximo passo é criar um plano de implementação detalhado que traduza esta arquitetura em tarefas executáveis.
