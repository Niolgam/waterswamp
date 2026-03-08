use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/audit-logs";

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, base, ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, format!("{}/stats", base), ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/failed-logins", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/suspicious-ips", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/user/{{user_id}}", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/cleanup", base),
            ACTION_POST
        ])
        .await?;

    tracing::info!("Políticas de Audit Logs carregadas");
    Ok(())
}
