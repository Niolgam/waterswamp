//! User Self-Service Handlers
//!
//! Handlers para gerenciamento de perfil e senha do próprio usuário.

use axum::{extract::State, http::StatusCode, Json};
use tracing::{error, info, instrument};
use validator::Validate;

use crate::{
    extractors::current_user::CurrentUser,
    infra::{errors::AppError, state::AppState},
};

use super::contracts::{
    ChangePasswordRequest, ChangePasswordResponse, ProfileResponse, UpdateProfileRequest,
    UpdateProfileResponse,
};

// =============================================================================
// HANDLERS
// =============================================================================

/// GET /api/v1/users/profile
///
/// Retorna as informações do perfil do usuário autenticado.
#[instrument(skip(state, current_user))]
pub async fn get_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<ProfileResponse>, AppError> {
    info!(user_id = %user.id, username = %user.username, "Buscando perfil do usuário");

    // Buscar dados completos do usuário
    let user_data: (String, String, bool, bool, chrono::NaiveDateTime) = sqlx::query_as(
        r#"
        SELECT username, email, email_verified, mfa_enabled, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .fetch_one(&state.db_pool_auth)
    .await
    .map_err(|e| {
        error!(user_id = %user.id, error = ?e, "Erro ao buscar perfil");
        AppError::NotFound("Usuário não encontrado".to_string())
    })?;

    Ok(Json(ProfileResponse {
        id: user.id,
        username: user_data.0,
        email: user_data.1,
        email_verified: user_data.2,
        mfa_enabled: user_data.3,
        created_at: user_data.4,
    }))
}

/// PUT /api/v1/users/profile
///
/// Atualiza o perfil do usuário autenticado (username e/ou email).
///
/// **Nota:** Se o email for alterado, será marcado como não verificado.
#[instrument(skip(state, current_user, payload))]
pub async fn update_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UpdateProfileResponse>, AppError> {
    // 1. Validar payload
    payload.validate().map_err(|e| {
        error!(user_id = %user.id, validation_errors = ?e, "Validação falhou");
        AppError::Validation(e.to_string())
    })?;

    // 2. Verificar se há algo para atualizar
    if payload.username.is_none() && payload.email.is_none() {
        return Err(AppError::ValidationError(
            "Pelo menos um campo (username ou email) deve ser fornecido".to_string(),
        ));
    }

    let user_id = user.id;
    info!(
        user_id = %user_id,
        new_username = ?payload.username,
        new_email = ?payload.email,
        "Atualizando perfil"
    );

    // 3. Buscar dados atuais do usuário
    let current_user_data: (String, String) =
        sqlx::query_as("SELECT username, email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&state.db_pool_auth)
            .await?;

    let current_username = current_user_data.0;
    let current_email = current_user_data.1;

    // 4. Verificar conflito de username
    if let Some(ref new_username) = payload.username {
        if new_username != &current_username {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)",
            )
            .bind(new_username)
            .bind(user_id)
            .fetch_one(&state.db_pool_auth)
            .await?;

            if exists {
                error!(user_id = %user_id, new_username = %new_username, "Username já em uso");
                return Err(AppError::Conflict("Username já está em uso".to_string()));
            }
        }
    }

    // 5. Verificar conflito de email
    let mut email_changed = false;
    if let Some(ref new_email) = payload.email {
        if new_email != &current_email {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM users WHERE LOWER(email) = LOWER($1) AND id != $2)",
            )
            .bind(new_email)
            .bind(user_id)
            .fetch_one(&state.db_pool_auth)
            .await?;

            if exists {
                error!(user_id = %user_id, new_email = %new_email, "Email já em uso");
                return Err(AppError::Conflict("Email já está em uso".to_string()));
            }
            email_changed = true;
        }
    }

    // 6. Atualizar username se fornecido
    if let Some(new_username) = &payload.username {
        if new_username != &current_username {
            sqlx::query("UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2")
                .bind(new_username)
                .bind(user_id)
                .execute(&state.db_pool_auth)
                .await?;

            info!(user_id = %user_id, new_username = %new_username, "Username atualizado");
        }
    }

    // 7. Atualizar email se fornecido
    if let Some(new_email) = &payload.email {
        if new_email != &current_email {
            sqlx::query(
                "UPDATE users SET email = $1, email_verified = FALSE, updated_at = NOW() WHERE id = $2",
            )
            .bind(new_email)
            .bind(user_id)
            .execute(&state.db_pool_auth)
            .await?;

            info!(
                user_id = %user_id,
                new_email = %new_email,
                "Email atualizado (requer verificação)"
            );
        }
    }

    // 8. Buscar dados atualizados
    let updated_data: (String, String, bool, bool, chrono::NaiveDateTime) = sqlx::query_as(
        r#"
        SELECT username, email, email_verified, mfa_enabled, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db_pool_auth)
    .await?;

    info!(user_id = %user_id, email_changed = %email_changed, "Perfil atualizado");

    Ok(Json(UpdateProfileResponse {
        id: user_id,
        username: updated_data.0,
        email: updated_data.1,
        email_verified: updated_data.2,
        mfa_enabled: updated_data.3,
        created_at: updated_data.4,
        email_changed,
    }))
}

