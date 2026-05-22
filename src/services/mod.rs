use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;

/// Inicializa cliente RPC da Solana
pub fn inicializar_cliente(url: &str) -> RpcClient {
    RpcClient::new(url.to_string())
}

/// Consulta saldo de uma conta
pub fn consultar_saldo(cliente: &RpcClient, conta: &Pubkey) -> u64 {
    cliente.get_balance(conta).expect("Erro ao consultar saldo")
}

/// Envia uma transação simples de transferência
pub fn enviar_transacao(
    cliente: &RpcClient,
    remetente: &Keypair,
    destinatario: &Pubkey,
    valor: u64,
) -> String {
    let instrucoes = system_instruction::transfer(&remetente.pubkey(), destinatario, valor);

    let mut transacao = Transaction::new_with_payer(&[instrucoes], Some(&remetente.pubkey()));

    let (recent_blockhash, _) = cliente
        .get_recent_blockhash()
        .expect("Erro ao obter blockhash");

    transacao.sign(&[remetente], recent_blockhash);

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
        let cliente = inicializar_cliente("https://api.devnet.solana.com");
        let conta = Pubkey::from_str("11111111111111111111111111111111").unwrap();
        let saldo = consultar_saldo(&cliente, &conta);
        assert!(saldo >= 0);
    }
}
