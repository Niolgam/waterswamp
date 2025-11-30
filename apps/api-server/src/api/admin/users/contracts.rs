use domain::models::UserDtoExtended;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use domain::models::CreateUserPayload as AdminCreateUserRequest;
pub use domain::models::UpdateUserPayload as AdminUpdateUserRequest;

#[derive(Debug, Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<UserDtoExtended>,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct BanUserRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserActionResponse {
    pub user_id: Uuid,
    pub action: String,
    pub success: bool,
}
