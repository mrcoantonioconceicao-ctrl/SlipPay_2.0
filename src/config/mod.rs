use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub host: String,
    pub port: String,
    pub solana_rpc_url: String,
    pub solana_network: String,
    pub vasp_nome: String,
    pub vasp_modo: String,
}

impl AppConfig {
    pub fn load() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL não configurada"),

            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),

            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),

            solana_rpc_url: env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),

            solana_network: env::var("SOLANA_NETWORK").unwrap_or_else(|_| "devnet".to_string()),

            vasp_nome: env::var("VASP_NOME").unwrap_or_else(|_| "SlipPay VASP".to_string()),

            vasp_modo: env::var("VASP_MODO").unwrap_or_else(|_| "simulado".to_string()),
        }
    }
}
