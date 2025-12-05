# Guia de Métricas Prometheus 📊

Este documento descreve todas as métricas disponíveis no Waterswamp e como usá-las para monitoramento e alertas.

## 🎯 Endpoint de Métricas

As métricas são expostas no endpoint:

```
GET /metrics
```

Configure o Prometheus para fazer scraping deste endpoint:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'waterswamp'
    static_configs:
      - targets: ['localhost:3000']
    scrape_interval: 15s
```

## 📈 Métricas Disponíveis

### 1. HTTP Requests

#### `http_requests_total` (Counter)

Total de requisições HTTP recebidas.

**Labels:**
- `method`: Método HTTP (GET, POST, PUT, DELETE, etc)
- `status`: Status code HTTP (200, 404, 500, etc)
- `path`: Caminho da requisição

**Exemplos:**

```promql
# Taxa de requisições por minuto
rate(http_requests_total[1m])

# Requisições por endpoint
sum by (path) (rate(http_requests_total[5m]))

# Taxa de erros 5xx
rate(http_requests_total{status=~"5.."}[1m])

# Taxa de sucesso (2xx)
rate(http_requests_total{status=~"2.."}[1m])
```

**Alert:**

```yaml
- alert: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 10
  for: 5m
  annotations:
    summary: "Alta taxa de erros 5xx"
    description: "{{ $value }} erros/segundo nos últimos 5 minutos"
```

---

#### `http_request_duration_seconds` (Histogram)

Latência das requisições HTTP.

**Labels:**
- `method`: Método HTTP
- `path`: Caminho da requisição

**Buckets:** 1ms, 5ms, 10ms, 50ms, 100ms, 500ms, 1s, 5s, 10s

**Exemplos:**

```promql
# P50 (mediana) de latência
histogram_quantile(0.5, rate(http_request_duration_seconds_bucket[5m]))

# P95 de latência
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# P99 de latência por endpoint
histogram_quantile(0.99, sum by (path, le) (rate(http_request_duration_seconds_bucket[5m])))

# Latência média
rate(http_request_duration_seconds_sum[5m]) / rate(http_request_duration_seconds_count[5m])
```

**Alert:**

```yaml
- alert: HighLatency
  expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
  for: 5m
  annotations:
    summary: "Latência alta detectada"
    description: "P95 está em {{ $value }}s"
```

---

### 2. Autenticação

#### `login_attempts_total` (Counter)

Total de tentativas de login.

**Labels:**
- `result`: Resultado (success, failure)

**Exemplos:**

```promql
# Taxa de login bem-sucedidos por minuto
rate(login_attempts_total{result="success"}[1m])

# Taxa de falhas de login
rate(login_attempts_total{result="failure"}[1m])

# Razão de sucesso
rate(login_attempts_total{result="success"}[5m]) /
rate(login_attempts_total[5m])

# Detecção de ataque de força bruta
rate(login_attempts_total{result="failure"}[1m]) > 10
```

**Alert:**

```yaml
- alert: BruteForceAttack
  expr: rate(login_attempts_total{result="failure"}[1m]) > 50
  for: 2m
  annotations:
    summary: "Possível ataque de força bruta"
    description: "{{ $value }} tentativas de login falhadas/segundo"
```

---

#### `token_refresh_total` (Counter)

Total de renovações de tokens.

**Labels:**
- `result`: Resultado (success, failure)

**Exemplos:**

```promql
# Taxa de refresh bem-sucedidos
rate(token_refresh_total{result="success"}[5m])

# Taxa de refresh failures (possível token theft)
rate(token_refresh_total{result="failure"}[5m])
```

---

#### `token_theft_detected_total` (Counter)

**NOVA MÉTRICA** - Detecções de possível roubo de tokens.

**Labels:**
- `detection_type`: Tipo de detecção (reuse, family_invalidation)

**Exemplos:**

```promql
# Total de detecções por hora
increase(token_theft_detected_total[1h])

# Detecções de reutilização de tokens
rate(token_theft_detected_total{detection_type="reuse"}[5m])

