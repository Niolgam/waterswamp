use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use super::add_crud_policies;
use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/vehicle-fines";

    // Fine Types
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/fine-types", base)).await?;

    // Fines
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/fines", base)).await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/fines/{{id}}/restore", base),
            ACTION_PUT
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/fines/{{id}}/status", base),
            ACTION_PUT
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/fines/{{id}}/history", base),
            ACTION_GET
        ])
        .await?;

    tracing::info!("Políticas de Vehicle Fines carregadas");
    Ok(())
}
