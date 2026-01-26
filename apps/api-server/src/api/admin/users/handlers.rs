use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use casbin::{CoreApi, MgmtApi};
use core_services::security::hash_password;
use domain::models::{ListUsersQuery, UserDetailDto, UserDto, UserDtoExtended};
use domain::pagination::Paginated;
use domain::ports::AuthRepositoryPort;
use domain::ports::UserRepositoryPort;
use persistence::repositories::{auth_repository::AuthRepository, user_repository::UserRepository};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use super::contracts::{AdminCreateUserRequest, AdminUpdateUserRequest, BanUserRequest, UserActionResponse};
use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

/// User response for list endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct UserListItem {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub roles: Vec<String>,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn user_to_user_detail_dto(user: UserDtoExtended) -> UserDetailDto {
    UserDetailDto {
        user: UserDto {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        },
        roles: vec![user.role],
    }
}

/// GET /admin/users
#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    tag = "Admin",
    params(
        ("limit" = Option<i64>, Query, description = "Limite de resultados por página"),
        ("offset" = Option<i64>, Query, description = "Offset para paginação"),
        ("search" = Option<String>, Query, description = "Termo de busca")
    ),
    responses(
        (status = 200, description = "Lista de usuários"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<Paginated<UserListItem>>, AppError> {
    let user_repo = UserRepository::new(state.db_pool_auth);

    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    // Handle "q" alias or empty search
    let search = params.search.filter(|s| !s.trim().is_empty());

    let (users_dto, total) = user_repo.list(limit, offset, search).await?;

    // Map users to include role information
    let items: Vec<UserListItem> = users_dto
        .into_iter()
        .map(|u| UserListItem {
            id: u.id,
            username: u.username.to_string(),
            email: u.email.to_string(),
            role: "user".to_string(),
            roles: vec!["user".to_string()],
            email_verified: false,
            mfa_enabled: false,
            created_at: u.created_at,
            updated_at: u.updated_at,
        })
        .collect();

    Ok(Json(Paginated::new(items, total, limit, offset)))
}

/// POST /admin/users
#[utoipa::path(
    post,
    path = "/api/v1/admin/users",
    tag = "Admin",
    request_body = AdminCreateUserRequest,
    responses(
        (status = 200, description = "Usuário criado com sucesso", body = UserDetailDto),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 409, description = "Username ou email já existe")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<AdminCreateUserRequest>,
) -> Result<(StatusCode, Json<UserDetailDto>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let user_repo = UserRepository::new(state.db_pool_auth);

    // CORREÇÃO: payload.username é um Username, passamos referência &Username
    if user_repo.exists_by_username(&payload.username).await? {
        return Err(AppError::Conflict("Username já existe".to_string()));
    }
    // CORREÇÃO: payload.email é um Email, passamos referência &Email
    if user_repo.exists_by_email(&payload.email).await? {
        return Err(AppError::Conflict("Email já existe".to_string()));
    }

    let hash = tokio::task::spawn_blocking(move || hash_password(&payload.password))
        .await
        .context("Task error")?
        .context("Hash error")?;

    // CORREÇÃO: Passando tipos fortes para o create
    let created = user_repo
        .create(&payload.username, &payload.email, &hash)
        .await?;

    let role = if payload.role.is_empty() {
        "user".to_string()
    } else {
        payload.role.clone()
    };
    if role != "user" {
        user_repo.update_role(created.id, &role).await?;
    }

    // Sync with Casbin
    {
        let mut enforcer = state.enforcer.write().await;
        enforcer
            .add_grouping_policy(vec![created.id.to_string(), role.clone()])
            .await
            .ok();
        let _ = enforcer.save_policy().await;
    }

    let user_extended = user_repo
        .find_extended_by_id(created.id)
        .await?
        .ok_or(AppError::Anyhow(anyhow::anyhow!("User not found")))?;

    Ok((StatusCode::OK, Json(user_to_user_detail_dto(user_extended))))
}

/// GET /admin/users/{id}
#[utoipa::path(
    get,
    path = "/api/v1/admin/users/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do usuário")
    ),
    responses(
        (status = 200, description = "Usuário encontrado", body = UserDetailDto),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Usuário não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDetailDto>, AppError> {
    let user_repo = UserRepository::new(state.db_pool_auth);
    let user = user_repo
        .find_extended_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("Usuário não encontrado".to_string()))?;

    Ok(Json(user_to_user_detail_dto(user)))
}

