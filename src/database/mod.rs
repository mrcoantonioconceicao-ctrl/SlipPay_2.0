use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

/// Cria pool PostgreSQL
pub async fn conectar() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL não configurada");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}

/// Verifica se o banco está respondendo
pub async fn verificar_conexao(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conectar() {
        if let Ok(pool) = conectar().await {
            assert!(verificar_conexao(&pool).await.is_ok());
        }
    }
}
