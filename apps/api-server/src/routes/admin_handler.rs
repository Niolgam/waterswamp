// apps/api-server/src/routes/admin_handler.rs

use crate::{error::AppError, state::AppState, web_models::CurrentUser};
use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use casbin::{MgmtApi, RbacApi}; // Importar RbacApi
use core_services::security::{hash_password, validate_password_strength};
use domain::models::{
    CreateUserPayload,
    ListUsersQuery,
    PaginatedUsers,
    PolicyRequest,
    UpdateUserPayload,
    UserDetailDto,
    UserDto, // Usado em PaginatedUsers
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
) -> Result<Json<UserDetailDto>, AppError> {
    let user_repo = UserRepository::new(&state.db_pool_auth);
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    let roles = {
        let enforcer = state.enforcer.read().await;
        enforcer.get_roles_for_user(&user_id.to_string(), None)
    };

    Ok(Json(UserDetailDto { user, roles }))
}

/// POST /api/admin/users
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<UserDetailDto>, AppError> {
    payload.validate()?;

    validate_password_strength(&payload.password)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user_repo = UserRepository::new(&state.db_pool_auth);

    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Conflict("Username já está em uso".to_string()));
    }
    if user_repo.exists_by_email(&payload.email).await? {
        return Err(AppError::Conflict("Email já está em uso".to_string()));
    }

    let password_clone = payload.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
        .await
        .context("Falha task hash")?
        .context("Erro ao gerar hash")?;

    // 1. Criar usuário no banco
    let user = user_repo
        .create(&payload.username, &payload.email, &password_hash)
        .await?;

    // 2. Adicionar papel no Casbin
    let role = payload.role.clone();
    {
        let mut enforcer = state.enforcer.write().await;
        enforcer
            .add_grouping_policy(vec![user.id.to_string(), role.clone()])
            .await
            .map_err(|e| anyhow::anyhow!("Erro no Casbin: {}", e))?;
    }

    // 3. Enviar email de boas-vindas
    state
        .email_service
        .send_welcome_email(payload.email, &user.username);

    tracing::info!(user_id = %user.id, role = %role, event_type = "user_created_by_admin", "Admin criou usuário");

    Ok(Json(UserDetailDto {
        user,
        roles: vec![role],
    }))
}

/// PUT /api/admin/users/{id}
pub async fn update_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<UserDetailDto>, AppError> {
    payload.validate()?;

    if current_user.id == user_id {
        return Err(AppError::Forbidden);
    }

    let user_repo = UserRepository::new(&state.db_pool_auth);

    if user_repo.find_by_id(user_id).await?.is_none() {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    // Atualiza username
    if let Some(ref new_username) = payload.username {
        if user_repo
            .exists_by_username_excluding(new_username, user_id)
            .await?
        {
            return Err(AppError::Conflict("Username já está em uso".to_string()));
        }
        user_repo.update_username(user_id, new_username).await?;
    }

    // Atualiza email
    if let Some(ref new_email) = payload.email {
        if user_repo
            .exists_by_email_excluding(new_email, user_id)
            .await?
        {
            return Err(AppError::Conflict("Email já está em uso".to_string()));
        }
        user_repo.update_email(user_id, new_email).await?;
    }

    // ⭐ CORREÇÃO: Lógica de 'new_password' preenchida
    if let Some(ref new_password) = payload.password {
        validate_password_strength(new_password)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let password_clone = new_password.clone();
        let password_hash = tokio::task::spawn_blocking(move || hash_password(&password_clone))
            .await
            .context("Falha task hash")?
            .context("Erro ao gerar hash")?;

        user_repo.update_password(user_id, &password_hash).await?;

        // ⭐ CORREÇÃO: 'AuthRepository' agora está a ser usado
        AuthRepository::new(&state.db_pool_auth)
            .revoke_all_user_tokens(user_id)
            .await
            .ok();
    }

    // ⭐ CORREÇÃO: Lógica de 'new_role' preenchida
    let mut updated_roles = Vec::new();
    if let Some(new_role) = payload.role {
        let user_id_str = user_id.to_string();
        let mut enforcer = state.enforcer.write().await;

        // 1. Remove papéis antigos
        let old_roles = enforcer.get_roles_for_user(&user_id_str, None);
        for r in old_roles {
            enforcer
                .remove_grouping_policy(vec![user_id_str.clone(), r])
                .await
                .map_err(|e| anyhow::anyhow!("Erro no Casbin: {}", e))?;
        }

        // 2. Adiciona papel novo
        enforcer
            .add_grouping_policy(vec![user_id_str, new_role.clone()])
            .await
            .map_err(|e| anyhow::anyhow!("Erro no Casbin: {}", e))?;
        updated_roles.push(new_role);
    } else {
        // Se não foi enviado um novo papel, apenas lemos os papéis existentes
        updated_roles = state
            .enforcer
            .read()
            .await
            .get_roles_for_user(&user_id.to_string(), None);
    }

    // Retorna usuário atualizado
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Usuário não encontrado (pós-update)".to_string()))?;

    tracing::info!(user_id = %user_id, event_type = "user_updated_by_admin", "Admin atualizou usuário");

    Ok(Json(UserDetailDto {
        user,
        roles: updated_roles,
    }))
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

    {
        let mut enforcer = state.enforcer.write().await;
        enforcer
            .remove_filtered_grouping_policy(0, vec![user_id.to_string()])
            .await
            .map_err(|e| anyhow::anyhow!("Erro no Casbin: {}", e))?;
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
        let user = user_repo.find_by_id(uuid).await?;
        return Ok(user.map(|u| u.id.to_string()));
    }
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
