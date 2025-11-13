use tokio::signal;
use tracing::info;

/// Aguarda um sinal de shutdown (SIGTERM, SIGINT/Ctrl+C)
/// e retorna quando o sinal for recebido
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Falha ao instalar handler de Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Falha ao instalar handler de SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("📡 Sinal Ctrl+C recebido, iniciando shutdown gracioso...");
        },
        _ = terminate => {
            info!("📡 Sinal SIGTERM recebido, iniciando shutdown gracioso...");
        },
    }
}

/// Versão com timeout para garantir que o shutdown não trave indefinidamente
pub async fn shutdown_signal_with_timeout(timeout_secs: u64) {
    use std::time::Duration;
    use tokio::time::timeout;

    let shutdown = shutdown_signal();

    if let Err(_) = timeout(Duration::from_secs(timeout_secs), shutdown).await {
        tracing::warn!(
            "⏱️ Timeout de {} segundos atingido, forçando shutdown...",
            timeout_secs
        );
    }

    info!("🛑 Servidor desligado.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_signal_with_timeout() {
        // Teste de timeout (não espera sinal real)
        let start = std::time::Instant::now();
        shutdown_signal_with_timeout(1).await;
        let elapsed = start.elapsed();

        // Deve ter esperado pelo menos 1 segundo
        assert!(elapsed.as_secs() >= 1);
    }
}
