mod finance;
mod security;
mod services;
mod ai;
mod interface;
mod governance;
mod ast;

#[tokio::main]
async fn main() {
    println!("🚀 SlipPay 2.0 iniciado...");

    // conexão PostgreSQL
    let pool =
        governance::conectar_db(
            "postgres://u0_a372@localhost/slippay"
        )
        .await;

    // auditoria
    governance::criar_tabela(&pool).await;

    // payments persistentes
    governance::criar_tabela_payments(&pool).await;

    // sobe API REST
    interface::iniciar_servidor(pool).await;
}
