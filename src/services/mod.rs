use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;

pub fn inicializar_cliente(url: &str) -> RpcClient {
    RpcClient::new(url.to_string())
}

pub fn consultar_saldo(cliente: &RpcClient, conta: &Pubkey) -> u64 {
    cliente.get_balance(conta).expect("Erro ao consultar saldo")
}

pub fn enviar_transacao(
    cliente: &RpcClient,
    remetente: &Keypair,
    destinatario: &Pubkey,
    valor: u64,
) -> String {
    let instrucoes = system_instruction::transfer(
        &remetente.pubkey(),
        destinatario,
        valor,
    );

    let blockhash = cliente
        .get_latest_blockhash()
        .expect("Erro ao obter blockhash");

    let transacao = Transaction::new_signed_with_payer(
        &[instrucoes],
        Some(&remetente.pubkey()),
        &[remetente],
        blockhash,
    );

    let assinatura = cliente
        .send_and_confirm_transaction(&transacao)
        .expect("Erro ao enviar transação");

    assinatura.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

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
}
