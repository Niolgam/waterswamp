use super::contracts::{PolicyListResponse, PolicyRequest, PolicyResponse};
use crate::infra::{errors::AppError, state::AppState};
use axum::{extract::State, Json};
use casbin::MgmtApi;
use validator::Validate;

/// GET /admin/policies
pub async fn list_policies(
    State(state): State<AppState>,
) -> Result<Json<PolicyListResponse>, AppError> {
    let enforcer = state.enforcer.read().await;

    let policies = enforcer.get_all_policy();

    Ok(Json(PolicyListResponse { policies }))
}

/// POST /admin/policies
pub async fn add_policy(
    State(state): State<AppState>,
    Json(payload): Json<PolicyRequest>,
) -> Result<Json<PolicyResponse>, AppError> {
    if let Err(e) = payload.validate() {
        return Err(AppError::Validation(e));
    }

    // Adquire lock de escrita
    let mut enforcer = state.enforcer.write().await;

    let added = enforcer
        .add_policy(vec![payload.sub, payload.obj, payload.act])
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    if added {
        Ok(Json(PolicyResponse {
            success: true,
            message: "Policy added successfully".to_string(),
        }))
    } else {
        Err(AppError::Conflict("Policy already exists".to_string()))
    }
}

/// DELETE /admin/policies
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
        Ok(Json(PolicyResponse {
            success: true,
            message: "Policy removed successfully".to_string(),
        }))
    } else {
        Err(AppError::NotFound("Policy not found".to_string()))
    }
}
