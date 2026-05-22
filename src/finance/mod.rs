use rust_decimal::Decimal;
use rust_decimal_macros::dec;

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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_valor_total() {
        let valor = dec!(100);
        let taxa = dec!(5); // 5%
        assert_eq!(valor_total(valor, taxa), dec!(105));
    }

    #[test]
    fn test_aplicar_desconto() {
        let valor = dec!(200);
        let desconto = dec!(10); // 10%
        assert_eq!(aplicar_desconto(valor, desconto), dec!(180));
    }

    #[test]
    fn test_converter_moeda() {
        let valor = dec!(50);
        let taxa_cambio = dec!(4.5); // exemplo: USD → BRL
        assert_eq!(converter_moeda(valor, taxa_cambio), dec!(225));
    }
}
