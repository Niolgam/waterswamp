use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

macro_rules! str_vec {
    ( $( $x:expr ),* $(,)? ) => {
        vec![ $( $x.to_string() ),* ]
    };
}

mod audit_logs;
mod budget_classifications;
mod catalog;
mod core;
mod drivers;
mod fleet;
mod fuelings;
mod geo_regions;
mod invoices;
mod locations;
mod organizational;
mod requisitions;
mod suppliers;
mod vehicle_fines;
mod warehouse;

use crate::utils::*;

/// Adds standard CRUD policies (GET list, POST, GET by id, PUT by id, DELETE by id)
async fn add_crud_policies(enforcer: &mut Enforcer, role: &str, base_path: &str) -> Result<()> {
    enforcer
        .add_policy(str_vec![role, base_path, ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![role, base_path, ACTION_POST])
        .await?;

    let by_id = format!("{}/{{id}}", base_path);
    enforcer
        .add_policy(str_vec![role, &by_id, ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![role, &by_id, ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![role, &by_id, ACTION_DELETE])
        .await?;
    Ok(())
}

/// Adds standard CRUD + tree endpoint policies
async fn add_crud_tree_policies(
    enforcer: &mut Enforcer,
    role: &str,
    base_path: &str,
) -> Result<()> {
    add_crud_policies(enforcer, role, base_path).await?;
    let tree = format!("{}/tree", base_path);
    enforcer
        .add_policy(str_vec![role, &tree, ACTION_GET])
        .await?;
    Ok(())
}

pub async fn seed_all_policies(enforcer: &mut Enforcer) -> Result<()> {
    core::seed(enforcer).await?;
    audit_logs::seed(enforcer).await?;
    locations::seed(enforcer).await?;
    geo_regions::seed(enforcer).await?;
    budget_classifications::seed(enforcer).await?;
    catalog::seed(enforcer).await?;
    organizational::seed(enforcer).await?;
    warehouse::seed(enforcer).await?;
    requisitions::seed(enforcer).await?;
    fleet::seed(enforcer).await?;
    suppliers::seed(enforcer).await?;
    drivers::seed(enforcer).await?;
    fuelings::seed(enforcer).await?;
    vehicle_fines::seed(enforcer).await?;
    invoices::seed(enforcer).await?;
    Ok(())
}
