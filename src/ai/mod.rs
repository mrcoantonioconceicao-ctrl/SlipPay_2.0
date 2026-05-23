use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::interface::Transacao;

pub fn analise_antifraude(transacoes: Vec<Transacao>) -> String {
    if transacoes.iter().any(|t| t.valor > dec!(10000)) {
        "Suspeita de fraude".to_string()
    } else {
        "Transações normais".to_string()
    }
}