/// PUT /api/v1/users/password
///
/// Altera a senha do usuário autenticado.
///
/// Requer a senha atual para verificação de segurança.
/// Após alteração, todas as sessões (refresh tokens) são revogadas.
#[instrument(skip(state, current_user, payload))]
pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<(StatusCode, Json<ChangePasswordResponse>), AppError> {
    use core_services::security::{hash_password, validate_password_strength, verify_password};

    // 1. Validar payload
    payload.validate().map_err(|e| {
        error!(user_id = %user.id, validation_errors = ?e, "Validação falhou");
        AppError::Validation(e.to_string())
    })?;

    let user_id = user.id;
    info!(user_id = %user_id, "Tentativa de alteração de senha");

    // 2. Buscar hash da senha atual
    let current_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db_pool_auth)
        .await?;

    // 3. Verificar senha atual
    let current_password = payload.current_password.clone();
    let password_valid =
        tokio::task::spawn_blocking(move || verify_password(&current_password, &current_hash))
            .await
            .map_err(|e| {
                error!(error = ?e, "Erro na task de verificação de senha");
                AppError::Anyhow(anyhow::anyhow!("Erro interno"))
            })?
            .map_err(|_| AppError::Unauthorized("Senha atual incorreta".to_string()))?;

    if !password_valid {
        error!(user_id = %user_id, "Verificação de senha atual falhou");
        return Err(AppError::Unauthorized("Senha atual incorreta".to_string()));
    }

    // 4. Verificar se nova senha é diferente da atual
    let new_password_for_check = payload.new_password.clone();
    let current_hash_for_check: String =
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&state.db_pool_auth)
            .await?;

    let same_as_current = tokio::task::spawn_blocking(move || {
        verify_password(&new_password_for_check, &current_hash_for_check)
    })
    .await
    .map_err(|e| {
        error!(error = ?e, "Erro na task de verificação");
        AppError::Anyhow(anyhow::anyhow!("Erro interno"))
    })?
    .unwrap_or(false);

    if same_as_current {
        error!(user_id = %user_id, "Nova senha igual à atual");
        return Err(AppError::Validation(
            "Nova senha deve ser diferente da senha atual".to_string(),
        ));
    }

    // 5. Validar força da nova senha
    validate_password_strength(&payload.new_password).map_err(|e| {
        error!(user_id = %user_id, error = %e, "Senha fraca");
        AppError::BadRequest(e)
    })?;

    // 6. Gerar hash da nova senha
    let new_password = payload.new_password.clone();
    let new_hash = tokio::task::spawn_blocking(move || hash_password(&new_password))
        .await
        .map_err(|e| {
            error!(error = ?e, "Erro na task de hash");
            AppError::Anyhow(anyhow::anyhow!("Erro interno"))
        })?
        .map_err(|e| {
            error!(error = ?e, "Erro ao gerar hash");
            AppError::Anyhow(e)
        })?;

    // 7. Atualizar senha no banco
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(user_id)
        .execute(&state.db_pool_auth)
        .await?;

    // 8. Revogar todos os refresh tokens
    sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE user_id = $1 AND revoked = FALSE")
        .bind(user_id)
        .execute(&state.db_pool_auth)
        .await?;

    info!(
        user_id = %user_id,
        "Senha alterada com sucesso, todas as sessões revogadas"
    );

    Ok((StatusCode::OK, Json(ChangePasswordResponse::default())))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_password_response_default() {
        let response = ChangePasswordResponse::default();
        assert!(response.message.contains("sucesso"));
    }
}
