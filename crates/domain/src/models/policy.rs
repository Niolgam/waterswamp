use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

lazy_static! {
    /// Valid subject pattern: alphanumeric, hyphens, underscores, and UUIDs
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
///
/// Policies follow the format: (subject, object, action)
/// - subject: The user ID, role name, or group identifier
/// - object: The resource path (e.g., "/api/v1/users", "/api/v1/admin/*")
/// - action: The operation (create, read, update, delete, list, manage, *)
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct PolicyRequest {
    /// The subject of the policy (user ID or role name)
    #[validate(length(min = 1, max = 100, message = "Subject must be between 1 and 100 characters"))]
    #[validate(regex(path = *SUBJECT_REGEX, message = "Subject contains invalid characters"))]
    pub sub: String,

    /// The object/resource being accessed (e.g., "/api/v1/users")
    #[validate(length(min = 1, max = 200, message = "Object must be between 1 and 200 characters"))]
    #[validate(regex(path = *OBJECT_REGEX, message = "Object contains invalid characters"))]
    pub obj: String,

    /// The action being performed (create, read, update, delete, list, manage, *)
    #[validate(length(min = 1, max = 50, message = "Action must be between 1 and 50 characters"))]
    #[validate(regex(path = *ACTION_REGEX, message = "Action must be one of: create, read, update, delete, list, manage, *"))]
    pub act: String,
}

impl PolicyRequest {
    /// Creates a new policy request.
    pub fn new(sub: impl Into<String>, obj: impl Into<String>, act: impl Into<String>) -> Self {
        Self {
            sub: sub.into(),
            obj: obj.into(),
            act: act.into(),
        }
    }
}

/// Response for policy check operations.
#[derive(Debug, Serialize, ToSchema)]
pub struct PolicyCheckResponse {
    /// Whether the policy allows the action
    pub allowed: bool,
    /// The policy that was checked
    pub policy: PolicyRequest,
}

/// Response for policy list operations.
#[derive(Debug, Serialize, ToSchema)]
pub struct PolicyListResponse {
    /// List of policies
    pub policies: Vec<PolicyRequest>,
    /// Total count
    pub total: usize,
}
