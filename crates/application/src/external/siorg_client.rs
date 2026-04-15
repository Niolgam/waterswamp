use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Serde helpers — a API SIORG retorna campos de código ora como string JSON,
// ora como número inteiro JSON. Estes helpers aceitam ambos.
// ---------------------------------------------------------------------------

/// Deserializa `Option<Vec<String>>` onde cada elemento pode ser string ou inteiro.
/// Usado para `telefone` e `email` em `SiorgContato` que às vezes vêm como inteiros.
fn deserialize_opt_vec_strings<'de, D>(
    d: D,
) -> std::result::Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{SeqAccess, Visitor};

    struct OptVec;
    impl<'de> Visitor<'de> for OptVec {
        type Value = Option<Vec<String>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null or array of strings/integers")
        }
        fn visit_none<E: serde::de::Error>(self) -> std::result::Result<Option<Vec<String>>, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Option<Vec<String>>, E> {
            Ok(None)
        }
        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            d: D2,
        ) -> std::result::Result<Option<Vec<String>>, D2::Error> {
            struct Seq;
            impl<'de> Visitor<'de> for Seq {
                type Value = Vec<String>;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("array of strings or integers")
                }
                fn visit_seq<A: SeqAccess<'de>>(
                    self,
                    mut seq: A,
                ) -> std::result::Result<Vec<String>, A::Error> {
                    let mut items = Vec::new();
                    while let Some(val) =
                        seq.next_element::<serde_json::Value>()?
                    {
                        let s = match val {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Null => continue,
                            other => other.to_string(),
                        };
                        items.push(s);
                    }
                    Ok(items)
                }
            }
            d.deserialize_seq(Seq).map(Some)
        }
    }
    d.deserialize_option(OptVec)
}


fn deserialize_string_or_int<'de, D>(d: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("string or integer")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<String, E> {
            Ok(v.to_owned())
        }
        fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<String, E> {
            Ok(v)
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<String, E> {
            Ok(v.to_string())
        }
    }
    d.deserialize_any(V)
}

fn deserialize_opt_string_or_int<'de, D>(
    d: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Option<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null, string, or integer")
        }
        fn visit_none<E: de::Error>(self) -> std::result::Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_unit<E: de::Error>(self) -> std::result::Result<Option<String>, E> {
            Ok(None)
        }
        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            d: D2,
        ) -> std::result::Result<Option<String>, D2::Error> {
            struct Inner;
            impl<'de> Visitor<'de> for Inner {
                type Value = String;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("string or integer")
                }
                fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<String, E> {
                    Ok(v.to_owned())
                }
                fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<String, E> {
                    Ok(v)
                }
                fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<String, E> {
                    Ok(v.to_string())
                }
                fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<String, E> {
                    Ok(v.to_string())
                }
            }
            d.deserialize_any(Inner).map(Some)
        }
    }
    d.deserialize_option(V)
}

// ---------------------------------------------------------------------------
// SSRF protection
// ---------------------------------------------------------------------------

const SIORG_ALLOWED_HOSTS: &[&str] = &[
    "estruturaorganizacional.dados.gov.br",
    "api.siorg.gov.br",
    "api.siorg.economia.gov.br",
];

fn ssrf_validate(url: &str, allowed_hosts: &[&str]) -> Result<()> {
    let host = extract_host(url)
        .ok_or_else(|| anyhow::anyhow!("SSRF: could not parse host from URL '{}'", url))?;

    if is_private_host(host) {
        anyhow::bail!("SSRF: URL '{}' resolves to a private/loopback address", url);
    }

    let allowed = allowed_hosts
        .iter()
        .any(|&a| host == a || host.ends_with(&format!(".{}", a)));

    if !allowed {
        anyhow::bail!(
            "SSRF: host '{}' is not in the allowed list {:?}",
            host,
            allowed_hosts
        );
    }
    Ok(())
}

fn extract_host(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let end = rest
        .find(|c| c == '/' || c == ':' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    let host = &rest[..end];
    if host.is_empty() { None } else { Some(host) }
}

fn is_private_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "0.0.0.0")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("169.254.")
        || host.starts_with("0.")
        || is_172_private(host)
}

