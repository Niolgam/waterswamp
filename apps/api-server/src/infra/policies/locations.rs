use anyhow::Result;
use casbin::Enforcer;

use super::add_crud_policies;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/locations";

    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/countries", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/states", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/cities", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/site-types", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/building-types", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/space-types", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/department-categories", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/sites", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/buildings", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/floors", base)).await?;
    add_crud_policies(enforcer, crate::utils::ROLE_ADMIN, &format!("{}/spaces", base)).await?;

    tracing::info!("Políticas de Location Management carregadas");
    Ok(())
}
