pub use crate::extractors::current_user::CurrentUser;
use crate::state::SharedEnforcer;
use crate::utils::*;
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

macro_rules! str_vec {
    ( $( $x:expr ),* $(,)? ) => {
        vec![ $( $x.to_string() ),* ]
    };
}

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

    // --- Definindo Políticas (Papéis) ---
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_DASHBOARD, ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_USER_PROFILE, ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_POLICIES, ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, RESOURCE_ADMIN_POLICIES, ACTION_DELETE])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_USER, RESOURCE_USER_PROFILE, ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/users", ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/users", ACTION_POST])
        .await?;

    // ATUALIZADO PARA SINTAXE {id} DO AXUM 0.7 (para manter consistência com as rotas)
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/users/{id}", ACTION_GET])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/users/{id}", ACTION_PUT])
        .await?;

    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/users/{id}", ACTION_DELETE])
        .await?;

    // --- AUDIT LOG POLICIES ---
    // Allow admin to view audit logs (list, pagination, filtering)
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/audit-logs", ACTION_GET])
        .await?;

    // Allow admin to view audit log statistics
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            "/api/admin/audit-logs/stats",
            ACTION_GET
        ])
        .await?;

    // Allow admin to view failed login attempts
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            "/api/admin/audit-logs/failed-logins",
            ACTION_GET
        ])
        .await?;

    // Allow admin to view suspicious IPs
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            "/api/admin/audit-logs/suspicious-ips",
            ACTION_GET
        ])
        .await?;

    // Allow admin to view user-specific audit logs
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            "/api/admin/audit-logs/user/{user_id}",
            ACTION_GET
        ])
        .await?;

    // Allow admin to view specific audit log entry
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            "/api/admin/audit-logs/{id}",
            ACTION_GET
        ])
        .await?;

    // Allow admin to cleanup old audit logs (destructive operation)
    enforcer
        .add_policy(str_vec![
            ROLE_ADMIN,
            "/api/admin/audit-logs/cleanup",
            ACTION_POST
        ])
        .await?;

    info!("Políticas de Audit Logs carregadas");

    // --- LOCATION POLICIES ---
    // Countries
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/countries", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/countries", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/countries/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/countries/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/countries/{id}", ACTION_DELETE])
        .await?;

    // States
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/states", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/states", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/states/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/states/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/states/{id}", ACTION_DELETE])
        .await?;

    // Cities
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/cities", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/cities", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/cities/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/cities/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/cities/{id}", ACTION_DELETE])
        .await?;

    // Site Types
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/site-types", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/site-types", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/site-types/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/site-types/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/site-types/{id}", ACTION_DELETE])
        .await?;

    // Building Types
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/building-types", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/building-types", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/building-types/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/building-types/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/building-types/{id}", ACTION_DELETE])
        .await?;

    // Space Types
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/space-types", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/space-types", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/space-types/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/space-types/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/space-types/{id}", ACTION_DELETE])
        .await?;

    // Department Categories
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/department-categories", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/department-categories", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/department-categories/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/department-categories/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/department-categories/{id}", ACTION_DELETE])
        .await?;

    // Sites (Phase 3A)
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/sites", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/sites", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/sites/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/sites/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/sites/{id}", ACTION_DELETE])
        .await?;

    // Buildings (Phase 3B)
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/buildings", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/buildings", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/buildings/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/buildings/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/buildings/{id}", ACTION_DELETE])
        .await?;

    // Floors (Phase 3C)
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/floors", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/floors", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/floors/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/floors/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/floors/{id}", ACTION_DELETE])
        .await?;

    // Spaces (Phase 3D)
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/spaces", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/spaces", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/spaces/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/spaces/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/locations/spaces/{id}", ACTION_DELETE])
        .await?;

    info!("Políticas de Location Management carregadas");

    // --- GEO REGIONS POLICIES ---
    // Countries
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/countries", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/countries", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/countries/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/countries/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/countries/{id}", ACTION_DELETE])
        .await?;

    // States
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/states", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/states", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/states/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/states/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/states/{id}", ACTION_DELETE])
        .await?;

    // Cities
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/cities", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/cities", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/cities/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/cities/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/geo_regions/cities/{id}", ACTION_DELETE])
        .await?;

    info!("Políticas de Geo Regions carregadas");

    // --- BUDGET CLASSIFICATIONS POLICIES ---
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/budget-classifications", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/budget-classifications", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/budget-classifications/tree", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/budget-classifications/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/budget-classifications/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/budget-classifications/{id}", ACTION_DELETE])
        .await?;

    info!("Políticas de Budget Classifications carregadas");

    // --- WAREHOUSE POLICIES ---
    // Material Groups
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/material-groups", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/material-groups", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/material-groups/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/material-groups/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/material-groups/{id}", ACTION_DELETE])
        .await?;

    // Materials
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/materials", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/materials", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/materials/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/materials/{id}", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/materials/{id}", ACTION_DELETE])
        .await?;

    // Warehouses
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/warehouses", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/warehouses/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/warehouses/{id}", ACTION_PUT])
        .await?;

    // Stock Movements
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/entry", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/exit", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/adjustment", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/transfer", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/{id}/maintenance", ACTION_PUT])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/{id}/block", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/stock/{id}/block", ACTION_DELETE])
        .await?;

    // Warehouse Reports
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/reports/stock-value", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/reports/stock-value/detail", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/reports/consumption", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/reports/most-requested", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/warehouse/reports/movement-analysis", ACTION_GET])
        .await?;

    info!("Políticas de Warehouse Management carregadas");

    // --- REQUISITION POLICIES ---
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions/{id}", ACTION_GET])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions/{id}", ACTION_DELETE])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions/{id}/approve", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions/{id}/reject", ACTION_POST])
        .await?;
    enforcer
        .add_policy(str_vec![ROLE_ADMIN, "/api/admin/requisitions/{id}/fulfill", ACTION_POST])
        .await?;

    info!("Políticas de Requisitions carregadas");

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
                // This is expected in parallel tests - policies already exist
            } else {
                // This is an actual error - propagate it
                return Err(e).context("Falha ao salvar políticas iniciais no banco de dados");
            }
        }
    }

    info!("Iniciando seeding de usuários de teste...");

    // --- ATUALIZAÇÃO AQUI: Usando Argon2id via security::hash_password ---
    // Como Argon2 pode ser pesado, ainda é bom fazer em spawn_blocking se configurado com parâmetros altos,
    // mas a crate 'argon2' com padrão é razoavelmente rápida para setup inicial.
    // Mantendo o padrão de spawn_blocking por segurança.
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
        .add_grouping_policy(str_vec![&alice_id.to_string(), ROLE_ADMIN])
        .await?;
    enforcer
        .add_grouping_policy(str_vec![&bob_id.to_string(), ROLE_USER])
        .await?;

    enforcer
        .add_grouping_policy(str_vec![ROLE_ADMIN, ROLE_USER])
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
