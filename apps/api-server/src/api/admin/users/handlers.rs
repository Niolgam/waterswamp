use anyhow::Context;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use casbin::MgmtApi;
use core_services::security::hash_password;
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};
use tracing::info;
use uuid::Uuid;
use validator::Validate; // <--- IMPORTANTE: Necessário para remover políticas

use super::contracts::{
    AdminCreateUserRequest, AdminUpdateUserRequest, AdminUserListResponse, UserActionResponse,
};
use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};
use domain::models::UserDtoExtended;

/// GET /admin/users
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<AdminUserListResponse>, AppError> {
    let user_repo = UserRepository::new(&state.db_pool_auth);

    // TODO: Implementar paginação real via Query params
    let (users_dto, total) = user_repo.list(100, 0, None).await?;

    // Mapeamento manual de UserDto para UserDtoExtended (mock para listagem)
    // Em produção, o repositório deve retornar o DTO correto ou fazer join
    let users: Vec<UserDtoExtended> = users_dto
        .into_iter()
        .map(|u| UserDtoExtended {
            id: u.id,
            username: u.username,
            email: u.email,
            role: "user".to_string(), // Default fallback se não vier do banco
            email_verified: false,
            email_verified_at: None,
            mfa_enabled: false,
            created_at: u.created_at,
            updated_at: u.updated_at,
        })
        .collect();

    Ok(Json(AdminUserListResponse { users, total }))
}

/// POST /admin/users
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<(StatusCode, Json<UserDtoExtended>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let user_repo = UserRepository::new(&state.db_pool_auth);

    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Conflict("Username já existe".to_string()));
    }
    if user_repo.exists_by_email(&payload.email).await? {
        return Err(AppError::Conflict("Email já existe".to_string()));
    }

    let hash = tokio::task::spawn_blocking(move || hash_password(&payload.password))
        .await
        .context("Falha na task de hash")?
        .context("Falha ao gerar hash")?;

    // Cria usuário básico
    let created = user_repo
        .create(&payload.username, &payload.email, &hash)
        .await?;

    // Aplica role se diferente de user
    if payload.role != "user" {
        user_repo.update_role(created.id, &payload.role).await?;
    }

    // Retorna o usuário criado (recuperando versão extendida)
    let user_extended =
        user_repo
            .find_extended_by_id(created.id)
            .await?
            .ok_or(AppError::Anyhow(anyhow::anyhow!(
                "Falha ao recuperar usuário criado"
            )))?;

    Ok((StatusCode::CREATED, Json(user_extended)))
}

/// GET /admin/users/{id}
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDtoExtended>, AppError> {
    let user_repo = UserRepository::new(&state.db_pool_auth);

    let user = user_repo
        .find_extended_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("Usuário não encontrado".to_string()))?;

    Ok(Json(user))
}

/// PUT /admin/users/{id}
pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<Json<UserActionResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let user_repo = UserRepository::new(&state.db_pool_auth);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    if let Some(username) = payload.username {
        user_repo.update_username(user_id, &username).await?;
    }
    if let Some(email) = payload.email {
        user_repo.update_email(user_id, &email).await?;
    }
    if let Some(role) = payload.role {
        user_repo.update_role(user_id, &role).await?;
        auth_repo.revoke_all_user_tokens(user_id).await?;
    }
    if let Some(password) = payload.password {
        let hash = tokio::task::spawn_blocking(move || hash_password(&password))
            .await
            .context("Task error")?
            .context("Hash error")?;
        user_repo.update_password(user_id, &hash).await?;
        auth_repo.revoke_all_user_tokens(user_id).await?;
    }

    Ok(Json(UserActionResponse {
        user_id,
        action: "update".to_string(),
        success: true,
    }))
}

/// DELETE /admin/users/{id}
pub async fn delete_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if current_user.id == user_id {
        return Err(AppError::BadRequest(
            "Não é possível deletar a si mesmo".to_string(),
        ));
    }

    let user_repo = UserRepository::new(&state.db_pool_auth);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    // Revogar tokens antes de deletar
    auth_repo.revoke_all_user_tokens(user_id).await?;

    let deleted = user_repo.delete(user_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    // Remover políticas do Casbin associadas ao usuário
    let mut enforcer = state.enforcer.write().await;
    enforcer
        .remove_filtered_policy(0, vec![user_id.to_string()])
        .await
        .map_err(|e: casbin::Error| AppError::Anyhow(anyhow::anyhow!(e)))?; // <--- CORREÇÃO: Tipo explícito

    info!(target_id = %user_id, admin_id = %current_user.id, "Usuário deletado por admin");

    Ok(StatusCode::NO_CONTENT)
}

/// PUT /admin/users/{id}/ban
pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserActionResponse>, AppError> {
    let user_repo = UserRepository::new(&state.db_pool_auth);
    let auth_repo = AuthRepository::new(&state.db_pool_auth);

    user_repo.disable_user(user_id).await?;
    auth_repo.revoke_all_user_tokens(user_id).await?;

    Ok(Json(UserActionResponse {
        user_id,
        action: "ban".to_string(),
        success: true,
    }))
}

/// PUT /admin/users/{id}/unban
pub async fn unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserActionResponse>, AppError> {
    let user_repo = UserRepository::new(&state.db_pool_auth);
    user_repo.enable_user(user_id).await?;

    Ok(Json(UserActionResponse {
        user_id,
        action: "unban".to_string(),
        success: true,
    }))
}
