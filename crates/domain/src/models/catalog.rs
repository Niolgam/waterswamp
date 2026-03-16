use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Unit of Measure DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UnitOfMeasureDto {
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub is_base_unit: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUnitOfMeasurePayload {
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub is_base_unit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUnitOfMeasurePayload {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub is_base_unit: Option<bool>,
}

// ============================
// Unit Conversion DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UnitConversionDto {
    pub id: Uuid,
    pub from_unit_id: Uuid,
    pub to_unit_id: Uuid,
    pub catmat_id: Option<Uuid>,
    pub description: Option<String>,
    pub conversion_factor: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UnitConversionWithDetailsDto {
    pub id: Uuid,
    pub from_unit_id: Uuid,
    pub from_unit_name: String,
    pub from_unit_symbol: String,
    pub to_unit_id: Uuid,
    pub to_unit_name: String,
    pub to_unit_symbol: String,
    pub catmat_id: Option<Uuid>,
    pub description: Option<String>,
    pub conversion_factor: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUnitConversionPayload {
    pub from_unit_id: Uuid,
    pub to_unit_id: Uuid,
    pub catmat_id: Option<Uuid>,
    pub description: Option<String>,
    pub conversion_factor: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUnitConversionPayload {
    pub catmat_id: Option<Uuid>,
    pub description: Option<String>,
    pub conversion_factor: rust_decimal::Decimal,
}

// ============================
// CATMAT DTOs (Catálogo de Materiais)
// ============================

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatGroupDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatGroupPayload {
    pub code: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatGroupPayload {
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Classes ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatClassDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatClassWithDetailsDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub pdm_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatClassPayload {
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatClassPayload {
    pub group_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- PDMs (Padrão Descritivo de Material) ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatPdmDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub code: String,
    pub description: String,
    /// Itens deste PDM entram no estoque ao serem lançados em NF
    pub is_stockable: bool,
    /// Itens são bens permanentes (patrimônio) — mutuamente exclusivo com is_stockable
    pub is_permanent: bool,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatPdmWithDetailsDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub class_name: String,
    pub class_code: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub code: String,
    pub description: String,
    pub is_stockable: bool,
    pub is_permanent: bool,
    pub is_active: bool,
    pub verification_status: String,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatPdmPayload {
    pub class_id: Uuid,
    pub code: String,
    pub description: String,
    /// Default: true — itens entram no estoque por padrão
    pub is_stockable: Option<bool>,
    /// Default: false — marcar apenas bens permanentes (patrimônio)
    pub is_permanent: Option<bool>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatPdmPayload {
    pub class_id: Option<Uuid>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_stockable: Option<bool>,
    pub is_permanent: Option<bool>,
    pub is_active: Option<bool>,
}

// --- Items ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatmatItemDto {
    pub id: Uuid,
    pub pdm_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub budget_classification_id: Option<Uuid>,
    pub code: String,
    pub description: String,
    pub is_sustainable: bool,
    pub code_ncm: Option<String>,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatItemWithDetailsDto {
    pub id: Uuid,
    pub pdm_id: Uuid,
    pub pdm_description: String,
    pub pdm_code: String,
    /// Classificação herdada do PDM: item entra no estoque quando lançado em NF
    pub pdm_is_stockable: bool,
    /// Classificação herdada do PDM: item é bem permanente (patrimônio)
    pub pdm_is_permanent: bool,
    pub class_id: Uuid,
    pub class_name: String,
    pub class_code: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub unit_of_measure_id: Uuid,
    pub unit_name: String,
    pub unit_symbol: String,
    pub budget_classification_id: Option<Uuid>,
    pub budget_classification_name: Option<String>,
    pub budget_classification_full_code: Option<String>,
    pub code: String,
    pub description: String,
    pub is_sustainable: bool,
    pub code_ncm: Option<String>,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatmatItemPayload {
    pub pdm_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub budget_classification_id: Option<Uuid>,
    pub code: String,
    pub description: String,
    pub is_sustainable: bool,
    pub code_ncm: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatmatItemPayload {
    pub pdm_id: Option<Uuid>,
    pub unit_of_measure_id: Option<Uuid>,
    pub budget_classification_id: Option<Uuid>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_sustainable: Option<bool>,
    pub code_ncm: Option<String>,
    pub is_active: Option<bool>,
}

// --- Tree ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatGroupTreeNode {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub classes: Vec<CatmatClassTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatmatClassTreeNode {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub pdm_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// CATSER DTOs (Catálogo de Serviços)
// ============================

// --- Sections ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserSectionDto {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserSectionWithDetailsDto {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub division_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserSectionPayload {
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserSectionPayload {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Divisions ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserDivisionDto {
    pub id: Uuid,
    pub section_id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserDivisionWithDetailsDto {
    pub id: Uuid,
    pub section_id: Uuid,
    pub section_name: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub group_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserDivisionPayload {
    pub section_id: Uuid,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserDivisionPayload {
    pub section_id: Option<Uuid>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserGroupDto {
    pub id: Uuid,
    pub division_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserGroupPayload {
    pub division_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserGroupPayload {
    pub division_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Classes ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserClassDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserClassWithDetailsDto {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub verification_status: String,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserClassPayload {
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserClassPayload {
    pub group_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

// --- Items (Serviços) ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct CatserItemDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub budget_classification_id: Option<Uuid>,
    pub code: String,
    pub code_cpc: Option<String>,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub search_links: Option<String>,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserItemWithDetailsDto {
    pub id: Uuid,
    pub class_id: Uuid,
    pub class_name: String,
    pub class_code: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_code: String,
    pub unit_of_measure_id: Uuid,
    pub unit_name: String,
    pub unit_symbol: String,
    pub budget_classification_id: Option<Uuid>,
    pub budget_classification_name: Option<String>,
    pub budget_classification_full_code: Option<String>,
    pub code: String,
    pub code_cpc: Option<String>,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub search_links: Option<String>,
    pub is_active: bool,
    pub verification_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCatserItemPayload {
    pub class_id: Uuid,
    pub unit_of_measure_id: Uuid,
    pub budget_classification_id: Option<Uuid>,
    pub code: String,
    pub code_cpc: Option<String>,
    pub description: String,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub search_links: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCatserItemPayload {
    pub class_id: Option<Uuid>,
    pub unit_of_measure_id: Option<Uuid>,
    pub budget_classification_id: Option<Uuid>,
    pub code: Option<String>,
    pub code_cpc: Option<String>,
    pub description: Option<String>,
    pub supplementary_description: Option<String>,
    pub specification: Option<String>,
    pub search_links: Option<String>,
    pub is_active: Option<bool>,
}

// --- Tree ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserSectionTreeNode {
    pub id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub divisions: Vec<CatserDivisionTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserDivisionTreeNode {
    pub id: Uuid,
    pub section_id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub groups: Vec<CatserGroupTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserGroupTreeNode {
    pub id: Uuid,
    pub division_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub classes: Vec<CatserClassTreeNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CatserClassTreeNode {
    pub id: Uuid,
    pub group_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub item_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================
// ComprasGov API Response DTOs
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovResponse<T> {
    pub resultado: Vec<T>,
    pub total_registros: i64,
    pub total_paginas: i64,
    pub paginas_restantes: i64,
}

// --- CATMAT API entities ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovGrupoMaterial {
    pub codigo_grupo: i64,
    pub nome_grupo: String,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovClasseMaterial {
    pub codigo_classe: i64,
    pub nome_classe: String,
    pub codigo_grupo: i64,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovPdmMaterial {
    pub codigo_pdm: i64,
    pub nome_pdm: String,
    pub codigo_classe: i64,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovItemMaterial {
    pub codigo_item: i64,
    pub nome_item: String,
    pub codigo_pdm: i64,
    pub codigo_classe: i64,
    pub codigo_grupo: i64,
    pub status: bool,
    pub sustentavel: bool,
}

// --- CATSER API entities ---

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovSectionService {
    pub codigo_secao: i64,
    pub nome_secao: String,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovDivisionService {
    pub codigo_divisao: i64,
    pub nome_divisao: String,
    pub codigo_secao: i64,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovGrupoServico {
    pub codigo_grupo: i64,
    pub nome_grupo: String,
    pub codigo_divisao: i64,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovClasseServico {
    pub codigo_classe: i64,
    pub nome_classe: String,
    pub codigo_grupo: i64,
    pub status: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComprasGovItemServico {
    pub codigo_item: i64,
    pub nome_item: String,
    pub codigo_classe: i64,
    pub codigo_grupo: i64,
    pub codigo_divisao: i64,
    pub codigo_secao: i64,
    pub codigo_cpc: Option<String>,
    pub status: bool,
}
