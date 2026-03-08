use anyhow::Result;
use casbin::Enforcer;

use super::add_crud_policies;
use crate::utils::ROLE_ADMIN;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/geo_regions";

    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/countries", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/states", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/cities", base)).await?;

    tracing::info!("Políticas de Geo Regions carregadas");
    Ok(())
}
