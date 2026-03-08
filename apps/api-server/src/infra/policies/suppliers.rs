use anyhow::Result;
use casbin::Enforcer;

use super::add_crud_policies;
use crate::utils::ROLE_ADMIN;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    add_crud_policies(enforcer, ROLE_ADMIN, "/api/admin/suppliers").await?;

    tracing::info!("Políticas de Supplier Management carregadas");
    Ok(())
}
