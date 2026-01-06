use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::value_objects::Email;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResendVerificationPayload {
    pub email: Email,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct VerifyEmailPayload {
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationResponse {
    pub verified: bool,
    pub message: String,
}
