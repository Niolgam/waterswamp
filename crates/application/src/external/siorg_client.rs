use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// SIORG API Response Types
// ============================================================================

/// Envelope de metadados presente em todas as respostas da API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgServico {
    pub codigo_erro: Option<i32>,
    pub mensagem: Option<String>,
}

/// Campos base de uma unidade organizacional (presentes na versão resumida e completa).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgUnidade {
    pub codigo_unidade: String,
    pub codigo_unidade_pai: Option<String>,
    pub codigo_orgao_entidade: Option<String>,
    pub codigo_tipo_unidade: Option<String>,
    pub nome: String,
    pub sigla: Option<String>,
    pub codigo_esfera: Option<String>,
    pub codigo_poder: Option<String>,
    pub nivel_normatizacao: Option<String>,
    pub versao_consulta: Option<String>,
    /// Presente em /alteracoes: "INCLUSAO", "ALTERACAO", "EXCLUSAO", etc.
    pub operacao: Option<String>,
}

impl SiorgUnidade {
    /// Converte `codigo_unidade` (String na API) para i32.
    pub fn siorg_code(&self) -> Option<i32> {
        self.codigo_unidade.parse().ok()
    }

    /// Converte `codigo_unidade_pai` (String na API) para i32.
    pub fn parent_siorg_code(&self) -> Option<i32> {
        self.codigo_unidade_pai.as_ref().and_then(|s| s.parse().ok())
    }

    /// Retorna true se a operação indicar extinção/remoção da unidade.
    pub fn is_exclusao(&self) -> bool {
        self.operacao
            .as_deref()
            .map(|op| op.eq_ignore_ascii_case("EXCLUSAO") || op.eq_ignore_ascii_case("EXTINCAO"))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgContato {
    pub telefone: Option<Vec<String>>,
    pub email: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgEndereco {
    pub logradouro: Option<String>,
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: Option<String>,
    pub cep: Option<String>,
    pub uf: Option<String>,
    pub municipio: Option<String>,
    pub pais: Option<String>,
    pub tipo_endereco: Option<String>,
}

/// Unidade organizacional com dados completos (resposta de /completa).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgUnidadeCompleta {
    #[serde(flatten)]
    pub base: SiorgUnidade,
    pub codigo_categoria_unidade: Option<String>,
    pub area_atuacao: Option<String>,
    pub competencia: Option<String>,
    pub missao: Option<String>,
    pub contato: Option<Vec<SiorgContato>>,
    pub endereco: Option<Vec<SiorgEndereco>>,
}

impl SiorgUnidadeCompleta {
    pub fn siorg_code(&self) -> Option<i32> {
        self.base.siorg_code()
    }

    pub fn parent_siorg_code(&self) -> Option<i32> {
        self.base.parent_siorg_code()
    }
}

// Wrappers de resposta da API

/// Resposta de /unidade-organizacional/{cod}/completa e /estrutura-organizacional/completa
#[derive(Debug, Deserialize)]
pub struct SiorgEstruturaCompletaResponse {
    pub servico: SiorgServico,
    /// A API usa a chave "unidade" (singular) mesmo retornando um array.
    pub unidade: Vec<SiorgUnidadeCompleta>,
}

/// Resposta de /estrutura-organizacional/alteracoes
#[derive(Debug, Deserialize)]
pub struct SiorgAlteracoesResponse {
    pub servico: SiorgServico,
    /// A API usa a chave "unidades" (plural) no endpoint de alterações.
    pub unidades: Vec<SiorgUnidade>,
}

// ============================================================================
// SIORG Client
// ============================================================================

#[derive(Debug, Clone)]
pub struct SiorgClient {
    client: Client,
    base_url: String,
}

impl SiorgClient {
    /// Cria um novo cliente da API SIORG.
    /// `_api_token` é aceito por compatibilidade, mas a API é pública e não exige autenticação.
    pub fn new(base_url: String, _api_token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, base_url })
    }

    // ========================================================================
    // Unidade Organizacional
    // ========================================================================

    /// Busca uma unidade organizacional e toda a hierarquia abaixo dela (dados completos).
    ///
    /// Endpoint: `GET /unidade-organizacional/{codigo}/completa`
    ///
    /// A resposta contém a própria unidade solicitada como primeiro elemento (ou identificada
    /// pelo `codigoUnidade`) seguida de todas as unidades-filha na hierarquia.
    /// Retorna `None` se a unidade não existir (HTTP 404).
    pub async fn get_unit_complete(
        &self,
        codigo: i32,
    ) -> Result<Option<SiorgUnidadeCompleta>> {
        let url = format!("{}/unidade-organizacional/{}/completa", self.base_url, codigo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch unit from SIORG")?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            anyhow::bail!("SIORG API error: {}", response.status());
        }

        let parsed = response
            .json::<SiorgEstruturaCompletaResponse>()
            .await
            .context("Failed to parse SIORG unit response")?;

        let codigo_str = codigo.to_string();
        let unit = parsed
            .unidade
            .into_iter()
            .find(|u| u.base.codigo_unidade == codigo_str);

        Ok(unit)
    }

