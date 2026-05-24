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

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("🚀 SlipPay 2.0 iniciado...");

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL não definida no .env");

    let host = env::var("HOST")
        .unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string());

    let solana_rpc = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_|
            "https://api.devnet.solana.com".to_string()
        );

    println!("📡 Rede: {}", env::var("SOLANA_NETWORK")
        .unwrap_or_else(|_| "devnet".to_string()));
    println!("🔗 RPC: {}", solana_rpc);
    println!("⚖️  Compliance: BCB 519/520/521 ativo");
    println!("🗄️  Banco: conectando...");

    let pool = governance::conectar_db(&database_url).await;

    governance::criar_tabela(&pool).await;
    governance::criar_tabela_payments(&pool).await;

    println!("🗄️  Banco: conectado ✓");
    println!("🌐 Servidor: http://{}:{}", host, port);

    interface::iniciar_servidor(pool, host, port, solana_rpc).await;
}
