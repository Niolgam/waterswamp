use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PolicyRequest {
    #[validate(length(min = 1))]
    #[serde(alias = "subject", alias = "sub")]
    pub sub: String,

    #[validate(length(min = 1))]
    #[serde(alias = "object", alias = "obj")]
    pub obj: String,

    #[validate(length(min = 1))]
    #[serde(alias = "action", alias = "act")]
    pub act: String,
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
