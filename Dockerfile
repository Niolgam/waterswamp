# =============================================================================
# DOCKERFILE MULTI-STAGE - WATERSWAMP API
# =============================================================================
# Este Dockerfile usa a estratégia multi-stage para criar uma imagem otimizada
# 
# Benefícios:
# - Imagem final pequena (~50MB vs ~2GB da imagem de build)
# - Tempo de build otimizado com cache de dependências
# - Binário otimizado para produção (LTO, strip)
# - Compatível com arquiteturas amd64 e arm64
#
# =============================================================================

# -----------------------------------------------------------------------------
# STAGE 1: PLANNER - Analisa dependências
# -----------------------------------------------------------------------------
FROM rust:1.83-bookworm AS planner

WORKDIR /app

# Instala cargo-chef (ferramenta de cache de dependências)
RUN cargo install cargo-chef

# Copia apenas os arquivos de manifesto para análise
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Gera o "recipe" (lista de dependências)
RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# STAGE 2: BUILDER - Compila dependências
# -----------------------------------------------------------------------------
FROM rust:1.83-bookworm AS cacher

WORKDIR /app

# Instala dependências de sistema necessárias para compilação
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Instala cargo-chef
RUN cargo install cargo-chef

# Copia o recipe do stage anterior
COPY --from=planner /app/recipe.json recipe.json

# Compila APENAS as dependências (isso fica em cache!)
RUN cargo chef cook --release --recipe-path recipe.json

# -----------------------------------------------------------------------------
# STAGE 3: BUILDER - Compila a aplicação
# -----------------------------------------------------------------------------
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Instala dependências de sistema
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copia as dependências compiladas do stage anterior
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

# Copia o código da aplicação
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY rbac_model.conf ./

# Compila a aplicação com otimizações máximas
ENV RUSTFLAGS="-C target-cpu=generic"
RUN cargo build --release

# Strip: Remove símbolos de debug
RUN strip target/release/waterswamp

# -----------------------------------------------------------------------------
# STAGE 4: RUNTIME - Imagem final (SLIM)
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Instala apenas as bibliotecas dinâmicas necessárias em runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Cria usuário não-root para segurança
RUN useradd -m -u 1000 appuser

# Copia o binário compilado do stage anterior
COPY --from=builder /app/target/release/waterswamp /usr/local/bin/waterswamp

# Copia o arquivo de modelo do Casbin
COPY --from=builder /app/rbac_model.conf /app/rbac_model.conf

# Troca para o usuário não-root
USER appuser

# Expõe a porta padrão
EXPOSE 3000

# Define o comando de execução
ENTRYPOINT ["/usr/local/bin/waterswamp"]

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1
