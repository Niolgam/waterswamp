use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BudgetClassificationResponse {
    /// ID da classificação orçamentária
    pub id: Uuid,
    /// ID do pai (null para nível 1)
    pub parent_id: Option<Uuid>,
    /// Código apenas deste nível
    pub code_part: String,
    /// Código completo (calculado automaticamente)
    pub full_code: String,
    /// Nome da classificação
    pub name: String,
    /// Nível (1 a 5)
    pub level: i32,
    /// Status ativo
    pub is_active: bool,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Data de atualização
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BudgetClassificationWithParentResponse {
    /// ID da classificação orçamentária
    pub id: Uuid,
    /// ID do pai
    pub parent_id: Option<Uuid>,
    /// Código apenas deste nível
    pub code_part: String,
    /// Código completo
    pub full_code: String,
    /// Nome da classificação
    pub name: String,
    /// Nível (1 a 5)
    pub level: i32,
    /// Status ativo
    pub is_active: bool,
    /// Nome do pai
    pub parent_name: Option<String>,
    /// Código completo do pai
    pub parent_full_code: Option<String>,
    /// Data de criação
    pub created_at: DateTime<Utc>,
    /// Data de atualização
    pub updated_at: DateTime<Utc>,
}
