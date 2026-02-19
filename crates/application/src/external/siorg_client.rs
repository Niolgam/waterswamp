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
    /// Extrai o código numérico de um campo que pode ser:
    /// - Um inteiro simples: `"471"`
    /// - Uma URI: `"https://estruturaorganizacional.dados.gov.br/id/unidade-organizacional/471"`
    fn extract_id(s: &str) -> Option<i32> {
        s.rsplit('/').next().and_then(|p| p.parse().ok())
    }

    /// Converte `codigo_unidade` para i32, suportando tanto inteiros quanto URIs.
    pub fn siorg_code(&self) -> Option<i32> {
        Self::extract_id(&self.codigo_unidade)
    }

    /// Converte `codigo_unidade_pai` para i32, suportando tanto inteiros quanto URIs.
    pub fn parent_siorg_code(&self) -> Option<i32> {
        self.codigo_unidade_pai.as_ref().and_then(|s| Self::extract_id(s))
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
// Versioning Types (Histórico de Estrutura Organizacional)
// ============================================================================

/// Informações de versão retornadas por /unidade-organizacional/{code}/versao
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgTipoVersao {
    pub versao_anterior: Option<String>,
    pub versao_posterior: Option<String>,
    /// Versão atual da unidade organizacional.
    pub versao_consulta: String,
    pub data_versao_anterior: Option<String>,
    pub data_versao_posterior: Option<String>,
    pub data_versao_consulta: Option<String>,
}

/// Resposta de /unidade-organizacional/{code}/versao
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgVersaoResponse {
    pub servico: SiorgServico,
    pub tipo_versao: SiorgTipoVersao,
}

/// Unidade alterada retornada por /unidade-organizacional/{code}/alteradas.
/// Estende `SiorgUnidade` com campos de rastreamento de hierarquia anterior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgUnidadeAlterada {
    #[serde(flatten)]
    pub base: SiorgUnidade,
    /// URI da unidade-pai anterior (preenchida quando a hierarquia mudou).
    pub codigo_unidade_pai_anterior: Option<String>,
    pub codigo_orgao_entidade_anterior: Option<String>,
}

impl SiorgUnidadeAlterada {
    pub fn siorg_code(&self) -> Option<i32> {
        self.base.siorg_code()
    }
}

impl From<SiorgUnidadeAlterada> for SiorgUnidadeCompleta {
    /// Converte dados resumidos de alteração em `SiorgUnidadeCompleta` com campos extras como None.
    /// Suficiente para upserts incrementais; campos como `area_atuacao` serão atualizados
    /// no próximo sync completo.
    fn from(a: SiorgUnidadeAlterada) -> Self {
        SiorgUnidadeCompleta {
            base: a.base,
            codigo_categoria_unidade: None,
            area_atuacao: None,
            competencia: None,
            missao: None,
            contato: None,
            endereco: None,
        }
    }
}

/// Resposta de /unidade-organizacional/{code}/alteradas
#[derive(Debug, Deserialize)]
pub struct SiorgAlteradasResponse {
    pub servico: SiorgServico,
    pub unidades: Vec<SiorgUnidadeAlterada>,
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
    // Histórico de Versões
    // ========================================================================

    /// Consulta a versão atual de uma unidade organizacional (órgão/entidade).
    ///
    /// Endpoint: `GET /unidade-organizacional/{codigo}/versao`
    ///
    /// Usado para verificar se há mudanças desde o último sync sem precisar
    /// baixar toda a estrutura. Se `versao_consulta` local == API, não há nada a fazer.
    pub async fn get_versao(&self, org_code: i32) -> Result<SiorgTipoVersao> {
        let url = format!("{}/unidade-organizacional/{}/versao", self.base_url, org_code);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch SIORG version")?;

        if !response.status().is_success() {
            anyhow::bail!("SIORG API error: {}", response.status());
        }

        let parsed = response
            .json::<SiorgVersaoResponse>()
            .await
            .context("Failed to parse SIORG version response")?;

        Ok(parsed.tipo_versao)
    }

    /// Retorna unidades alteradas de um órgão desde uma versão específica.
    ///
    /// Endpoint: `GET /unidade-organizacional/{codigo}/alteradas?versaoConsulta={versao}`
    ///
    /// Mais eficiente que `get_estrutura_completa` para syncs recorrentes:
    /// retorna apenas unidades criadas, alteradas ou extintas desde `from_versao`.
    /// O campo `operacao` indica o tipo de mudança ("INCLUSAO", "ALTERACAO", "EXCLUSAO").
    pub async fn get_alteradas(
        &self,
        org_code: i32,
        from_versao: &str,
    ) -> Result<Vec<SiorgUnidadeAlterada>> {
        let url = format!(
            "{}/unidade-organizacional/{}/alteradas?versaoConsulta={}",
            self.base_url, org_code, from_versao
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch SIORG changed units")?;

        if !response.status().is_success() {
            anyhow::bail!("SIORG API error: {}", response.status());
        }

        let parsed = response
            .json::<SiorgAlteradasResponse>()
            .await
            .context("Failed to parse SIORG changed units response")?;

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
