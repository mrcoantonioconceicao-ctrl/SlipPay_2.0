use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use sqlx::{Pool, Postgres};
use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::ai;
use crate::finance;
use crate::governance::{self, Payment};
use crate::pix;
use crate::security;
use crate::services;

type AppState = Pool<Postgres>;

#[derive(Deserialize)]
pub struct Transacao {
    pub conta: String,
    pub valor: Decimal,
}

#[derive(Deserialize)]
pub struct CheckoutRequest {
    pub merchant_id: String,
    pub wallet_destino: String,
    pub token: String,
    pub network: String,
    pub amount: Decimal,
}

#[derive(Deserialize)]
pub struct ConfirmRequest {
    pub payment_id: String,
    pub tx_hash: String,
    pub payer: String,
    pub amount: Decimal,
    pub memo: String,
}

#[derive(Deserialize)]
pub struct PixRequest {
    pub payment_id: String,
    pub merchant_id: String,
    pub chave_pix: String,
    pub valor_usdc: Decimal,
    pub taxa_cambio: Decimal,
}

#[derive(Serialize)]
pub struct CheckoutResponse {
    pub payment_id: String,
    pub merchant_id: String,
    pub wallet_destino: String,
    pub token: String,
    pub network: String,
    pub amount: Decimal,
    pub taxa_slippay: Decimal,
    pub valor_merchant: Decimal,
    pub memo: String,
    pub expires_at: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct ConfirmResponse {
    pub status: String,
    pub risk_score: u8,
    pub tx_hash: String,
    pub confirmacoes: u64,
}

fn autenticar(headers: &HeaderMap) -> bool {
    match headers.get("X-Api-Key") {
        Some(value) => match value.to_str() {
            Ok(key) => security::validar_api_key(key),
            Err(_) => false,
        },
        None => false,
    }
}

async fn home() -> impl IntoResponse {
    "SlipPay API 2.0 rodando 🚀"
}

async fn health() -> impl IntoResponse {
    "ok"
}

async fn saldo(
    headers: HeaderMap,
    Json(payload): Json<Transacao>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            "API Key inválida ou ausente".to_string(),
        );
    }

    let pubkey = match Pubkey::from_str(&payload.conta) {
        Ok(pk) => pk,
        Err(_) => return (StatusCode::BAD_REQUEST, "Conta inválida".to_string()),
    };

    let cliente = services::inicializar_cliente(
        "https://api.devnet.solana.com"
    );

    let saldo = services::consultar_saldo(&cliente, &pubkey);
    (StatusCode::OK, saldo.to_string())
}

async fn antifraude(
    headers: HeaderMap,
    Json(transacoes): Json<Vec<Transacao>>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(serde_json::json!({
            "error": "API Key inválida ou ausente"
        }));
    }
    let resultado = ai::analise_antifraude(transacoes);
    Json(serde_json::json!({ "resultado": resultado }))
}

async fn checkout(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Json(payload): Json<CheckoutRequest>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(serde_json::json!({
            "error": "API Key inválida ou ausente"
        }));
    }

    let payment_id = Uuid::new_v4().to_string();
    let memo = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(15);

    let breakdown = finance::calcular_breakdown(payload.amount);

    let payment = Payment {
        payment_id: payment_id.clone(),
        merchant_id: payload.merchant_id.clone(),
        wallet_destino: payload.wallet_destino.clone(),
        token: payload.token.clone(),
        network: payload.network.clone(),
        amount: payload.amount,
        memo: memo.clone(),
        expires_at,
        status: "pending".to_string(),
        created_at: Utc::now(),
    };

    governance::salvar_payment(&pool, payment).await;

    Json(serde_json::json!(CheckoutResponse {
        payment_id,
        merchant_id: payload.merchant_id,
        wallet_destino: payload.wallet_destino,
        token: payload.token,
        network: payload.network,
        amount: breakdown.valor_original,
        taxa_slippay: breakdown.taxa_slippay,
        valor_merchant: breakdown.valor_merchant,
        memo,
        expires_at: expires_at.to_rfc3339(),
        status: "pending".to_string(),
    }))
}

async fn webhook_confirm(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Json(payload): Json<ConfirmRequest>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(serde_json::json!({
            "error": "API Key inválida ou ausente"
        }));
    }

    let payment = match governance::buscar_payment(
        &pool,
        &payload.payment_id,
    )
    .await
    {
        Some(p) => p,
        None => {
            return Json(serde_json::json!({
                "error": "payment_id inválido"
            }))
        }
    };

    if payment.memo != payload.memo {
        return Json(serde_json::json!({
            "error": "memo inválido"
        }));
    }

    if payment.amount != payload.amount {
        return Json(serde_json::json!({
            "error": "valor divergente"
        }));
    }

    if Utc::now() > payment.expires_at {
        return Json(serde_json::json!({
            "error": "pagamento expirado"
        }));
    }

    let cliente = services::inicializar_cliente(
        "https://api.devnet.solana.com"
    );

    let verificacao = services::verificar_transacao(
        &cliente,
        &payload.tx_hash,
    );

    if !verificacao.valida {
        return Json(serde_json::json!({
            "error": format!(
                "transação inválida on-chain: {}",
                verificacao.erro.unwrap_or_default()
            )
        }));
    }

    let limite = Decimal::from(10000);
    let risco = if payload.amount > limite { 90u8 } else { 10u8 };

    if risco > 80 {
        return Json(serde_json::json!({
            "error": "suspeita de fraude"
        }));
    }

    governance::atualizar_status_payment(
        &pool,
        &payload.payment_id,
        "paid",
    )
    .await;

    Json(serde_json::json!(ConfirmResponse {
        status: "paid".to_string(),
        risk_score: risco,
        tx_hash: payload.tx_hash,
        confirmacoes: verificacao.confirmacoes,
    }))
}

async fn get_payment(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(serde_json::json!({
            "error": "API Key inválida ou ausente"
        }));
    }

    match governance::buscar_payment(&pool, &id).await {
        Some(payment) => Json(serde_json::json!({
            "payment_id": payment.payment_id,
            "status": payment.status,
            "amount": payment.amount,
            "merchant_id": payment.merchant_id,
            "expires_at": payment.expires_at.to_rfc3339()
        })),
        None => Json(serde_json::json!({
            "error": "payment não encontrado"
        })),
    }
}

async fn pix_offramp(
    headers: HeaderMap,
    Json(payload): Json<PixRequest>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(serde_json::json!({
            "error": "API Key inválida ou ausente"
        }));
    }

    let pedido = pix::criar_pedido_pix(
        &payload.payment_id,
        &payload.merchant_id,
        &payload.chave_pix,
        payload.valor_usdc,
        payload.taxa_cambio,
    );

    let resultado = pix::enviar_para_vasp(&pedido).await;

    Json(serde_json::json!({
        "sucesso": resultado.sucesso,
        "pedido_id": resultado.pedido_id,
        "valor_brl": resultado.valor_brl,
        "valor_liquido_brl": resultado.valor_liquido_brl,
        "mensagem": resultado.mensagem,
    }))
}

pub async fn iniciar_servidor(pool: Pool<Postgres>) {
    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .route("/saldo", post(saldo))
        .route("/antifraude", post(antifraude))
        .route("/checkout", post(checkout))
        .route("/webhook/confirm", post(webhook_confirm))
        .route("/payment/:id", get(get_payment))
        .route("/pix/offramp", post(pix_offramp))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Servidor rodando em http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
