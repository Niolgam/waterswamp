use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/dashboard";

    // GET  /dashboard/stock-summary
    // GET  /dashboard/daily-movements
    // GET  /dashboard/supplier-performance
    // POST /dashboard/refresh           — atualiza materialized views (somente admin)
    for (path, method) in &[
        (format!("{}/stock-summary", base), ACTION_GET),
        (format!("{}/daily-movements", base), ACTION_GET),
        (format!("{}/supplier-performance", base), ACTION_GET),
        (format!("{}/refresh", base), ACTION_POST),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_ADMIN, path, method])
            .await?;
    }

    // ROLE_USER pode ler os dados do dashboard (visualização)
    for path in &[
        format!("{}/stock-summary", base),
        format!("{}/daily-movements", base),
        format!("{}/supplier-performance", base),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_USER, path, ACTION_GET])
            .await?;
    }

    tracing::info!("Políticas de Dashboard (Materialized Views) carregadas");
    Ok(())
}
