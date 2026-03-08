use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use super::add_crud_policies;
use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/warehouse";

    // Material Groups
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/material-groups", base)).await?;

    // Materials
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/materials", base)).await?;

    // Warehouses (POST, GET by id, PUT by id — no list GET or DELETE)
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/warehouses", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/warehouses/{{id}}", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/warehouses/{{id}}", base),
            ACTION_PUT
        ])
        .await?;

    // Stock Movements
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/entry", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/exit", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/adjustment", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/transfer", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/{{id}}", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/{{id}}/maintenance", base),
            ACTION_PUT
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/{{id}}/block", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stock/{{id}}/block", base),
            ACTION_DELETE
        ])
        .await?;

    // Warehouse Reports
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/reports/stock-value", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/reports/stock-value/detail", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/reports/consumption", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/reports/most-requested", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/reports/movement-analysis", base),
            ACTION_GET
        ])
        .await?;

    tracing::info!("Políticas de Warehouse Management carregadas");
    Ok(())
}
