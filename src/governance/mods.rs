use sqlx::{Pool, Postgres, Row, postgres::PgPoolOptions};
use rust_decimal::Decimal;
use chrono::{Utc, DateTime};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct LogTransacao {
    pub id: String,
    pub conta_origem: String,
    pub conta_destino: String,
    pub valor: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Payment {
    pub payment_id: String,
    pub merchant_id: String,
    pub wallet_destino: String,
    pub token: String,
    pub network: String,
    pub amount: Decimal,
    pub memo: String,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Conecta no PostgreSQL
pub async fn conectar_db(url: &str) -> Pool<Postgres> {
    match PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
    {
        Ok(pool) => {
            info!("Banco conectado com sucesso");
            pool
        }
        Err(e) => {
            error!("Erro ao conectar ao banco: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn criar_tabela(pool: &Pool<Postgres>) {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS auditoria (
            id TEXT PRIMARY KEY,
            conta_origem TEXT NOT NULL,
            conta_destino TEXT NOT NULL,
            valor NUMERIC NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL
        )
        "#
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Tabela auditoria OK"),
        Err(e) => error!("Erro ao criar tabela auditoria: {}", e),
    }
}

pub async fn registrar_transacao(
    pool: &Pool<Postgres>,
    log: LogTransacao,
) {
    match sqlx::query(
        r#"
        INSERT INTO auditoria (
            id,
            conta_origem,
            conta_destino,
            valor,
            timestamp
        )
        VALUES ($1, $2, $3, $4, $5)
        "#
    )
    .bind(&log.id)
    .bind(&log.conta_origem)
    .bind(&log.conta_destino)
    .bind(log.valor)
    .bind(log.timestamp)
    .execute(pool)
    .await
    {
        Ok(_) => info!("Transação registrada: {}", log.id),
        Err(e) => error!("Erro ao registrar transação: {}", e),
    }
}

pub async fn criar_tabela_payments(pool: &Pool<Postgres>) {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS payments (
            payment_id TEXT PRIMARY KEY,
            merchant_id TEXT NOT NULL,
            wallet_destino TEXT NOT NULL,
            token TEXT NOT NULL,
            network TEXT NOT NULL,
            amount NUMERIC NOT NULL,
            memo TEXT NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            status TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL
        )
        "#
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Tabela payments OK"),
        Err(e) => error!("Erro ao criar tabela payments: {}", e),
    }
}

pub async fn salvar_payment(
    pool: &Pool<Postgres>,
    payment: Payment,
) {
    match sqlx::query(
        r#"
        INSERT INTO payments (
            payment_id,
            merchant_id,
            wallet_destino,
            token,
            network,
            amount,
            memo,
            expires_at,
            status,
            created_at
        )
        VALUES (
            $1,$2,$3,$4,$5,
            $6,$7,$8,$9,$10
        )
        "#
    )
    .bind(&payment.payment_id)
    .bind(&payment.merchant_id)
    .bind(&payment.wallet_destino)
    .bind(&payment.token)
    .bind(&payment.network)
    .bind(payment.amount)
    .bind(&payment.memo)
    .bind(payment.expires_at)
    .bind(&payment.status)
    .bind(payment.created_at)
    .execute(pool)
    .await
    {
        Ok(_) => info!("Payment salvo: {}", payment.payment_id),
        Err(e) => error!("Erro ao salvar payment: {}", e),
    }
}

pub async fn buscar_payment(
    pool: &Pool<Postgres>,
    id: &str,
) -> Option<Payment> {
    match sqlx::query(
        r#"SELECT * FROM payments WHERE payment_id = $1"#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    {
        Ok(row) => row.map(|r| Payment {
            payment_id: r.get("payment_id"),
            merchant_id: r.get("merchant_id"),
            wallet_destino: r.get("wallet_destino"),
            token: r.get("token"),
            network: r.get("network"),
            amount: r.get("amount"),
            memo: r.get("memo"),
            expires_at: r.get("expires_at"),
            status: r.get("status"),
            created_at: r.get("created_at"),
        }),
        Err(e) => {
            error!("Erro ao buscar payment {}: {}", id, e);
            None
        }
    }
}

pub async fn atualizar_status_payment(
    pool: &Pool<Postgres>,
    id: &str,
    status: &str,
) {
    match sqlx::query(
        r#"UPDATE payments SET status = $1 WHERE payment_id = $2"#
    )
    .bind(status)
    .bind(id)
    .execute(pool)
    .await
    {
        Ok(_) => info!("Payment {} atualizado para {}", id, status),
        Err(e) => error!("Erro ao atualizar payment {}: {}", id, e),
    }
}

/// Lista os pagamentos mais recentes
pub async fn listar_payments(
    pool: &Pool<Postgres>,
    limite: i64,
) -> Vec<Payment> {
    match sqlx::query(
        r#"
        SELECT *
        FROM payments
        ORDER BY created_at DESC
        LIMIT $1
        "#
    )
    .bind(limite)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|r| Payment {
                payment_id: r.get("payment_id"),
                merchant_id: r.get("merchant_id"),
                wallet_destino: r.get("wallet_destino"),
                token: r.get("token"),
                network: r.get("network"),
                amount: r.get("amount"),
                memo: r.get("memo"),
                expires_at: r.get("expires_at"),
                status: r.get("status"),
                created_at: r.get("created_at"),
            })
            .collect(),

        Err(e) => {
            error!("Erro ao listar payments: {}", e);
            Vec::new()
        }
    }
}
