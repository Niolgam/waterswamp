use anyhow::Result;
use casbin::Enforcer;

use super::{add_crud_policies, add_crud_tree_policies};
use crate::utils::ROLE_ADMIN;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/catalog";

    // Units
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/units", base)).await?;

    // Catalog Groups (with tree)
    add_crud_tree_policies(enforcer, ROLE_ADMIN, &format!("{}/groups", base)).await?;

    // Catalog Items
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/items", base)).await?;

    // Units of Measure
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/units-of-measure", base)).await?;

    // Unit Conversions
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/conversions", base)).await?;

    // --- CATMAT ---
    add_crud_tree_policies(enforcer, ROLE_ADMIN, &format!("{}/catmat/groups", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/catmat/classes", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/catmat/pdms", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/catmat/items", base)).await?;

    // --- CATSER ---
    add_crud_tree_policies(enforcer, ROLE_ADMIN, &format!("{}/catser/sections", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/catser/divisions", base)).await?;
    add_crud_tree_policies(enforcer, ROLE_ADMIN, &format!("{}/catser/groups", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/catser/classes", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/catser/items", base)).await?;

    tracing::info!("Políticas de Catalog carregadas (incluindo CATMAT e CATSER)");
    Ok(())
}
