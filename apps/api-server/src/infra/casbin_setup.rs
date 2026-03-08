pub use crate::extractors::current_user::CurrentUser;
use crate::state::SharedEnforcer;
use anyhow::{Context, Result};
use axum::http::request::Parts;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use core_services::security;
use sqlx::PgPool;
use sqlx_adapter::SqlxAdapter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use super::policies;

fn find_model_path() -> Result<PathBuf> {
    let possible_paths = vec![
        PathBuf::from("rbac_model.conf"),
        PathBuf::from("../rbac_model.conf"),
        PathBuf::from("../../rbac_model.conf"),
        PathBuf::from("src/rbac_model.conf"),
        PathBuf::from("../src/rbac_model.conf"),
    ];

    for path in possible_paths {
        if path.exists() {
            info!("Arquivo de modelo Casbin encontrado em: {:?}", path);
            return Ok(path);
        }
    }

    anyhow::bail!(
        "Arquivo rbac_model.conf não encontrado. Tentei: raiz do projeto, ../rbac_model.conf, src/"
    )
}

pub async fn setup_casbin(pool: PgPool) -> Result<SharedEnforcer> {
    let adapter = SqlxAdapter::new_with_pool(pool.clone()).await?;
    let model_path = find_model_path()?;
    let model = DefaultModel::from_file(model_path).await?;
    let mut enforcer = Enforcer::new(model, adapter).await?;

    seed_policies(&mut enforcer, &pool).await?;

    let enforcer_arc = Arc::new(RwLock::new(enforcer));
    Ok(enforcer_arc)
}

async fn seed_policies(enforcer: &mut Enforcer, pool: &PgPool) -> Result<()> {
    // Limpa políticas antigas para evitar duplicação no reinício
    enforcer.clear_policy().await?;

    // Carrega todas as políticas de cada domínio
    policies::seed_all_policies(enforcer).await?;

    match enforcer.save_policy().await {
        Ok(_) => {
            info!("Políticas do Casbin carregadas e salvas no banco.");
        }
        Err(e) => {
            let error_msg = e.to_string();

            // PostgreSQL error code 23505 = unique constraint violation
            // This happens when tests run in parallel and try to insert duplicate policies
            if error_msg.contains("23505")
                || error_msg.contains("unique")
                || error_msg.contains("duplicar")
                || error_msg.contains("duplicate key")
            {
                info!("Políticas já existem no banco (execução paralela de testes)");
            } else {
                return Err(e).context("Falha ao salvar políticas iniciais no banco de dados");
            }
        }
    }

    info!("Iniciando seeding de usuários de teste...");

    let alice_hash =
        tokio::task::spawn_blocking(move || security::hash_password("password123")).await??;
    let bob_hash =
        tokio::task::spawn_blocking(move || security::hash_password("password123")).await??;

    let alice_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ('alice', 'alice@temp.example.com', $1)
        ON CONFLICT (username) DO UPDATE
        SET password_hash = EXCLUDED.password_hash
        RETURNING id
        "#,
    )
    .bind(&alice_hash)
    .fetch_one(pool)
    .await?;

    let bob_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ('bob','bob@temp.example.com', $1)
        ON CONFLICT (username) DO UPDATE
        SET password_hash = EXCLUDED.password_hash
        RETURNING id
        "#,
    )
    .bind(&bob_hash)
    .fetch_one(pool)
    .await?;

    info!("Usuários criados: alice={}, bob={}", alice_id, bob_id);

    enforcer
        .add_grouping_policy(vec![alice_id.to_string(), "ROLE_ADMIN".to_string()])
        .await?;
    enforcer
        .add_grouping_policy(vec![bob_id.to_string(), "ROLE_USER".to_string()])
        .await?;

    enforcer
        .add_grouping_policy(vec!["ROLE_ADMIN".to_string(), "ROLE_USER".to_string()])
        .await?;

    if let Err(e) = enforcer.save_policy().await {
        let error_msg = e.to_string();
        if error_msg.contains("23505") || error_msg.contains("unique constraint") {
            info!("Grupos já existem no banco, ignorando erro.");
        } else {
            return Err(e).context("Falha ao salvar grupos");
        }
    }

    info!("Grupos do Casbin associados aos UUIDs dos usuários.");

    Ok(())
}

pub fn casbin_subject_getter(parts: &Parts) -> String {
    if let Some(user) = parts.extensions.get::<CurrentUser>() {
        user.id.to_string()
    } else {
        "anonymous".to_string()
    }
}