# Detecções de invalidação de família
rate(token_theft_detected_total{detection_type="family_invalidation"}[5m])
```

**Alert:**

```yaml
- alert: TokenTheftDetected
  expr: increase(token_theft_detected_total[5m]) > 0
  annotations:
    summary: "Possível roubo de token detectado"
    description: "{{ $value }} detecções nos últimos 5 minutos"
```

---

### 3. Password Operations

#### `password_hash_duration_seconds` (Histogram)

**NOVA MÉTRICA** - Tempo de operações de hash/verify de senha.

**Labels:**
- `operation`: Tipo de operação (hash, verify)

**Buckets:** 10ms, 50ms, 100ms, 150ms, 200ms, 250ms, 300ms, 400ms, 500ms, 1s

**Exemplos:**

```promql
# P95 de tempo de hash
histogram_quantile(0.95, rate(password_hash_duration_seconds_bucket{operation="hash"}[5m]))

# Tempo médio de verificação
rate(password_hash_duration_seconds_sum{operation="verify"}[5m]) /
rate(password_hash_duration_seconds_count{operation="verify"}[5m])

# Comparação hash vs verify
histogram_quantile(0.95, rate(password_hash_duration_seconds_bucket[5m])) by (operation)
```

**Alert:**

```yaml
- alert: SlowPasswordHashing
  expr: histogram_quantile(0.95, rate(password_hash_duration_seconds_bucket{operation="hash"}[5m])) > 0.5
  for: 10m
  annotations:
    summary: "Password hashing muito lento"
    description: "P95 está em {{ $value }}s. Considere ajustar parâmetros Argon2"
```

---

### 4. Casbin (Autorização)

#### `policy_cache_hits_total` (Counter)

Total de cache hits/misses do Casbin.

**Labels:**
- `result`: Resultado (hit, miss)

**Exemplos:**

```promql
# Taxa de cache hit
rate(policy_cache_hits_total{result="hit"}[5m]) /
rate(policy_cache_hits_total[5m])

# Cache hit rate em percentual
100 * (
  rate(policy_cache_hits_total{result="hit"}[5m]) /
  rate(policy_cache_hits_total[5m])
)

# Taxa de cache misses
rate(policy_cache_hits_total{result="miss"}[5m])
```

**Alert:**

```yaml
- alert: LowCacheHitRate
  expr: |
    (
      rate(policy_cache_hits_total{result="hit"}[5m]) /
      rate(policy_cache_hits_total[5m])
    ) < 0.8
  for: 10m
  annotations:
    summary: "Cache hit rate baixo"
    description: "Hit rate está em {{ $value }}%"
```

---

#### `casbin_policies_count` (Gauge)

Número atual de políticas carregadas no Casbin.

**Exemplos:**

```promql
# Número de políticas
casbin_policies_count

# Crescimento de políticas por hora
increase(casbin_policies_count[1h])
```

---

#### `casbin_enforcement_duration_seconds` (Histogram)

**NOVA MÉTRICA** - Tempo de verificação de permissões.

**Labels:**
- `result`: Resultado (allowed, denied)

**Buckets:** 0.1ms, 0.5ms, 1ms, 5ms, 10ms, 50ms, 100ms

**Exemplos:**

```promql
# P95 de tempo de enforcement
histogram_quantile(0.95, rate(casbin_enforcement_duration_seconds_bucket[5m]))

# Tempo médio por resultado
rate(casbin_enforcement_duration_seconds_sum[5m]) /
rate(casbin_enforcement_duration_seconds_count[5m]) by (result)
```

---

### 5. MFA (Multi-Factor Authentication)

#### `mfa_operations_total` (Counter)

**NOVA MÉTRICA** - Total de operações MFA.

**Labels:**
- `operation`: Tipo de operação (enable, disable, verify)
- `result`: Resultado (success, failure)

**Exemplos:**

```promql
# Taxa de habilitação de MFA
rate(mfa_operations_total{operation="enable",result="success"}[1h])

# Taxa de falhas na verificação MFA
rate(mfa_operations_total{operation="verify",result="failure"}[5m])

