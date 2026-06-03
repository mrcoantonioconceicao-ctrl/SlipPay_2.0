use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use tracing::error;

/// Gera par de chaves Ed25519
pub fn gerar_chaves() -> (PublicKey, SecretKey) {
    let mut csprng = OsRng;
    let keypair = Keypair::generate(&mut csprng);

    (keypair.public, keypair.secret)
}

/// Assina mensagem
pub fn assinar_mensagem(mensagem: &[u8], chave_privada: &SecretKey) -> Option<Signature> {
    let secret_bytes = chave_privada.to_bytes();
    let public_key = PublicKey::from(chave_privada);

    let keypair_bytes = [secret_bytes.as_ref(), public_key.as_bytes()].concat();

    match Keypair::from_bytes(&keypair_bytes) {
        Ok(keypair) => Some(keypair.sign(mensagem)),
        Err(e) => {
            error!("Erro ao reconstruir keypair: {}", e);
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

/// SHA256
pub fn gerar_hash(mensagem: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(mensagem);
    hasher.finalize().to_vec()
}

/// Comparação resistente a timing attack
pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff = 0u8;

    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }

    diff == 0
}

/// AES-256-GCM Encrypt
pub fn criptografar(dados: &[u8], chave: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>, String> {
    let cipher =
        Aes256Gcm::new_from_slice(chave).map_err(|e| format!("Erro ao criar cipher: {}", e))?;

    let nonce = Nonce::clone_from_slice(nonce);

    cipher.encrypt(&nonce, dados).map_err(|e| {
        error!("Erro ao criptografar: {}", e);
        format!("Erro ao criptografar: {}", e)
    })
}

/// AES-256-GCM Decrypt
pub fn descriptografar(
    dados: &[u8],
    chave: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, String> {
    let cipher =
        Aes256Gcm::new_from_slice(chave).map_err(|e| format!("Erro ao criar cipher: {}", e))?;

    let nonce = Nonce::clone_from_slice(nonce);

    cipher.decrypt(&nonce, dados).map_err(|e| {
        error!("Erro ao descriptografar: {}", e);
        format!("Erro ao descriptografar: {}", e)
    })
}

/// API Keys válidas (MVP)
const API_KEYS_VALIDAS: [&str; 2] = ["slippay-dev-key-2026", "slippay-merchant-key-001"];

/// Validação de API Key
pub fn validar_api_key(api_key: &str) -> bool {
    API_KEYS_VALIDAS.iter().any(|k| secure_compare(api_key, k))
}

/// Geração determinística de API Key
pub fn gerar_api_key(merchant_id: &str, secret: &str) -> String {
    let input = format!("slippay:{}:{}", merchant_id, secret);

    let hash = gerar_hash(input.as_bytes());

    format!("sp_{}", hex::encode(&hash[..16]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assinatura_valida() {
        let (pk, sk) = gerar_chaves();

        let msg = b"SlipPay Security";

        let assinatura = assinar_mensagem(msg, &sk).unwrap();

        assert!(verificar_mensagem(msg, &assinatura, &pk));
    }

    #[test]
    fn test_hash() {
        let hash = gerar_hash(b"teste");

        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_secure_compare() {
        assert!(secure_compare("abc", "abc"));

        assert!(!secure_compare("abc", "xyz"));
    }

    #[test]
    fn test_criptografia() {
        let chave = [0u8; 32];
        let nonce = [1u8; 12];

        let dados = b"SlipPay Secret";

        let enc = criptografar(dados, &chave, &nonce).unwrap();

        let dec = descriptografar(&enc, &chave, &nonce).unwrap();

        assert_eq!(dec, dados);
    }

    #[test]
    fn test_api_key_valida() {
        assert!(validar_api_key("slippay-dev-key-2026"));
    }

    #[test]
    fn test_api_key_invalida() {
        assert!(!validar_api_key("fake-key"));
    }

    #[test]
    fn test_gerar_api_key() {
        let key = gerar_api_key("merchant01", "secret");

        assert!(key.starts_with("sp_"));
    }
}