    // ========================================================================
    // Estrutura Organizacional
    // ========================================================================

    /// Retorna todos os dados completos de todas as unidades de um órgão/entidade.
    ///
    /// Endpoint: `GET /estrutura-organizacional/completa?codigoUnidade={codigo}`
    ///
    /// Uma única chamada retorna toda a estrutura hierárquica, eliminando a necessidade
    /// de paginação e de chamadas individuais por unidade.
    pub async fn get_estrutura_completa(
        &self,
        codigo_unidade: i32,
    ) -> Result<Vec<SiorgUnidadeCompleta>> {
        let url = format!(
            "{}/estrutura-organizacional/completa?codigoUnidade={}",
            self.base_url, codigo_unidade
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch organizational structure from SIORG")?;

        if response.status() == 404 {
            return Ok(vec![]);
        }

        if !response.status().is_success() {
            anyhow::bail!("SIORG API error: {}", response.status());
        }

        let parsed = response
            .json::<SiorgEstruturaCompletaResponse>()
            .await
            .context("Failed to parse SIORG organizational structure response")?;

        Ok(parsed.unidade)
    }

    // ========================================================================
    // Alterações (Sync Incremental)
    // ========================================================================

    /// Retorna unidades incluídas, alteradas e excluídas a partir de uma versão de referência.
    ///
    /// Endpoint: `GET /estrutura-organizacional/alteracoes?versaoReferencia={versao}`
    ///
    /// O campo `operacao` em cada unidade indica o tipo de mudança
    /// ("INCLUSAO", "ALTERACAO", "EXCLUSAO", etc.).
    pub async fn get_alteracoes(&self, versao_referencia: &str) -> Result<Vec<SiorgUnidade>> {
        let url = format!(
            "{}/estrutura-organizacional/alteracoes?versaoReferencia={}",
            self.base_url, versao_referencia
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch SIORG changes")?;

        if !response.status().is_success() {
            anyhow::bail!("SIORG API error: {}", response.status());
        }

        let parsed = response
            .json::<SiorgAlteracoesResponse>()
            .await
            .context("Failed to parse SIORG changes response")?;

        Ok(parsed.unidades)
    }

    // ========================================================================
    // Health Check
    // ========================================================================

    /// Verifica disponibilidade da API SIORG.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/tipo-unidade", self.base_url);

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
// Mock Implementation for Testing
// ============================================================================

#[cfg(test)]
pub struct MockSiorgClient;

#[cfg(test)]
impl MockSiorgClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_unit_complete(
        &self,
        codigo: i32,
    ) -> Result<Option<SiorgUnidadeCompleta>> {
        Ok(Some(SiorgUnidadeCompleta {
            base: SiorgUnidade {
                codigo_unidade: codigo.to_string(),
                codigo_unidade_pai: None,
                codigo_orgao_entidade: Some(codigo.to_string()),
                codigo_tipo_unidade: Some("1".to_string()),
                nome: "Universidade Federal de Mato Grosso".to_string(),
                sigla: Some("UFMT".to_string()),
                codigo_esfera: Some("1".to_string()),
                codigo_poder: Some("1".to_string()),
                nivel_normatizacao: None,
                versao_consulta: None,
                operacao: None,
            },
            codigo_categoria_unidade: None,
            area_atuacao: Some("FIM".to_string()),
            competencia: None,
            missao: None,
            contato: None,
            endereco: None,
        }))
    }

    pub async fn get_estrutura_completa(
        &self,
        codigo_unidade: i32,
    ) -> Result<Vec<SiorgUnidadeCompleta>> {
        Ok(vec![SiorgUnidadeCompleta {
            base: SiorgUnidade {
                codigo_unidade: "789012".to_string(),
                codigo_unidade_pai: Some(codigo_unidade.to_string()),
                codigo_orgao_entidade: Some(codigo_unidade.to_string()),
                codigo_tipo_unidade: Some("2".to_string()),
                nome: "Reitoria".to_string(),
                sigla: Some("REITORIA".to_string()),
                codigo_esfera: Some("1".to_string()),
                codigo_poder: Some("1".to_string()),
                nivel_normatizacao: None,
                versao_consulta: None,
                operacao: None,
            },
            codigo_categoria_unidade: None,
            area_atuacao: Some("MEIO".to_string()),
            competencia: None,
            missao: None,
            contato: None,
            endereco: None,
        }])
    }

    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