# Razão de sucesso na verificação
rate(mfa_operations_total{operation="verify",result="success"}[5m]) /
rate(mfa_operations_total{operation="verify"}[5m])
```

---

### 6. Erros

#### `http_errors_total` (Counter)

Total de erros HTTP por tipo.

**Labels:**
- `error_type`: Tipo de erro (validation, unauthorized, forbidden, internal)

**Exemplos:**

```promql
# Taxa de erros de validação
rate(http_errors_total{error_type="validation"}[5m])

# Taxa de erros 401 (não autorizados)
rate(http_errors_total{error_type="unauthorized"}[5m])

# Taxa de erros internos
rate(http_errors_total{error_type="internal"}[5m])
```

---

## 📊 Dashboards Grafana

### Dashboard de Overview

```json
{
  "panels": [
    {
      "title": "Request Rate",
      "targets": [
        {
          "expr": "sum(rate(http_requests_total[5m]))"
        }
      ]
    },
    {
      "title": "P95 Latency",
      "targets": [
        {
          "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))"
        }
      ]
    },
    {
      "title": "Error Rate",
      "targets": [
        {
          "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m]))"
        }
      ]
    },
    {
      "title": "Login Success Rate",
      "targets": [
        {
          "expr": "rate(login_attempts_total{result=\"success\"}[5m]) / rate(login_attempts_total[5m])"
        }
      ]
    }
  ]
}
```

### Dashboard de Segurança

```json
{
  "panels": [
    {
      "title": "Login Failures",
      "targets": [
        {
          "expr": "rate(login_attempts_total{result=\"failure\"}[5m])"
        }
      ]
    },
    {
      "title": "Token Theft Detections",
      "targets": [
        {
          "expr": "increase(token_theft_detected_total[1h])"
        }
      ]
    },
    {
      "title": "MFA Verification Failures",
      "targets": [
        {
          "expr": "rate(mfa_operations_total{operation=\"verify\",result=\"failure\"}[5m])"
        }
      ]
    }
  ]
}
```

## 🚨 Alertas Recomendados

```yaml
groups:
  - name: waterswamp_alerts
    rules:
      # Performance
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Alta latência detectada"

      # Segurança
      - alert: BruteForceAttack
        expr: rate(login_attempts_total{result="failure"}[1m]) > 50
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Possível ataque de força bruta"

      - alert: TokenTheftDetected
        expr: increase(token_theft_detected_total[5m]) > 0
        labels:
          severity: critical
        annotations:
          summary: "Possível roubo de token detectado"

      # Disponibilidade
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Alta taxa de erros 5xx"

      # Cache
      - alert: LowCacheHitRate
        expr: (rate(policy_cache_hits_total{result="hit"}[5m]) / rate(policy_cache_hits_total[5m])) < 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Cache hit rate baixo no Casbin"
```

## 🔍 Troubleshooting com Métricas

### Performance lenta

1. Verificar P95 de latência:
   ```promql
   histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) by (path)
   ```

2. Identificar endpoints lentos:
   ```promql
   topk(5, histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) by (path))
   ```

3. Verificar tempo de password hashing:
   ```promql
   histogram_quantile(0.95, rate(password_hash_duration_seconds_bucket{operation="hash"}[5m]))
   ```

### Segurança

1. Detectar ataques:
   ```promql
   rate(login_attempts_total{result="failure"}[1m]) > 10
   ```

2. Verificar token theft:
   ```promql
   increase(token_theft_detected_total[1h])
   ```

3. Monitorar MFA:
   ```promql
   rate(mfa_operations_total{operation="verify",result="failure"}[5m])
   ```

### Cache

1. Cache hit rate:
   ```promql
   100 * (rate(policy_cache_hits_total{result="hit"}[5m]) / rate(policy_cache_hits_total[5m]))
   ```

2. Identificar se cache está sendo usado:
   ```promql
   rate(policy_cache_hits_total[5m])
   ```

## 📚 Recursos Adicionais

- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [Grafana Dashboards](https://grafana.com/grafana/dashboards/)
- [Alerting Rules Guide](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
