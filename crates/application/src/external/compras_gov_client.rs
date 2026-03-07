use anyhow::{Context, Result};
use domain::models::catalog::{
    ComprasGovClasseMaterial, ComprasGovClasseServico, ComprasGovDivisionService,
    ComprasGovGrupoMaterial, ComprasGovGrupoServico, ComprasGovItemMaterial,
    ComprasGovItemServico, ComprasGovPdmMaterial, ComprasGovResponse, ComprasGovSectionService,
};
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ComprasGovClient {
    client: Client,
    catmat_base_url: String,
    catser_base_url: String,
}

impl ComprasGovClient {
    pub fn new(catmat_base_url: String, catser_base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build ComprasGov HTTP client")?;

        Ok(Self {
            client,
            catmat_base_url,
            catser_base_url,
        })
    }

    pub fn update_urls(&mut self, catmat_base_url: String, catser_base_url: String) {
        self.catmat_base_url = catmat_base_url;
        self.catser_base_url = catser_base_url;
    }

    // ========================================================================
    // CATMAT - Material endpoints
    // ========================================================================

    pub async fn search_grupos_material(
        &self,
        codigo_grupo: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovGrupoMaterial>> {
        let mut url = format!("{}/1_consultarGrupoMaterial", self.catmat_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_grupo {
            params.push(format!("codigoGrupo={}", c));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "grupos material").await
    }

    pub async fn search_classes_material(
        &self,
        codigo_classe: Option<i64>,
        codigo_grupo: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovClasseMaterial>> {
        let mut url = format!("{}/2_consultarClasseMaterial", self.catmat_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_classe {
            params.push(format!("codigoClasse={}", c));
        }
        if let Some(g) = codigo_grupo {
            params.push(format!("codigoGrupo={}", g));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "classes material").await
    }

    pub async fn search_pdms_material(
        &self,
        codigo_pdm: Option<i64>,
        codigo_classe: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovPdmMaterial>> {
        let mut url = format!("{}/3_consultarPDMMaterial", self.catmat_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_pdm {
            params.push(format!("codigoPdm={}", c));
        }
        if let Some(cl) = codigo_classe {
            params.push(format!("codigoClasse={}", cl));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "PDMs material").await
    }

    pub async fn search_itens_material(
        &self,
        codigo_item: Option<i64>,
        codigo_pdm: Option<i64>,
        codigo_classe: Option<i64>,
        codigo_grupo: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovItemMaterial>> {
        let mut url = format!("{}/4_consultarItemMaterial", self.catmat_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_item {
            params.push(format!("codigoItem={}", c));
        }
        if let Some(p) = codigo_pdm {
            params.push(format!("codigoPdm={}", p));
        }
        if let Some(cl) = codigo_classe {
            params.push(format!("codigoClasse={}", cl));
        }
        if let Some(g) = codigo_grupo {
            params.push(format!("codigoGrupo={}", g));
        }
        if let Some(pg) = pagina {
            params.push(format!("pagina={}", pg));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "itens material").await
    }

    // ========================================================================
    // CATSER - Service endpoints
    // ========================================================================

    pub async fn search_sections_service(
        &self,
        section_code: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovSectionService>> {
        let mut url = format!("{}/1_consultarSecaoServico", self.catser_base_url);
        let mut params = vec![];
        if let Some(c) = section_code {
            params.push(format!("codigoSecao={}", c));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "seções serviço").await
    }

    pub async fn search_divisions_service(
        &self,
        division_code: Option<i64>,
        section_code: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovDivisionService>> {
        let mut url = format!("{}/2_consultarDivisaoServico", self.catser_base_url);
        let mut params = vec![];
        if let Some(c) = division_code {
            params.push(format!("codigoDivisao={}", c));
        }
        if let Some(s) = section_code {
            params.push(format!("codigoSecao={}", s));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "divisões serviço").await
    }

    pub async fn search_grupos_servico(
        &self,
        codigo_grupo: Option<i64>,
        division_code: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovGrupoServico>> {
        let mut url = format!("{}/3_consultarGrupoServico", self.catser_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_grupo {
            params.push(format!("codigoGrupo={}", c));
        }
        if let Some(d) = division_code {
            params.push(format!("codigoDivisao={}", d));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "grupos serviço").await
    }

    pub async fn search_classes_servico(
        &self,
        codigo_classe: Option<i64>,
        codigo_grupo: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovClasseServico>> {
        let mut url = format!("{}/4_consultarClasseServico", self.catser_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_classe {
            params.push(format!("codigoClasse={}", c));
        }
        if let Some(g) = codigo_grupo {
            params.push(format!("codigoGrupo={}", g));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "classes serviço").await
    }

    pub async fn search_itens_servico(
        &self,
        codigo_item: Option<i64>,
        codigo_classe: Option<i64>,
        codigo_grupo: Option<i64>,
        pagina: Option<i64>,
    ) -> Result<ComprasGovResponse<ComprasGovItemServico>> {
        let mut url = format!("{}/5_consultarItemServico", self.catser_base_url);
        let mut params = vec![];
        if let Some(c) = codigo_item {
            params.push(format!("codigoItem={}", c));
        }
        if let Some(cl) = codigo_classe {
            params.push(format!("codigoClasse={}", cl));
        }
        if let Some(g) = codigo_grupo {
            params.push(format!("codigoGrupo={}", g));
        }
        if let Some(p) = pagina {
            params.push(format!("pagina={}", p));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get_json(&url, "itens serviço").await
    }

    // ========================================================================
    // Health Check
    // ========================================================================

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/1_consultarGrupoMaterial?pagina=1", self.catmat_base_url);

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .context("Failed to check ComprasGov health")?;

        Ok(response.status().is_success())
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        context: &str,
    ) -> Result<T> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch {} from ComprasGov", context))?;

        if !response.status().is_success() {
            anyhow::bail!("ComprasGov API error for {}: {}", context, response.status());
        }

        response
            .json::<T>()
            .await
            .with_context(|| format!("Failed to parse ComprasGov {} response", context))
    }
}
