use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

/// Cria pool PostgreSQL
pub async fn conectar() -> Result<PgPool, sqlx::Error> {
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            return Err(sqlx::Error::Configuration(
                "DATABASE_URL não configurada".into(),
            ));
        }
    };

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
}

/// Verifica se o banco está respondendo
pub async fn verificar_conexao(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conectar() {
        // Se não existir DATABASE_URL,
        // apenas ignora o teste.
        if std::env::var("DATABASE_URL").is_err() {
            return;
        }

        let pool = conectar()
            .await
            .expect("Falha ao conectar");

        assert!(verificar_conexao(&pool).await.is_ok());
    }
}
