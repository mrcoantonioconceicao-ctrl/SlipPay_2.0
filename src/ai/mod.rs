use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::interface::Transacao;

/// Score de risco de uma transação
pub struct RiscoTransacao {
    pub score: u8,          // 0-100
    pub nivel: String,      // baixo, medio, alto, critico
    pub motivos: Vec<String>,
    pub aprovada: bool,
}

/// Analisa uma lista de transações — retorna resultado simples
pub fn analise_antifraude(transacoes: Vec<Transacao>) -> String {
    if transacoes.iter().any(|t| t.valor > dec!(10000)) {
        "Suspeita de fraude".to_string()
    } else {
        "Transações normais".to_string()
    }
}

/// Análise avançada de risco por transação individual
pub fn analisar_risco(
    valor: Decimal,
    historico: &[Decimal],
    wallet: &str,
    network: &str,
) -> RiscoTransacao {
    let mut score: u8 = 0;
    let mut motivos: Vec<String> = Vec::new();

    // Fator 1: valor absoluto
    if valor > dec!(50000) {
        score = score.saturating_add(40);
        motivos.push("valor acima de $50.000".to_string());
    } else if valor > dec!(10000) {
        score = score.saturating_add(25);
        motivos.push("valor acima de $10.000".to_string());
    } else if valor > dec!(5000) {
        score = score.saturating_add(10);
        motivos.push("valor acima de $5.000".to_string());
    }

    // Fator 2: desvio em relação ao histórico
    if !historico.is_empty() {
        let media: Decimal = historico.iter().sum::<Decimal>()
            / Decimal::from(historico.len() as u64);

        if media > dec!(0) {
            let desvio = ((valor - media) / media * dec!(100)).abs();

            if desvio > dec!(500) {
                score = score.saturating_add(30);
                motivos.push(format!(
                    "valor {}% acima da média histórica",
                    desvio.round()
                ));
            } else if desvio > dec!(200) {
                score = score.saturating_add(15);
                motivos.push(format!(
                    "valor {}% acima da média histórica",
                    desvio.round()
                ));
            }
        }
    }

    // Fator 3: wallet suspeita (endereço muito curto ou padrão)
    if wallet.len() < 32 {
        score = score.saturating_add(20);
        motivos.push("wallet com formato suspeito".to_string());
    }

    // Fator 4: network desconhecida
    if network != "solana" && network != "stellar" {
        score = score.saturating_add(15);
        motivos.push(format!("network desconhecida: {}", network));
    }

    // Fator 5: volume alto no histórico recente
    if historico.len() >= 5 {
        let soma_recente: Decimal = historico
            .iter()
            .rev()
            .take(5)
            .sum();

        if soma_recente > dec!(100000) {
            score = score.saturating_add(20);
            motivos.push("volume alto nas últimas 5 transações".to_string());
        }
    }

    let nivel = match score {
        0..=20 => "baixo".to_string(),
        21..=50 => "medio".to_string(),
        51..=80 => "alto".to_string(),
        _ => "critico".to_string(),
    };

    let aprovada = score <= 80;

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
