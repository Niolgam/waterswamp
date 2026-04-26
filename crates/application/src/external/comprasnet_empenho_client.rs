use anyhow::{Context, Result};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::time::Duration;

const COMPRASNET_ALLOWED_HOSTS: &[&str] = &[
    "api.compras.gov.br",
    "compras.gov.br",
    "www.compras.gov.br",
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
    let end = rest.find(['/', ':', '?', '#']).unwrap_or(rest.len());
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

/// Result of an empenho (commitment) balance validation against Comprasnet (RF-030).
#[derive(Debug, Clone)]
pub struct EmpenhoValidationResult {
    pub commitment_number: String,
    /// Total committed amount in BRL
    pub committed_amount: Decimal,
    /// Amount already consumed by previous invoices
    pub consumed_amount: Decimal,
    /// Balance remaining = committed_amount - consumed_amount
    pub available_balance: Decimal,
    /// Whether the requested amount fits within the available balance
    pub is_sufficient: bool,
}

/// Raw API response shape — Comprasnet empenho endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComprasnetEmpenhoResponse {
    numero_empenho: String,
    valor_empenho: Decimal,
    valor_empenhado_consumido: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct ComprasnetEmpenhoClient {
    client: Client,
    base_url: String,
    api_token: Option<String>,
}

impl ComprasnetEmpenhoClient {
    pub fn new(base_url: String, api_token: Option<String>) -> Result<Self> {
        ssrf_validate(&base_url, COMPRASNET_ALLOWED_HOSTS)
            .context("SSRF validation failed for Comprasnet empenho base URL")?;

        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .context("Failed to build Comprasnet HTTP client")?;

        Ok(Self { client, base_url, api_token })
    }

    pub fn update_config(&mut self, base_url: String, api_token: Option<String>) -> Result<()> {
        ssrf_validate(&base_url, COMPRASNET_ALLOWED_HOSTS)
            .context("SSRF validation failed for updated Comprasnet base URL")?;
        self.base_url = base_url;
        self.api_token = api_token;
        Ok(())
    }

    /// Validates an empenho (commitment) against the Comprasnet API.
    /// Returns `EmpenhoValidationResult` with balance details.
    pub async fn validate_empenho(
        &self,
        commitment_number: &str,
        requested_amount: Decimal,
    ) -> Result<EmpenhoValidationResult> {
        let url = format!(
            "{}/contratacao/v1/empenhos/{}",
            self.base_url.trim_end_matches('/'),
            commitment_number
        );

        let mut req = self.client.get(&url);
        if let Some(ref token) = self.api_token {
            req = req.bearer_auth(token);
        }

        let resp = req
            .send()
            .await
            .context("Failed to reach Comprasnet empenho API")?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Empenho '{}' não encontrado na base Comprasnet", commitment_number);
        }

        if !resp.status().is_success() {
            anyhow::bail!(
                "API Comprasnet retornou erro {} ao consultar empenho '{}'",
                resp.status(),
                commitment_number
            );
        }

        let data: ComprasnetEmpenhoResponse = resp
            .json()
            .await
            .context("Failed to parse Comprasnet empenho response")?;

        let committed = data.valor_empenho;
        let consumed = data.valor_empenhado_consumido.unwrap_or(Decimal::ZERO);
        let available = committed - consumed;

        Ok(EmpenhoValidationResult {
            commitment_number: data.numero_empenho,
            committed_amount: committed,
            consumed_amount: consumed,
            available_balance: available,
            is_sufficient: requested_amount <= available,
        })
    }
}