/// PUT /admin/users/{id}
#[utoipa::path(
    put,
    path = "/api/v1/admin/users/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do usuário")
    ),
    request_body = AdminUpdateUserRequest,
    responses(
        (status = 200, description = "Usuário atualizado com sucesso", body = UserDetailDto),
        (status = 400, description = "Dados inválidos"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador"),
        (status = 404, description = "Usuário não encontrado"),
        (status = 409, description = "Username ou email já está em uso")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<Json<UserDetailDto>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    let auth_repo = AuthRepository::new(state.db_pool_auth.clone());
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    if user_repo.find_by_id(user_id).await?.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // 1. Username
    // payload.username é Option<Username>. 'username' é &Username.
    if let Some(ref username) = payload.username {
        // Verifica duplicidade antes de atualizar
        if user_repo
            .exists_by_username_excluding(username, user_id)
            .await?
        {
            return Err(AppError::Conflict("Username already taken".to_string()));
        }
        user_repo.update_username(user_id, username).await?;
    }

    // 2. Email
    // payload.email é Option<Email>. 'email' é &Email.
    if let Some(ref email) = payload.email {
        if user_repo.exists_by_email_excluding(email, user_id).await? {
            return Err(AppError::Conflict("Email already taken".to_string()));
        }
        user_repo.update_email(user_id, email).await?;
    }

    // 3. Role
    if let Some(ref role) = payload.role {
        user_repo.update_role(user_id, role).await?;
        auth_repo.revoke_all_user_tokens(user_id).await?;

        let mut enforcer = state.enforcer.write().await;
        enforcer
            .remove_filtered_grouping_policy(0, vec![user_id.to_string()])
            .await
            .ok();
        enforcer
            .add_grouping_policy(vec![user_id.to_string(), role.clone()])
            .await
            .ok();
        let _ = enforcer.save_policy().await;
    }

    // 4. Password
    if let Some(ref password) = payload.password {
        let hash = tokio::task::spawn_blocking({
            let pwd = password.clone();
            move || hash_password(&pwd)
        })
        .await
        .context("Task error")?
        .context("Hash error")?;

        user_repo.update_password(user_id, &hash).await?;
        auth_repo.revoke_all_user_tokens(user_id).await?;
    }

    let updated_user = user_repo
        .find_extended_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user_to_user_detail_dto(updated_user)))
}

/// DELETE /admin/users/{id}
#[utoipa::path(
    delete,
    path = "/api/v1/admin/users/{id}",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do usuário")
    ),
    responses(
        (status = 204, description = "Usuário deletado com sucesso"),
        (status = 401, description = "Não autenticado"),
        (status = 403, description = "Sem permissão de administrador ou tentando deletar a si mesmo"),
        (status = 404, description = "Usuário não encontrado")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    if current_user.id == user_id {
        return Err(AppError::Forbidden(
            "Não é possível deletar a si mesmo".to_string(),
        ));
    }

    let auth_repo = AuthRepository::new(state.db_pool_auth.clone());
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    auth_repo.revoke_all_user_tokens(user_id).await?;

    let deleted = user_repo.delete(user_id).await?;
    if !deleted {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    {
        let mut enforcer = state.enforcer.write().await;
        enforcer
            .remove_filtered_policy(0, vec![user_id.to_string()])
            .await
            .ok();
        enforcer
            .remove_filtered_grouping_policy(0, vec![user_id.to_string()])
            .await
            .ok();
        // Save policies to persist changes
        if let Err(e) = enforcer.save_policy().await {
            tracing::error!("Failed to save policy after user deletion: {:?}", e);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /admin/users/{id}/ban
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/ban",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do usuário")
    ),
    request_body = BanUserRequest,
    responses(
        (status = 200, description = "User banned successfully", body = UserActionResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "No admin permission or trying to ban yourself"),
        (status = 404, description = "User not found or already banned")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn ban_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<BanUserRequest>,
) -> Result<Json<UserActionResponse>, AppError> {
    // Prevent self-ban
    if current_user.id == user_id {
        return Err(AppError::Forbidden("Cannot ban yourself".to_string()));
    }

    let auth_repo = AuthRepository::new(state.db_pool_auth.clone());
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    // Ban the user
    user_repo
        .ban_user(user_id, payload.reason)
        .await
        .map_err(|e| match e {
            domain::errors::RepositoryError::NotFound => {
                AppError::NotFound("User not found or already banned".to_string())
            }
            _ => AppError::from(e),
        })?;

    // Revoke all tokens to force logout
    auth_repo.revoke_all_user_tokens(user_id).await?;

    Ok(Json(UserActionResponse {
        user_id,
        action: "ban".to_string(),
        success: true,
        message: Some("User has been banned and all sessions revoked".to_string()),
    }))
}

/// POST /admin/users/{id}/unban
#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{id}/unban",
    tag = "Admin",
    params(
        ("id" = Uuid, Path, description = "ID do usuário")
    ),
    responses(
        (status = 200, description = "User unbanned successfully", body = UserActionResponse),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "No admin permission"),
        (status = 404, description = "User not found or not banned")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserActionResponse>, AppError> {
    let user_repo = UserRepository::new(state.db_pool_auth.clone());

    // Unban the user
    user_repo.unban_user(user_id).await.map_err(|e| match e {
        domain::errors::RepositoryError::NotFound => {
            AppError::NotFound("User not found or not banned".to_string())
        }
        _ => AppError::from(e),
    })?;

    Ok(Json(UserActionResponse {
        user_id,
        action: "unban".to_string(),
        success: true,
        message: Some("User has been unbanned".to_string()),
    }))
}
