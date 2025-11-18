# 🛡️ Rate Limiting por Endpoint

## Visão Geral

O Waterswamp implementa rate limiting baseado em IP com diferentes limites por tipo de endpoint.

## Configuração por Endpoint

| Endpoint | Limite | Janela | Uso |
|----------|--------|--------|-----|
| `/login`, `/register` | 5 req | 10s | Proteção contra brute-force |
| `/forgot-password` | 3 req | 60s | Previne spam de emails |
| `/auth/mfa/verify` | 5 req | 30s | Previne brute-force de códigos |
| `/resend-verification` | 5 req | 60s | Previne spam de verificação |
| `/api/admin/*` | 10 req | 2s | Proteção admin (300 req/min) |
| API autenticada | 50 req | 200ms | Uso geral (~15k req/min) |
| `/health`, `/metrics` | 100 req | 1s | Monitoramento frequente |

## Extração de IP

### Com Proxy Reverso (Produção)
```bash
# .env
TRUST_PROXY=true
```

**Ordem de prioridade:**
1. `CF-Connecting-IP` (Cloudflare)
2. `X-Real-IP` (Nginx)
3. `X-Forwarded-For` (primeiro IP)
4. IP da conexão TCP

### Sem Proxy Reverso (Desenvolvimento Local)
```bash
# .env
TRUST_PROXY=false
```

Usa apenas o IP da conexão TCP (mais seguro se não há proxy confiável).

## Configuração Nginx
```nginx
location / {
    proxy_pass http://waterswamp:3000;
    
    # Headers para rate limiting
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header Host $host;
}
```

## Configuração Cloudflare

Cloudflare adiciona automaticamente `CF-Connecting-IP` com o IP real do cliente.

## Headers de Resposta

Quando rate limit é atingido:
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 60
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0

{
  "error": "Too Many Requests",
  "message": "Você excedeu o limite de requisições. Tente novamente mais tarde.",
  "retry_after_seconds": 60,
  "code": "RATE_LIMIT_EXCEEDED"
}
```

## Métricas Prometheus
```promql
# Total de rate limits atingidos
rate(rate_limit_hits_total[5m])

# Rate limits por tipo de endpoint
rate(rate_limit_hits_total{endpoint_type="login"}[5m])

# Top 10 IPs bloqueados
topk(10, sum by (ip_address) (rate_limit_hits_total))
```

## Desabilitar em Testes
```bash
# .env ou comando
export DISABLE_RATE_LIMIT=true
cargo test
```

## Ajuste de Limites

Edite `apps/api-server/src/rate_limit.rs`:
```rust
// Exemplo: Login mais permissivo
pub fn login_rate_limiter() -> RateLimitLayer {
    let config = GovernorConfigBuilder::default()
        .key_extractor(RobustIpExtractor)
        .period(Duration::from_secs(10))
        .burst_size(10)  // 5 → 10 requisições
        .finish()
        .unwrap();
    
    GovernorLayer::new(config)
}
```

## Alertas Recomendados
```yaml
# prometheus/alerts.yml
- alert: HighRateLimitHits
  expr: rate(rate_limit_hits_total[5m]) > 10
  for: 5m
  annotations:
    summary: "Muitos rate limits atingidos ({{ $value }} hits/s)"

- alert: SuspiciousIPActivity
  expr: sum by (ip_address) (rate(rate_limit_hits_total{endpoint_type="login"}[5m])) > 5
  for: 5m
  annotations:
    summary: "IP suspeito: {{ $labels.ip_address }} ({{ $value }} hits/s)"
```

## Troubleshooting

### Rate limit não funciona

1. Verifique `DISABLE_RATE_LIMIT=false`
2. Confirme que IP está sendo extraído: `curl -v http://localhost:3000/login`
3. Verifique logs: `docker logs waterswamp | grep rate_limit`

### IP sempre 127.0.0.1

1. Configure `TRUST_PROXY=true`
2. Verifique headers do proxy reverso:
```bash
   curl -H "X-Real-IP: 1.2.3.4" http://localhost:3000/login
```

### Rate limit muito agressivo

Ajuste os limites em `rate_limit.rs` conforme necessidade do seu tráfego.
