use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer_checked;
use std::env;
use std::str::FromStr;
use tracing::{error, info, warn};

pub const USDC_MINT_DEVNET: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

pub const USDC_MINT_MAINNET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[derive(Debug, Clone)]
pub struct VerificacaoTx {
    pub valida: bool,
    pub confirmacoes: u64,
    pub erro: Option<String>,
}

pub fn inicializar_cliente(url: &str) -> RpcClient {
    RpcClient::new(url.to_string())
}

pub fn obter_mint_usdc() -> Result<Pubkey, String> {
    let network = env::var("SOLANA_NETWORK").unwrap_or_else(|_| "devnet".to_string());

    let mint = if network == "mainnet" {
        USDC_MINT_MAINNET
    } else {
        USDC_MINT_DEVNET
    };

    mint.parse::<Pubkey>()
        .map_err(|e| format!("Mint inválido: {}", e))
}

/// Consulta saldo SOL
pub fn consultar_saldo(cliente: &RpcClient, conta: &Pubkey) -> u64 {
    match cliente.get_balance(conta) {
        Ok(saldo) => saldo,
        Err(e) => {
            error!("Erro ao consultar saldo {}: {}", conta, e);
            0
        }
    }
}

/// Consulta saldo USDC
pub fn consultar_saldo_usdc(cliente: &RpcClient, conta: &Pubkey) -> u64 {
    let mint = match obter_mint_usdc() {
        Ok(m) => m,
        Err(e) => {
            error!("{}", e);
            return 0;
        }
    };

    let ata = get_associated_token_address(conta, &mint);

    match cliente.get_token_account_balance(&ata) {
        Ok(balance) => balance.amount.parse::<u64>().unwrap_or(0),
        Err(e) => {
            warn!("Erro ao consultar saldo USDC {}: {}", conta, e);
            0
        }
    }
}

/// Verifica transação existente
pub fn verificar_transacao(cliente: &RpcClient, tx_hash: &str) -> VerificacaoTx {
    let assinatura = match Signature::from_str(tx_hash) {
        Ok(sig) => sig,
        Err(_) => {
            return VerificacaoTx {
                valida: false,
                confirmacoes: 0,
                erro: Some("tx_hash inválido".to_string()),
            };
        }
    };

    match cliente.get_transaction(
        &assinatura,
        solana_transaction_status::UiTransactionEncoding::Json,
    ) {
        Ok(tx) => {
            let sucesso = tx
                .transaction
                .meta
                .as_ref()
                .map(|m| m.err.is_none())
                .unwrap_or(false);

            let confirmacoes = tx.slot;

            VerificacaoTx {
                valida: sucesso,
                confirmacoes,
                erro: if sucesso {
                    None
                } else {
                    Some("transação falhou on-chain".to_string())
                },
            }
        }
        Err(e) => VerificacaoTx {
            valida: false,
            confirmacoes: 0,
            erro: Some(format!("Erro ao buscar tx: {}", e)),
        },
    }
}

/// Envia SOL
pub fn enviar_transacao(
    cliente: &RpcClient,
    remetente: &Keypair,
    destinatario: &Pubkey,
    valor: u64,
) -> Result<String, String> {
    if valor == 0 {
        return Err("valor deve ser maior que zero".to_string());
    }

    let instrucoes = system_instruction::transfer(&remetente.pubkey(), destinatario, valor);

    let blockhash = cliente.get_latest_blockhash().map_err(|e| e.to_string())?;

    let tx = Transaction::new_signed_with_payer(
        &[instrucoes],
        Some(&remetente.pubkey()),
        &[remetente],
        blockhash,
    );

    let assinatura = cliente
        .send_and_confirm_transaction(&tx)
        .map_err(|e| e.to_string())?;

    info!("SOL enviado {}", assinatura);

    Ok(assinatura.to_string())
}

/// Envia USDC
pub fn enviar_usdc(
    cliente: &RpcClient,
    remetente: &Keypair,
    destinatario: &Pubkey,
    valor: u64,
) -> Result<String, String> {
    if valor == 0 {
        return Err("valor deve ser maior que zero".to_string());
    }

    let mint = obter_mint_usdc()?;

    let origem_ata = get_associated_token_address(&remetente.pubkey(), &mint);

    let destino_ata = get_associated_token_address(destinatario, &mint);

    let instrucao = transfer_checked(
        &spl_token::id(),
        &origem_ata,
        &mint,
        &destino_ata,
        &remetente.pubkey(),
        &[],
        valor,
        6,
    )
    .map_err(|e| e.to_string())?;

    let blockhash = cliente.get_latest_blockhash().map_err(|e| e.to_string())?;

    let tx = Transaction::new_signed_with_payer(
        &[instrucao],
        Some(&remetente.pubkey()),
        &[remetente],
        blockhash,
    );

    let assinatura = cliente
        .send_and_confirm_transaction(&tx)
        .map_err(|e| e.to_string())?;

    info!("USDC enviado {} -> {}", valor, assinatura);

    Ok(assinatura.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_hash_invalido() {
        let cliente = inicializar_cliente("https://api.devnet.solana.com");

        let resultado = verificar_transacao(&cliente, "hash_invalido");

        assert!(!resultado.valida);
    }

    #[test]
    fn test_mint_usdc() {
        let resultado = obter_mint_usdc();

        assert!(resultado.is_ok());
    }
}
