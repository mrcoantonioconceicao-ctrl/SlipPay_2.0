mod ai;
mod ast;
mod compliance;
mod database;
mod finance;
mod governance;
mod interface;
mod pix;
mod security;
mod services;

use dotenvy::dotenv;
use std::env;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // =========================================================
    // LOGGING
    // =========================================================
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    dotenv().ok();

    info!("🚀 SlipPay 2.0 iniciando...");

    // =========================================================
    // CONFIGURAÇÕES
    // =========================================================

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            error!("DATABASE_URL não encontrada");

            error!(
                "Configure no .env:
DATABASE_URL=postgres://usuario:senha@localhost/slippay"
            );

            std::process::exit(1);
        }
    };

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let solana_rpc =
        env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let network = env::var("SOLANA_NETWORK").unwrap_or_else(|_| "devnet".to_string());

    info!("🌐 Host: {}", host);
    info!("🚪 Porta: {}", port);
    info!("📡 Rede Solana: {}", network);
    info!("🔗 RPC: {}", solana_rpc);

    // =========================================================
    // DATABASE
    // =========================================================

    info!("🗄️ Conectando PostgreSQL...");

    let pool = governance::conectar_db(&database_url).await;

    governance::criar_tabela(&pool).await;
    governance::criar_tabela_payments(&pool).await;

    info!("✅ PostgreSQL conectado");

    // =========================================================
    // PIX
    // =========================================================

    let vasp_nome = env::var("VASP_NOME").unwrap_or_else(|_| "SlipPay VASP".to_string());

    let vasp_modo = env::var("VASP_MODO").unwrap_or_else(|_| "simulado".to_string());

    info!("🏦 PIX Off-Ramp ativo");
    info!("🏦 VASP: {}", vasp_nome);
    info!("⚙️ Modo VASP: {}", vasp_modo);

    // =========================================================
    // COMPLIANCE
    // =========================================================

    info!("⚖️ Compliance BACEN 519/520/521 ativo");

    // =========================================================
    // SERVIDOR
    // =========================================================

    info!("🌍 API disponível em:");
    info!("http://{}:{}", host, port);

    match interface::iniciar_servidor(pool, host, port, solana_rpc).await {
        Ok(_) => {
            info!("Servidor encerrado");
        }

        Err(e) => {
            error!("Erro fatal: {}", e);
            std::process::exit(1);
        }
    }
}
