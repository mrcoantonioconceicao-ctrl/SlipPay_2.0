use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Taxa SlipPay: 1.5%
pub const TAXA_SLIPPAY: Decimal = dec!(1.5);

/// Calcula a taxa sobre um valor
pub fn calcular_taxa(valor: Decimal, taxa_percentual: Decimal) -> Decimal {
    valor * (taxa_percentual / dec!(100))
}

/// Retorna o valor total com a taxa aplicada
pub fn valor_total(valor: Decimal, taxa_percentual: Decimal) -> Decimal {
    valor + calcular_taxa(valor, taxa_percentual)
}

/// Aplica desconto sobre um valor
pub fn aplicar_desconto(valor: Decimal, desconto_percentual: Decimal) -> Decimal {
    valor - (valor * (desconto_percentual / dec!(100)))
}

/// Converte valores para uma moeda com taxa de câmbio
pub fn converter_moeda(valor: Decimal, taxa_cambio: Decimal) -> Decimal {
    valor * taxa_cambio
}

/// Breakdown completo de um pagamento SlipPay
pub struct BreakdownPagamento {
    pub valor_original: Decimal,
    pub taxa_slippay: Decimal,
    pub valor_merchant: Decimal,
    pub total_payer: Decimal,
}

/// Calcula o breakdown completo de um pagamento
pub fn calcular_breakdown(valor: Decimal) -> BreakdownPagamento {
    let taxa = calcular_taxa(valor, TAXA_SLIPPAY);
    let valor_merchant = valor - taxa;

    BreakdownPagamento {
        valor_original: valor,
        taxa_slippay: taxa,
        valor_merchant,
        total_payer: valor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valor_total() {
        let valor = dec!(100);
        let taxa = dec!(5);
        assert_eq!(valor_total(valor, taxa), dec!(105));
    }

    #[test]
    fn test_aplicar_desconto() {
        let valor = dec!(200);
        let desconto = dec!(10);
        assert_eq!(aplicar_desconto(valor, desconto), dec!(180));
    }

    #[test]
    fn test_converter_moeda() {
        let valor = dec!(50);
        let taxa_cambio = dec!(4.5);
        assert_eq!(converter_moeda(valor, taxa_cambio), dec!(225));
    }

    #[test]
    fn test_taxa_slippay() {
        let valor = dec!(100);
        let breakdown = calcular_breakdown(valor);
        assert_eq!(breakdown.taxa_slippay, dec!(1.5));
        assert_eq!(breakdown.valor_merchant, dec!(98.5));
    }

    #[test]
    fn test_breakdown_1000_usdc() {
        let valor = dec!(1000);
        let breakdown = calcular_breakdown(valor);
        assert_eq!(breakdown.taxa_slippay, dec!(15));
        assert_eq!(breakdown.valor_merchant, dec!(985));
    }
}
