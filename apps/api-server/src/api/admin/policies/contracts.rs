use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PolicyRequest {
    #[validate(length(min = 1))]
    pub sub: String, // Subject (role/user)
    #[validate(length(min = 1))]
    pub obj: String, // Object (resource)
    #[validate(length(min = 1))]
    pub act: String, // Action (read/write)
}

#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PolicyListResponse {
    pub policies: Vec<Vec<String>>,
}
