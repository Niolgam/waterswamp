use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use super::add_crud_policies;
use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/fleet";

    // Reference data
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/categories", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/makes", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/models", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/colors", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/fuel-types", base)).await?;
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/transmission-types", base)).await?;

    // Vehicles CRUD
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/vehicles", base)).await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/vehicles/search", base),
            ACTION_GET
        ])
        .await?;

    // Vehicle status and history
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/vehicles/{{id}}/status", base),
            ACTION_PUT
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/vehicles/{{id}}/history", base),
            ACTION_GET
        ])
        .await?;

    // Vehicle documents
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/vehicles/{{id}}/documents", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/vehicles/{{id}}/documents", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/vehicles/{{vehicle_id}}/documents/{{doc_id}}", base),
            ACTION_DELETE
        ])
        .await?;

    tracing::info!("Políticas de Fleet Management carregadas");
    Ok(())
}
