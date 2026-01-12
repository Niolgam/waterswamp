use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// SIORG API Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiorgOrganization {
    pub codigo_siorg: i32,
    pub sigla: String,
    pub nome: String,
    pub cnpj: Option<String>,
    pub codigo_ug: Option<i32>,
    pub ativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiorgUnit {
    pub codigo_siorg: i32,
    pub codigo_unidade_pai: Option<i32>,
    pub nome: String,
    pub tipo_unidade: String,
    pub area_atuacao: String,
    pub ativo: bool,
    pub nivel_hierarquico: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiorgUnitType {
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiorgCategory {
    pub nome: String,
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiorgApiResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

// ============================================================================
// SIORG Client
// ============================================================================

#[derive(Debug, Clone)]
pub struct SiorgClient {
    client: Client,
    base_url: String,
    api_token: Option<String>,
}

impl SiorgClient {
    /// Creates a new SIORG API client
    pub fn new(base_url: String, api_token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            base_url,
            api_token,
        })
    }

    /// Helper to add authentication headers
    fn add_auth_header(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.api_token {
            request.header("Authorization", format!("Bearer {}", token))
        } else {
            request
        }
    }

    // ========================================================================
    // Organizations
    // ========================================================================

    /// Fetch organization by SIORG code
    pub async fn get_organization(&self, siorg_code: i32) -> Result<Option<SiorgOrganization>> {
        let url = format!("{}/api/v1/organizacoes/{}", self.base_url, siorg_code);

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to fetch organization from SIORG")?;

        if response.status().is_success() {
            let org = response
                .json::<SiorgOrganization>()
                .await
                .context("Failed to parse SIORG organization response")?;
            Ok(Some(org))
        } else if response.status() == 404 {
            Ok(None)
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    /// List all organizations with pagination
    pub async fn list_organizations(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<SiorgApiResponse<SiorgOrganization>> {
        let url = format!(
            "{}/api/v1/organizacoes?page={}&page_size={}",
            self.base_url, page, page_size
        );

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to list organizations from SIORG")?;

        if response.status().is_success() {
            response
                .json::<SiorgApiResponse<SiorgOrganization>>()
                .await
                .context("Failed to parse SIORG organizations list")
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    // ========================================================================
    // Organizational Units
    // ========================================================================

    /// Fetch organizational unit by SIORG code
    pub async fn get_unit(&self, siorg_code: i32) -> Result<Option<SiorgUnit>> {
        let url = format!("{}/api/v1/unidades/{}", self.base_url, siorg_code);

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to fetch unit from SIORG")?;

        if response.status().is_success() {
            let unit = response
                .json::<SiorgUnit>()
                .await
                .context("Failed to parse SIORG unit response")?;
            Ok(Some(unit))
        } else if response.status() == 404 {
            Ok(None)
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    /// List units for a specific organization
    pub async fn list_units_by_organization(
        &self,
        org_siorg_code: i32,
        page: i32,
        page_size: i32,
    ) -> Result<SiorgApiResponse<SiorgUnit>> {
        let url = format!(
            "{}/api/v1/organizacoes/{}/unidades?page={}&page_size={}",
            self.base_url, org_siorg_code, page, page_size
        );

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to list units from SIORG")?;

        if response.status().is_success() {
            response
                .json::<SiorgApiResponse<SiorgUnit>>()
                .await
                .context("Failed to parse SIORG units list")
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    /// Fetch unit hierarchy (parent and children)
    pub async fn get_unit_hierarchy(&self, siorg_code: i32) -> Result<Vec<SiorgUnit>> {
        let url = format!("{}/api/v1/unidades/{}/hierarquia", self.base_url, siorg_code);

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to fetch unit hierarchy from SIORG")?;

        if response.status().is_success() {
            response
                .json::<Vec<SiorgUnit>>()
                .await
                .context("Failed to parse SIORG unit hierarchy")
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    // ========================================================================
    // Unit Types
    // ========================================================================

    /// List all unit types
    pub async fn list_unit_types(&self) -> Result<Vec<SiorgUnitType>> {
        let url = format!("{}/api/v1/tipos-unidade", self.base_url);

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to list unit types from SIORG")?;

        if response.status().is_success() {
            response
                .json::<Vec<SiorgUnitType>>()
                .await
                .context("Failed to parse SIORG unit types")
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    // ========================================================================
    // Categories
    // ========================================================================

    /// List all unit categories
    pub async fn list_categories(&self) -> Result<Vec<SiorgCategory>> {
        let url = format!("{}/api/v1/categorias", self.base_url);

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to list categories from SIORG")?;

        if response.status().is_success() {
            response
                .json::<Vec<SiorgCategory>>()
                .await
                .context("Failed to parse SIORG categories")
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    // ========================================================================
    // Changes / Sync
    // ========================================================================

    /// Fetch recent changes since a specific timestamp
    pub async fn get_changes_since(
        &self,
        since_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<SiorgChangeEvent>> {
        let url = format!(
            "{}/api/v1/mudancas?desde={}",
            self.base_url,
            since_timestamp.to_rfc3339()
        );

        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request
            .send()
            .await
            .context("Failed to fetch changes from SIORG")?;

        if response.status().is_success() {
            response
                .json::<Vec<SiorgChangeEvent>>()
                .await
                .context("Failed to parse SIORG changes")
        } else {
            anyhow::bail!("SIORG API error: {}", response.status())
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .context("Failed to check SIORG health")?;

        Ok(response.status().is_success())
    }
}

// ============================================================================
// Change Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiorgChangeEvent {
    pub tipo_entidade: String, // "ORGANIZATION", "UNIT", "CATEGORY", "TYPE"
    pub codigo_siorg: i32,
    pub tipo_mudanca: String, // "CREATION", "UPDATE", "EXTINCTION", etc.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dados: serde_json::Value,
}

// ============================================================================
// Mock Implementation for Testing
// ============================================================================

#[cfg(test)]
pub struct MockSiorgClient;

#[cfg(test)]
impl MockSiorgClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_organization(&self, _siorg_code: i32) -> Result<Option<SiorgOrganization>> {
        Ok(Some(SiorgOrganization {
            codigo_siorg: 123456,
            sigla: "UFMT".to_string(),
            nome: "Universidade Federal de Mato Grosso".to_string(),
            cnpj: Some("03000000000191".to_string()),
            codigo_ug: Some(154048),
            ativo: true,
        }))
    }

    pub async fn get_unit(&self, _siorg_code: i32) -> Result<Option<SiorgUnit>> {
        Ok(Some(SiorgUnit {
            codigo_siorg: 789012,
            codigo_unidade_pai: None,
            nome: "Reitoria".to_string(),
            tipo_unidade: "ADMINISTRATION".to_string(),
            area_atuacao: "SUPPORT".to_string(),
            ativo: true,
            nivel_hierarquico: Some(1),
        }))
    }

    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
