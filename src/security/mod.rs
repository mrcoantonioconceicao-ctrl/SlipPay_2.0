use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use aes_gcm::{
    aead::Aead,
    Aes256Gcm,
    Key,
    KeyInit,
    Nonce,
};
use tracing::error;

/// Gera um par de chaves Ed25519
pub fn gerar_chaves() -> (PublicKey, SecretKey) {
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);
    (keypair.public, keypair.secret)
}

/// Assina uma mensagem com a chave privada
/// Retorna Option em vez de panicar
pub fn assinar_mensagem(
    mensagem: &[u8],
    chave_privada: &SecretKey,
) -> Option<Signature> {
    let secret_bytes = chave_privada.to_bytes();
    let public_key = PublicKey::from(chave_privada);
    let keypair_bytes = [
        secret_bytes.as_ref(),
        public_key.as_bytes(),
    ]
    .concat();

    match Keypair::from_bytes(&keypair_bytes) {
        Ok(keypair) => Some(keypair.sign(mensagem)),
        Err(e) => {
            error!("Erro ao reconstruir Keypair: {}", e);
            None
        }
    }
}

/// Verifica assinatura
pub fn verificar_mensagem(
    mensagem: &[u8],
    assinatura: &Signature,
    chave_publica: &PublicKey,
) -> bool {
    chave_publica.verify(mensagem, assinatura).is_ok()
}

/// Gera SHA-256
pub fn gerar_hash(mensagem: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(mensagem);
    hasher.finalize().to_vec()
}

/// AES-256-GCM encrypt — retorna Result em vez de panicar
pub fn criptografar(
    dados: &[u8],
    chave: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(chave);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    cipher.encrypt(nonce, dados)
        .map_err(|e| {
            error!("Erro ao criptografar: {}", e);
            format!("Erro ao criptografar: {}", e)
        })
}

/// AES-256-GCM decrypt — retorna Result em vez de panicar
pub fn descriptografar(
    dados: &[u8],
    chave: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, String> {
    let key = Key::<Aes256Gcm>::from_slice(chave);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce);

    cipher.decrypt(nonce, dados)
        .map_err(|e| {
            error!("Erro ao descriptografar: {}", e);
            format!("Erro ao descriptografar: {}", e)
        })
}

/// Valida API Key recebida no header
pub fn validar_api_key(api_key: &str) -> bool {
    let keys_validas = vec![
        "slippay-dev-key-2026",
        "slippay-merchant-key-001",
    ];
    keys_validas.contains(&api_key)
}

/// Gera uma API Key baseada em hash SHA-256
pub fn gerar_api_key(merchant_id: &str, secret: &str) -> String {
    let input = format!("{}:{}", merchant_id, secret);
    let hash = gerar_hash(input.as_bytes());
    hex::encode(&hash[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assinatura_valida() {
        let (pk, sk) = gerar_chaves();
        let mensagem = "SlipPay 2.0 - Segurança".as_bytes();
        let assinatura = assinar_mensagem(mensagem, &sk)
            .expect("Falha ao assinar");
        assert!(verificar_mensagem(mensagem, &assinatura, &pk));
    }

    #[test]
    fn test_hash() {
        let mensagem = "Teste de hash".as_bytes();
        let hash = gerar_hash(mensagem);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_criptografia_simetrica() {
        let chave: [u8; 32] = [0; 32];
        let nonce: [u8; 12] = [1; 12];
        let dados = "SlipPay dados sensíveis".as_bytes();
        let criptografado = criptografar(dados, &chave, &nonce)
            .expect("Falha ao criptografar");
        let descriptografado = descriptografar(&criptografado, &chave, &nonce)
            .expect("Falha ao descriptografar");
        assert_eq!(descriptografado, dados);
    }

    #[test]
    fn test_api_key_valida() {
        assert!(validar_api_key("slippay-dev-key-2026"));
    }

    #[test]
    fn test_api_key_invalida() {
        assert!(!validar_api_key("chave-falsa"));
    }
}
