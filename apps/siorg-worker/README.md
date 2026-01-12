# SIORG Sync Worker (Standalone)

Worker dedicado para processamento assíncrono da fila de sincronização com SIORG.

## 📋 Visão Geral

Este worker processa continuamente itens da tabela `siorg_sync_queue`, sincronizando dados com a API SIORG do governo brasileiro. Ele é projetado para rodar como um processo separado do API server, permitindo escalabilidade horizontal e isolamento de recursos.

## 🎯 Funcionalidades

- **Processamento em Lote**: Processa múltiplos itens da fila por vez
- **Retry Logic**: Reprocessa itens falhados com exponential backoff
- **FOR UPDATE SKIP LOCKED**: Permite múltiplos workers rodando em paralelo sem conflitos
- **Cleanup Automático**: Remove itens expirados da fila periodicamente
- **Logging Estruturado**: Suporta formato JSON para integração com sistemas de logging
- **Graceful Shutdown**: Para processamento de forma segura em deploy/restart

## 🚀 Como Usar

### Instalação Local

1. **Configure as variáveis de ambiente:**
```bash
cd apps/siorg-worker
cp .env.example .env
# Edite .env com suas configurações
```

2. **Execute o worker:**
```bash
cargo run --bin siorg-worker
```

### Docker

1. **Build da imagem:**
```bash
docker build -t waterswamp-siorg-worker -f apps/siorg-worker/Dockerfile .
```

2. **Execute o container:**
```bash
docker run --env-file apps/siorg-worker/.env waterswamp-siorg-worker
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: siorg-worker
spec:
  replicas: 3  # Escale horizontalmente conforme necessário
  selector:
    matchLabels:
      app: siorg-worker
  template:
    metadata:
      labels:
        app: siorg-worker
    spec:
      containers:
      - name: worker
        image: waterswamp-siorg-worker:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        - name: SIORG_API_TOKEN
          valueFrom:
            secretKeyRef:
              name: siorg-credentials
              key: token
        - name: WORKER_BATCH_SIZE
          value: "10"
        - name: LOG_FORMAT
          value: "json"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

## ⚙️ Configuração

Todas as configurações são feitas via variáveis de ambiente:

### Database

| Variável | Descrição | Padrão | Obrigatório |
|----------|-----------|--------|-------------|
| `DATABASE_URL` | URL de conexão PostgreSQL | - | ✅ |

### SIORG API

| Variável | Descrição | Padrão | Obrigatório |
|----------|-----------|--------|-------------|
| `SIORG_API_URL` | URL base da API SIORG | `https://api.siorg.gov.br` | ❌ |
| `SIORG_API_TOKEN` | Token de autenticação | - | ❌ |

### Worker

| Variável | Descrição | Padrão | Obrigatório |
|----------|-----------|--------|-------------|
| `WORKER_BATCH_SIZE` | Número de itens por lote | `10` | ❌ |
| `WORKER_POLL_INTERVAL_SECS` | Intervalo entre polls (segundos) | `5` | ❌ |
| `WORKER_MAX_RETRIES` | Tentativas máximas por item | `3` | ❌ |
| `WORKER_RETRY_BASE_DELAY_MS` | Delay base para retry (ms) | `1000` | ❌ |
| `WORKER_RETRY_MAX_DELAY_MS` | Delay máximo para retry (ms) | `60000` | ❌ |
| `WORKER_ENABLE_CLEANUP` | Habilita limpeza de itens expirados | `true` | ❌ |
| `WORKER_CLEANUP_INTERVAL_SECS` | Intervalo de limpeza (segundos) | `3600` | ❌ |

### Logging

| Variável | Descrição | Padrão | Obrigatório |
|----------|-----------|--------|-------------|
| `RUST_LOG` | Nível de logging | `info,siorg_worker=debug` | ❌ |
| `LOG_FORMAT` | Formato de log (`text` ou `json`) | `text` | ❌ |

## 📊 Monitoramento

### Logs

O worker emite logs estruturados com informações sobre:
- Itens processados com sucesso
- Falhas e erros (com stack traces)
- Conflitos detectados
- Estatísticas de processamento por lote
- Operações de cleanup

Exemplo de log (formato JSON):
```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "INFO",
  "message": "Batch complete: 10 processed, 8 succeeded, 1 failed, 1 conflicts, 0 skipped",
  "target": "application::workers::siorg_sync_worker"
}
```

