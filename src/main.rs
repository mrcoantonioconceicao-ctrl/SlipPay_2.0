mod ai;
mod ast;
mod compliance;
mod config;
mod database;
mod finance;
mod governance;
mod interface;
mod pix;
mod security;
mod services;

use config::AppConfig;
use dotenvy::dotenv;

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

    let config = AppConfig::load();

    info!("🚀 SlipPay 2.0 iniciando...");

    // =========================================================
    // CONFIGURAÇÕES
    // =========================================================

    let database_url = config.database_url.clone();
    let host = config.host.clone();
    let port = config.port.clone();
    let solana_rpc = config.solana_rpc_url.clone();
    let network = config.solana_network.clone();

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

    let vasp_nome = config.vasp_nome.clone();
    let vasp_modo = config.vasp_modo.clone();

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
