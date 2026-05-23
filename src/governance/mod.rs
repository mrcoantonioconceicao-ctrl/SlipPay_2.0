use sqlx::{Pool, Postgres, Row, postgres::PgPoolOptions};
use rust_decimal::Decimal;
use chrono::{Utc, DateTime};

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

pub async fn conectar_db(url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .expect("Erro ao conectar ao banco")
}

pub async fn criar_tabela(pool: &Pool<Postgres>) {
    sqlx::query(
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
    .expect("Erro ao criar auditoria");
}

pub async fn registrar_transacao(
    pool: &Pool<Postgres>,
    log: LogTransacao,
) {
    sqlx::query(
        r#"
        INSERT INTO auditoria (
            id, conta_origem, conta_destino, valor, timestamp
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
    .expect("Erro ao registrar auditoria");
}

pub async fn criar_tabela_payments(pool: &Pool<Postgres>) {
    sqlx::query(
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
    .expect("Erro ao criar tabela payments");
}

pub async fn salvar_payment(
    pool: &Pool<Postgres>,
    payment: Payment,
) {
    sqlx::query(
        r#"
        INSERT INTO payments (
            payment_id, merchant_id, wallet_destino,
            token, network, amount, memo,
            expires_at, status, created_at
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
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
    .expect("Erro ao salvar payment");
}

pub async fn buscar_payment(
    pool: &Pool<Postgres>,
    id: &str,
) -> Option<Payment> {
    let row = sqlx::query(
        r#"SELECT * FROM payments WHERE payment_id = $1"#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .expect("Erro ao buscar payment");

    row.map(|r| Payment {
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
}

pub async fn atualizar_status_payment(
    pool: &Pool<Postgres>,
    id: &str,
    status: &str,
) {
    sqlx::query(
        r#"UPDATE payments SET status = $1 WHERE payment_id = $2"#
    )
    .bind(status)
    .bind(id)
    .execute(pool)
    .await
    .expect("Erro ao atualizar payment");
}
