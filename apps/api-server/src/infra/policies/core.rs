use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use crate::utils::*;

use super::add_crud_policies;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    // Dashboard
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_DASHBOARD, ACTION_GET])
        .await?;

    // User profile
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_USER_PROFILE, ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_USER, RESOURCE_USER_PROFILE, ACTION_GET])
        .await?;

    // Admin policies management
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_POLICIES, ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_POLICIES, ACTION_DELETE])
        .await?;

    // Users CRUD
    add_crud_policies(enforcer, ROLE_ADMIN, "/api/admin/users").await?;

    tracing::info!("Políticas Core carregadas");
    Ok(())
}
