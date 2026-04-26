use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/stock-alerts";

    // GET  /stock-alerts           — lista alertas (com filtros)
    // POST /stock-alerts           — cria alerta manual
    // GET  /stock-alerts/:id       — detalhe de alerta
    // POST /stock-alerts/:id/acknowledge
    // POST /stock-alerts/:id/resolve
    // POST /stock-alerts/sla-check — dispara verificação de SLA vencido
    for (path, method) in &[
        (base.to_string(), ACTION_GET),
        (base.to_string(), ACTION_POST),
        (format!("{}/{{id}}", base), ACTION_GET),
        (format!("{}/{{id}}/acknowledge", base), ACTION_POST),
        (format!("{}/{{id}}/resolve", base), ACTION_POST),
        (format!("{}/sla-check", base), ACTION_POST),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_ADMIN, path, method])
            .await?;
    }

    // ROLE_USER pode visualizar alertas do seu almoxarifado (leitura)
    enforcer
        .add_policy(str_vec![ROLE_USER, base, ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_USER, format!("{}/{{id}}", base), ACTION_GET])
        .await?;

    tracing::info!("Políticas de Alertas de Estoque / SLA carregadas");
    Ok(())
}