fn is_172_private(host: &str) -> bool {
    if let Some(rest) = host.strip_prefix("172.") {
        if let Some(second) = rest.split('.').next() {
            if let Ok(n) = second.parse::<u8>() {
                return (16..=31).contains(&n);
            }
        }
    }
    false
}

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
    /// Pode ser retornado como string `"471"` ou inteiro `471` dependendo do endpoint.
    #[serde(deserialize_with = "deserialize_string_or_int")]
    pub codigo_unidade: String,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub codigo_unidade_pai: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub codigo_orgao_entidade: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub codigo_tipo_unidade: Option<String>,
    pub nome: String,
    pub sigla: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub codigo_esfera: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
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
    #[serde(default, deserialize_with = "deserialize_opt_vec_strings")]
    pub telefone: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_opt_vec_strings")]
    pub email: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgEndereco {
    pub logradouro: Option<String>,
    /// Número do endereço — pode vir como inteiro (ex: 2367) ou string.
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub numero: Option<String>,
    pub complemento: Option<String>,
    pub bairro: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub cep: Option<String>,
    /// Código IBGE da UF — pode vir como inteiro (ex: 51) ou sigla (ex: "MT").
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub uf: Option<String>,
    /// Código IBGE do município — pode vir como inteiro (ex: 5103403) ou nome.
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub municipio: Option<String>,
    /// Código IBGE / ISO do país — pode vir como inteiro ou nome.
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
    pub pais: Option<String>,
    pub tipo_endereco: Option<String>,
}

/// Unidade organizacional com dados completos (resposta de /completa).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiorgUnidadeCompleta {
    #[serde(flatten)]
    pub base: SiorgUnidade,
    #[serde(default, deserialize_with = "deserialize_opt_string_or_int")]
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

/// Extrai unidades de um `serde_json::Value` já parseado.
///
/// Suporta:
/// - campo `"unidade"` (singular) com valor objeto `{}` ou array `[…]`
/// - campo `"unidades"` (plural) com valor objeto `{}` ou array `[…]`
///
/// Isso contorna a limitação do serde onde `#[serde(alias)] + #[serde(deserialize_with)]`
/// não chama o deserializador customizado quando o campo é encontrado via alias.
fn extract_units_from_value(
    v: &serde_json::Value,
    body_preview: &str,
) -> Result<Vec<SiorgUnidadeCompleta>> {
    let units_val = v
        .get("unidade")
        .or_else(|| v.get("unidades"))
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));

    let units_array: Vec<serde_json::Value> = match units_val {
        serde_json::Value::Array(arr) => arr,
        obj @ serde_json::Value::Object(_) => vec![obj],
        serde_json::Value::Null => vec![],
        other => {
            anyhow::bail!(
                "Expected object or array for units field, got {:?}. Body: {}",
                other,
                body_preview
            )
        }
    };

    let mut units = Vec::with_capacity(units_array.len());
    for (i, item) in units_array.into_iter().enumerate() {
        let unit: SiorgUnidadeCompleta = serde_json::from_value(item).with_context(|| {
            format!("Failed to parse unit at index {}. Body: {}", i, body_preview)
        })?;
        units.push(unit);
    }
    Ok(units)
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
        ssrf_validate(&base_url, SIORG_ALLOWED_HOSTS)
            .context("SSRF validation failed for SIORG_API_URL")?;

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

        let body = response
            .text()
            .await
            .context("Failed to read SIORG unit response body")?;

        let preview: String = body.chars().take(2000).collect();

        let v: serde_json::Value = serde_json::from_str(&body).with_context(|| {
            format!("Failed to parse SIORG unit response as JSON. Body: {}", preview)
        })?;

        let units = extract_units_from_value(&v, &preview)?;

        // Usa siorg_code() para suportar tanto "471" quanto a URI completa
        // "https://estruturaorganizacional.dados.gov.br/id/unidade-organizacional/471"
        let unit = units.into_iter().find(|u| u.siorg_code() == Some(codigo));

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

        let body = response
            .text()
            .await
            .context("Failed to read SIORG structure response body")?;

        let preview: String = body.chars().take(2000).collect();

        let v: serde_json::Value = serde_json::from_str(&body).with_context(|| {
            format!(
                "Failed to parse SIORG organizational structure response as JSON. Body: {}",
                preview
            )
        })?;

        let units = extract_units_from_value(&v, &preview)?;

        Ok(units)
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

        let body = response
            .text()
            .await
            .context("Failed to read SIORG changes response body")?;

        let parsed = serde_json::from_str::<SiorgAlteracoesResponse>(&body)
            .with_context(|| {
                format!(
                    "Failed to parse SIORG changes response. Body (first 500 chars): {}",
                    body.chars().take(2000).collect::<String>()
                )
            })?;

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

        let body = response
            .text()
            .await
            .context("Failed to read SIORG version response body")?;

        let parsed = serde_json::from_str::<SiorgVersaoResponse>(&body)
            .with_context(|| {
                format!(
                    "Failed to parse SIORG version response. Body (first 500 chars): {}",
                    body.chars().take(2000).collect::<String>()
                )
            })?;

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

        let body = response
            .text()
            .await
            .context("Failed to read SIORG changed units response body")?;

        let parsed = serde_json::from_str::<SiorgAlteradasResponse>(&body)
            .with_context(|| {
                format!(
                    "Failed to parse SIORG changed units response. Body (first 500 chars): {}",
                    body.chars().take(2000).collect::<String>()
                )
            })?;

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
