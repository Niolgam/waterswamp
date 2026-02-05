# Guia de Autenticação - Frontend

Este documento descreve como integrar o frontend Angular com o sistema de autenticação do Waterswamp API.

## Visão Geral

O sistema oferece **duas formas de autenticação**:

| Método | Uso Recomendado | Armazenamento |
|--------|-----------------|---------------|
| **JWT (Bearer Token)** | APIs mobile, integrações de terceiros | localStorage/memória |
| **Session (Cookie)** | Aplicações web SPA | HttpOnly Cookie (automático) |

**Recomendação para Angular SPA:** Use autenticação por **Session (Cookie)** por ser mais segura contra XSS.

---

## Autenticação por Session (Cookie) - RECOMENDADO

### 1. Login

**Endpoint:** `POST /api/v1/auth/session/login`

**Request:**
```json
{
  "username": "usuario@email.com",
  "password": "senha123",
  "remember_me": false
}
```

| Campo | Tipo | Obrigatório | Descrição |
|-------|------|-------------|-----------|
| `username` | string | Sim | Username ou email |
| `password` | string | Sim | Senha do usuário |
| `remember_me` | boolean | Não | `true` = sessão de 30 dias, `false` = 24 horas |

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Login successful",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "joao.silva",
  "csrf_token": "a1b2c3d4e5f6...",
  "expires_at": 1706918400
}
```

**Cookies Definidos pelo Servidor:**

| Cookie | Valor | Flags |
|--------|-------|-------|
| `__Host-session` | Token de sessão | `HttpOnly`, `Secure`, `SameSite=Strict`, `Path=/` |
| `csrf_token` | Token CSRF | `Secure`, `SameSite=Strict`, `Path=/` (legível pelo JS) |

**Importante:**
- O cookie `__Host-session` é **HttpOnly** - o JavaScript NÃO consegue ler
- O cookie `csrf_token` é legível pelo JavaScript para enviar no header
- O `csrf_token` também é retornado no body para conveniência

**Erros Possíveis:**

| Status | Descrição |
|--------|-----------|
| 400 | Payload inválido (validação) |
| 401 | Credenciais inválidas |
| 400 | MFA habilitado (use endpoint JWT) |

---

### 2. Requisições Autenticadas

Após o login, o browser envia automaticamente o cookie `__Host-session` em todas as requisições para o mesmo domínio.

**Para requisições que MODIFICAM dados (POST, PUT, DELETE, PATCH):**

O header `X-CSRF-Token` é **OBRIGATÓRIO**:

```http
POST /api/v1/resource HTTP/1.1
Content-Type: application/json
X-CSRF-Token: a1b2c3d4e5f6...

{"data": "..."}
```

**Para requisições GET:**

Não precisa do header CSRF:

```http
GET /api/v1/resource HTTP/1.1
```

**Configuração Angular HttpClient:**

O Angular deve ser configurado com:
- `withCredentials: true` - para enviar cookies cross-origin
- Interceptor para adicionar `X-CSRF-Token` em requisições mutantes

---

### 3. Verificar Sessão Atual

**Endpoint:** `GET /api/v1/auth/session/me`

**Request:** Apenas o cookie (enviado automaticamente)

**Response (200 OK):**
```json
{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "joao.silva",
  "created_at": 1706832000,
  "expires_at": 1706918400,
  "last_activity_at": 1706835600
}
```

**Erros:**

| Status | Descrição |
|--------|-----------|
| 401 | Sessão inválida ou expirada |

**Uso:** Chamar ao iniciar a aplicação para verificar se o usuário está autenticado.

---

### 4. Listar Sessões Ativas

**Endpoint:** `GET /api/v1/auth/session/list`

**Response (200 OK):**
```json
{
  "sessions": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
      "ip_address": "192.168.1.100",
      "created_at": 1706832000,
      "last_activity_at": 1706835600,
      "is_current": true
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "user_agent": "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)...",
      "ip_address": "10.0.0.50",
      "created_at": 1706800000,
      "last_activity_at": 1706820000,
      "is_current": false
    }
  ]
}
```

**Uso:** Mostrar ao usuário onde ele está logado (gerenciamento de sessões).

---

### 5. Logout (Sessão Atual)

**Endpoint:** `POST /api/v1/auth/session/logout`

**Request:** Apenas o cookie (enviado automaticamente)

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Logout successful"
}
```

**Cookies Removidos:** `__Host-session`, `csrf_token` (Max-Age=0)

---

### 6. Logout de Todas as Sessões

**Endpoint:** `POST /api/v1/auth/session/logout-all`

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Logged out of 3 session(s)",
  "revoked_count": 3
}
```

**Uso:** Botão "Sair de todos os dispositivos" nas configurações de segurança.

---

### 7. Revogar Sessão Específica

**Endpoint:** `DELETE /api/v1/auth/session/{session_id}`

**Response:** `204 No Content`

**Erros:**

| Status | Descrição |
|--------|-----------|
| 401 | Não autenticado |
| 404 | Sessão não encontrada (ou não pertence ao usuário) |

**Uso:** Botão "Encerrar" ao lado de cada sessão na lista.

---

## Autenticação JWT (Token Bearer)

Use este método para:
- Aplicações mobile
- Integrações de API
- Quando MFA está habilitado

### 1. Login JWT

**Endpoint:** `POST /api/v1/auth/login`

**Request:**
```json
{
  "username": "usuario@email.com",
  "password": "senha123"
}
```

**Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "joao.silva",
    "email": "joao@email.com"
  }
}
```

