mod finance;
mod security;
mod services;
mod ai;
mod interface;
mod governance;
mod ast;
mod pix;
mod compliance;

use dotenvy::dotenv;
use std::env;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Inicializa logging estruturado (Orion: substituir println!)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    dotenv().ok();

    info!("🚀 SlipPay 2.0 iniciado...");

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => {
            error!("DATABASE_URL não definida: {}", e);
            std::process::exit(1);
        }
    };

    let host = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());

    let solana_rpc = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    info!("📡 Rede: {}", env::var("SOLANA_NETWORK")
        .unwrap_or_else(|_| "devnet".to_string()));
    info!("🔗 RPC: {}", solana_rpc);
    info!("⚖️  Compliance: BCB 519/520/521 ativo");
    info!("🗄️  Banco: conectando...");

    let pool = governance::conectar_db(&database_url).await;

    governance::criar_tabela(&pool).await;
    governance::criar_tabela_payments(&pool).await;

    info!("🗄️  Banco: conectado ✓");
    info!("🌐 Servidor: http://{}:{}", host, port);

    if let Err(e) = interface::iniciar_servidor(
        pool, host, port, solana_rpc
    ).await {
        error!("Erro fatal no servidor: {}", e);
        std::process::exit(1);
    }
}
