use crate::{error::AppError, models::PolicyRequest, state::AppState};
use axum::{extract::State, http::StatusCode, Json};
use casbin::MgmtApi;
use uuid::Uuid;
use validator::Validate;

/// POST /api/admin/policies
/// Adiciona uma nova regra de política (p)
pub async fn add_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;

    let subject_uuid = match resolve_subject(&state, &payload.subject).await? {
        Some(uuid) => uuid,
        None => {
            return Err(AppError::NotFound(format!(
                "Usuário '{}' não encontrado",
                payload.subject
            )));
        }
    };

    // Verifica se a política já existe antes de tentar adicionar
    let already_exists = {
        let e = state.enforcer.read().await;
        e.has_policy(vec![
            subject_uuid.clone(),
            payload.object.clone(),
            payload.action.clone(),
        ])
    };

    if already_exists {
        // Política já existe - operação idempotente, retorna 200 OK
        tracing::debug!(
            "Política já existe: sub={}, obj={}, act={}",
            subject_uuid,
            payload.object,
            payload.action
        );
        return Ok(StatusCode::OK);
    }

    // Tenta adicionar a nova política
    let inserted = {
        let mut e = state.enforcer.write().await;
        e.add_policy(vec![
            subject_uuid.clone(),
            payload.object.clone(),
            payload.action.clone(),
        ])
        .await
        .map_err(|e| anyhow::anyhow!("Erro no Casbin: {}", e))?
    };

    if inserted {
        invalidate_cache(&state, &subject_uuid, &payload.object, &payload.action).await;
        tracing::info!(
            "Nova política adicionada: sub={}, obj={}, act={}",
            subject_uuid,
            payload.object,
            payload.action
        );
        Ok(StatusCode::CREATED) // 201
    } else {
        // Race condition: política foi adicionada entre o check e o add
        // Trata como sucesso idempotente
        Ok(StatusCode::OK) // 200
    }
}

/// DELETE /api/admin/policies
/// Remove uma regra de política existente
pub async fn remove_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;

    let subject_uuid = match resolve_subject(&state, &payload.subject).await? {
        Some(uuid) => uuid,
        None => {
            // Se o usuário não existe, a política também não pode existir
            // Retorna 404 diretamente (não é erro, é resultado esperado)
            return Ok(StatusCode::NOT_FOUND); // 404
        }
    };

    // Tenta remover a política
    let removed = {
        let mut e = state.enforcer.write().await;
        e.remove_policy(vec![
            subject_uuid.clone(),
            payload.object.clone(),
            payload.action.clone(),
        ])
        .await
        .map_err(|e| anyhow::anyhow!("Erro no Casbin: {}", e))?
    };

    if removed {
        // Política foi removida com sucesso
        invalidate_cache(&state, &subject_uuid, &payload.object, &payload.action).await;
        Ok(StatusCode::NO_CONTENT) // 204
    } else {
        // Política não existia para remover
        Ok(StatusCode::NOT_FOUND) // 404
    }
}

/// Helper: Resolve subject (aceita UUID ou username)
///
/// RETORNA Option<String> em vez de Result<String, AppError>
/// - Some(uuid) = usuário encontrado
/// - None = usuário não existe (NÃO é erro de autenticação!)
///
async fn resolve_subject(state: &AppState, subject: &str) -> Result<Option<String>, AppError> {
    // Tenta converter para UUID
    if let Ok(uuid) = Uuid::parse_str(subject) {
        // É um UUID válido - verifica se existe no banco
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
            .bind(uuid)
            .fetch_one(&state.db_pool_auth)
            .await?; // ⚠️ Erro de DB é propagado (isto sim é erro real)

        if exists {
            return Ok(Some(uuid.to_string()));
        } else {
            // UUID válido mas usuário não existe
            return Ok(None);
        }
    }

    // Não é UUID - tenta buscar por username
    let user: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(subject)
        .fetch_optional(&state.db_pool_auth)
        .await?; // ⚠️ Erro de DB é propagado

    // Retorna o UUID se encontrou, ou None se não encontrou
    Ok(user.map(|(id,)| id.to_string()))
}

/// Invalida o cache para uma política específica
async fn invalidate_cache(state: &AppState, subject: &str, object: &str, action: &str) {
    let cache_key = format!("{}:{}:{}", subject, object, action);
    let mut cache = state.policy_cache.write().await;
    if cache.remove(&cache_key).is_some() {
        tracing::debug!("Cache invalidado para: {}", cache_key);
    }
}
