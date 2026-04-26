use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/legacy-import";

    // Importação de legados: somente ROLE_ADMIN pode executar e consultar
    //
    // POST /legacy-import/suppliers       — importa fornecedores (JSON)
    // POST /legacy-import/catalog-items   — importa itens de catálogo (JSON)
    // POST /legacy-import/initial-stock   — importa saldos iniciais (JSON)
    // GET  /legacy-import/jobs            — lista jobs de importação
    // GET  /legacy-import/jobs/:id        — detalhe de um job
    for (path, method) in &[
        (format!("{}/suppliers", base), ACTION_POST),
        (format!("{}/catalog-items", base), ACTION_POST),
        (format!("{}/initial-stock", base), ACTION_POST),
        (format!("{}/jobs", base), ACTION_GET),
        (format!("{}/jobs/{{id}}", base), ACTION_GET),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_ADMIN, path, method])
            .await?;
    }

    tracing::info!("Políticas de Importação de Legados carregadas");
    Ok(())
}
