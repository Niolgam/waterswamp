use crate::{error::AppError, state::AppState, web_models::CurrentUser};
use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use casbin::MgmtApi;
use core_services::security::{hash_password, validate_password_strength};
use domain::models::{
    CreateUserPayload, ListUsersQuery, PaginatedUsers, PolicyRequest, UpdateUserPayload, UserDto,
};
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};
use uuid::Uuid;
use validator::Validate;

/// POST /api/admin/policies
pub async fn add_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;

    let user_repo = UserRepository::new(&state.db_pool_auth);
    let subject_uuid = match resolve_subject(&user_repo, &payload.subject).await? {
        Some(uuid) => uuid,
        None => {
            return Err(AppError::NotFound(format!(
                "Usuário '{}' não encontrado",
                payload.subject
            )));
        }
    };

    let already_exists = {
        let e = state.enforcer.read().await;
        e.has_policy(vec![
            subject_uuid.clone(),
            payload.object.clone(),
            payload.action.clone(),
        ])
    };

    if already_exists {
        return Ok(StatusCode::OK);
    }

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
        Ok(StatusCode::CREATED)
    } else {
        Ok(StatusCode::OK)
    }
}

/// DELETE /api/admin/policies
pub async fn remove_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;

    let user_repo = UserRepository::new(&state.db_pool_auth);
    let subject_uuid = match resolve_subject(&user_repo, &payload.subject).await? {
        Some(uuid) => uuid,
        None => return Ok(StatusCode::NOT_FOUND),
    };

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
        invalidate_cache(&state, &subject_uuid, &payload.object, &payload.action).await;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

/// GET /api/admin/users
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<PaginatedUsers>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let user_repo = UserRepository::new(&state.db_pool_auth);
    let (users, total) = user_repo
        .list(limit, offset, params.search.as_ref())
        .await?;

    Ok(Json(PaginatedUsers {
        users,
        total,
        limit,
        offset,
    }))
}

/// GET /api/admin/users/{id}
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDto>, AppError> {
    let user_repo = UserRepository::new(&state.db_pool_auth);
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    Ok(Json(user))
}

/// POST /api/admin/users
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<UserDto>, AppError> {
    payload.validate()?;

    validate_password_strength(&payload.password)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    let user_repo = UserRepository::new(&state.db_pool_auth);

    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Anyhow(anyhow::anyhow!("Username já existe")));
    }

    let password_clone = payload.password.clone();
    // USO DO NOVO HASH
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha task hash")?
        .context("Erro ao gerar hash")?;

    let user = user_repo.create(&payload.username, &password_hash).await?;

    tracing::info!(user_id = %user.id, event_type = "user_created_by_admin", "Admin criou usuário");

    Ok(Json(user))
}

/// PUT /api/admin/users/{id}
pub async fn update_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<UserDto>, AppError> {
    payload.validate()?;

    if current_user.id == user_id {
        return Err(AppError::Forbidden);
    }

    let user_repo = UserRepository::new(&state.db_pool_auth);

    // Verifica existência
    if user_repo.find_by_id(user_id).await?.is_none() {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    // Atualiza username
    if let Some(ref new_username) = payload.username {
        if user_repo
            .exists_by_username_excluding(new_username, user_id)
            .await?
        {
            return Err(AppError::Anyhow(anyhow::anyhow!("Username já está em uso")));
        }
        user_repo.update_username(user_id, new_username).await?;
    }

    // Atualiza senha
    if let Some(ref new_password) = payload.password {
        validate_password_strength(new_password)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

        let password_clone = new_password.clone();
        // USO DO NOVO HASH
        let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
            .await
            .context("Falha task hash")?
            .context("Erro ao gerar hash")?;

        user_repo.update_password(user_id, &password_hash).await?;

        AuthRepository::new(&state.db_pool_auth)
            .revoke_all_user_tokens(user_id)
            .await
            .ok();
    }

    // Retorna usuário atualizado
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    tracing::info!(user_id = %user_id, event_type = "user_updated_by_admin", "Admin atualizou usuário");

    Ok(Json(user))
}

/// DELETE /api/admin/users/{id}
pub async fn delete_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if current_user.id == user_id {
        return Err(AppError::Forbidden);
    }

    let user_repo = UserRepository::new(&state.db_pool_auth);
    if !user_repo.delete(user_id).await? {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    tracing::warn!(user_id = %user_id, event_type = "user_deleted_by_admin", "Admin deletou usuário");

    Ok(StatusCode::NO_CONTENT)
}

// --- Helpers ---

async fn resolve_subject(
    user_repo: &UserRepository<'_>,
    subject: &str,
) -> Result<Option<String>, AppError> {
    if let Ok(uuid) = Uuid::parse_str(subject) {
        // Se for UUID, verifica se existe pelo ID
        let user = user_repo.find_by_id(uuid).await?;
        return Ok(user.map(|u| u.id.to_string()));
    }

    // Se não for UUID, busca pelo username
    let user = user_repo.find_by_username(subject).await?;
    Ok(user.map(|u| u.id.to_string()))
}

async fn invalidate_cache(state: &AppState, subject: &str, object: &str, action: &str) {
    let cache_key = format!("{}:{}:{}", subject, object, action);
    let mut cache = state.policy_cache.write().await;
    if cache.remove(&cache_key).is_some() {
        tracing::debug!("Cache invalidado para: {}", cache_key);
    }
}
