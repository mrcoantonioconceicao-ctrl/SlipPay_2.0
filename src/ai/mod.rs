use crate::interface::Transacao;

/// Função simples de análise antifraude
pub fn analise_antifraude(transacoes: Vec<Transacao>) -> String {
    if transacoes.iter().any(|t| t.valor > 10000.0) {
        "Suspeita de fraude".to_string()
    } else {
        "Transações normais".to_string()
    }
}
