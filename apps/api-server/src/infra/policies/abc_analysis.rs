use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/abc-analysis";

    // POST /abc-analysis/run         — executa análise ABC (somente admin)
    // GET  /abc-analysis/results     — resultados da última análise
    // GET  /abc-analysis/latest-run  — timestamp da última execução
    for (path, method) in &[
        (format!("{}/run", base), ACTION_POST),
        (format!("{}/results", base), ACTION_GET),
        (format!("{}/latest-run", base), ACTION_GET),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_ADMIN, path, method])
            .await?;
    }

    // ROLE_USER pode consultar resultados (somente leitura)
    for path in &[
        format!("{}/results", base),
        format!("{}/latest-run", base),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_USER, path, ACTION_GET])
            .await?;
    }

    tracing::info!("Políticas de Curva ABC carregadas");
    Ok(())
}
