# Guia Completo: Sistema Robusto e Testável em Rust

> Requisitos, práticas, estruturas e estratégias de teste para sistemas modernos, escaláveis e de longo prazo em Rust

---

## Índice

1. [Pilares de Definição](#pilares-de-definição-o-que-torna-um-sistema-rust-robusto)
2. [Requisitos Fundamentais](#requisitos-fundamentais)
3. [Práticas Essenciais](#práticas-essenciais)
4. [Estrutura de Projeto](#estrutura-de-projeto)
5. [Ferramentas Modernas](#ferramentas-modernas)
6. [Escalabilidade](#escalabilidade)
7. [Manutenibilidade de Longo Prazo](#manutenibilidade-de-longo-prazo)
8. [Padrões Específicos do Rust](#padrões-específicos-do-rust)
9. [Type-Driven Development](#type-driven-development-tornando-estados-inválidos-irrepresentáveis)
10. [Stack Recomendado (Padrão Ouro)](#stack-recomendado-padrão-ouro)
11. [Comparação de Alternativas](#comparação-de-alternativas)
12. [Estratégia de Testes](#estratégia-de-testes)
13. [Tipos de Testes Detalhados](#tipos-de-testes-detalhados)
14. [Métricas de Qualidade](#métricas-de-qualidade)
15. [CI/CD Pipeline](#cicd-pipeline)

---

## Pilares de Definição: O que Torna um Sistema Rust Robusto

Antes de mergulhar nas práticas, é fundamental entender os **pilares conceituais** que definem um sistema robusto, moderno, escalável e de longo prazo em Rust.

### 🛡️ Robustez (Confiabilidade)

Em Rust, robustez significa que o sistema **não falha silenciosamente** e lida com estados inválidos **em tempo de compilação** sempre que possível.

**Características:**
- **Type-Driven Development**: O sistema de tipos torna estados inválidos irrepresentáveis
- **Tratamento de Erros Exaustivo**: Zero `.unwrap()` em produção, sempre usar `Result<T, E>`
- **Testes Abrangentes**: Unitários, integração e property-based testing
- **Falhas Controladas**: Panic apenas para bugs irrecuperáveis, não para erros esperados

### 🚀 Modernidade & Eficiência

Significa aproveitar o modelo de **Ownership sem lutar contra ele** e usar abstrações de custo zero.

**Características:**
- **Async/Await**: Runtime assíncrono (Tokio) para sistemas I/O-bound
- **Zero-Cost Abstractions**: Iteradores, closures e generics que compilam para código otimizado
- **Uso Inteligente de Memória**: `&str` vs `String`, `Cow<'a, T>` para dados raramente mutados
- **Ownership Consciente**: Minimizar clones, usar referências quando possível

### 📈 Escalabilidade & Longo Prazo

Capacidade de crescer em **carga (throughput)** e em **complexidade de código** ao longo dos anos.

**Características:**
- **Modularização**: Cargo Workspaces com crates de responsabilidade única
- **Compilação Incremental**: Mudanças em uma crate não recompilam todo o projeto
- **Estabilidade de Dependências**: Uso correto de `Cargo.lock` e seleção criteriosa
- **Documentação Viva**: `cargo doc` e doctests mantêm docs sincronizadas com código
- **Arquitetura Flexível**: Hexagonal/Clean Architecture permite trocar implementações

---

## Requisitos Fundamentais

Um sistema robusto, moderno e de longo prazo em Rust se fundamenta em vários pilares essenciais.

### 1. Segurança de Tipos e Memória

- **Aproveitar o sistema de ownership/borrowing** do Rust ao máximo
- **Minimizar uso de `unsafe`** - documentar e encapsular quando absolutamente necessário
- **Usar tipos que representam estados inválidos de forma impossível** (parse, don't validate)
- **Preferir `Result<T, E>` e `Option<T>`** ao invés de pânico
- **Zero-cost abstractions** sempre que possível

```rust
// ❌ Evitar: Estado inválido possível
struct User {
    email: String, // pode ser inválido
    age: i32,      // pode ser negativo
}

// ✅ Preferir: Estado inválido impossível
struct User {
    email: ValidatedEmail,
    age: PositiveAge,
}

pub struct ValidatedEmail(String);
pub struct PositiveAge(u8);
```

### 2. Tratamento de Erros

O gerenciamento de erros estruturado é vital para manutenção de longo prazo. A separação entre erros de biblioteca e erros de aplicação garante robustez.

#### **Bibliotecas (Libs): Use `thiserror`**

Bibliotecas devem definir erros específicos e enumerados que os consumidores possam tratar programaticamente.

```rust
use thiserror::Error;

// ✅ Para bibliotecas: erros tipados e específicos
#[derive(Error, Debug)]
pub enum UserError {
    #[error("Email inválido: {0}")]
    InvalidEmail(String),
    
    #[error("Usuário não encontrado: {id}")]
    NotFound { id: u64 },
    
    #[error("Erro de banco de dados")]
    DatabaseError(#[from] sqlx::Error),
}

// API da biblioteca retorna erros específicos
pub fn register_user(email: &str) -> Result<User, UserError> {
    let validated = validate_email(email)
        .map_err(|_| UserError::InvalidEmail(email.to_string()))?;
    // ...
    Ok(user)
}
```

#### **Aplicações (Binários): Use `anyhow`**

Aplicações devem usar `anyhow` (ou `eyre`) para capturar contextos de erro e facilitar diagnóstico em logs, já que raramente precisam tratar cada tipo de erro individualmente no topo da pilha.

```rust
use anyhow::{Context, Result};

// ✅ Para aplicações: contexto rico para debugging
async fn process_order(order_id: u64) -> Result<()> {
    let order = fetch_order(order_id)
        .await
        .context("Failed to fetch order")?;
    
    let payment = process_payment(&order)
        .await
        .context(format!("Failed to process payment for order {}", order_id))?;
    
    save_payment(&payment)
        .await
        .context("Failed to save payment to database")?;
    
    Ok(())
}

// Erro será exibido como:
// Error: Failed to save payment to database
// Caused by:
//     Failed to process payment for order 12345
//     Caused by:
//         Connection timeout
```

#### **Regras de Ouro**

- **Propagação explícita** com operador `?`
- **Nunca usar `.unwrap()` ou `.expect()` em produção** (com exceções raras - veja abaixo)
- **Distinção clara** entre erros recuperáveis e irrecuperáveis
- **Logs estruturados** de erros com contexto
- **Error types ricos** com informações para debugging

#### **Quando `.unwrap()` é Aceitável**

Existem casos específicos onde `.unwrap()` ou `.expect()` são aceitáveis:

```rust
// ✅ Literais que sempre são válidas
let url = Url::parse("https://api.example.com").unwrap();
let regex = Regex::new(r"^\d+$").expect("Invalid regex pattern");

// ✅ Locks que não podem falhar (em single-threaded ou se você controla)
let data = arc_rwlock.read().expect("Lock poisoned - fatal error");

// ✅ Em testes
#[test]
fn test_something() {
    let result = parse_data("valid").unwrap();
    assert_eq!(result.value, 42);
}

// ✅ Setup de aplicação (panic early)
fn main() {
    let config = load_config().expect("Failed to load config - cannot start");
    let db = connect_db(&config.db_url).expect("Failed to connect to database");
    // ...
}

// ❌ NUNCA em código de produção que processa input
fn process_request(data: &str) -> Response {
    let parsed = parse_json(data).unwrap();  // ❌ ERRADO! Input pode ser inválido
    // ...
}

// ✅ Use Result para input externo
fn process_request(data: &str) -> Result<Response, Error> {
    let parsed = parse_json(data)?;  // ✅ CORRETO
    // ...
}
```

### 3. Concorrência Segura

A escalabilidade vertical exige uso eficiente de núcleos de CPU com segurança garantida em tempo de compilação.

#### **Preferir Canais sobre Estado Compartilhado**

- **Canais** (mpsc, broadcast, oneshot) para comunicação entre tasks/threads
- **Evitar Arc<Mutex>** sempre que possível - use message passing
- **RwLock** quando estado compartilhado for inevitável e houver **muitas leituras e poucas escritas**
- **Evitar deadlocks** com design cuidadoso, timeouts e hierarquia de locks
- **Rayon** para paralelismo de CPU-bound

```rust
use tokio::sync::mpsc;

// ✅ PREFERIR: Message passing com canais
async fn process_messages() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Producer
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(format!("message {}", i)).await.unwrap();
        }
    });
    
    // Consumer
    while let Some(msg) = rx.recv().await {
        process(msg).await;
    }
}

// ⚠️ Usar RwLock quando necessário (muitas leituras, poucas escritas)
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct CacheService {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl CacheService {
    // Muitas leituras simultâneas (sem bloqueio entre si)
    async fn get(&self, key: &str) -> Option<String> {
        self.cache.read().unwrap().get(key).cloned()
    }
    
    // Poucas escritas (bloqueia todas as leituras)
    async fn set(&self, key: String, value: String) {
        self.cache.write().unwrap().insert(key, value);
    }
}

// 🚀 Rayon para processamento paralelo de CPU
use rayon::prelude::*;

fn process_batch(items: Vec<Item>) -> Vec<Result> {
    items.par_iter()
        .map(|item| expensive_computation(item))
        .collect()
}
```

---

## Práticas Essenciais

### Arquitetura e Design

#### **1. Modularização**
- **Workspace** com múltiplos crates separando responsabilidades
- **Domínio puro** sem dependências externas
- **Infraestrutura** isolada da lógica de negócio
- **API/Apresentação** como camada fina

#### **2. Dependency Injection**
- **Traits** para abstrair dependências
- **Constructor injection** para configuração
- **Facilita testes** com mocks

```rust
// Definir trait para abstração
pub trait UserRepository {
    async fn find_by_id(&self, id: u64) -> Result<User, Error>;
    async fn save(&self, user: &User) -> Result<(), Error>;
}

// Injetar dependência
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    pub async fn get_user(&self, id: u64) -> Result<User, Error> {
        self.repository.find_by_id(id).await
    }
}
```

#### **3. Hexagonal/Clean Architecture**
- **Separar lógica de negócio** de detalhes técnicos
- **Portas e adaptadores** para flexibilidade
- **Inversão de dependências** (domínio não depende de infra)

#### **4. Domain-Driven Design**
- **Tipos que expressam regras** de negócio
- **Entities, Value Objects, Aggregates**
- **Linguagem ubíqua** refletida no código

```rust
// Value Object com validação
pub struct Email(String);

impl Email {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        if !value.contains('@') {
            return Err(ValidationError::InvalidEmail);
        }
        Ok(Email(value))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Aggregate Root
pub struct Order {
    id: OrderId,
    items: Vec<OrderItem>,
    status: OrderStatus,
}

impl Order {
    pub fn add_item(&mut self, item: OrderItem) -> Result<(), OrderError> {
        if self.status != OrderStatus::Draft {
            return Err(OrderError::CannotModifyConfirmedOrder);
        }
        self.items.push(item);
        Ok(())
    }
}
```

### Código Limpo

#### **Princípios SOLID em Rust**

```rust
// Single Responsibility
struct EmailSender { /* apenas envia emails */ }
struct EmailValidator { /* apenas valida */ }

// Open/Closed (com traits)
trait PaymentProcessor {
    fn process(&self, amount: f64) -> Result<(), Error>;
}

struct CreditCardProcessor;
struct PayPalProcessor;

impl PaymentProcessor for CreditCardProcessor { /* ... */ }
impl PaymentProcessor for PayPalProcessor { /* ... */ }

// Liskov Substitution (naturalmente com traits)
fn process_payment<P: PaymentProcessor>(processor: &P, amount: f64) {
    processor.process(amount).unwrap();
}

// Interface Segregation (traits pequenos e específicos)
trait Readable {
    fn read(&self) -> Vec<u8>;
}

trait Writable {
    fn write(&mut self, data: &[u8]);
}

// Dependency Inversion (já mostrado acima)
```

### Documentação

- **Doc comments (`///`)** para APIs públicas
- **Exemplos executáveis** em documentação
- **README.md** com quickstart e exemplos
- **Architecture Decision Records (ADRs)** para decisões importantes
- **CHANGELOG.md** seguindo Keep a Changelog

```rust
/// Calcula o desconto aplicado a um valor.
///
/// # Arguments
///
/// * `price` - O preço original
/// * `discount_rate` - Taxa de desconto entre 0.0 e 1.0
///
/// # Returns
///
/// O preço com desconto aplicado
///
/// # Examples
///
/// ```
/// use meu_crate::calculate_discount;
///
/// let final_price = calculate_discount(100.0, 0.2);
/// assert_eq!(final_price, 80.0);
/// ```
///
/// # Panics
///
/// Entra em pânico se `discount_rate` estiver fora do intervalo [0.0, 1.0]
pub fn calculate_discount(price: f64, discount_rate: f64) -> f64 {
    assert!(discount_rate >= 0.0 && discount_rate <= 1.0);
    price * (1.0 - discount_rate)
}
```

---

## Estrutura de Projeto

### Layout Recomendado para Projetos Médios/Grandes

O uso de Workspace divide o projeto em múltiplas crates menores com responsabilidades claras, trazendo benefícios significativos:

**Benefícios do Workspace:**
- ✅ **Compilação incremental por crate** - mudanças em uma crate não recompilam todo o projeto
- ✅ **Tempos de build mais rápidos** em desenvolvimento
- ✅ **Limites claros de dependência** - previne acoplamento acidental
- ✅ **Reutilização de código** entre binários e bibliotecas
- ✅ **Testes isolados** - facilita identificar problemas
- ✅ **Versionamento independente** das crates internas

```
meu-projeto/
├── Cargo.toml                 # Workspace root
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── LICENSE
│
├── crates/
│   ├── domain/                # Lógica de negócio pura (sem dependências externas)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entities/      # Entities e Value Objects
│   │       ├── services/      # Domain Services
│   │       └── errors.rs
│   │
│   ├── application/           # Casos de uso (use cases)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── commands/      # Write operations
│   │       ├── queries/       # Read operations
│   │       └── dto.rs         # Data Transfer Objects
│   │
│   ├── infrastructure/        # Implementações concretas
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── database/      # PostgreSQL, Redis, etc
│   │       ├── messaging/     # RabbitMQ, Kafka, etc
│   │       ├── http_client/   # Clientes HTTP externos
│   │       └── config.rs
│   │
│   └── api/                   # Camada de apresentação
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── routes/        # Endpoints HTTP
│           ├── middleware/    # Auth, logging, etc
│           └── handlers/      # Request handlers
│
├── tests/                     # Testes de integração
│   ├── common/
│   │   └── mod.rs            # Fixtures e helpers
│   ├── api_tests.rs
│   └── integration_tests.rs
│
├── benches/                   # Benchmarks
│   └── performance.rs
│
├── docs/                      # Documentação adicional
│   ├── architecture/
│   │   ├── ADR-001-escolha-banco.md
│   │   └── diagrams/
│   └── api/
│       └── openapi.yaml
│
├── scripts/                   # Scripts úteis
│   ├── setup.sh
│   └── migrate.sh
│
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

### Cargo.toml do Workspace

```toml
[workspace]
members = [
    "crates/domain",
    "crates/application",
    "crates/infrastructure",
    "crates/api",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Seu Nome <email@example.com>"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Dependências compartilhadas
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## Ferramentas Modernas

### Desenvolvimento

```bash
# Ferramentas essenciais
cargo install cargo-watch      # Hot reload
cargo install cargo-nextest    # Testes mais rápidos
cargo install cargo-audit      # Verificar vulnerabilidades
cargo install cargo-deny       # Verificar licenças
cargo install cargo-outdated   # Deps desatualizadas
cargo install cargo-edit       # Adicionar/remover deps
cargo install cargo-expand     # Expandir macros

# Qualidade de código
cargo install cargo-llvm-cov   # Cobertura de código
cargo install cargo-mutants    # Mutation testing
cargo install cargo-flamegraph # Profiling

# Documentação
cargo install mdbook           # Livros/docs
```

### Clippy (Linter)

O linter oficial do Rust deve ser configurado para **falhar no CI** em qualquer warning, garantindo qualidade consistente.

```toml
# .cargo/config.toml
[target.'cfg(all())']
rustflags = ["-D", "warnings"]  # Tratar warnings como erros no CI

# Ou em Cargo.toml (Rust 1.74+)
[lints.rust]
unsafe_code = "forbid"  # Proibir unsafe (exceto onde explicitamente permitido)

[lints.clippy]
all = "deny"      # ❌ Falhar em qualquer warning
pedantic = "warn" # ⚠️ Avisar sobre código pedante
cargo = "warn"    # ⚠️ Avisar sobre issues do Cargo
nursery = "warn"  # ⚠️ Lints experimentais

# Permitir alguns lints específicos se necessário
[lints.clippy]
too_many_arguments = "allow"  # Às vezes necessário
```

**Rodar localmente:**
```bash
# Ver todos os warnings
cargo clippy --all-targets --all-features

# Falhar em warnings (igual ao CI)
cargo clippy --all-targets --all-features -- -D warnings

# Aplicar correções automáticas
cargo clippy --fix
```

### Rustfmt (Formatação)

```toml
# rustfmt.toml
max_width = 100
tab_spaces = 4
edition = "2021"
use_small_heuristics = "Default"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### Observabilidade

Um sistema moderno precisa ser **observável** para ser mantido em produção. Substitua logs de texto simples por eventos estruturados.

#### **Tracing Estruturado**

Use `tracing` para anexar contexto (request IDs, user IDs) a todo um fluxo de execução, especialmente vital em código async.

```rust
use tracing::{info, warn, error, debug, instrument, span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Setup inicial da aplicação
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
        )
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
}

// Instrumentação automática
#[instrument(skip(db), fields(user_id = %user_id))]
async fn create_user(db: &Database, user_id: u64, email: String) -> Result<User, Error> {
    info!("Creating user with email: {}", email);
    
    let user = db.insert_user(email).await
        .map_err(|e| {
            error!("Failed to create user: {:?}", e);
            e
        })?;
    
    info!(user_id = user.id, "User created successfully");
    Ok(user)
}

// Spans manuais para contexto complexo
async fn process_order(order_id: u64) -> Result<()> {
    let span = span!(Level::INFO, "process_order", order_id);
    let _enter = span.enter();
    
    info!("Starting order processing");
    
    // Todo este código mantém o contexto do order_id
    validate_order(order_id).await?;
    charge_payment(order_id).await?;
    ship_order(order_id).await?;
    
    info!("Order processed successfully");
    Ok(())
}
```

#### **Métricas com OpenTelemetry**

Integração com OpenTelemetry para exportar métricas de runtime (latência, throughput, uso de recursos).

```rust
use opentelemetry::{global, KeyValue, metrics::{Counter, Histogram}};
use opentelemetry_sdk::metrics::MeterProvider;

// Setup de métricas
fn init_metrics() -> MeterProvider {
    let provider = opentelemetry_sdk::metrics::MeterProvider::builder()
        .with_reader(
            opentelemetry_sdk::metrics::PeriodicReader::builder(
                opentelemetry_stdout::MetricsExporter::default(),
                opentelemetry_sdk::runtime::Tokio,
            )
            .build(),
        )
        .build();
    
    global::set_meter_provider(provider.clone());
    provider
}

// Uso de métricas
struct Metrics {
    requests_total: Counter<u64>,
    request_duration: Histogram<f64>,
}

impl Metrics {
    fn new() -> Self {
        let meter = global::meter("my_service");
        Self {
            requests_total: meter
                .u64_counter("requests_total")
                .with_description("Total number of requests")
                .init(),
            request_duration: meter
                .f64_histogram("request_duration_seconds")
                .with_description("Request duration in seconds")
                .init(),
        }
    }
    
    fn record_request(&self, endpoint: &str, duration: f64) {
        let labels = &[KeyValue::new("endpoint", endpoint.to_string())];
        self.requests_total.add(1, labels);
        self.request_duration.record(duration, labels);
    }
}

// Integração com framework web (Axum exemplo)
async fn track_metrics(
    State(metrics): State<Arc<Metrics>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();
    let path = request.uri().path().to_string();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed().as_secs_f64();
    metrics.record_request(&path, duration);
    
    response
}
```

#### **Integração Completa**

```rust
// Tracing + OpenTelemetry + Logs
use tracing_subscriber::layer::SubscriberExt;
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_observability() {
    // Configurar OpenTelemetry tracer
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("my_service")
        .install_simple()
        .expect("Failed to initialize tracer");
    
    // Combinar tracing com OpenTelemetry
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .with(EnvFilter::from_default_env())
        .init();
}
```

#### **Ferramentas Recomendadas**

- **Logs**: `tracing` + `tracing-subscriber`
- **Métricas**: `opentelemetry` + Prometheus/StatsD
- **Tracing Distribuído**: Jaeger, Tempo, ou Zipkin
- **Error Tracking**: Sentry (`sentry` crate)
- **APM**: Datadog, New Relic (com integrações OpenTelemetry)

---

## Escalabilidade

### Performance

#### **1. Profiling**

```bash
# Flamegraph
cargo install flamegraph
cargo flamegraph

# Perf (Linux)
perf record -g target/release/meu_app
perf report

# Valgrind (Memory)
valgrind --tool=cachegrind target/release/meu_app
```

#### **2. Otimizações**

```rust
// ❌ Evitar: Alocações desnecessárias
fn process(data: &str) -> String {
    data.to_string().to_uppercase() // 2 alocações
}

// ✅ Preferir: Minimizar alocações
fn process(data: &str) -> String {
    data.to_uppercase() // 1 alocação
}

// 🚀 Uso de &str vs String
fn greet(name: &str) {  // ✅ Aceita &str e String (via deref)
    println!("Hello, {}!", name);
}

fn greet_owned(name: String) {  // ❌ Força alocação
    println!("Hello, {}!", name);
}

// 🐄 Cow<'a, T> para dados que raramente mudam
use std::borrow::Cow;

fn process_config(config: Cow<str>) -> Cow<str> {
    if config.contains("DEBUG") {
        // Apenas aloca se precisar modificar
        Cow::Owned(config.replace("DEBUG", "PROD"))
    } else {
        // Retorna borrowed - zero alocações
        config
    }
}

// Uso
let config1 = "MODE=PROD";
let result1 = process_config(Cow::Borrowed(config1)); // Sem alocação

let config2 = "MODE=DEBUG";
let result2 = process_config(Cow::Borrowed(config2)); // Aloca apenas aqui

// Zero-copy quando possível
use bytes::Bytes;

fn handle_data(data: Bytes) {
    // Bytes permite compartilhar sem copiar
    let shared = data.clone(); // apenas incrementa ref count
    send_to_service_a(data);
    send_to_service_b(shared);
}

// Lazy evaluation com iteradores
let sum: i32 = (1..1_000_000)
    .filter(|x| x % 2 == 0)
    .take(100)
    .sum(); // Não processa todos os 1M de números
```

#### **3. Async e Concorrência**

```rust
use tokio::task;

// Processar em paralelo
async fn process_batch(items: Vec<Item>) -> Vec<Result> {
    let futures: Vec<_> = items
        .into_iter()
        .map(|item| task::spawn(process_item(item)))
        .collect();
    
    // Aguardar todos
    let results = futures::future::join_all(futures).await;
    results
}

// Rate limiting
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(Quota::per_second(nonzero!(10u32)));
limiter.until_ready().await;
```

### Arquitetura Escalável

#### **1. Design Stateless**

```rust
// ❌ Evitar: Estado na aplicação
static mut COUNTER: i32 = 0;

// ✅ Preferir: Estado em cache/BD externo
async fn get_counter(redis: &Redis) -> i32 {
    redis.get("counter").await
}
```

#### **2. Message Queues**

```rust
use lapin::{Connection, Channel};

async fn publish_event(channel: &Channel, event: OrderCreated) {
    let payload = serde_json::to_vec(&event).unwrap();
    channel.basic_publish(
        "orders",
        "order.created",
        Default::default(),
        &payload,
        Default::default(),
    ).await.unwrap();
}
```

#### **3. Cache Estratégico**

```rust
use redis::AsyncCommands;

async fn get_user_cached(
    redis: &mut Redis,
    db: &Database,
    user_id: u64,
) -> Result<User, Error> {
    let cache_key = format!("user:{}", user_id);
    
    // Tentar cache primeiro
    if let Ok(Some(cached)) = redis.get::<_, String>(&cache_key).await {
        return Ok(serde_json::from_str(&cached)?);
    }
    
    // Cache miss: buscar do banco
    let user = db.find_user(user_id).await?;
    
    // Salvar no cache
    let serialized = serde_json::to_string(&user)?;
    redis.set_ex(&cache_key, serialized, 300).await?; // TTL: 5 min
    
    Ok(user)
}
```

#### **4. Connection Pooling**

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect("postgresql://localhost/mydb")
    .await?;

// Reusar conexões
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
    .fetch_one(&pool)
    .await?;
```

---

## Manutenibilidade de Longo Prazo

### Gerenciamento de Dependências

```toml
[dependencies]
# Princípio: Mínimo necessário, máxima qualidade

# ✅ Crates bem mantidos com grande adoção
tokio = "1.35"
serde = "1.0"

# ⚠️ Avaliar antes de adicionar:
# - Última atualização
# - Número de downloads
# - Issues abertas
# - Licença compatível

[dependencies.some-crate]
version = "0.5"
default-features = false  # Habilitar apenas o necessário
features = ["json", "compression"]
```

### Versionamento Semântico

```toml
# Seguir SemVer estritamente
[package]
version = "1.2.3"  # MAJOR.MINOR.PATCH

# MAJOR: Breaking changes (incompatibilidade de API)
# MINOR: Novas features (backwards compatible)
# PATCH: Bug fixes (não adiciona features)
```

**Práticas Recomendadas:**
- Documente breaking changes no CHANGELOG
- Use `#[deprecated]` antes de remover APIs públicas
- Mantenha compatibilidade por pelo menos 2 minor versions

### Configuração Hierárquica

Use `config-rs` para gerenciar configurações de múltiplas fontes de forma estruturada:

```toml
[dependencies]
config = "0.13"
serde = { version = "1.0", features = ["derive"] }
```

```rust
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LogConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        
        let config = Config::builder()
            // 1. Arquivo padrão (sempre carregado)
            .add_source(File::with_name("config/default"))
            
            // 2. Arquivo específico do ambiente (opcional)
            .add_source(
                File::with_name(&format!("config/{}", env))
                    .required(false)
            )
            
            // 3. Arquivo local para overrides (não commitado, opcional)
            .add_source(File::with_name("config/local").required(false))
            
            // 4. Variáveis de ambiente com prefixo APP_ (maior precedência)
            // APP_SERVER__PORT=3000 sobrescreve server.port
            .add_source(
                Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true)
            )
            
            .build()?;
        
        config.try_deserialize()
    }
}

// Uso
fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    println!("Starting server on {}:{}", config.server.host, config.server.port);
    Ok(())
}
```

**Estrutura de arquivos de configuração:**
```
config/
├── default.toml          # Valores padrão (committed)
├── development.toml      # Configs de dev (committed)
├── production.toml       # Configs de prod (committed)
├── test.toml            # Configs de teste (committed)
└── local.toml           # Overrides locais (gitignored)
```

**Exemplo `config/default.toml`:**
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "postgresql://localhost/myapp_dev"
max_connections = 20
timeout_seconds = 30

[logging]
level = "info"
format = "json"
```

**Precedência (do menor para o maior):**
1. `config/default.toml`
2. `config/{environment}.toml`
3. `config/local.toml`
4. Variáveis de ambiente `APP_*`

### Feature Flags

```toml
[features]
default = ["json"]
json = ["serde_json"]
xml = ["quick-xml"]
full = ["json", "xml"]

# Uso
#[cfg(feature = "json")]
pub mod json_support {
    // ...
}
```

### Changelog

```markdown
# Changelog

## [Unreleased]

## [1.2.0] - 2024-01-15

### Added
- Suporte para autenticação OAuth2
- Novos endpoints de relatórios

### Changed
- Melhorada performance de queries em 30%

### Deprecated
- `old_api()` será removida na versão 2.0

### Fixed
- Corrigido leak de memória em websockets

### Security
- Atualizada dependência com vulnerabilidade CVE-2024-1234
```

---

## Padrões Específicos do Rust

### 1. Newtype Pattern

```rust
// Encapsular tipos primitivos para type safety
pub struct UserId(u64);
pub struct ProductId(u64);

// Impede erros de tipo
fn get_user(id: UserId) -> User { /* ... */ }
// get_user(ProductId(5)); // ❌ Erro de compilação
```

### 2. Builder Pattern

```rust
pub struct Server {
    host: String,
    port: u16,
    timeout: Duration,
    max_connections: usize,
}

pub struct ServerBuilder {
    host: String,
    port: u16,
    timeout: Option<Duration>,
    max_connections: Option<usize>,
}

impl ServerBuilder {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            timeout: None,
            max_connections: None,
        }
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }
    
    pub fn build(self) -> Server {
        Server {
            host: self.host,
            port: self.port,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            max_connections: self.max_connections.unwrap_or(100),
        }
    }
}

// Uso
let server = ServerBuilder::new("localhost".into(), 8080)
    .timeout(Duration::from_secs(60))
    .max_connections(200)
    .build();
```

### 3. Type State Pattern

```rust
// Estados em tempo de compilação
pub struct Connection<State> {
    _state: PhantomData<State>,
}

pub struct Disconnected;
pub struct Connected;
pub struct Authenticated;

impl Connection<Disconnected> {
    pub fn new() -> Self {
        Connection { _state: PhantomData }
    }
    
    pub fn connect(self) -> Connection<Connected> {
        // lógica de conexão
        Connection { _state: PhantomData }
    }
}

impl Connection<Connected> {
    pub fn authenticate(self, credentials: &str) -> Connection<Authenticated> {
        // lógica de autenticação
        Connection { _state: PhantomData }
    }
}

impl Connection<Authenticated> {
    pub fn send_data(&self, data: &[u8]) {
        // só pode enviar dados se autenticado
    }
}

// Uso: garante ordem de operações em tempo de compilação
let conn = Connection::new()
    .connect()
    .authenticate("token");
conn.send_data(b"Hello");
```

### 4. Extension Traits

```rust
// Adicionar funcionalidade a tipos externos
trait StringExt {
    fn is_valid_email(&self) -> bool;
}

impl StringExt for String {
    fn is_valid_email(&self) -> bool {
        self.contains('@') && self.contains('.')
    }
}

// Uso
let email = "user@example.com".to_string();
if email.is_valid_email() {
    // ...
}
```

### 5. Interior Mutability

```rust
use std::cell::RefCell;
use std::rc::Rc;

// Single-threaded
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
data.borrow_mut().push(4);

// Multi-threaded
use std::sync::{Arc, RwLock};

let data = Arc::new(RwLock::new(vec![1, 2, 3]));
let data_clone = data.clone();

std::thread::spawn(move || {
    data_clone.write().unwrap().push(4);
});
```

---

## Type-Driven Development: Tornando Estados Inválidos Irrepresentáveis

Um dos pilares mais importantes de robustez em Rust é usar o sistema de tipos para **prevenir bugs em tempo de compilação**. O mantra é: "parse, don't validate" e "make illegal states unrepresentable".

### Princípio Fundamental

Ao invés de validar dados em runtime repetidamente, **construa tipos que só podem existir em estados válidos**.

```rust
// ❌ MAU: Estado inválido é possível
struct User {
    email: String,      // pode ser inválido
    age: i32,           // pode ser negativo
    status: String,     // pode ser "ativo", "inativo", typo...
}

// ✅ BOM: Estado inválido é impossível
struct User {
    email: Email,              // garantido válido
    age: Age,                  // garantido positivo
    status: UserStatus,        // apenas valores válidos
}

// Value Objects validados
pub struct Email(String);

impl Email {
    pub fn new(value: String) -> Result<Self, ValidationError> {
        if !value.contains('@') || !value.contains('.') {
            return Err(ValidationError::InvalidEmail);
        }
        Ok(Email(value))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub struct Age(u8);  // u8 já garante 0-255

impl Age {
    pub fn new(value: u8) -> Result<Self, ValidationError> {
        if value > 120 {
            return Err(ValidationError::UnrealisticAge);
        }
        Ok(Age(value))
    }
}

// Enum ao invés de strings mágicas
#[derive(Debug, Clone, Copy)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}
```

### Type State Pattern Avançado

Use tipos fantasma para rastrear estados em tempo de compilação:

```rust
use std::marker::PhantomData;

// Estados
pub struct Disconnected;
pub struct Connected;
pub struct Authenticated;

// Conexão que rastreia seu estado no tipo
pub struct Connection<State> {
    inner: TcpStream,
    _state: PhantomData<State>,
}

// Apenas disponível quando desconectado
impl Connection<Disconnected> {
    pub fn new(addr: &str) -> io::Result<Self> {
        Ok(Connection {
            inner: TcpStream::connect(addr)?,
            _state: PhantomData,
        })
    }
    
    pub fn connect(self) -> Result<Connection<Connected>, Error> {
        // lógica de conexão
        Ok(Connection {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}

// Apenas disponível quando conectado
impl Connection<Connected> {
    pub fn authenticate(self, token: &str) -> Result<Connection<Authenticated>, Error> {
        // lógica de autenticação
        Ok(Connection {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}

// Apenas disponível quando autenticado
impl Connection<Authenticated> {
    pub fn send_data(&mut self, data: &[u8]) -> io::Result<()> {
        self.inner.write_all(data)
    }
    
    pub fn receive_data(&mut self) -> io::Result<Vec<u8>> {
        // implementação
    }
}

// Uso: O compilador força a ordem correta!
let conn = Connection::new("localhost:8080")?
    .connect()?
    .authenticate("token")?;

conn.send_data(b"Hello");  // ✅ Compila

// let conn = Connection::new("localhost:8080")?;
// conn.send_data(b"Hello");  // ❌ Erro de compilação!
```

### Builder Pattern com Tipos Fantasma

```rust
// Estados do builder
pub struct Incomplete;
pub struct Complete;

pub struct EmailBuilder<State = Incomplete> {
    local: Option<String>,
    domain: Option<String>,
    _state: PhantomData<State>,
}

impl EmailBuilder<Incomplete> {
    pub fn new() -> Self {
        Self {
            local: None,
            domain: None,
            _state: PhantomData,
        }
    }
    
    pub fn local(mut self, local: String) -> Self {
        self.local = Some(local);
        self
    }
    
    pub fn domain(mut self, domain: String) -> EmailBuilder<Complete> {
        EmailBuilder {
            local: self.local,
            domain: Some(domain),
            _state: PhantomData,
        }
    }
}

// build() só disponível quando Complete
impl EmailBuilder<Complete> {
    pub fn build(self) -> Email {
        Email(format!("{}@{}", 
            self.local.unwrap(), 
            self.domain.unwrap()
        ))
    }
}

// Uso
let email = EmailBuilder::new()
    .local("user".into())
    .domain("example.com".into())
    .build();  // ✅ Compila

// let email = EmailBuilder::new()
//     .local("user".into())
//     .build();  // ❌ Erro: build não existe em Incomplete
```

### Evitar Enums Booleanos e Strings Mágicas

```rust
// ❌ MAU: Booleans ambíguos
fn process_order(send_email: bool, is_priority: bool) { }
process_order(true, false);  // Qual é qual?

// ✅ BOM: Tipos nomeados
enum EmailPreference { Send, DontSend }
enum Priority { Normal, High }

fn process_order(email: EmailPreference, priority: Priority) { }
process_order(EmailPreference::Send, Priority::Normal);  // Claro!

// ❌ MAU: Strings mágicas
fn set_status(status: &str) {
    match status {
        "active" => { },
        "inactive" => { },
        _ => panic!("Invalid status"),  // Erro em runtime!
    }
}

// ✅ BOM: Enum exhaustivo
enum Status { Active, Inactive }

fn set_status(status: Status) {
    match status {
        Status::Active => { },
        Status::Inactive => { },
        // Compilador garante que todos os casos são cobertos
    }
}
```

### NonZero Types

```rust
use std::num::NonZeroU32;

// Garante que divisão é segura
fn calculate_average(total: u32, count: NonZeroU32) -> u32 {
    total / count.get()  // Nunca divide por zero!
}

// Construção segura
let count = NonZeroU32::new(5).expect("Count cannot be zero");
let avg = calculate_average(100, count);
```

### Resumo: Benefícios do Type-Driven Development

- ✅ **Bugs prevenidos em compile-time** ao invés de runtime
- ✅ **Documentação executável** - tipos dizem o que é válido
- ✅ **Refatoração segura** - mudanças de tipo propagam automaticamente
- ✅ **Menos testes necessários** - impossível testar estados inválidos
- ✅ **API mais clara** - tipos guiam o uso correto
- ✅ **Zero overhead** - tipos fantasma são eliminados na compilação

---

## Stack Recomendado (Padrão Ouro)

Para um sistema produtivo moderno, estas são as escolhas consolidadas pela indústria que garantem **robustez**, **performance** e **manutenibilidade de longo prazo**.

### Runtime e Async

**Tokio** - O runtime assíncrono padrão de fato
```toml
tokio = { version = "1.35", features = ["full"] }
```
- ✅ Mais maduro e testado em produção
- ✅ Ecossistema rico (hyper, tonic, etc)
- ✅ Performance excelente
- ❌ Alternativa: `async-std` (mais simples, menos features)

### Web Frameworks

**Axum** - Framework moderno e ergonômico
```toml
axum = "0.7"
tower = "0.4"  # Middleware
tower-http = "0.5"  # CORS, compression, etc
```
- ✅ Construído sobre tokio/hyper
- ✅ Type-safe, ergonômico
- ✅ Extractor pattern poderoso
- ❌ Alternativa: `actix-web` (mais maduro, ligeiramente mais rápido)

### Banco de Dados

**SQLx** - SQL com verificação em tempo de compilação
```toml
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros"] }
```
- ✅ Type-safe queries verificadas em compile-time
- ✅ Async nativo
- ✅ Migrations integradas
- ✅ Controle total do SQL
- ❌ Alternativa: `diesel` (ORM tradicional, sync primeiro)

**SeaORM** - ORM moderno async
```toml
sea-orm = "0.12"
```
- ✅ Async nativo
- ✅ Migrations e CLI
- ✅ Active Record pattern
- ⚠️ Menos controle que SQLx

### Serialização

**Serde** - Padrão onipresente
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```
- ✅ Performance excelente
- ✅ Suporta praticamente todos os formatos
- ✅ Código gerado em compile-time

### Gerenciamento de Erros

```toml
# Para bibliotecas
thiserror = "1.0"

# Para aplicações
anyhow = "1.0"
```

### Observabilidade

```toml
# Tracing estruturado
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# OpenTelemetry
opentelemetry = "0.21"
opentelemetry-jaeger = "0.20"
tracing-opentelemetry = "0.22"
```

### Configuração

**config-rs** - Gerenciamento hierárquico de configurações
```toml
config = "0.13"
```

```rust
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

fn load_config() -> Result<AppConfig, config::ConfigError> {
    let config = Config::builder()
        // Arquivo padrão
        .add_source(File::with_name("config/default"))
        // Arquivo específico do ambiente (opcional)
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        // Variáveis de ambiente com prefixo APP_
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;
    
    config.try_deserialize()
}

// Uso:
// config/default.toml:
// [server]
// host = "0.0.0.0"
// port = 8080
//
// Ou via env: APP_SERVER__PORT=3000
```

### HTTP Client

```toml
reqwest = { version = "0.11", features = ["json"] }
```

### Cache

```toml
redis = { version = "0.24", features = ["tokio-comp"] }
# Ou
moka = "0.12"  # In-memory cache
```

### Testes

```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
wiremock = "0.6"
proptest = "1.4"
criterion = "0.5"
testcontainers = "0.15"
```

### CLI (se aplicável)

```toml
clap = { version = "4.4", features = ["derive"] }
```

---

## Comparação de Alternativas

Nem sempre a escolha "padrão ouro" é a melhor para todo cenário. Aqui estão comparações práticas:

### Web Frameworks

| Framework | Quando Usar | Prós | Contras |
|-----------|-------------|------|---------|
| **Axum** | Projetos modernos, APIs REST/GraphQL | Type-safe, ergonômico, Tower ecosystem | Comunidade menor que Actix |
| **Actix-web** | Alta performance, projetos maduros | Muito rápido, battle-tested, maduro | API menos ergonômica |
| **Rocket** | Protótipos rápidos, APIs simples | Muito ergonômico, fácil de começar | Performance menor, menos async |
| **Warp** | Quando já usa Tokio/Hyper | Leve, composável | Curva de aprendizado íngreme |

**Recomendação**: Axum para novos projetos, Actix se performance é crítica.

### Banco de Dados

| Crate | Quando Usar | Prós | Contras |
|-------|-------------|------|---------|
| **SQLx** | Controle total, queries complexas | Compile-time verificação, async nativo, flexível | Requer banco rodando para compile-time checks |
| **Diesel** | ORMs tradicionais, type safety máxima | Extremamente type-safe, migrations robustas | Sync-first, curva de aprendizado |
| **SeaORM** | ORM async moderno | Async nativo, Active Record, CLI excelente | Menos maduro que Diesel |
| **Tokio-Postgres** | Baixo nível, máxima performance | Minimal overhead, controle total | Sem ORM, manual |

**Recomendação**: SQLx para flexibilidade + safety, Diesel se você precisa de ORM sync robusto.

### Runtime Async

| Runtime | Quando Usar | Prós | Contras |
|---------|-------------|------|---------|
| **Tokio** | Padrão para produção | Ecosystem gigante, maduro, features ricas | Binários maiores |
| **async-std** | Simplicidade, APIs familiares | API similar a std, mais simples | Ecosystem menor |
| **smol** | Aplicações pequenas, embarcados | Muito leve, simples | Features limitadas |

**Recomendação**: Tokio, sempre. Só considere alternativas em casos específicos (embedded).

### Serialização

| Crate | Quando Usar | Prós | Contras |
|-------|-------------|------|---------|
| **Serde** | 99% dos casos | Universal, performático, flexible | Macros aumentam compile time |
| **Prost** | Protobuf específico | Geração de código, interop | Apenas Protobuf |
| **Bincode** | Serialização binária Rust-to-Rust | Muito rápido, compacto | Apenas Rust |

**Recomendação**: Serde sempre, Bincode apenas para comunicação interna Rust.

### Logging/Tracing

| Crate | Quando Usar | Prós | Contras |
|-------|-------------|------|---------|
| **tracing** | Sistemas modernos, async | Estruturado, async-aware, contexto rico | Complexidade inicial |
| **log** | Aplicações simples, compatibilidade | Universal, simples | Sem estrutura, sem contexto async |
| **env_logger** | CLIs simples | Extremamente simples | Features mínimas |

**Recomendação**: `tracing` para qualquer sistema em produção, `log` apenas para ferramentas simples.

### Error Handling

| Crate | Quando Usar | Prós | Contras |
|-------|-------------|------|---------|
| **thiserror** | Bibliotecas | Errors tipados, pattern matching | Verboso |
| **anyhow** | Aplicações | Contexto rico, ergonômico | Type erasure |
| **eyre** | Aplicações com error reports | Reports bonitos, hooks | Overhead ligeiro |

**Recomendação**: thiserror para libs, anyhow para apps (ou eyre se quiser reports mais ricos).

### HTTP Client

| Crate | Quando Usar | Prós | Contras |
|-------|-------------|------|---------|
| **reqwest** | Alto nível, facilidade | Ergonômico, features ricas | Mais pesado |
| **hyper** | Baixo nível, performance | Rápido, controle total | Verboso, complexo |
| **ureq** | Sync, CLIs simples | Bloqueante, zero deps | Não async |

**Recomendação**: reqwest para 90% dos casos.

### Configuração

| Crate | Quando Usar | Prós | Contras |
|-------|-------------|------|---------|
| **config-rs** | Múltiplas fontes, hierarquia | Flexível, múltiplos formatos | Setup inicial complexo |
| **figment** | Rocket, ou type-safe config | Type-safe profiles | Menos features |
| **dotenv** | Configs simples | Extremamente simples | Apenas .env files |

**Recomendação**: config-rs para produção, dotenv apenas para desenvolvimento local.

---

## Stack Completa - Exemplo

```toml
[dependencies]
# Runtime
tokio = { version = "1.35", features = ["full"] }

# Web
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "macros"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
opentelemetry = "0.21"

# Config
config = "0.13"

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
criterion = "0.5"
```

---

## Estratégia de Testes

### Pirâmide de Testes

```
        /\
       /E2E\      5-10%  - Testes End-to-End
      /------\
     /  Int.  \   15-20% - Testes de Integração
    /----------\
   /   Unit     \ 70-80% - Testes Unitários
  /--------------\
```

### Distribuição Recomendada

**Objetivo**: Maximizar confiança e velocidade de feedback

- **Testes Unitários (70-80%)**: Rápidos, isolados, testam lógica específica
- **Testes de Integração (15-20%)**: Testam interação entre módulos
- **Testes E2E (5-10%)**: Testam fluxos completos como usuário

### Quantidade por Tamanho de Projeto

#### **Projeto Pequeno (< 5k LOC)**
- 50-100 testes unitários
- 10-20 testes de integração
- 2-5 testes E2E
- **Total**: ~70-125 testes

#### **Projeto Médio (5k-20k LOC)**
- 200-500 testes unitários
- 30-80 testes de integração
- 5-15 testes E2E
- 10-20 property tests
- **Total**: ~250-600 testes

#### **Projeto Grande (> 20k LOC)**
- 500-2000+ testes unitários
- 100-300 testes de integração
- 20-50 testes E2E
- 20-50 property tests
- **Total**: ~650-2400+ testes

---

## Tipos de Testes Detalhados

### 1. Testes Unitários (70-80%)

**Objetivo**: Testar funções/métodos isoladamente

```rust
// Em src/lib.rs ou em módulos
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_discount() {
        let result = calculate_discount(100.0, 0.1);
        assert_eq!(result, 90.0);
    }

    #[test]
    #[should_panic(expected = "Invalid discount")]
    fn test_invalid_discount_panics() {
        calculate_discount(100.0, 1.5);
    }

    #[test]
    fn test_error_handling() {
        let result = parse_email("invalid");
        assert!(result.is_err());
        
        match result {
            Err(EmailError::MissingAtSign) => (),
            _ => panic!("Expected MissingAtSign error"),
        }
    }
    
    #[test]
    fn test_edge_cases() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }
}
```

**O que testar:**
- ✅ Lógica de negócio pura
- ✅ Validações e parsing
- ✅ Transformações de dados
- ✅ Casos extremos (edge cases)
- ✅ Tratamento de erros
- ✅ Cada branch de if/match
- ✅ Boundary conditions

**Meta de cobertura**: 80-90% das linhas de código de lógica

### 2. Testes de Integração (15-20%)

**Objetivo**: Testar interação entre módulos

```rust
// Em tests/integration_test.rs
use meu_projeto::*;

#[test]
fn test_user_registration_flow() {
    let db = setup_test_db();
    let service = UserService::new(db);
    
    let user = service.register("test@example.com", "password123")
        .expect("Registration failed");
    
    assert_eq!(user.email, "test@example.com");
    assert!(user.id > 0);
    assert!(service.find_by_email("test@example.com").is_ok());
}

#[tokio::test]
async fn test_api_endpoint() {
    let app = spawn_test_server().await;
    
    let response = app
        .post("/api/users")
        .json(&json!({
            "email": "test@example.com",
            "password": "secure123"
        }))
        .await;
    
    assert_eq!(response.status(), 201);
    let body: User = response.json().await;
    assert_eq!(body.email, "test@example.com");
}

// Testar com banco real (usando testcontainers)
#[tokio::test]
async fn test_database_operations() {
    let container = testcontainers::clients::Cli::default()
        .run(postgres_image());
    
    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/test",
        container.get_host_port_ipv4(5432)
    );
    
    let pool = PgPool::connect(&connection_string).await.unwrap();
    
    // Executar migrations
    sqlx::migrate!().run(&pool).await.unwrap();
    
    // Testar operações
    let user = create_user(&pool, "test@example.com").await.unwrap();
    assert!(user.id > 0);
}
```

**O que testar:**
- ✅ Fluxos completos de casos de uso
- ✅ Integração com banco de dados
- ✅ Integração entre camadas (domain ↔ infra)
- ✅ Serialização/deserialização
- ✅ APIs internas
- ✅ Transações e rollbacks

**Ferramentas úteis:**
- `testcontainers`: Containers Docker para testes
- `mockall` ou `mockito`: Mocks e stubs
- `wiremock`: Mock de HTTP servers
- `fake`: Geração de dados fake

```rust
use mockall::*;

#[automock]
trait UserRepository {
    fn find(&self, id: u64) -> Option<User>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockUserRepository::new();
    mock.expect_find()
        .with(eq(1))
        .returning(|_| Some(User { id: 1, name: "Test".into() }));
    
    let service = UserService::new(mock);
    let user = service.get_user(1).unwrap();
    assert_eq!(user.name, "Test");
}
```

### 3. Testes End-to-End (5-10%)

**Objetivo**: Testar sistema completo como usuário

```rust
#[tokio::test]
async fn test_complete_order_flow() {
    // Setup: Servidor completo rodando
    let app = spawn_full_application().await;
    let client = TestClient::new(app);
    
    // 1. Registrar usuário
    let register_response = client
        .post("/auth/register")
        .json(&json!({
            "email": "customer@test.com",
            "password": "secure123"
        }))
        .await;
    assert_eq!(register_response.status(), 201);
    
    // 2. Login
    let login_response = client
        .post("/auth/login")
        .json(&json!({
            "email": "customer@test.com",
            "password": "secure123"
        }))
        .await;
    let token: String = login_response.json().await;
    
    // 3. Adicionar produto ao carrinho
    let add_response = client
        .post("/cart/items")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "product_id": 123,
            "quantity": 2
        }))
        .await;
    assert_eq!(add_response.status(), 200);
    
    // 4. Fazer checkout
    let checkout_response = client
        .post("/orders/checkout")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "payment_method": "credit_card",
            "card_token": "tok_test_123"
        }))
        .await;
    assert_eq!(checkout_response.status(), 201);
    
    // 5. Verificar pedido criado
    let order: Order = checkout_response.json().await;
    assert_eq!(order.status, OrderStatus::Confirmed);
    assert_eq!(order.items.len(), 1);
    assert_eq!(order.items[0].quantity, 2);
}
```

**O que testar:**
- ✅ Jornadas críticas do usuário
- ✅ Fluxos de ponta a ponta
- ✅ Cenários de falha e recuperação
- ✅ Interações entre múltiplos sistemas

### 4. Property-Based Testing

**Objetivo**: Testar propriedades invariantes com dados aleatórios

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode_roundtrip(data: Vec<u8>) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(data, decoded);
    }

    #[test]
    fn test_sort_preserves_length(mut vec: Vec<i32>) {
        let original_len = vec.len();
        vec.sort();
        prop_assert_eq!(vec.len(), original_len);
    }
    
    #[test]
    fn test_addition_commutative(a: i32, b: i32) {
        prop_assert_eq!(a + b, b + a);
    }
    
    #[test]
    fn test_email_validation_consistency(email in "[a-z]{5}@[a-z]{5}\\.com") {
        let validated1 = validate_email(&email);
        let validated2 = validate_email(&email);
        prop_assert_eq!(validated1.is_ok(), validated2.is_ok());
    }
}
```

**Quando usar:**
- ✅ Funções com propriedades matemáticas (comutatividade, associatividade, idempotência)
- ✅ Parsers e serialização
- ✅ Algoritmos de ordenação/busca
- ✅ Validações complexas
- ✅ Encoders/decoders

**Quantidade**: 10-20 testes property-based estratégicos

### 5. Testes de Performance/Benchmark

```rust
// Em benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_parse(c: &mut Criterion) {
    c.bench_function("parse large json", |b| {
        let json = load_test_data();
        b.iter(|| parse_json(black_box(&json)))
    });
}

fn benchmark_with_parameters(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let data = vec![0u8; size];
            b.iter(|| hash_data(black_box(&data)));
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_parse, benchmark_with_parameters);
criterion_main!(benches);
```

**Quando fazer:**
- ✅ Operações críticas de performance (hot paths)
- ✅ Comparar implementações alternativas
- ✅ Prevenir regressões de performance
- ✅ Otimizações de algoritmos

**Quantidade**: 5-15 benchmarks para operações críticas

### 6. Testes de Contrato

Para microsserviços e APIs:

```rust
use serde_json::json;

#[test]
fn test_api_contract_user_response() {
    let response = get_user_api_response(1);
    
    // Verifica estrutura do contrato
    assert!(response["id"].is_number());
    assert!(response["email"].is_string());
    assert!(response["created_at"].is_string());
    
    // Validar contra JSON Schema
    let schema = load_json_schema("schemas/user_v1.json");
    assert!(validate_json_schema(&response, &schema).is_ok());
}

#[test]
fn test_backwards_compatibility() {
    // Garantir que novos campos não quebram clientes antigos
    let old_client_expected_fields = vec!["id", "email", "name"];
    let response = get_user_api_response(1);
    
    for field in old_client_expected_fields {
        assert!(response.get(field).is_some(), "Missing field: {}", field);
    }
}
```

### 7. Testes de Snapshot

```rust
use insta::assert_snapshot;

#[test]
fn test_render_html_template() {
    let data = TemplateData {
        title: "Test Page",
        items: vec!["Item 1", "Item 2"],
    };
    
    let html = render_template(&data);
    assert_snapshot!(html);
}

#[test]
fn test_generated_sql_query() {
    let query = QueryBuilder::new()
        .select(&["id", "name", "email"])
        .from("users")
        .where_clause("age > 18")
        .order_by("name")
        .build();
    
    assert_snapshot!(query);
}
```

---

## Práticas de Teste em Rust

### 1. Organização de Fixtures

```rust
// tests/common/mod.rs
use sqlx::PgPool;

pub struct TestDb {
    pool: PgPool,
}

impl TestDb {
    pub async fn new() -> Self {
        let pool = PgPool::connect("postgresql://localhost/test").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        Self { pool }
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        // Cleanup
    }
}

pub fn create_test_user() -> User {
    User {
        id: 1,
        email: "test@example.com".into(),
        name: "Test User".into(),
    }
}

// tests/integration_test.rs
mod common;

#[tokio::test]
async fn my_test() {
    let db = common::TestDb::new().await;
    let user = common::create_test_user();
    // ...
}
```

### 2. Testes Assíncronos

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = fetch_data().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_operations() {
    let results = futures::future::join_all(vec![
        async_task_1(),
        async_task_2(),
        async_task_3(),
    ]).await;
    
    assert!(results.iter().all(|r| r.is_ok()));
}

// Testar timeout
#[tokio::test]
async fn test_operation_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        slow_operation()
    ).await;
    
    assert!(result.is_err()); // Deve dar timeout
}
```

### 3. Testes Parametrizados

```rust
use rstest::rstest;

#[rstest]
#[case(0, 0)]
#[case(1, 1)]
#[case(2, 2)]
#[case(5, 120)]
#[case(10, 3628800)]
fn test_factorial(#[case] input: u32, #[case] expected: u32) {
    assert_eq!(factorial(input), expected);
}

#[rstest]
#[case("user@example.com", true)]
#[case("invalid.email", false)]
#[case("no-at-sign.com", false)]
#[case("@no-local-part.com", false)]
fn test_email_validation(#[case] email: &str, #[case] expected: bool) {
    assert_eq!(is_valid_email(email), expected);
}
```

### 4. Test Helpers e Macros

```rust
// Macro para testes que precisam de setup/teardown
macro_rules! db_test {
    ($name:ident, $test:expr) => {
        #[tokio::test]
        async fn $name() {
            let db = setup_test_db().await;
            $test(db).await;
            teardown_test_db().await;
        }
    };
}

db_test!(test_create_user, |db| async move {
    let user = create_user(&db, "test@example.com").await;
    assert!(user.is_ok());
});

// Builder para dados de teste
struct UserBuilder {
    email: String,
    name: String,
    age: u8,
}

impl UserBuilder {
    fn new() -> Self {
        Self {
            email: "default@test.com".into(),
            name: "Default User".into(),
            age: 25,
        }
    }
    
    fn with_email(mut self, email: &str) -> Self {
        self.email = email.into();
        self
    }
    
    fn with_age(mut self, age: u8) -> Self {
        self.age = age;
        self
    }
    
    fn build(self) -> User {
        User {
            email: self.email,
            name: self.name,
            age: self.age,
        }
    }
}

// Uso
#[test]
fn test_adult_user() {
    let user = UserBuilder::new()
        .with_email("adult@test.com")
        .with_age(30)
        .build();
    
    assert!(user.is_adult());
}
```

### 5. Padrão AAA (Arrange-Act-Assert)

```rust
#[test]
fn test_order_total_calculation() {
    // Arrange: Preparar dados
    let mut order = Order::new();
    order.add_item(OrderItem {
        product_id: 1,
        quantity: 2,
        unit_price: 50.0,
    });
    order.add_item(OrderItem {
        product_id: 2,
        quantity: 1,
        unit_price: 30.0,
    });
    
    // Act: Executar ação
    let total = order.calculate_total();
    
    // Assert: Verificar resultado
    assert_eq!(total, 130.0);
}
```

---

## Métricas de Qualidade

### Cobertura de Código

```bash
# Instalar
cargo install cargo-llvm-cov

# Gerar relatório HTML
cargo llvm-cov --html

# Gerar relatório LCOV (para CI)
cargo llvm-cov --lcov --output-path coverage.lcov

# Por crate específico
cargo llvm-cov --package meu_crate

# Ignorar arquivos
cargo llvm-cov --ignore-filename-regex tests
```

**Metas de Cobertura:**
- **Mínima aceitável**: 70%
- **Ideal**: 80-90%
- **Código crítico (domínio)**: 90%+
- **Infraestrutura**: 60-70%

### Mutation Testing

```bash
cargo install cargo-mutants

# Executar mutation testing
cargo mutants

# Verificar se testes matam os mutantes
cargo mutants --check
```

**O que é**: Introduz pequenas mudanças (mutações) no código e verifica se os testes falham. Se não falharem, os testes são insuficientes.

### Métricas de Qualidade

```bash
# Complexidade ciclomática
cargo install cargo-geiger
cargo geiger

# Linhas de código
tokei

# Auditoria de segurança
cargo audit

# Dependências desatualizadas
cargo outdated

# Licenças
cargo-deny check
```

---

## CI/CD Pipeline

### GitHub Actions Completo

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: testdb
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          override: true
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test '*'
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/testdb
      
      - name: Run doc tests
        run: cargo test --doc

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy
      - run: cargo clippy --all-targets --all-features -- -D warnings

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      
      - name: Upload to codecov.io
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  benchmarks:
    name: Benchmarks
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
      
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/report/index.html
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - test
  - quality
  - security

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  paths:
    - .cargo/
    - target/

test:unit:
  stage: test
  image: rust:latest
  script:
    - cargo test --lib

test:integration:
  stage: test
  image: rust:latest
  services:
    - postgres:15
  variables:
    POSTGRES_DB: testdb
    POSTGRES_PASSWORD: postgres
  script:
    - cargo test --test '*'

quality:fmt:
  stage: quality
  image: rust:latest
  script:
    - rustup component add rustfmt
    - cargo fmt -- --check

quality:clippy:
  stage: quality
  image: rust:latest
  script:
    - rustup component add clippy
    - cargo clippy --all-targets -- -D warnings

quality:coverage:
  stage: quality
  image: rust:latest
  script:
    - cargo install cargo-llvm-cov
    - cargo llvm-cov --lcov --output-path coverage.lcov
  coverage: '/\d+\.\d+% coverage/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage.lcov

security:audit:
  stage: security
  image: rust:latest
  script:
    - cargo install cargo-audit
    - cargo audit
```

---

## Quando NÃO Testar

É importante saber o que **não** precisa de testes:

- ❌ **Getters/setters triviais** sem lógica
- ❌ **Código gerado automaticamente** (ex: macros derive)
- ❌ **Código que apenas delega** para outra função
- ❌ **Configuração simples** (structs de config)
- ❌ **DTOs sem lógica** (Plain Old Data)
- ❌ **Constantes**
- ❌ **Código privado simples** já coberto por testes de funções públicas

---

## Regras de Ouro para Testes

### 1. FIRST Principles

- **F**ast: Testes devem rodar rapidamente
- **I**ndependent: Não devem depender uns dos outros
- **R**epeatable: Mesmo resultado toda vez
- **S**elf-validating: Pass ou fail, nada manual
- **T**imely: Escritos junto com o código (ou antes, TDD)

### 2. Princípios Adicionais

1. **Testes devem ser mais simples que o código testado**
2. **Um conceito por teste** (não necessariamente um assert)
3. **Nomes descritivos**: `test_user_registration_fails_with_invalid_email`
4. **AAA Pattern**: Arrange-Act-Assert
5. **Testes são documentação viva** - devem ser legíveis
6. **Evitar lógica em testes** (if/loops)
7. **DRY com moderação** - duplicação às vezes é OK para clareza

### 3. Boas Práticas

```rust
// ✅ BOM: Nome descritivo, AAA claro
#[test]
fn test_order_total_with_discount_and_tax() {
    // Arrange
    let order = Order::new()
        .add_item(100.0, 2)
        .with_discount(0.1)
        .with_tax_rate(0.08);
    
    // Act
    let total = order.calculate_total();
    
    // Assert
    assert_eq!(total, 194.4); // (200 - 20) * 1.08
}

// ❌ RUIM: Nome genérico, sem estrutura clara
#[test]
fn test1() {
    let o = Order::new();
    o.add_item(100.0, 2);
    o.with_discount(0.1);
    o.with_tax_rate(0.08);
    assert_eq!(o.calculate_total(), 194.4);
}
```

---

## Regras de Ouro da Robustez em Rust

Antes de finalizar, aqui estão os princípios fundamentais que todo desenvolvedor Rust deve internalizar:

### 1. **Parse, Don't Validate**
```rust
// ❌ Validar repetidamente
fn process_email(email: &str) {
    if !is_valid_email(email) { return; }
    send_email(email);
}

fn send_email(email: &str) {
    if !is_valid_email(email) { return; }  // Validação duplicada!
    // ...
}

// ✅ Validar uma vez, usar tipo seguro
fn process_email(email: Email) {
    send_email(email);
}

fn send_email(email: Email) {
    // Garantido válido, não precisa re-validar
}
```

### 2. **Make Illegal States Unrepresentable**
Use o sistema de tipos para tornar bugs impossíveis, não difíceis.

### 3. **Fail Fast, Fail Safe**
Prefira panic durante inicialização a falhar silenciosamente em produção.

### 4. **Errors are Values, Not Exceptions**
Trate erros como dados usando `Result<T, E>`, não como exceções.

### 5. **Prefer Message Passing to Shared State**
Canais > Arc<Mutex> sempre que possível.

### 6. **Zero-Cost Abstractions Don't Mean Zero Cost**
Abstrações de custo zero ainda têm custo de desenvolvimento - use quando valer a pena.

### 7. **Compiler Is Your Friend, Not Your Enemy**
Se não compila, geralmente há um bom motivo. Lute contra o borrow checker no início, colabore depois.

### 8. **Documentation Is Code**
Doctests garantem que exemplos funcionam. Sem desculpas para docs desatualizadas.

### 9. **Test Behavior, Not Implementation**
Teste o "o quê", não o "como". Refatoração não deve quebrar testes.

### 10. **Performance Is a Feature, But Correctness Comes First**
Otimize depois de medir. Premature optimization é a raiz de todo mal.

---

## Resumo Executivo

### Checklist de Sistema Robusto

- [ ] **Type-Driven Development** - Estados inválidos irrepresentáveis
- [ ] **Arquitetura modular** com separação clara de responsabilidades (Hexagonal/Clean)
- [ ] **Workspace** dividido em crates com compilação incremental
- [ ] **Tipos fortes** que previnem bugs em tempo de compilação
- [ ] **Tratamento de erros** explícito (thiserror para libs, anyhow para apps)
- [ ] **Concorrência segura** (preferir canais sobre Arc<Mutex>)
- [ ] **Testes automatizados** com 80%+ de cobertura (unitários, integração, E2E, property-based)
- [ ] **CI/CD** rodando testes, linting (clippy --deny) e security checks
- [ ] **Documentação** completa (código, API, arquitetura, ADRs)
- [ ] **Observabilidade** (tracing estruturado, OpenTelemetry, métricas)
- [ ] **Gerenciamento de dependências** consciente e atualizado
- [ ] **Performance** medida (benchmarks) e otimizada (Cow, zero-copy)
- [ ] **Segurança** auditada regularmente (cargo audit)
- [ ] **Stack consolidada** (Tokio, Axum, SQLx, Serde, Tracing)

### Ferramentas Essenciais

```bash
# Instalar todas de uma vez
cargo install \
  cargo-watch \
  cargo-nextest \
  cargo-audit \
  cargo-llvm-cov \
  cargo-outdated \
  cargo-edit \
  cargo-deny \
  cargo-expand

# Opcional mas recomendado
cargo install cargo-flamegraph  # Profiling
cargo install cargo-mutants     # Mutation testing
```

### Comandos Diários

```bash
# Desenvolvimento
cargo watch -x test              # Testes contínuos
cargo clippy --fix               # Corrigir warnings
cargo fmt                        # Formatar código

# Qualidade
cargo test                       # Todos os testes
cargo llvm-cov --html            # Cobertura
cargo audit                      # Vulnerabilidades

# Release
cargo build --release            # Build otimizado
cargo doc --open                 # Gerar documentação
```

---

## Conclusão

Um sistema robusto em Rust combina:

1. **Type Safety Máxima** - Estados inválidos irrepresentáveis em compile-time
2. **Arquitetura Limpa** - Hexagonal/Clean com Workspace modularizado
3. **Testes Abrangentes** - Unitários, integração, E2E, property-based (80%+ cobertura)
4. **Observabilidade Rica** - Tracing estruturado + OpenTelemetry + Métricas
5. **Stack Consolidada** - Tokio, Axum, SQLx, Serde, Config-rs, Tracing
6. **Erros Estruturados** - thiserror (libs) + anyhow (apps)
7. **Concorrência Segura** - Message passing > estado compartilhado
8. **Otimizações Inteligentes** - Cow, zero-copy, lazy evaluation
9. **CI/CD Rigoroso** - Clippy --deny, audit, coverage, benchmarks
10. **Práticas de Longo Prazo** - Documentação viva, ADRs, SemVer estrito

### Princípios-Chave para Lembrar

- 🛡️ **Parse, don't validate** - Valide uma vez na entrada, use tipos seguros depois
- 🚀 **Zero-cost abstractions** - Performance sem sacrificar ergonomia
- 📈 **Compilação incremental** - Workspace acelera builds em projetos grandes
- 🔍 **Fail fast, fail safe** - Erros explícitos > falhas silenciosas
- 🧪 **Test pyramid** - 70% unitários, 20% integração, 10% E2E
- 📊 **Observe everything** - Logs não bastam, use tracing + métricas
- 🔧 **Tooling matters** - Clippy, rustfmt, audit são obrigatórios, não opcionais

**Lembre-se**: A robustez vem da combinação de todas essas práticas, não de uma única técnica. Comece com o básico (type safety, testes unitários, tratamento de erros) e evolua incrementalmente.

**Qualidade > Quantidade**: 80% de cobertura bem feita é melhor que 100% de testes ruins. Um tipo bem desenhado previne mais bugs que milhares de testes.

---

**Próximos Passos Sugeridos:**

1. ✅ **Setup inicial**: Crie Workspace com estrutura hexagonal
2. ✅ **CI/CD**: Configure desde o dia 1 (testes + clippy + audit)
3. ✅ **Type-Driven**: Modele domínio com tipos que impedem estados inválidos
4. ✅ **Testes**: TDD ou testes logo após implementação (nunca deixe para depois)
5. ✅ **Observabilidade**: Integre tracing desde o início
6. ✅ **Documentação**: ADRs para decisões importantes
7. ✅ **Review**: Revise e refatore regularmente (debt técnico acumula rápido)
8. ✅ **Dependências**: Auditoria mensal com cargo-outdated e cargo-audit
9. ✅ **Performance**: Benchmark hot paths antes de otimizar (premature optimization is evil)
10. ✅ **Comunidade**: Contribua e aprenda - Rust tem uma das melhores comunidades tech

**Recursos Adicionais:**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [tokio.rs](https://tokio.rs) - Async runtime
- [crates.io](https://crates.io) - Registro de pacotes
