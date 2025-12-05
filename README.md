# Waterswamp 🌊

Sistema de autenticação e autorização robusto desenvolvido em Rust, com foco em segurança e performance.

## 🚀 Recursos Principais

- **Autenticação JWT** com refresh tokens rotativos
- **Autorização RBAC** usando Casbin com cache de políticas
- **MFA/2FA** com TOTP e códigos de backup
- **Hashing Seguro** com Argon2id (OWASP compliant)
- **Rate Limiting** granular por endpoint
- **Métricas Prometheus** para observabilidade
- **Auditoria** completa de ações de usuários
- **Verificação de Email** e recuperação de senha
- **Token Theft Detection** com invalidação de família

## 📋 Requisitos

- Rust 1.75+
- PostgreSQL 14+
- (Opcional) Redis para cache distribuído
- (Opcional) SMTP server para emails

## ⚙️ Configuração

### Variáveis de Ambiente

```bash
# Database
WS_AUTH_DATABASE_URL=postgresql://user:pass@localhost/waterswamp_auth
WS_LOGS_DATABASE_URL=postgresql://user:pass@localhost/waterswamp_logs

# JWT Keys (gerar com scripts/generate_keys.sh)
WS_JWT_PRIVATE_KEY=<caminho_para_private_key.pem>
WS_JWT_PUBLIC_KEY=<caminho_para_public_key.pem>

# Token Expiry (opcional - tem defaults sensíveis)
WS_ACCESS_TOKEN_EXPIRY_SECONDS=3600        # 1 hora (default)
WS_REFRESH_TOKEN_EXPIRY_SECONDS=604800     # 7 dias (default)
WS_PASSWORD_RESET_EXPIRY_SECONDS=900       # 15 minutos (default)
WS_MFA_CHALLENGE_EXPIRY_SECONDS=300        # 5 minutos (default)
WS_EMAIL_VERIFICATION_EXPIRY_SECONDS=86400 # 24 horas (default)

# Email (opcional)
WS_SMTP_HOST=smtp.gmail.com
WS_SMTP_PORT=587
WS_SMTP_USERNAME=your-email@gmail.com
WS_SMTP_PASSWORD=your-app-password

# Logging
RUST_LOG=info
RUST_LOG_FORMAT=json  # ou "text" para desenvolvimento
ENVIRONMENT=production  # ou "development"
```

### Instalação

```bash
# Clonar repositório
git clone https://github.com/yourusername/waterswamp.git
cd waterswamp

# Instalar dependências
cargo build --release

# Gerar chaves JWT
./scripts/generate_keys.sh

# Configurar banco de dados
createdb waterswamp_auth
createdb waterswamp_logs
sqlx migrate run

# Iniciar servidor
cargo run --release
```

## 🏗️ Arquitetura

O projeto segue **Clean Architecture** com **Domain-Driven Design**:

```
waterswamp/
├── apps/
│   └── api-server/          # API REST com Axum
│       ├── api/             # Handlers e rotas
│       ├── infra/           # Config, state, telemetry
│       └── middleware/      # Auth, RBAC, rate limiting
├── crates/
│   ├── domain/              # Entidades e value objects
│   ├── application/         # Serviços de aplicação
│   ├── persistence/         # Repositórios e SQLx
│   ├── core-services/       # JWT, security, crypto
│   └── email-service/       # Envio de emails
└── docs/                    # Documentação adicional
```

### Camadas

- **Domain**: Regras de negócio puras, sem dependências externas
- **Application**: Casos de uso e orquestração de serviços
- **Persistence**: Acesso a dados com SQLx
- **Infrastructure**: HTTP, logging, métricas, configuração
- **Core Services**: Utilitários compartilhados (JWT, crypto, etc)

## 📊 Métricas e Observabilidade

O sistema expõe métricas Prometheus em `/metrics`:

### Métricas Disponíveis

| Métrica | Tipo | Descrição |
|---------|------|-----------|
| `http_requests_total` | Counter | Total de requisições HTTP |
| `http_request_duration_seconds` | Histogram | Latência das requisições |
| `login_attempts_total` | Counter | Tentativas de login (success/failure) |
| `token_refresh_total` | Counter | Renovações de tokens |
| `token_theft_detected_total` | Counter | Detecções de roubo de tokens |
| `password_hash_duration_seconds` | Histogram | Tempo de hash/verify de senhas |
| `policy_cache_hits_total` | Counter | Cache hits do Casbin |
| `casbin_policies_count` | Gauge | Número de políticas carregadas |
| `casbin_enforcement_duration_seconds` | Histogram | Tempo de verificação de permissões |
| `mfa_operations_total` | Counter | Operações MFA |

### Exemplo de Queries Prometheus

```promql
# Taxa de login por minuto
rate(login_attempts_total[1m])

# P95 latência de requisições
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Taxa de cache hit do Casbin
rate(policy_cache_hits_total{result="hit"}[5m]) /
rate(policy_cache_hits_total[5m])

# Detecções de token theft por hora
increase(token_theft_detected_total[1h])
```

## 🔒 Segurança

### Password Hashing

- **Algoritmo**: Argon2id (OWASP recommendation 2024)
- **Parâmetros**: 64 MiB memory, 3 iterations, 4 threads
- **Performance**: ~200-300ms por operação (tunável via env vars)

### Validação de Senha

- Comprimento mínimo: 8 caracteres
- Bloqueio de senhas comuns (top 100)
- Score zxcvbn mínimo: 3/4 (Strong)

### Token Theft Detection

- Refresh tokens com família (family-based rotation)
- Detecção de reutilização de tokens
- Invalidação automática de toda a família em caso de suspeita

## 📚 Documentação Adicional

- [Guia de Autenticação](docs/guia-auth-user.md)
- [Rate Limiting](docs/RATE_LIMITING.md)
- [Benchmarks Argon2](docs/ARGON2_BENCHMARKS.md)
- [Deployment](docs/deployment.md)
- [Guia Rust](docs/guia-rust.md)
- [Code Review](CODE_REVIEW_REPORT.md)

## 🧪 Testes

```bash
# Rodar todos os testes
cargo test

# Testes com coverage
cargo tarpaulin --out Html

# Testes de integração
cargo test --test integration_tests

# Testes de performance
cargo bench
```

## 📝 Exemplos de Uso

### Registro de Usuário

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "email": "john@example.com",
    "password": "MyS3cur3P@ssw0rd!"
  }'
```

### Login

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "johndoe",
    "password": "MyS3cur3P@ssw0rd!"
  }'
```

### Refresh Token

```bash
curl -X POST http://localhost:3000/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<your_refresh_token>"
  }'
```

## 🤝 Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

### Guidelines

- Siga as convenções do [Guia Rust](docs/guia-rust.md)
- Adicione testes para novas funcionalidades
- Mantenha coverage acima de 80%
- Use `cargo fmt` e `cargo clippy` antes de commitar

## 📄 Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## ✨ Agradecimentos

- [Axum](https://github.com/tokio-rs/axum) - Framework web
- [SQLx](https://github.com/launchbadge/sqlx) - SQL toolkit
- [Casbin](https://casbin.org/) - Sistema de autorização
- [Argon2](https://github.com/RustCrypto/password-hashes) - Password hashing
- [TOTP-RS](https://github.com/constantoine/totp-rs) - MFA/2FA

## 📞 Suporte

Para questões e suporte:
- Abra uma [issue](https://github.com/yourusername/waterswamp/issues)
- Email: support@waterswamp.com
- Discord: [Junte-se ao servidor](https://discord.gg/waterswamp)
