# 🚀 Guia de Deploy - Waterswamp API

Este guia cobre todos os cenários de deploy da Waterswamp API, desde desenvolvimento local até produção em Kubernetes.

---

## 📋 Índice

1. [Desenvolvimento Local](#desenvolvimento-local)
2. [Deploy com Docker Compose](#deploy-com-docker-compose)
3. [Deploy em Kubernetes](#deploy-em-kubernetes)
4. [Deploy em Cloud Providers](#deploy-em-cloud-providers)
5. [CI/CD com GitHub Actions](#cicd-com-github-actions)
6. [Monitoramento e Logs](#monitoramento-e-logs)
7. [Troubleshooting](#troubleshooting)

---

## 🏠 Desenvolvimento Local

### Opção 1: Cargo Run (Mais Rápido para Desenvolvimento)

```bash
# 1. Subir apenas os bancos de dados
docker-compose up -d db_auth db_logs

# 2. Executar migrações
cargo make migrate-auth
cargo make migrate-logs

# 3. Rodar a aplicação
cargo run

# Ou com hot-reload:
cargo watch -x run
```

### Opção 2: Docker Compose (Ambiente Completo)

```bash
# Subir tudo (bancos + API + Adminer)
docker-compose up -d

# Ver logs
docker-compose logs -f api

# Acessar:
# - API: http://localhost:3000
# - Adminer: http://localhost:8080
```

---

## 🐳 Deploy com Docker Compose

### Passo 1: Preparar o Ambiente

```bash
# 1. Clonar o repositório
git clone https://github.com/seu-usuario/waterswamp.git
cd waterswamp

# 2. Criar .env
cp .env.example .env
nano .env  # Editar com valores de produção
```

### Passo 2: Build da Imagem

```bash
# Build local
docker build -t waterswamp-api:latest .

# Ou com tag específica
docker build -t waterswamp-api:1.0.0 .
```

### Passo 3: Deploy

```bash
# Subir os serviços
docker-compose up -d

# Verificar status
docker-compose ps

# Ver logs
docker-compose logs -f api
```

### Passo 4: Executar Migrações

```bash
# Opção A: Executar dentro do container
docker-compose exec api /bin/bash
# Dentro do container:
export DATABASE_URL=$WS_AUTH_DATABASE_URL
sqlx migrate run --source migrations_auth

# Opção B: Da máquina host (se tiver sqlx-cli instalado)
export DATABASE_URL=$(grep WS_AUTH_DATABASE_URL .env | cut -d '=' -f2)
sqlx migrate run --source migrations_auth
```

---

## ☸️ Deploy em Kubernetes

### Pré-requisitos

- Cluster Kubernetes rodando (minikube, k3s, GKE, EKS, AKS)
- `kubectl` configurado
- Imagem Docker publicada em um registry (Docker Hub, GHCR, ECR)

### Passo 1: Preparar Secrets

```bash
# Gerar JWT Secret
JWT_SECRET=$(openssl rand -hex 64)

# Codificar em base64
echo -n "$JWT_SECRET" | base64

# Editar k8s-deployment.yaml e substituir secrets
# Ou criar secrets via kubectl:
kubectl create secret generic waterswamp-secrets \
  --from-literal=WS_JWT_SECRET="$JWT_SECRET" \
  --from-literal=WS_AUTH_DATABASE_URL="postgres://..." \
  --from-literal=WS_LOGS_DATABASE_URL="postgres://..." \
  -n waterswamp
```

### Passo 2: Deploy

```bash
# Aplicar todos os recursos
kubectl apply -f k8s-deployment.yaml

# Verificar status
kubectl get all -n waterswamp
kubectl get pods -n waterswamp -w
```

### Passo 3: Expor o Serviço

**Opção A: Port-forward (Teste Local)**

```bash
kubectl port-forward -n waterswamp svc/waterswamp-api-service 3000:80
# Acessar: http://localhost:3000
```

**Opção B: Ingress (Produção)**

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: waterswamp-ingress
  namespace: waterswamp
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.seudominio.com
    secretName: waterswamp-tls
  rules:
  - host: api.seudominio.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: waterswamp-api-service
            port:
              number: 80
```

```bash
kubectl apply -f ingress.yaml
```

### Passo 4: Executar Migrações

```bash
# Criar Job de migração (uma vez)
kubectl create job migrate-auth \
  --from=cronjob/database-migrations \
  -n waterswamp

# Ou executar manualmente em um pod
kubectl exec -it -n waterswamp deployment/waterswamp-api -- /bin/sh
# Dentro do pod:
# (Execute as migrações manualmente)
```

---

## ☁️ Deploy em Cloud Providers

### AWS (ECS + RDS)

```bash
# 1. Criar RDS PostgreSQL
aws rds create-db-instance \
  --db-instance-identifier waterswamp-auth \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --master-username postgres \
  --master-user-password <senha-forte> \
  --allocated-storage 20

# 2. Push da imagem para ECR
aws ecr create-repository --repository-name waterswamp-api
docker tag waterswamp-api:latest <account-id>.dkr.ecr.<region>.amazonaws.com/waterswamp-api:latest
docker push <account-id>.dkr.ecr.<region>.amazonaws.com/waterswamp-api:latest

# 3. Criar Task Definition e Service no ECS
# (Use o Console AWS ou Terraform)
```

### Google Cloud (Cloud Run + Cloud SQL)

```bash
# 1. Criar Cloud SQL PostgreSQL
gcloud sql instances create waterswamp-auth \
  --database-version=POSTGRES_16 \
  --tier=db-f1-micro \
  --region=us-central1

# 2. Push para Artifact Registry
gcloud builds submit --tag gcr.io/<project-id>/waterswamp-api

# 3. Deploy no Cloud Run
gcloud run deploy waterswamp-api \
  --image gcr.io/<project-id>/waterswamp-api \
  --platform managed \
  --region us-central1 \
  --add-cloudsql-instances <project-id>:us-central1:waterswamp-auth \
  --set-env-vars WS_AUTH_DATABASE_URL="postgres://..." \
  --allow-unauthenticated
```

### Azure (Container Apps + PostgreSQL)

```bash
# 1. Criar PostgreSQL Flexible Server
az postgres flexible-server create \
  --resource-group waterswamp-rg \
  --name waterswamp-postgres \
  --location eastus \
  --admin-user postgres \
  --admin-password <senha-forte>

# 2. Push para Azure Container Registry
az acr build --registry waterswampacr --image waterswamp-api:latest .

# 3. Deploy no Container Apps
az containerapp create \
  --name waterswamp-api \
  --resource-group waterswamp-rg \
  --environment waterswamp-env \
  --image waterswampacr.azurecr.io/waterswamp-api:latest \
  --target-port 3000 \
  --env-vars WS_AUTH_DATABASE_URL="postgres://..." \
  --ingress external
```

---

## 🔄 CI/CD com GitHub Actions

### Configuração

1. **Copiar o workflow**:
   ```bash
   mkdir -p .github/workflows
   cp github-actions-ci.yml .github/workflows/ci.yml
   ```

2. **Configurar Secrets** no GitHub:
   - Settings → Secrets and variables → Actions → New repository secret
   
   **Secrets necessários**:
   - `KUBECONFIG_STAGING` (opcional)
   - `KUBECONFIG_PRODUCTION` (opcional)
   - `CODECOV_TOKEN` (opcional)

3. **Push para GitHub**:
   ```bash
   git add .
   git commit -m "Add CI/CD pipeline"
   git push origin main
   ```

### Fluxo Automático

- **Push em `develop`** → Testes + Build + Deploy em Staging
- **Push em `main`** → Testes + Build
- **Create Release** → Testes + Build + Deploy em Produção

---

## 📊 Monitoramento e Logs

### Ver Logs (Docker Compose)

```bash
# Logs em tempo real
docker-compose logs -f api

# Últimas 100 linhas
docker-compose logs --tail=100 api

# Logs em formato JSON (produção)
docker-compose logs --no-log-prefix api | jq .
```

### Ver Logs (Kubernetes)

```bash
# Logs de todos os pods
kubectl logs -n waterswamp -l app=waterswamp-api --tail=100 -f

# Logs de um pod específico
kubectl logs -n waterswamp <pod-name> -f

# Logs em formato JSON
kubectl logs -n waterswamp -l app=waterswamp-api --tail=100 | jq .
```

### Métricas de Health

```bash
# Health check
curl http://localhost:3000/health

# Liveness
curl http://localhost:3000/health/live

# Readiness
curl http://localhost:3000/health/ready
```

### Integração com Ferramentas

**Datadog**:
```yaml
# No Deployment, adicionar:
env:
- name: DD_AGENT_HOST
  valueFrom:
    fieldRef:
      fieldPath: status.hostIP
- name: DD_SERVICE
  value: waterswamp-api
- name: DD_ENV
  value: production
```

**Prometheus**:
```yaml
# ServiceMonitor para Prometheus Operator
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: waterswamp-api
  namespace: waterswamp
spec:
  selector:
    matchLabels:
      app: waterswamp-api
  endpoints:
  - port: http
    path: /metrics
```

---

## 🐛 Troubleshooting

### Problema: "Connection refused" ao banco

```bash
# Verificar se o banco está rodando
docker-compose ps db_auth

# Testar conexão
psql "postgres://postgres:senha@localhost:5432/auth_db"

# Ver logs do banco
docker-compose logs db_auth
```

### Problema: "JWT invalid"

```bash
# Verificar secret
echo $WS_JWT_SECRET | wc -c  # Deve ter 128+ caracteres

# Regenerar secret
openssl rand -hex 64
```

### Problema: Pod não inicia no Kubernetes

```bash
# Ver eventos
kubectl describe pod -n waterswamp <pod-name>

# Ver logs
kubectl logs -n waterswamp <pod-name>

# Ver configuração
kubectl get configmap -n waterswamp waterswamp-config -o yaml
kubectl get secret -n waterswamp waterswamp-secrets -o yaml
```

### Problema: Imagem Docker muito grande

```bash
# Verificar tamanho
docker images waterswamp-api

# Verificar layers
docker history waterswamp-api:latest

# Rebuild com cache limpo
docker build --no-cache -t waterswamp-api:latest .
```

---

## 📚 Recursos Adicionais

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust in Production](https://doc.rust-lang.org/book/)

---

**Última atualização**: 2024-11-07  
**Versão**: 1.0.0
