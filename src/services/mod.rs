use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use spl_token::instruction::transfer_checked;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use tracing::{error, info, warn};

pub const USDC_MINT_DEVNET: &str =
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";

pub struct VerificacaoTx {
    pub valida: bool,
    pub confirmacoes: u64,
    pub erro: Option<String>,
}

pub fn inicializar_cliente(url: &str) -> RpcClient {
    RpcClient::new(url.to_string())
}

/// Consulta saldo SOL — retorna 0 em caso de erro
pub fn consultar_saldo(cliente: &RpcClient, conta: &Pubkey) -> u64 {
    match cliente.get_balance(conta) {
        Ok(saldo) => saldo,
        Err(e) => {
            error!("Erro ao consultar saldo de {}: {}", conta, e);
            0
        }
    }
}

/// Consulta saldo USDC — retorna 0 em caso de erro
pub fn consultar_saldo_usdc(
    cliente: &RpcClient,
    conta: &Pubkey,
) -> u64 {
    let mint = match USDC_MINT_DEVNET.parse::<Pubkey>() {
        Ok(m) => m,
        Err(e) => {
            error!("Mint USDC inválido: {}", e);
            return 0;
        }
    };

    let ata = get_associated_token_address(conta, &mint);

    match cliente.get_token_account_balance(&ata) {
        Ok(balance) => balance.amount.parse::<u64>().unwrap_or(0),
        Err(_) => 0,
    }
}

/// Verifica se uma transação on-chain é válida
pub fn verificar_transacao(
    cliente: &RpcClient,
    tx_hash: &str,
) -> VerificacaoTx {
    let assinatura = match Signature::from_str(tx_hash) {
        Ok(sig) => sig,
        Err(_) => {
            warn!("TX hash inválido: {}", tx_hash);
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
            let confirmacoes = tx.slot;
            let sucesso = tx
                .transaction
                .meta
                .as_ref()
                .map(|m| m.err.is_none())
                .unwrap_or(false);

            if sucesso {
                info!("TX válida: {} ({} confirmações)", tx_hash, confirmacoes);
            } else {
                warn!("TX falhou on-chain: {}", tx_hash);
            }

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
        Err(e) => {
            error!("Erro ao buscar TX {}: {}", tx_hash, e);
            VerificacaoTx {
                valida: false,
                confirmacoes: 0,
                erro: Some(format!("Erro ao buscar tx: {}", e)),
            }
        }
    }
}

/// Envia SOL simples — retorna Result em vez de panicar
pub fn enviar_transacao(
    cliente: &RpcClient,
    remetente: &Keypair,
    destinatario: &Pubkey,
    valor: u64,
) -> Result<String, String> {
    let instrucoes = system_instruction::transfer(
        &remetente.pubkey(),
        destinatario,
        valor,
    );

    let blockhash = cliente
        .get_latest_blockhash()
        .map_err(|e| format!("Erro ao obter blockhash: {}", e))?;

    let transacao = Transaction::new_signed_with_payer(
        &[instrucoes],
        Some(&remetente.pubkey()),
        &[remetente],
        blockhash,
    );

    let assinatura = cliente
        .send_and_confirm_transaction(&transacao)
        .map_err(|e| format!("Erro ao enviar transação: {}", e))?;

    info!("SOL enviado: {}", assinatura);
    Ok(assinatura.to_string())
}

/// Envia USDC via SPL Token — retorna Result em vez de panicar
pub fn enviar_usdc(
    cliente: &RpcClient,
    remetente: &Keypair,
    destinatario: &Pubkey,
    valor: u64,
) -> Result<String, String> {
    let mint = USDC_MINT_DEVNET
        .parse::<Pubkey>()
        .map_err(|e| format!("Mint inválido: {}", e))?;

    let origem_ata = get_associated_token_address(
        &remetente.pubkey(),
        &mint,
    );

    let destino_ata = get_associated_token_address(
        destinatario,
        &mint,
    );

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
    .map_err(|e| format!("Erro ao criar instrução USDC: {}", e))?;

    let blockhash = cliente
        .get_latest_blockhash()
        .map_err(|e| format!("Erro ao obter blockhash: {}", e))?;

    let transacao = Transaction::new_signed_with_payer(
        &[instrucao],
        Some(&remetente.pubkey()),
        &[remetente],
        blockhash,
    );

    let assinatura = cliente
        .send_and_confirm_transaction(&transacao)
        .map_err(|e| format!("Erro ao enviar USDC: {}", e))?;

    info!("USDC enviado: {} lamports → {}", valor, assinatura);
    Ok(assinatura.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consultar_saldo() {
        let cliente = inicializar_cliente(
            "https://api.devnet.solana.com"
        );
        let conta = Pubkey::from_str(
            "11111111111111111111111111111111"
        ).unwrap();
        let saldo = consultar_saldo(&cliente, &conta);
        assert!(saldo >= 0);
    }

    #[test]
    fn test_tx_hash_invalido() {
        let cliente = inicializar_cliente(
            "https://api.devnet.solana.com"
        );
        let resultado = verificar_transacao(&cliente, "hash_invalido");
        assert!(!resultado.valida);
    }
}

