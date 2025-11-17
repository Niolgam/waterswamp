use axum_test::TestServer;
use domain::models::{Claims, TokenType};
use jsonwebtoken::DecodingKey;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use waterswamp::casbin_setup::setup_casbin;
use waterswamp::config::config::Config;
use waterswamp::routes::build;
use waterswamp::state::AppState;
// [Fix] Import AuditService
use waterswamp::handlers::audit_services::AuditService;

use async_trait::async_trait;
use email_service::EmailSender;
use tera::Context as TeraContext;

#[derive(Clone)]
#[allow(dead_code)]
pub struct MockEmail {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub context: TeraContext,
}

#[derive(Clone, Default)]
pub struct MockEmailService {
    pub messages: Arc<Mutex<Vec<MockEmail>>>,
}

impl MockEmailService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EmailSender for MockEmailService {
    async fn send_email(
        &self,
        to_email: String,
        subject: String,
        template: &str,
        context: TeraContext,
    ) -> anyhow::Result<()> {
        let mut guard = self.messages.lock().await;
        guard.push(MockEmail {
            to: to_email,
            subject,
            template: template.to_string(),
            context,
        });
        Ok(())
    }

    fn send_welcome_email(&self, to_email: String, username: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        let service = self.clone();
        let subject = "Bem-vindo ao Waterswamp!".to_string();
        let template = "welcome.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    fn send_password_reset_email(&self, to_email: String, username: &str, token: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        let reset_link = format!("http://mock.test/reset?token={}", token);
        context.insert("reset_link", &reset_link);
        let service = self.clone();
        let subject = "Redefina sua senha do Waterswamp".to_string();
        let template = "reset_password.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    fn send_verification_email(&self, to_email: String, username: &str, token: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        let verification_link = format!("http://mock.test/verify-email?token={}", token);
        context.insert("verification_link", &verification_link);
        let service = self.clone();
        let subject = "Verifique seu email - Waterswamp".to_string();
        let template = "email_verification.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }

    fn send_mfa_enabled_email(&self, to_email: String, username: &str) {
        let mut context = TeraContext::new();
        context.insert("username", username);
        context.insert("enabled_at", "2025-01-01 00:00:00 UTC");
        let service = self.clone();
        let subject = "MFA Ativado - Waterswamp".to_string();
        let template = "mfa_enabled.html".to_string();
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }
}

#[allow(dead_code)]
pub struct TestApp {
    pub api: TestServer,
    pub db_auth: PgPool,
    pub db_logs: PgPool,
    pub admin_token: String,
    pub user_token: String,
    pub email_service: Arc<MockEmailService>,
}

pub async fn spawn_app() -> TestApp {
    dotenvy::dotenv().ok();

    std::env::set_var("DISABLE_RATE_LIMIT", "true");

    let config = Config::from_env().expect("Falha ao carregar config de teste");

    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db");

    // [Fix] Run migrations to create tables
    sqlx::migrate!("../../crates/persistence/migrations_auth")
        .run(&pool_auth)
        .await
        .expect("Falha ao rodar migrations no auth_db");

    sqlx::migrate!("../../crates/persistence/migrations_logs")
        .run(&pool_logs)
        .await
        .expect("Falha ao rodar migrations no logs_db");

    // [Fix] Removed TRUNCATE calls to prevent race conditions in parallel tests.
    // In a real production test env, you would use transactional tests or unique DBs per test.
    // sqlx::query("TRUNCATE TABLE users, casbin_rule, refresh_tokens CASCADE")
    //    .execute(&pool_auth)
    //    .await
    //    .ok();
    // sqlx::query("TRUNCATE TABLE audit_logs CASCADE")
    //    .execute(&pool_logs)
    //    .await
    //    .ok();

    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");

    let encoding_key = EncodingKey::from_ed_pem(private_pem).expect("Chave privada inválida");
    let decoding_key = DecodingKey::from_ed_pem(public_pem).expect("Chave pública inválida");

    let email_service = Arc::new(MockEmailService::new());

    // [Fix] Initialize audit service
    let audit_service = AuditService::new(pool_logs.clone());

    let app_state = AppState {
        enforcer,
        policy_cache: Arc::new(RwLock::new(HashMap::new())),
        db_pool_auth: pool_auth.clone(),
        db_pool_logs: pool_logs.clone(),
        encoding_key,
        decoding_key,
        email_service: email_service.clone(),
        // [Fix] Add field to AppState
        audit_service,
    };

    let app = build(app_state);
    let api = TestServer::new(app).unwrap();

    let alice_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&pool_auth)
        .await
        .expect("Alice não encontrada no seed");

    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&pool_auth)
        .await
        .expect("Bob não encontrado no seed");

    TestApp {
        api,
        db_auth: pool_auth,
        db_logs: pool_logs,
        admin_token: generate_test_token(alice_id),
        user_token: generate_test_token(bob_id),
        email_service: email_service,
    }
}

pub fn generate_test_token(user_id: Uuid) -> String {
    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let encoding_key = EncodingKey::from_ed_pem(private_pem).unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: user_id,
        exp: now + 3600,
        iat: now,
        token_type: TokenType::Access,
    };

    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &encoding_key).unwrap()
}
