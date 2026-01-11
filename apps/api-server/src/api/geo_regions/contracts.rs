use domain::value_objects::{LocationName, StateCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ============================
// Country Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CountryResponse {
    /// ID do país
    pub id: Uuid,
    /// Nome do país
    pub name: LocationName,
    /// Código ISO 3166-1 alpha-2
    pub iso2: String,
    /// Código Bacen
    pub bacen_code: i32,
    /// Data de criação
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Data de atualização
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// State Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StateResponse {
    /// ID do estado
    pub id: Uuid,
    /// Nome do estado
    pub name: LocationName,
    /// Sigla do estado
    pub abbreviation: StateCode,
    /// Código IBGE
    pub ibge_code: i32,
    /// ID do país
    pub country_id: Uuid,
    /// Data de criação
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Data de atualização
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StateWithCountryResponse {
    /// ID do estado
    pub id: Uuid,
    /// Nome do estado
    pub name: LocationName,
    /// Sigla do estado
    pub abbreviation: StateCode,
    /// Código IBGE
    pub ibge_code: i32,
    /// ID do país
    pub country_id: Uuid,
    /// Nome do país
    pub country_name: LocationName,
    /// Código ISO2 do país
    pub country_iso2: String,
    /// Código Bacen do país
    pub country_bacen_code: i32,
    /// Data de criação
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Data de atualização
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================
// City Response DTOs
// ============================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CityResponse {
    /// ID da cidade
    pub id: Uuid,
    /// Nome da cidade
    pub name: LocationName,
    /// Código IBGE
    pub ibge_code: i32,
    /// ID do estado
    pub state_id: Uuid,
    /// Data de criação
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Data de atualização
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CityWithStateResponse {
    /// ID da cidade
    pub id: Uuid,
    /// Nome da cidade
    pub name: LocationName,
    /// Código IBGE
    pub ibge_code: i32,
    /// ID do estado
    pub state_id: Uuid,
    /// Nome do estado
    pub state_name: LocationName,
    /// Sigla do estado
    pub state_abbreviation: StateCode,
    /// Código IBGE do estado
    pub state_ibge_code: i32,
    /// ID do país
    pub country_id: Uuid,
    /// Nome do país
    pub country_name: LocationName,
    /// Código ISO2 do país
    pub country_iso2: String,
    /// Código Bacen do país
    pub country_bacen_code: i32,
    /// Data de criação
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Data de atualização
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
