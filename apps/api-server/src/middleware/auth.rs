pub use crate::extractors::current_user::CurrentUser;
use crate::{infra::errors::AppError, state::AppState};
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use casbin::CoreApi;
use domain::models::TokenType;

pub async fn mw_authenticate(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(req.headers())
        .ok_or_else(|| AppError::Unauthorized("Token não encontrado".to_string()))?;

    // Verifica o token usando o JwtService (suporta rotação de chaves)
    let claims = state
        .jwt_service
        .verify_token(&token, TokenType::Access)
        .map_err(|_| AppError::Unauthorized("Token inválido ou expirado".to_string()))?;

    let user_id = claims.sub;

    // Fetch username from database
    let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db_pool_auth)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "Erro ao buscar username");
            AppError::Anyhow(anyhow::anyhow!("Erro interno"))
        })?
        .ok_or_else(|| AppError::Unauthorized("Usuário não encontrado".to_string()))?;

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

/// Middleware de Autorização usando Casbin
pub async fn mw_authorize(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<CurrentUser>()
        .ok_or_else(|| anyhow::anyhow!("CurrentUser não encontrado nas extensões"))?;

    let subject = user.id.to_string();
    let object = req.uri().path().to_string();
    let action = req.method().to_string();

    let cache_key = format!("{}:{}:{}", subject, object, action);

    let cached_decision = {
        let cache = state.policy_cache.read().await;
        cache.get(&cache_key).copied()
    };

    let allowed = match cached_decision {
        Some(decision) => {
            tracing::debug!("Cache hit para: {}", cache_key);
            decision
        }
        None => {
            tracing::debug!("Cache miss para: {}", cache_key);

            let decision = {
                let enforcer_guard = state.enforcer.read().await;
                enforcer_guard
                    .enforce(vec![subject.clone(), object.clone(), action.clone()])
                    .map_err(|e| anyhow::anyhow!("Erro no Casbin Enforcer: {}", e))?
            };

            {
                let mut cache = state.policy_cache.write().await;
                cache.insert(cache_key.clone(), decision);
            }

            decision
        }
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

    tracing::info!(
        "Acesso permitido: sub={}, obj={}, act={}",
        subject,
        object,
        action
    );
    Ok(next.run(req).await)
}
