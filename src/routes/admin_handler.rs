use std::path::Path;

use crate::{
    error::AppError,
    models::{
        CreateUserPayload, CurrentUser, ListUsersQuery, PaginatedUsers, PolicyRequest,
        UpdateUserPayload, UserDto,
    },
    state::AppState,
};
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

/// GET /api/admin/users
/// Lista todos os usuários (paginado)
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<PaginatedUsers>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    // Query base
    let mut query = "SELECT id, username, created_at, updated_at FROM users".to_string();

    // Se houver busca, adicionar WHERE
    if let Some(ref search) = params.search {
        query.push_str(&format!(" WHERE username ILIKE '%{}%'", search));
    }

    query.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");

    let users = sqlx::query_as::<_, UserDto>(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db_pool_auth)
        .await?;

    // Count total
    let mut count_query = "SELECT COUNT(*) FROM users".to_string();
    if let Some(ref search) = params.search {
        count_query.push_str(&format!(" WHERE username ILIKE '%{}%'", search));
    }

    let total: i64 = sqlx::query_scalar(&count_query)
        .fetch_one(&state.db_pool_auth)
        .await?;

    Ok(Json(PaginatedUsers {
        users,
        total,
        limit,
        offset,
    }))
}

/// GET /api/admin/users/:id
/// Busca um usuário específico
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDto>, AppError> {
    let user = sqlx::query_as::<_, UserDto>(
        "SELECT id, username, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool_auth)
    .await?
    .ok_or_else(|| AppError::NotFound("Usuário não encontrado".to_string()))?;

    Ok(Json(user))
}

/// POST /api/admin/users
/// Cria um novo usuário (admin pode definir role depois via Casbin)
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<UserDto>, AppError> {
    payload.validate()?;

    // Validar senha forte
    crate::security::validate_password_strength(&payload.password)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    // Verificar duplicata
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
        .bind(&payload.username)
        .fetch_one(&state.db_pool_auth)
        .await?;

    if exists {
        return Err(AppError::Anyhow(anyhow::anyhow!("Username já existe")));
    }

    // Hash senha
    let password_clone = payload.password.clone();
    let password_hash =
        tokio::task::spawn_blocking(move || bcrypt::hash(password_clone, bcrypt::DEFAULT_COST))
            .await??;

    // Inserir
    let user = sqlx::query_as::<_, UserDto>(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ($1, $2)
        RETURNING id, username, created_at, updated_at
        "#,
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&state.db_pool_auth)
    .await?;

    tracing::info!(
        user_id = %user.id,
        username = %user.username,
        event_type = "user_created_by_admin",
        "Admin criou novo usuário"
    );

    Ok(Json(user))
}

/// PUT /api/admin/users/:id
/// Atualiza um usuário
pub async fn update_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<UserDto>, AppError> {
    payload.validate()?;

    // Admin não pode modificar a si mesmo via esta rota
    if current_user.id == user_id {
        return Err(AppError::Forbidden);
    }

    // Verificar se usuário existe
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(&state.db_pool_auth)
        .await?;

    if !exists {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    // Update username (se fornecido)
    if let Some(ref new_username) = payload.username {
        // Verificar duplicata
        let username_taken: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)",
        )
        .bind(new_username)
        .bind(user_id)
        .fetch_one(&state.db_pool_auth)
        .await?;

        if username_taken {
            return Err(AppError::Anyhow(anyhow::anyhow!("Username já está em uso")));
        }

        sqlx::query("UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_username)
            .bind(user_id)
            .execute(&state.db_pool_auth)
            .await?;
    }

    // Update password (se fornecido)
    if let Some(ref new_password) = payload.password {
        crate::security::validate_password_strength(new_password)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

        let password_clone = new_password.clone();
        let password_hash =
            tokio::task::spawn_blocking(move || bcrypt::hash(password_clone, bcrypt::DEFAULT_COST))
                .await??;

        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&password_hash)
            .bind(user_id)
            .execute(&state.db_pool_auth)
            .await?;

        // Revogar refresh tokens do usuário
        crate::routes::auth_handler::revoke_all_user_tokens(&state.db_pool_auth, user_id)
            .await
            .ok();
    }

    // Buscar usuário atualizado
    let user = sqlx::query_as::<_, UserDto>(
        "SELECT id, username, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.db_pool_auth)
    .await?;

    tracing::info!(
        user_id = %user_id,
        event_type = "user_updated_by_admin",
        "Admin atualizou usuário"
    );

    Ok(Json(user))
}

/// DELETE /api/admin/users/:id
/// Deleta um usuário
pub async fn delete_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Admin não pode deletar a si mesmo
    if current_user.id == user_id {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db_pool_auth)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Usuário não encontrado".to_string()));
    }

    tracing::warn!(
        user_id = %user_id,
        event_type = "user_deleted_by_admin",
        "Admin deletou usuário"
    );

    Ok(StatusCode::NO_CONTENT)
}
