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
use waterswamp::config::Config;
use waterswamp::routes::build_router;
use waterswamp::state::AppState;

use async_trait::async_trait;
use email_service::EmailSender;
use tera::Context as TeraContext;

#[derive(Clone)]
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
        // Em vez de enviar, apenas guardamos a mensagem na Vec
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

        // O spawn é importante para simular o comportamento real
        tokio::spawn(async move {
            let _ = service
                .send_email(to_email, subject, &template, context)
                .await;
        });
    }
}

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

    std::env::set_var("DISABLE_RATE_LIMIT", "true"); // Opcional, dependendo se quer testar rate limit ou não

    let config = Config::from_env().expect("Falha ao carregar config de teste");

    let pool_auth = PgPool::connect(&config.auth_db)
        .await
        .expect("Falha ao conectar ao auth_db");
    let pool_logs = PgPool::connect(&config.logs_db)
        .await
        .expect("Falha ao conectar ao logs_db");

    // Limpa o banco antes de cada teste para garantir isolamento (opcional mas recomendado)
    sqlx::query("TRUNCATE TABLE users, casbin_rule, refresh_tokens CASCADE")
        .execute(&pool_auth)
        .await
        .ok();

    let enforcer = setup_casbin(pool_auth.clone())
        .await
        .expect("Falha ao inicializar Casbin");

    // Carrega chaves de teste
    let private_pem = include_bytes!("../tests/keys/private_test.pem");
    let public_pem = include_bytes!("../tests/keys/public_test.pem");

    let encoding_key = EncodingKey::from_ed_pem(private_pem).expect("Chave privada inválida");
    let decoding_key = DecodingKey::from_ed_pem(public_pem).expect("Chave pública inválida");

    let email_service = Arc::new(MockEmailService::new());

    let app_state = AppState {
        enforcer,
        policy_cache: Arc::new(RwLock::new(HashMap::new())),
        db_pool_auth: pool_auth.clone(),
        db_pool_logs: pool_logs.clone(),
        encoding_key,
        decoding_key,
        email_service: email_service.clone(),
    };

    let app = build_router(app_state);
    let api = TestServer::new(app).unwrap();

    // --- Busca os IDs da Alice e Bob que foram criados no setup_casbin ---
    let alice_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'alice'")
        .fetch_one(&pool_auth)
        .await
        .expect("Alice não encontrada no seed");

    let bob_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'bob'")
        .fetch_one(&pool_auth)
        .await
        .expect("Bob não encontrado no seed");

    // --- Gera tokens para eles ---
    TestApp {
        api,
        db_auth: pool_auth,
        db_logs: pool_logs,
        admin_token: generate_test_token(alice_id),
        user_token: generate_test_token(bob_id),
        email_service: email_service,
    }
}

// Seu helper de geração de token
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
