use anyhow::Result;
use casbin::Enforcer;

use super::add_crud_tree_policies;
use crate::utils::ROLE_ADMIN;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    add_crud_tree_policies(
        enforcer,
        ROLE_ADMIN,
        "/api/admin/budget-classifications",
    )
    .await?;

    tracing::info!("Políticas de Budget Classifications carregadas");
    Ok(())
}
