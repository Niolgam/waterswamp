use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/requisitions";

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, base, ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, base, ACTION_POST])
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
            format!("{}/{{id}}", base),
            ACTION_DELETE
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/approve", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/reject", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/fulfill", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/cancel", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/history", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/rollback-points", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/rollback", base),
            ACTION_POST
        ])
        .await?;

    // Requisition items
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/items", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{req_id}}/items/{{item_id}}/delete", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{req_id}}/items/{{item_id}}/restore", base),
            ACTION_POST
        ])
        .await?;

    tracing::info!("Políticas de Requisitions carregadas");
    Ok(())
}
