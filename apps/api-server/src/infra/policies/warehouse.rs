use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use super::add_crud_policies;
use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    // Base legada (singular) para rotas que ainda não foram migradas para o plural
    let legacy_base = "/api/admin/warehouse";
    
    // Base correta (plural) usada pelo Router principal de almoxarifados e pelos testes
    let wh_base = "/api/admin/warehouses";

    // Material Groups & Materials (Mantido do original)
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/material-groups", legacy_base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/materials", legacy_base)).await?;

    // ========================================================================
    // WAREHOUSES - CRUD Completo (Alinhado com router() em mod.rs)
    // ========================================================================
    enforcer.add_policy(str_vec![ROLE_ADMIN, wh_base, ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, wh_base, ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/{{id}}", wh_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/{{id}}", wh_base), ACTION_PUT]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/{{id}}", wh_base), ACTION_DELETE]).await?;

    // ========================================================================
    // WAREHOUSE STOCKS (Alinhado com router() em mod.rs)
    // ========================================================================
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/{{id}}/stocks", wh_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stocks/{{stock_id}}", wh_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stocks/{{stock_id}}", wh_base), ACTION_PATCH]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stocks/{{stock_id}}/block", wh_base), ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stocks/{{stock_id}}/unblock", wh_base), ACTION_POST]).await?;

    // ========================================================================
    // STOCK MOVEMENTS (Mantidos do original)
    // ========================================================================
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/entry", legacy_base), ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/exit", legacy_base), ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/adjustment", legacy_base), ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/transfer", legacy_base), ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/{{id}}", legacy_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/{{id}}/maintenance", legacy_base), ACTION_PUT]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/{{id}}/block", legacy_base), ACTION_POST]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/stock/{{id}}/block", legacy_base), ACTION_DELETE]).await?;

    // ========================================================================
    // WAREHOUSE REPORTS (Mantidos do original)
    // ========================================================================
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/reports/stock-value", legacy_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/reports/stock-value/detail", legacy_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/reports/consumption", legacy_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/reports/most-requested", legacy_base), ACTION_GET]).await?;
    enforcer.add_policy(str_vec![ROLE_ADMIN, format!("{}/reports/movement-analysis", legacy_base), ACTION_GET]).await?;

    tracing::info!("Políticas de Warehouse Management carregadas");
    Ok(())
}
