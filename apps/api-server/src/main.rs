use std::env;
use std::net::SocketAddr;
use std::process;
use waterswamp::run;

#[tokio::main]
async fn main() {
    // ⭐ 0. Carregar variáveis de ambiente do arquivo .env
    // Usamos .ok() para ignorar o erro se o arquivo não existir.
    // Isso é IMPORTANTE: em produção (Docker/AWS), o arquivo .env geralmente
    // não existe (as vars são injetadas pelo ambiente), então não queremos que quebre.
    // dotenvy::dotenv().ok();
    dotenvy::from_path("apps/api-server/.env").ok();
    // 1. Lógica de Resolução da Porta (agora já enxerga o .env se existir)
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("⏳ Bootstrapping Waterswamp na porta {}...", port);

    // 2. Executa a aplicação
    if let Err(e) = run(addr).await {
        eprintln!("❌ Erro fatal na aplicação: {:?}", e);
        process::exit(1);
    }
}
