use anyhow::Result;
use casbin::{Enforcer, MgmtApi};

use super::add_crud_policies;
use crate::utils::*;

pub async fn seed(enforcer: &mut Enforcer) -> Result<()> {
    let base = "/api/admin/invoices";

    // Standard CRUD: GET list, POST, GET by id, PUT by id, DELETE by id
    add_crud_policies(enforcer, ROLE_ADMIN, base).await?;

    // Invoice items
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            format!("{}/{{id}}/items", base),
            ACTION_GET
        ])
        .await?;

    // State machine transitions
    for action in &[
        "start-checking",
        "finish-checking",
        "post",
        "reject",
        "cancel",
    ] {
        enforcer
            .add_policy(str_vec![
                ROLE_ADMIN,
                format!("{}/{{id}}/{}", base, action),
                ACTION_POST
            ])
            .await?;
    }

    tracing::info!("Políticas de Invoice Management carregadas");
    Ok(())
}
