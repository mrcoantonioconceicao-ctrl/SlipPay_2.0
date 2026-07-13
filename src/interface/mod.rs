use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use sqlx::{Pool, Postgres};
use std::env;
use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use uuid::Uuid;

use crate::ai;
use crate::finance;
use crate::governance::{self, Payment};
use crate::pix;
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

/// Requisição de off-ramp PIX.
/// IMPORTANTE: valor_usdc NÃO vem do cliente — é lido do payment já
/// confirmado on-chain (governance::buscar_payment). O cliente só
/// informa a chave PIX de destino e a taxa de câmbio da cotação atual.
#[derive(Deserialize)]
pub struct PixRequest {
    pub payment_id: String,
    pub chave_pix: String,
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
    let api_keys: Vec<String> = env::var("API_KEYS")
        .unwrap_or_else(|_| "slippay-dev-key-2026".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    match headers.get("X-Api-Key") {
        Some(value) => match value.to_str() {
            Ok(key) => api_keys.contains(&key.to_string()),
            Err(_) => false,
        },
        None => false,
    }
}

fn get_rpc_url() -> String {
    env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())
}

fn erro_nao_autorizado() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": "API Key inválida ou ausente"
    }))
}

async fn home() -> impl IntoResponse {
    "SlipPay API 2.0 rodando 🚀"
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": "2.0",
        "network": env::var("SOLANA_NETWORK")
            .unwrap_or_else(|_| "devnet".to_string()),
    }))
}

async fn saldo(headers: HeaderMap, Json(payload): Json<Transacao>) -> impl IntoResponse {
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

    let cliente = services::inicializar_cliente(&get_rpc_url());
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

    info!("Checkout criado: {}", payment_id);

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

// Subfunções extraídas do webhook_confirm (Orion: reduzir função de 81 linhas)

fn validar_payment(
    payment: &governance::Payment,
    payload: &ConfirmRequest,
) -> Result<(), serde_json::Value> {
    if payment.memo != payload.memo {
        return Err(serde_json::json!({ "error": "memo inválido" }));
    }
    if payment.amount != payload.amount {
        return Err(serde_json::json!({ "error": "valor divergente" }));
    }
    if Utc::now() > payment.expires_at {
        return Err(serde_json::json!({ "error": "pagamento expirado" }));
    }
    Ok(())
}

fn verificar_risco(amount: Decimal) -> Result<u8, serde_json::Value> {
    let limite = Decimal::from(10000);
    let risco = if amount > limite { 90u8 } else { 10u8 };
    if risco > 80 {
        return Err(serde_json::json!({ "error": "suspeita de fraude" }));
    }
    Ok(risco)
}

async fn webhook_confirm(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Json(payload): Json<ConfirmRequest>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(erro_nao_autorizado().0);
    }

    // 1. Busca payment
    let payment = match governance::buscar_payment(&pool, &payload.payment_id).await {
        Some(p) => p,
        None => return Json(serde_json::json!({ "error": "payment_id inválido" })),
    };

    // 2. Valida payment
    if let Err(e) = validar_payment(&payment, &payload) {
        return Json(e);
    }

    // 3. Verifica TX on-chain
    let cliente = services::inicializar_cliente(&get_rpc_url());
    let verificacao = services::verificar_transacao(&cliente, &payload.tx_hash);

    if !verificacao.valida {
        error!("TX inválida: {}", payload.tx_hash);
        return Json(serde_json::json!({
            "error": format!(
                "transação inválida on-chain: {}",
                verificacao.erro.unwrap_or_default()
            )
        }));
    }

    // 4. Verifica risco
    let risco = match verificar_risco(payload.amount) {
        Ok(r) => r,
        Err(e) => return Json(e),
    };

    // 5. Atualiza status
    governance::atualizar_status_payment(&pool, &payload.payment_id, "paid").await;

    info!("Payment confirmado: {}", payload.payment_id);

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
        return Json(erro_nao_autorizado().0);
    }

    match governance::buscar_payment(&pool, &id).await {
        Some(payment) => Json(serde_json::json!({
            "payment_id": payment.payment_id,
            "status": payment.status,
            "amount": payment.amount,
            "merchant_id": payment.merchant_id,
            "expires_at": payment.expires_at.to_rfc3339()
        })),
        None => Json(serde_json::json!({ "error": "payment não encontrado" })),
    }
}

/// Off-ramp PIX. Só aceita payments já confirmados on-chain (status "paid").
/// O valor_usdc usado é o valor CONFIRMADO em `governance`, nunca o que
/// o cliente manda no payload — isso evita que alguém com a API key
/// force um off-ramp com valor inventado.
async fn pix_offramp(
    headers: HeaderMap,
    State(pool): State<AppState>,
    Json(payload): Json<PixRequest>,
) -> impl IntoResponse {
    if !autenticar(&headers) {
        return Json(erro_nao_autorizado().0);
    }

    let payment = match governance::buscar_payment(&pool, &payload.payment_id).await {
        Some(p) => p,
        None => return Json(serde_json::json!({ "error": "payment_id inválido" })),
    };

    if payment.status != "paid" {
        return Json(serde_json::json!({
            "error": format!(
                "payment {} ainda não confirmado on-chain (status atual: {})",
                payment.payment_id, payment.status
            )
        }));
    }

    if payload.taxa_cambio <= Decimal::ZERO {
        return Json(serde_json::json!({ "error": "taxa_cambio inválida" }));
    }

    let pedido = pix::criar_pedido_pix(
        &payment.payment_id,
        &payment.merchant_id,
        &payload.chave_pix,
        payment.amount, // valor confirmado on-chain, não o payload do cliente
        payload.taxa_cambio,
    );

    let resultado = pix::enviar_para_vasp(&pedido).await;

    info!("PIX off-ramp: {}", pedido.id);

    Json(serde_json::json!({
        "sucesso": resultado.sucesso,
        "pedido_id": resultado.pedido_id,
        "valor_brl": resultado.valor_brl,
        "valor_liquido_brl": resultado.valor_liquido_brl,
        "mensagem": resultado.mensagem,
        "vasp_tx_id": resultado.vasp_tx_id,
        "eta_segundos": resultado.eta_segundos,
    }))
}

pub async fn iniciar_servidor(
    pool: Pool<Postgres>,
    host: String,
    port: String,
    _solana_rpc: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .route("/saldo", post(saldo))
        .route("/antifraude", post(antifraude))
        .route("/checkout", post(checkout))
        .route("/webhook/confirm", post(webhook_confirm))
        .route("/payment/:id", get(get_payment))
        .route("/pix/offramp", post(pix_offramp))
        .layer(cors)
        .with_state(pool);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().map_err(|e| {
        error!("Endereço inválido: {}", e);
        e
    })?;

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        error!("Erro ao bind: {}", e);
        e
    })?;

    info!("✅ SlipPay 2.0 pronto em http://{}", addr);

    axum::serve(listener, app).await.map_err(|e| {
        error!("Erro no servidor: {}", e);
        e
    })?;

    Ok(())
}
