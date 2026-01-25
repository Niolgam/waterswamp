use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

lazy_static! {
    /// Valid subject pattern: alphanumeric, hyphens, underscores, colons, and UUIDs
    static ref SUBJECT_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9_\-:]+$"
    ).unwrap();

    /// Valid object pattern: resource paths like /api/v1/users or wildcards
    static ref OBJECT_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9_\-/:*]+$"
    ).unwrap();

    /// Valid action pattern: CRUD operations
    static ref ACTION_REGEX: Regex = Regex::new(
        r"^(create|read|update|delete|list|manage|\*)$"
    ).unwrap();
}

/// Request to add or check a policy rule in the RBAC system.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PolicyRequest {
    /// The subject of the policy (user ID or role name)
    #[validate(length(min = 1, max = 100, message = "Subject must be between 1 and 100 characters"))]
    #[validate(regex(path = *SUBJECT_REGEX, message = "Subject contains invalid characters"))]
    #[serde(alias = "subject", alias = "sub")]
    pub sub: String,

    /// The object/resource being accessed (e.g., "/api/v1/users")
    #[validate(length(min = 1, max = 200, message = "Object must be between 1 and 200 characters"))]
    #[validate(regex(path = *OBJECT_REGEX, message = "Object contains invalid characters"))]
    #[serde(alias = "object", alias = "obj")]
    pub obj: String,

    /// The action being performed (create, read, update, delete, list, manage, *)
    #[validate(length(min = 1, max = 50, message = "Action must be between 1 and 50 characters"))]
    #[validate(regex(path = *ACTION_REGEX, message = "Action must be one of: create, read, update, delete, list, manage, *"))]
    #[serde(alias = "action", alias = "act")]
    pub act: String,
}

/// Response for policy operations.
#[derive(Debug, Serialize, ToSchema)]
pub struct PolicyResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Descriptive message
    pub message: String,
}

/// Response containing a list of policies.
#[derive(Debug, Serialize, ToSchema)]
pub struct PolicyListResponse {
    /// List of policies as [subject, object, action] tuples
    pub policies: Vec<Vec<String>>,
}
