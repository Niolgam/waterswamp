use axum::{extract::State, http::StatusCode, Json};
use casbin::{Adapter, CoreApi, MgmtApi};
use domain::ports::UserRepositoryPort;
use persistence::repositories::user_repository::UserRepository;
use validator::Validate;

use super::contracts::{PolicyListResponse, PolicyRequest, PolicyResponse};
use crate::infra::{errors::AppError, state::AppState};

async fn clear_policy_cache(state: &AppState) {
    let mut cache = state.policy_cache.write().await;
    cache.clear();
    tracing::debug!("Policy cache cleared");
}

pub async fn list_policies(
    State(state): State<AppState>,
) -> Result<Json<PolicyListResponse>, AppError> {
    let enforcer = state.enforcer.read().await;
    let policies = enforcer.get_all_policy();
    Ok(Json(PolicyListResponse { policies }))
}

pub async fn add_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<(StatusCode, Json<PolicyResponse>), AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    // Verifica se o Subject (sub) existe no banco de dados antes de criar a policy
    if let Ok(user_id) = uuid::Uuid::parse_str(&payload.sub) {
        // Caso 1: É um UUID
        let user_repo = UserRepository::new(&state.db_pool_auth);
        if user_repo.find_by_id(user_id).await?.is_none() {
            return Err(AppError::NotFound("User not found (UUID)".to_string()));
        }
    } else {
        // Caso 2: Não é UUID, tenta validar como Username
        // Tenta converter String -> Username (Value Object)

        if let Ok(username) = domain::value_objects::Username::try_from(payload.sub.as_str()) {
            let user_repo = UserRepository::new(&state.db_pool_auth);

            // Agora passamos &username (que é do tipo correto), não &payload.sub
            if user_repo.find_by_username(&username).await?.is_none() {
                return Err(AppError::NotFound("User not found (Username)".to_string()));
            }
        } else {
            // Caso 3: Não é UUID e nem formato de Username válido.
            // Pode ser um nome de Role (ex: "admin", "manager") ou grupo.
            // Nesse caso, não validamos na tabela de usuários.
            tracing::debug!(
                "Subject '{}' is not a valid username format, skipping DB check (assumed role)",
                payload.sub
            );
        }
    }

    let mut enforcer = state.enforcer.write().await;

    let policy_exists = enforcer.has_policy(vec![
        payload.sub.clone(),
        payload.obj.clone(),
        payload.act.clone(),
    ]);

    if policy_exists {
        tracing::info!(
            "Policy already exists: sub={}, obj={}, act={}",
            payload.sub,
            payload.obj,
            payload.act
        );
        return Ok((
            StatusCode::OK,
            Json(PolicyResponse {
                success: true,
                message: "Policy already exists".to_string(),
            }),
        ));
    }

    let result = enforcer
        .add_policy(vec![
            payload.sub.clone(),
            payload.obj.clone(),
            payload.act.clone(),
        ])
        .await;

    match result {
        Ok(true) => {
            // Save policies to persist changes
            match enforcer.save_policy().await {
                Ok(_) => {
                    tracing::debug!("Policy saved to database successfully");
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // PostgreSQL error 23505 = unique constraint violation
                    if error_msg.contains("23505")
                        || error_msg.contains("unique")
                        || error_msg.contains("duplicate")
                    {
                        tracing::warn!("Policy already exists in database (concurrent insert)");
                    } else {
                        tracing::error!("Failed to save policy to database: {:?}", e);
                    }
                }
            }

            clear_policy_cache(&state).await;

            Ok((
                StatusCode::CREATED,
                Json(PolicyResponse {
                    success: true,
                    message: "Policy added successfully".to_string(),
                }),
            ))
        }
        Ok(false) => {
            // Already exists -> 200 OK (Idempotent)
            Ok((
                StatusCode::OK,
                Json(PolicyResponse {
                    success: true,
                    message: "Policy already exists".to_string(),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to add policy: {:?}", e);
            Err(AppError::Internal("Failed to add policy".to_string()))
        }
    }
}

pub async fn remove_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<StatusCode, AppError> {
    let mut enforcer = state.enforcer.write().await;
    let removed = enforcer
        .remove_policy(vec![payload.sub, payload.obj, payload.act])
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    if removed {
        // Save policies to persist changes
        if let Err(e) = enforcer.save_policy().await {
            tracing::error!("Failed to save policy: {:?}", e);
            // Don't fail the request - policy is removed from memory
        }

        clear_policy_cache(&state).await;
        tracing::info!("Policy cache cleared after removal");

        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Policy not found".to_string()))
    }
}
