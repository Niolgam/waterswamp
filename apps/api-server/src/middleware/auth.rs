pub use crate::extractors::current_user::CurrentUser;
use crate::{infra::errors::AppError, state::AppState};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method},
    middleware::Next,
    response::Response,
};
use casbin::CoreApi;
use core_services::session::{config as session_config, encryption, hash_token, validate_csrf};
use domain::models::TokenType;
use domain::ports::SessionRepositoryPort;
use persistence::repositories::session_repository::SessionRepository;
use tower_cookies::Cookies;

pub async fn mw_authenticate(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(req.headers())
        .ok_or_else(|| AppError::Unauthorized("Token not found".to_string()))?;

    // Verify token using JwtService (supports key rotation)
    let claims = state
        .jwt_service
        .verify_token(&token, TokenType::Access)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    let user_id = claims.sub;
    // Username já está disponível no JWT - não precisa query no banco! (N+1 fix)
    let username = claims.username.clone();

    let current_user = CurrentUser {
        id: user_id,
        username,
    };

    // Injeta os claims e o usuário atual na requisição
    req.extensions_mut().insert(claims);
    req.extensions_mut().insert(current_user);

    Ok(next.run(req).await)
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Session-based authentication middleware
///
/// Authenticates using HttpOnly session cookies. Falls back to JWT if no session cookie.
/// Also validates CSRF token for mutating requests (POST, PUT, DELETE, PATCH).
pub async fn mw_session_authenticate(
    State(state): State<AppState>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Try session cookie first
    if let Some(session_cookie) = cookies.get(session_config::SESSION_COOKIE_NAME) {
        let session_token = session_cookie.value().to_string();
        let session_token_hash = hash_token(&session_token);

        let session_repo = SessionRepository::new(state.db_pool_auth.clone());

        // Find valid session
        if let Some(session) = session_repo.find_by_token_hash(&session_token_hash).await? {
            // Validate CSRF for mutating requests
            if is_mutating_request(req.method()) {
                validate_csrf_header(req.headers(), &session.csrf_token_hash)?;
            }

            // Decrypt access token
            let encryption_key = get_encryption_key(&state).await?;
            let access_token = encryption::decrypt(&session.access_token_encrypted, &encryption_key)
                .map_err(|_| AppError::Unauthorized("Session decryption failed".to_string()))?;

            // Verify the decrypted JWT is still valid
            let claims = state
                .jwt_service
                .verify_token(&access_token, TokenType::Access)
                .map_err(|_| AppError::Unauthorized("Session token expired".to_string()))?;

            let current_user = CurrentUser {
                id: claims.sub,
                username: claims.username.clone(),
            };

            // Touch session (extend expiry via sliding window)
            let _ = session_repo
                .touch_session(session.id, Some(session_config::SLIDING_WINDOW_MINUTES))
                .await;

            req.extensions_mut().insert(claims);
            req.extensions_mut().insert(current_user);
            req.extensions_mut().insert(session.id); // Store session ID for later use

            return Ok(next.run(req).await);
        }
    }

    // Fall back to JWT Bearer token
    let token = extract_token(req.headers())
        .ok_or_else(|| AppError::Unauthorized("No valid session or token".to_string()))?;

    let claims = state
        .jwt_service
        .verify_token(&token, TokenType::Access)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    let current_user = CurrentUser {
        id: claims.sub,
        username: claims.username.clone(),
    };

    req.extensions_mut().insert(claims);
    req.extensions_mut().insert(current_user);

    Ok(next.run(req).await)
}

/// Checks if the request method is a mutating operation
fn is_mutating_request(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    )
}

/// Validates the CSRF token from X-CSRF-Token header
fn validate_csrf_header(headers: &HeaderMap, expected_hash: &str) -> Result<(), AppError> {
    let csrf_token = headers
        .get("X-CSRF-Token")
        .or_else(|| headers.get("X-Csrf-Token"))
        .or_else(|| headers.get("x-csrf-token"))
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Forbidden("CSRF token required".to_string()))?;

    validate_csrf(csrf_token, expected_hash)
        .map_err(|_| AppError::Forbidden("Invalid CSRF token".to_string()))
}

/// Gets the encryption key from the database
async fn get_encryption_key(state: &AppState) -> Result<Vec<u8>, AppError> {
    let session_repo = SessionRepository::new(state.db_pool_auth.clone());

    session_repo
        .get_active_encryption_key()
        .await?
        .map(|k| k.key_material)
        .ok_or_else(|| AppError::Internal("No encryption key available".to_string()))
}

/// Middleware de Autorização usando Casbin
pub async fn mw_authorize(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<CurrentUser>()
        .ok_or_else(|| anyhow::anyhow!("CurrentUser not found in request extensions"))?;

    let subject = user.id.to_string();
    let object = req.uri().path().to_string();
    let action = req.method().to_string();

    let cache_key = format!("{}:{}:{}", subject, object, action);

    // Tentar obter do cache (moka)
    let allowed = if let Some(decision) = state.policy_cache.get(&cache_key).await {
        tracing::debug!("Cache hit para: {}", cache_key);
        decision
    } else {
        tracing::debug!("Cache miss para: {}", cache_key);

        // Consultar Casbin
        let decision = {
            let enforcer_guard = state.enforcer.read().await;
            enforcer_guard
                .enforce(vec![subject.clone(), object.clone(), action.clone()])
                .map_err(|e| anyhow::anyhow!("Erro no Casbin Enforcer: {}", e))?
        };

        // Inserir no cache (moka insere automaticamente com TTL)
        state.policy_cache.insert(cache_key.clone(), decision).await;

        decision
    };

    if !allowed {
        tracing::warn!(
            "Acesso negado: sub={}, obj={}, act={}",
            subject,
            object,
            action
        );
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    tracing::debug!(
        "Acesso permitido: sub={}, obj={}, act={}",
        subject,
        object,
        action
    );
    Ok(next.run(req).await)
}