### Métricas (Futuro)

Planejado para implementação:
- Taxa de processamento (itens/segundo)
- Taxa de sucesso/falha
- Latência média de processamento
- Tamanho da fila ao longo do tempo

## 🔄 Escalabilidade

### Múltiplos Workers

É seguro rodar múltiplas instâncias do worker em paralelo:

```bash
# Terminal 1
WORKER_BATCH_SIZE=5 cargo run --bin siorg-worker

# Terminal 2
WORKER_BATCH_SIZE=5 cargo run --bin siorg-worker

# Terminal 3
WORKER_BATCH_SIZE=5 cargo run --bin siorg-worker
```

**Como funciona:**
- Cada worker usa `FOR UPDATE SKIP LOCKED` para adquirir itens da fila
- Se um worker já estiver processando um item, outros workers o ignoram
- Não há risco de processamento duplicado

### Recomendações de Escala

| Carga | Workers Recomendados | Batch Size | Poll Interval |
|-------|---------------------|------------|---------------|
| Baixa (< 100/dia) | 1 | 10 | 10s |
| Média (100-1000/dia) | 2-3 | 10-20 | 5s |
| Alta (1000-5000/dia) | 3-5 | 20-50 | 2s |
| Muito Alta (> 5000/dia) | 5-10 | 50-100 | 1s |

## 🐛 Troubleshooting

### Worker não processa itens

1. **Verifique se há itens PENDING na fila:**
```sql
SELECT COUNT(*) FROM siorg_sync_queue WHERE status = 'PENDING';
```

2. **Verifique se itens não estão expirados:**
```sql
SELECT * FROM siorg_sync_queue
WHERE status = 'PENDING'
  AND (expires_at IS NULL OR expires_at > NOW());
```

3. **Verifique logs do worker:**
```bash
RUST_LOG=debug cargo run --bin siorg-worker
```

### Itens sempre falhando

1. **Verifique o erro específico:**
```sql
SELECT id, siorg_code, attempts, last_error, error_details
FROM siorg_sync_queue
WHERE status = 'FAILED'
ORDER BY created_at DESC
LIMIT 10;
```

2. **Verifique conectividade com SIORG:**
```bash
curl -H "Authorization: Bearer $SIORG_API_TOKEN" https://api.siorg.gov.br/health
```

### Performance degradada

1. **Verifique uso de recursos:**
```bash
docker stats siorg-worker
```

2. **Aumente recursos ou número de workers**

3. **Ajuste batch size e poll interval**

## 🔐 Segurança

- ✅ Credenciais carregadas de variáveis de ambiente (não hardcoded)
- ✅ Conexões HTTPS com SIORG API
- ✅ Logs não expõem dados sensíveis
- ✅ Suporta secrets do Kubernetes

## 📝 Desenvolvimento

### Build de Produção

```bash
cargo build --release --bin siorg-worker
./target/release/siorg-worker
```

### Testes

```bash
# Unit tests do worker core
cargo test --package application --lib workers

# Integration tests
cargo test --package siorg-worker
```

### Debugging

```bash
# Com logs detalhados
RUST_LOG=trace cargo run --bin siorg-worker

# Com backtrace em erros
RUST_BACKTRACE=1 cargo run --bin siorg-worker
```

## 📚 Arquitetura

```
┌─────────────────────────────────────────────────┐
│          siorg_sync_queue (PostgreSQL)          │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │  PENDING  PROCESSING  COMPLETED  FAILED  │  │
│  └──────────────────────────────────────────┘  │
└──────────────────┬──────────────────────────────┘
                   │
         ┌─────────┴──────────┐
         │                    │
    ┌────▼─────┐        ┌─────▼────┐
    │ Worker 1 │        │ Worker N │
    │          │   ...  │          │
    │ [Batch]  │        │ [Batch]  │
    └────┬─────┘        └─────┬────┘
         │                    │
         └─────────┬──────────┘
                   │
           ┌───────▼───────┐
           │  SIORG API    │
           │               │
           └───────────────┘
```

## 🔗 Links Relacionados

- [API Server (com worker embutido)](../api-server/README.md)
- [Documentação SIORG](https://api.siorg.gov.br/docs)
- [Arquitetura do Sistema](../../docs/architecture.md)

## 📄 Licença

Este projeto faz parte do Waterswamp e segue a mesma licença do projeto principal.
