use crate::interface::Transacao;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

pub struct RiscoTransacao {
    pub score: u8,
    pub nivel: String,
    pub motivos: Vec<String>,
    pub aprovada: bool,
}

pub fn analise_antifraude(transacoes: Vec<Transacao>) -> String {
    if transacoes.iter().any(|t| t.valor > dec!(10000)) {
        "Suspeita de fraude".to_string()
    } else {
        "Transações normais".to_string()
    }
}

/// Fator 1: score por valor absoluto
fn score_valor(valor: Decimal, motivos: &mut Vec<String>) -> u8 {
    if valor > dec!(50000) {
        motivos.push("valor acima de $50.000".to_string());
        40
    } else if valor > dec!(10000) {
        motivos.push("valor acima de $10.000".to_string());
        25
    } else if valor > dec!(5000) {
        motivos.push("valor acima de $5.000".to_string());
        10
    } else {
        0
    }
}

/// Fator 2: score por desvio histórico
fn score_historico(valor: Decimal, historico: &[Decimal], motivos: &mut Vec<String>) -> u8 {
    if historico.is_empty() {
        return 0;
    }

    let media: Decimal = historico.iter().sum::<Decimal>() / Decimal::from(historico.len() as u64);

    if media <= dec!(0) {
        return 0;
    }

    let desvio = ((valor - media) / media * dec!(100)).abs();

    if desvio > dec!(500) {
        motivos.push(format!(
            "valor {}% acima da média histórica",
            desvio.round()
        ));
        30
    } else if desvio > dec!(200) {
        motivos.push(format!(
            "valor {}% acima da média histórica",
            desvio.round()
        ));
        15
    } else {
        0
    }
}

/// Fator 3: score por formato de wallet
fn score_wallet(wallet: &str, motivos: &mut Vec<String>) -> u8 {
    if wallet.len() < 32 {
        motivos.push("wallet com formato suspeito".to_string());
        20
    } else {
        0
    }
}

/// Fator 4: score por network
fn score_network(network: &str, motivos: &mut Vec<String>) -> u8 {
    if network != "solana" && network != "stellar" {
        motivos.push(format!("network desconhecida: {}", network));
        15
    } else {
        0
    }
}

/// Fator 5: score por volume recente
fn score_volume_recente(historico: &[Decimal], motivos: &mut Vec<String>) -> u8 {
    if historico.len() < 5 {
        return 0;
    }

    let soma_recente: Decimal = historico.iter().rev().take(5).sum();

    if soma_recente > dec!(100000) {
        motivos.push("volume alto nas últimas 5 transações".to_string());
        20
    } else {
        0
    }
}

/// Converte score em nível textual
fn score_para_nivel(score: u8) -> String {
    match score {
        0..=20 => "baixo".to_string(),
        21..=50 => "medio".to_string(),
        51..=80 => "alto".to_string(),
        _ => "critico".to_string(),
    }
}

/// Análise avançada de risco — subfunções extraídas (Orion)
pub fn analisar_risco(
    valor: Decimal,
    historico: &[Decimal],
    wallet: &str,
    network: &str,
) -> RiscoTransacao {
    let mut motivos: Vec<String> = Vec::new();

    let score: u8 = [
        score_valor(valor, &mut motivos),
        score_historico(valor, historico, &mut motivos),
        score_wallet(wallet, &mut motivos),
        score_network(network, &mut motivos),
        score_volume_recente(historico, &mut motivos),
    ]
    .iter()
    .fold(0u8, |acc, &s| acc.saturating_add(s));

    let nivel = score_para_nivel(score);
    let aprovada = score <= 80;

    info!("Risco analisado: score={} nivel={}", score, nivel);

    RiscoTransacao {
        score,
        nivel,
        motivos,
        aprovada,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risco_baixo() {
        let resultado = analisar_risco(
            dec!(100),
            &[dec!(90), dec!(110), dec!(95)],
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "solana",
        );
        assert_eq!(resultado.nivel, "baixo");
        assert!(resultado.aprovada);
    }

    #[test]
    fn test_risco_alto_valor() {
        let resultado = analisar_risco(
            dec!(60000),
            &[],
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "solana",
        );
        assert!(resultado.score >= 40);
    }

    #[test]
    fn test_risco_desvio_historico() {
        let resultado = analisar_risco(
            dec!(10000),
            &[dec!(100), dec!(120), dec!(90)],
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "solana",
        );
        assert!(resultado.score >= 25);
        assert!(!resultado.motivos.is_empty());
    }

    #[test]
    fn test_risco_network_desconhecida() {
        let resultado = analisar_risco(
            dec!(100),
            &[],
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
            "ethereum",
        );
        assert!(resultado.score >= 15);
    }
}
