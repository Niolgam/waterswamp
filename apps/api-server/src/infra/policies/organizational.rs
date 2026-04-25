use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use super::add_crud_policies;
use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/organizational";

    // System Settings (uses {key} instead of {id})
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/settings", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/settings", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/settings/{{key}}", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/settings/{{key}}", base),
            ACTION_PUT
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/settings/{{key}}", base),
            ACTION_DELETE
        ])
        .await?;

    // Organizations
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/organizations", base)).await?;

    // Unit Categories
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/unit-categories", base)).await?;

    // Unit Types
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/unit-types", base)).await?;

    // Organizational Units (with tree and extra endpoints)
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/units", base)).await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/units/tree", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/units/{{id}}/children", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/units/{{id}}/path", base),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/units/{{id}}/deactivate", base),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/units/{{id}}/activate", base),
            ACTION_POST
        ])
        .await?;

    // SIORG Sync
    let sync = format!("{}/sync", base);
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/organization", sync),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/unit", sync),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/org-units", sync),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/health", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/from-db", sync),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/organization/*", sync),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/organization/*/units", sync),
            ACTION_POST
        ])
        .await?;

    // Sync Queue Management
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/queue", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/queue/stats", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/queue/*", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/queue/*", sync),
            ACTION_DELETE
        ])
        .await?;

    // Conflict Resolution
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/conflicts", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/conflicts/*", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/conflicts/*/resolve", sync),
            ACTION_POST
        ])
        .await?;

    // Sync History
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/history", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/history/*", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/history/*/review", sync),
            ACTION_POST
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/history/entity/*/*", sync),
            ACTION_GET
        ])
        .await?;

    // Sync Statistics
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stats/detailed", sync),
            ACTION_GET
        ])
        .await?;
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/stats/health", sync),
            ACTION_GET
        ])
        .await?;

    // Natureza Jurídica
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/natureza-juridica", base)).await?;

    // Poder
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/poder", base)).await?;

    // Esfera
    add_crud_policies(enforcer, ROLE_ADMIN, &format!("{}/esfera", base)).await?;

    tracing::info!("Políticas de Organizational Management carregadas");
    Ok(())
}
