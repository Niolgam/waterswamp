use axum::{extract::State, http::StatusCode, Json};
use casbin::{Adapter, CoreApi, MgmtApi};
use persistence::repositories::user_repository::UserRepository;
use validator::Validate;

use super::contracts::{PolicyListResponse, PolicyRequest, PolicyResponse};
use crate::infra::{errors::AppError, state::AppState};

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

    // Explicit check for non-existent user if subject looks like a UUID
    if let Ok(user_id) = uuid::Uuid::parse_str(&payload.sub) {
        let user_repo = UserRepository::new(&state.db_pool_auth);
        if user_repo.find_by_id(user_id).await?.is_none() {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    }

    let mut enforcer = state.enforcer.write().await;

    let result = enforcer
        .add_policy(vec![payload.sub, payload.obj, payload.act])
        .await;

    match result {
        Ok(true) => {
            // Save policies to persist changes
            if let Err(e) = enforcer.save_policy().await {
                tracing::error!("Failed to save policy: {:?}", e);
                // Don't fail the request - policy is in memory
            }
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
) -> Result<Json<PolicyResponse>, AppError> {
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
        Ok(Json(PolicyResponse {
            success: true,
            message: "Policy removed successfully".to_string(),
        }))
    } else {
        Err(AppError::NotFound("Policy not found".to_string()))
    }
}
