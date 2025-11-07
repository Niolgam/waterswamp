use crate::constants::*;
use crate::models::CurrentUser;
use crate::state::SharedEnforcer;
use anyhow::Result;
use axum::http::request::Parts;
use bcrypt::{DEFAULT_COST, hash};
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use sqlx::PgPool;
use sqlx_adapter::SqlxAdapter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

macro_rules! str_vec {
    ( $( $x:expr ),* $(,)? ) => {
        vec![ $( $x.to_string() ),* ]
    };
}

/// Encontra o caminho do arquivo rbac_model.conf
/// Tenta vários locais possíveis para funcionar tanto em produção quanto em testes
fn find_model_path() -> Result<PathBuf> {
    let possible_paths = vec![
        PathBuf::from("rbac_model.conf"),        // Diretório atual
        PathBuf::from("../rbac_model.conf"),     // Um nível acima (para testes)
        PathBuf::from("../../rbac_model.conf"),  // Dois níveis acima
        PathBuf::from("src/rbac_model.conf"),    // Na pasta src
        PathBuf::from("../src/rbac_model.conf"), // src um nível acima
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

    // Seed policies and users
    seed_policies(&mut enforcer, &pool).await?;

    let enforcer_arc = Arc::new(RwLock::new(enforcer));
    Ok(enforcer_arc)
}

async fn seed_policies(enforcer: &mut Enforcer, pool: &PgPool) -> Result<()> {
    enforcer.clear_policy().await?;

    // --- Definindo Políticas (Papéis) ---
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_DASHBOARD, ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_USER, RESOURCE_USER_PROFILE, ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_POLICIES, ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_POLICIES, ACTION_DELETE])
        .await?;

    // Salva as políticas no banco de dados
    enforcer.save_policy().await?;

    info!("Políticas do Casbin carregadas e salvas no banco.");
    info!("Iniciando seeding de usuários de teste...");

    // Cria os hashes das senhas
    let alice_hash =
        tokio::task::spawn_blocking(move || hash("password123", DEFAULT_COST)).await??;
    let bob_hash = tokio::task::spawn_blocking(move || hash("password123", DEFAULT_COST)).await??;

    // Insere os usuários e captura os IDs gerados
    let alice_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, password_hash)
        VALUES ('alice', $1)
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
        INSERT INTO users (username, password_hash)
        VALUES ('bob', $1)
        ON CONFLICT (username) DO UPDATE
        SET password_hash = EXCLUDED.password_hash
        RETURNING id
        "#,
    )
    .bind(&bob_hash)
    .fetch_one(pool)
    .await?;

    info!("Usuários criados: alice={}, bob={}", alice_id, bob_id);

    // --- Agora usa os UUIDs reais nas políticas do Casbin ---
    enforcer
        .add_grouping_policy(str_vec![&alice_id.to_string(), ROLE_ADMIN])
        .await?;
    enforcer
        .add_grouping_policy(str_vec![&bob_id.to_string(), ROLE_USER])
        .await?;

    enforcer.save_policy().await?;

    info!("Grupos do Casbin associados aos UUIDs dos usuários.");

    Ok(())
}

/// Função "getter" que o `axum-casbin` usará para
/// extrair o "subject" (sujeito) da requisição.
pub fn casbin_subject_getter(parts: &Parts) -> String {
    if let Some(user) = parts.extensions.get::<CurrentUser>() {
        // Usa UUID como subject (mais estável)
        user.id.to_string()
    } else {
        "anonymous".to_string()
    }
}
