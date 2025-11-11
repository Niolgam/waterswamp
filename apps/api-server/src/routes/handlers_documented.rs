// =============================================================================
// EXEMPLOS DE HANDLERS DOCUMENTADOS COM UTOIPA
// =============================================================================
// Este arquivo mostra como adicionar documentação OpenAPI aos handlers

use axum::{extract::State, Json};
use utoipa;

use crate::{
    error::AppError,
    state::AppState,
};
use domain::models::{LoginPayload, LoginResponse, RefreshTokenPayload, RefreshTokenResponse},



// -----------------------------------------------------------------------------
// EXEMPLO 1: Rota Pública Simples
// -----------------------------------------------------------------------------

/// Endpoint público de teste
///
/// Esta rota não requer autenticação e pode ser usada para verificar
/// se a API está respondendo.
#[utoipa::path(
    get,
    path = "/public",
    tag = "Public",
    responses(
        (status = 200, description = "Mensagem de sucesso", body = String,
         example = json!("Esta rota é pública."))
    ),
    summary = "Rota pública de teste",
)]
pub async fn handler_public() -> &'static str {
    "Esta rota é pública."
}

// -----------------------------------------------------------------------------
// EXEMPLO 2: Login com Autenticação
// -----------------------------------------------------------------------------

/// Autentica usuário e retorna tokens JWT
///
/// Este endpoint valida as credenciais do usuário e, se válidas,
/// retorna um access token (válido por 1 hora) e um refresh token
/// (válido por 30 dias).
///
/// # Exemplo de uso
///
/// ```json
/// {
///   "username": "alice",
///   "password": "password123"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/login",
    tag = "Auth",
    request_body(
        content = LoginPayload,
        description = "Credenciais de login do usuário",
        example = json!({
            "username": "alice",
            "password": "password123"
        })
    ),
    responses(
        (status = 200, description = "Login bem-sucedido", body = LoginResponse,
         example = json!({
             "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
             "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
             "token_type": "Bearer",
             "expires_in": 3600
         })),
        (status = 401, description = "Credenciais inválidas",
         example = json!({"error": "Usuário ou senha inválidos."})),
        (status = 400, description = "Payload inválido (validação falhou)",
         example = json!({"error": "Senha muito curta"})),
    ),
    summary = "Autenticar usuário",
    description = "Valida credenciais e retorna access token + refresh token"
)]
pub async fn handler_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    // Implementação real está em auth_handler.rs
    todo!()
}

// -----------------------------------------------------------------------------
// EXEMPLO 3: Renovação de Token
// -----------------------------------------------------------------------------

/// Renova access token usando refresh token
///
/// Quando o access token expira (após 1 hora), use este endpoint
/// para obter um novo access token sem fazer login novamente.
///
/// # Exemplo de uso
///
/// ```json
/// {
///   "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/refresh-token",
    tag = "Auth",
    request_body(
        content = RefreshTokenPayload,
        description = "Refresh token obtido no login",
        example = json!({
            "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
        })
    ),
    responses(
        (status = 200, description = "Token renovado com sucesso", body = RefreshTokenResponse,
         example = json!({
             "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
             "token_type": "Bearer",
             "expires_in": 3600
         })),
        (status = 401, description = "Refresh token inválido, expirado ou revogado",
         example = json!({"error": "Refresh token inválido"})),
    ),
    summary = "Renovar access token",
    description = "Obtém novo access token sem fazer login novamente"
)]
pub async fn handler_refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<Json<RefreshTokenResponse>, AppError> {
    // Implementação real está em auth_handler.rs
    todo!()
}

// -----------------------------------------------------------------------------
// EXEMPLO 4: Rota Protegida com JWT
// -----------------------------------------------------------------------------

/// Retorna perfil do usuário autenticado
///
/// Esta rota requer autenticação via JWT Bearer token.
/// O token deve ser enviado no header Authorization.
#[utoipa::path(
    get,
    path = "/users/profile",
    tag = "User",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Perfil do usuário", body = String,
         example = json!("Este é o seu perfil de usuário.")),
        (status = 401, description = "Token não fornecido ou inválido",
         example = json!({"error": "Token não encontrado"})),
        (status = 403, description = "Sem permissão para acessar este recurso",
         example = json!({"error": "Acesso negado"})),
    ),
    summary = "Obter perfil do usuário",
    description = "Retorna informações do perfil do usuário autenticado"
)]
pub async fn handler_user_profile() -> &'static str {
    "Este é o seu perfil de usuário."
}

// -----------------------------------------------------------------------------
// EXEMPLO 5: Rota Admin com Permissões
// -----------------------------------------------------------------------------

/// Dashboard administrativo
///
/// Esta rota requer:
/// - Autenticação (JWT token)
/// - Permissão de admin (verificada pelo Casbin)
#[utoipa::path(
    get,
    path = "/admin/dashboard",
    tag = "Admin",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Dashboard do admin", body = String,
         example = json!("Você está no dashboard de admin.")),
        (status = 401, description = "Token não fornecido ou inválido"),
        (status = 403, description = "Sem permissão de admin"),
    ),
    summary = "Acessar dashboard administrativo",
    description = "Painel de controle para administradores (requer permissão admin)"
)]
pub async fn handler_admin_dashboard() -> &'static str {
    "Você está no dashboard de admin."
}

// -----------------------------------------------------------------------------
// EXEMPLO 6: Health Check com Schema Customizado
// -----------------------------------------------------------------------------

use crate::routes::health_handler::{CasbinHealth, DatabaseHealth, HealthResponse};

/// Verifica saúde da API
///
/// Retorna status detalhado de todos os componentes:
/// - Bancos de dados (auth_db e logs_db)
/// - Casbin (autorização)
/// - Versão da aplicação
/// - Uptime do servidor
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "API saudável", body = HealthResponse,
         example = json!({
             "status": "healthy",
             "database": {
                 "auth_db": true,
                 "logs_db": true
             },
             "casbin": {
                 "operational": true,
                 "policy_count": 4
             },
             "version": "1.0.0",
             "uptime_seconds": 3600
         })),
        (status = 503, description = "API com problemas (banco ou Casbin indisponível)"),
    ),
    summary = "Health check detalhado",
    description = "Verifica status de todos os componentes da API"
)]
pub async fn handler_health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, axum::http::StatusCode> {
    // Implementação real está em health_handler.rs
    todo!()
}

// =============================================================================
// NOTAS DE USO
// =============================================================================
//
// Para adicionar documentação a um handler:
//
// 1. Adicione a macro #[utoipa::path(...)] antes da função
// 2. Configure:
//    - Método HTTP (get, post, put, delete, etc)
//    - Path da rota
//    - Tag (para agrupar no Swagger UI)
//    - request_body (se aplicável)
//    - responses (todos os códigos de status possíveis)
//    - security (se requer autenticação)
//
// 3. Adicione o handler em ApiDoc (openapi.rs):
//    paths(
//        seu_handler::handler_name,
//    )
//
// 4. Se usar schemas customizados, adicione em components:
//    components(
//        schemas(
//            SeuSchema,
//        )
//    )
//
// =============================================================================
