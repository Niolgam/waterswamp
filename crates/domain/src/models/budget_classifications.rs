use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// ============================
// DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct BudgetClassificationDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub code_part: String,
    pub full_code: String,
    pub name: String,
    pub level: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Budget classification with parent information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct BudgetClassificationWithParentDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub code_part: String,
    pub full_code: String,
    pub name: String,
    pub level: i32,
    pub is_active: bool,
    pub parent_name: Option<String>,
    pub parent_full_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tree node for hierarchical representation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BudgetClassificationTreeNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub code_part: String,
    pub full_code: String,
    pub name: String,
    pub level: i32,
    pub is_active: bool,
    #[schema(no_recursion)]
    pub children: Vec<BudgetClassificationTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// Payloads
// ============================

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct CreateBudgetClassificationPayload {
    /// Parent ID (null for level 1 - Categoria Econômica)
    pub parent_id: Option<Uuid>,

    /// Code for this level only (e.g., "30")
    #[validate(length(min = 1, max = 10))]
    pub code_part: String,

    /// Name of the classification
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Active status
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Validate, Deserialize, ToSchema)]
pub struct UpdateBudgetClassificationPayload {
    /// Parent ID (null for level 1)
    pub parent_id: Option<Uuid>,

    /// Code for this level only
    #[validate(length(min = 1, max = 10))]
    pub code_part: Option<String>,

    /// Name
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    /// Active status
    pub is_active: Option<bool>,
}

// ============================
// Query Params
// ============================

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListBudgetClassificationsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
    pub parent_id: Option<Uuid>,
    pub level: Option<i32>,
    pub is_active: Option<bool>,
}

// ============================
// Paginated Response
// ============================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedBudgetClassifications {
    pub items: Vec<BudgetClassificationWithParentDto>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// ============================
// Helper Functions
// ============================

fn default_true() -> bool {
    true
}