**Se MFA estiver habilitado (200 OK com flag):**
```json
{
  "mfa_required": true,
  "mfa_token": "temp_token_for_mfa...",
  "mfa_methods": ["totp", "email"]
}
```

---

### 2. Requisições Autenticadas (JWT)

```http
GET /api/v1/resource HTTP/1.1
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

### 3. Refresh Token

**Endpoint:** `POST /api/v1/auth/refresh-token`

**Request:**
```json
{
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4..."
}
```

**Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "bmV3IHJlZnJlc2ggdG9rZW4...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

---

### 4. Logout JWT

**Endpoint:** `POST /api/v1/auth/logout`

**Request:**
```json
{
  "refresh_token": "dGhpcyBpcyBhIHJlZnJlc2ggdG9rZW4..."
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Logout successful"
}
```

---

## Recuperação de Senha

### 1. Solicitar Reset

**Endpoint:** `POST /api/v1/auth/forgot-password`

**Request:**
```json
{
  "email": "usuario@email.com"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "If the email exists, a reset link has been sent"
}
```

**Nota:** Sempre retorna sucesso para não revelar se o email existe.

---

### 2. Redefinir Senha

**Endpoint:** `POST /api/v1/auth/reset-password`

**Request:**
```json
{
  "token": "reset_token_from_email...",
  "new_password": "NovaSenha@123"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "message": "Password reset successful"
}
```

---

## Registro de Usuário

**Endpoint:** `POST /api/v1/auth/register`

**Request:**
```json
{
  "username": "joao.silva",
  "email": "joao@email.com",
  "password": "Senha@Forte123",
  "first_name": "João",
  "last_name": "Silva"
}
```

**Validações de Senha:**
- Mínimo 8 caracteres
- Pelo menos 1 letra maiúscula
- Pelo menos 1 letra minúscula
- Pelo menos 1 número
- Pelo menos 1 caractere especial

**Response (201 Created):**
```json
{
  "success": true,
  "message": "Registration successful. Please verify your email.",
  "user_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Erros:**

| Status | Descrição |
|--------|-----------|
| 400 | Validação falhou |
| 409 | Username ou email já existe |

---

## Tratamento de Erros

Todos os erros seguem o formato:

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid or expired session"
  }
}
```

### Códigos de Erro Comuns

| HTTP Status | Código | Ação no Frontend |
|-------------|--------|------------------|
| 401 | `UNAUTHORIZED` | Redirecionar para login |
| 403 | `FORBIDDEN` | Mostrar mensagem de permissão negada |
| 403 | `CSRF_INVALID` | Recarregar página (obter novo CSRF token) |
| 429 | `RATE_LIMITED` | Mostrar "Muitas tentativas, aguarde X segundos" |

---

## Fluxo Recomendado para Angular SPA

```
1. App Inicializa
   └── GET /session/me
       ├── 200 OK → Usuário autenticado, carregar app
       └── 401 → Mostrar tela de login

2. Login
   └── POST /session/login
       ├── 200 OK → Salvar csrf_token, redirecionar para home
       └── 401 → Mostrar erro "Credenciais inválidas"

3. Navegação
   └── Interceptor adiciona X-CSRF-Token em POST/PUT/DELETE/PATCH
       └── 401 em qualquer requisição → Redirecionar para login

4. Logout
   └── POST /session/logout
       └── Limpar estado local, redirecionar para login
```

---

## Configurações CORS

O backend aceita requisições de:
- `http://localhost:4200` (desenvolvimento)
- Domínios configurados em produção

**Headers permitidos (Access-Control-Allow-Headers):**

| Header | Uso |
|--------|-----|
| `Content-Type` | Tipo do conteúdo da requisição |
| `Authorization` | Token JWT (Bearer) |
| `Accept` | Tipos de resposta aceitos |
| `X-Content-Type-Options` | Prevenção de MIME sniffing |
| `X-CSRF-Token` | Token CSRF para session auth |
| `X-Requested-With` | Identificador de requisição AJAX |

**Métodos permitidos:**
- GET, POST, PUT, DELETE, PATCH, OPTIONS

---

## Expiração e Renovação de Sessão

| Configuração | Valor |
|--------------|-------|
| Sessão padrão | 24 horas |
| Sessão "lembrar-me" | 30 dias |
| Sliding window | 30 minutos de atividade estende a sessão |

A sessão é automaticamente estendida a cada requisição autenticada (sliding expiration).

---

## Segurança - Checklist Frontend

- [ ] Configurar `withCredentials: true` no HttpClient
- [ ] Interceptor para adicionar `X-CSRF-Token` em requisições mutantes
- [ ] Ler CSRF token do cookie `csrf_token` ou do response do login
- [ ] Tratar 401 globalmente (redirecionar para login)
- [ ] Não armazenar tokens JWT em localStorage (se usar session cookies)
- [ ] Limpar estado local no logout
