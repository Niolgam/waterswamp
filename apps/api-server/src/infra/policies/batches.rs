use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let wh_base = "/api/admin/warehouses";

    // ============================================================
    // BATCH STOCKS — FEFO (RF-021)
    // GET /warehouses/:id/batch-stocks/:catalog_item_id
    // POST /warehouses/:id/batch-stocks/fefo-exit
    // GET /warehouses/:id/batch-stocks/:catalog_item_id/:batch_number
    // POST /warehouses/:id/batch-stocks/:catalog_item_id/:batch_number/quarantine
    // POST /warehouses/:id/batch-stocks/:catalog_item_id/:batch_number/release-quarantine
    // GET  /warehouses/:id/batch-stocks/near-expiry
    // ============================================================
    for (path, method) in &[
        (format!("{}/{{id}}/batch-stocks/{{catalog_item_id}}", wh_base), ACTION_GET),
        (format!("{}/{{id}}/batch-stocks/fefo-exit", wh_base), ACTION_POST),
        (format!("{}/{{id}}/batch-stocks/{{catalog_item_id}}/{{batch_number}}", wh_base), ACTION_GET),
        (format!("{}/{{id}}/batch-stocks/{{catalog_item_id}}/{{batch_number}}/quarantine", wh_base), ACTION_POST),
        (format!("{}/{{id}}/batch-stocks/{{catalog_item_id}}/{{batch_number}}/release-quarantine", wh_base), ACTION_POST),
        (format!("{}/{{id}}/batch-stocks/near-expiry", wh_base), ACTION_GET),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_ADMIN, path, method])
            .await?;
    }

    // ============================================================
    // BATCH QUALITY OCCURRENCES (RF-043)
    // POST /batch-occurrences
    // GET  /batch-occurrences
    // GET  /batch-occurrences/:id
    // POST /batch-occurrences/:id/resolve
    // POST /batch-occurrences/:id/close
    // ============================================================
    let occ_base = "/api/admin/batch-occurrences";
    for (path, method) in &[
        (occ_base.to_string(), ACTION_GET),
        (occ_base.to_string(), ACTION_POST),
        (format!("{}/{{id}}", occ_base), ACTION_GET),
        (format!("{}/{{id}}/resolve", occ_base), ACTION_POST),
        (format!("{}/{{id}}/close", occ_base), ACTION_POST),
    ] {
        enforcer
            .add_policy(str_vec![ROLE_ADMIN, path, method])
            .await?;
    }

    tracing::info!("Políticas de Batch / FEFO carregadas");
    Ok(())
}
