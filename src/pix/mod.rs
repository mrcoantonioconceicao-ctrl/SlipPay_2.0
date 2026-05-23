use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use uuid::Uuid;

/// Status de um pedido de off-ramp PIX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusPix {
    Pendente,
    Processando,
    Concluido,
    Falhou,
}

/// Pedido de conversão USDC → BRL via PIX
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedidoPix {
    pub id: String,
    pub payment_id: String,
    pub merchant_id: String,
    pub chave_pix: String,
    pub valor_usdc: Decimal,
    pub taxa_cambio: Decimal,
    pub valor_brl: Decimal,
    pub taxa_vasp: Decimal,
    pub valor_liquido_brl: Decimal,
    pub status: StatusPix,
    pub criado_em: DateTime<Utc>,
}

/// Resultado do processamento PIX
#[derive(Debug, Serialize)]
pub struct ResultadoPix {
    pub sucesso: bool,
    pub pedido_id: String,
    pub valor_brl: Decimal,
    pub valor_liquido_brl: Decimal,
    pub mensagem: String,
}

/// Taxa do VASP parceiro: 0.5%
pub const TAXA_VASP: Decimal = dec!(0.5);

/// Calcula o valor em BRL dado USDC e taxa de câmbio
pub fn calcular_valor_brl(
    valor_usdc: Decimal,
    taxa_cambio: Decimal,
) -> Decimal {
    valor_usdc * taxa_cambio
}

/// Calcula taxa do VASP
pub fn calcular_taxa_vasp(valor_brl: Decimal) -> Decimal {
    valor_brl * (TAXA_VASP / dec!(100))
}

/// Cria um pedido de off-ramp PIX
pub fn criar_pedido_pix(
    payment_id: &str,
    merchant_id: &str,
    chave_pix: &str,
    valor_usdc: Decimal,
    taxa_cambio: Decimal,
) -> PedidoPix {
    let valor_brl = calcular_valor_brl(valor_usdc, taxa_cambio);
    let taxa_vasp = calcular_taxa_vasp(valor_brl);
    let valor_liquido_brl = valor_brl - taxa_vasp;

    PedidoPix {
        id: Uuid::new_v4().to_string(),
        payment_id: payment_id.to_string(),
        merchant_id: merchant_id.to_string(),
        chave_pix: chave_pix.to_string(),
        valor_usdc,
        taxa_cambio,
        valor_brl,
        taxa_vasp,
        valor_liquido_brl,
        status: StatusPix::Pendente,
        criado_em: Utc::now(),
    }
}

/// Simula envio do pedido ao VASP parceiro. Em produção faria chamada HTTP ao VASP.
pub async fn enviar_para_vasp(
    pedido: &PedidoPix,
) -> ResultadoPix {
    if pedido.chave_pix.is_empty() {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: "Chave PIX inválida".to_string(),
        };
    }

    if pedido.valor_usdc <= dec!(0) {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: "Valor inválido".to_string(),
        };
    }

    ResultadoPix {
        sucesso: true,
        pedido_id: pedido.id.clone(),
        valor_brl: pedido.valor_brl,
        valor_liquido_brl: pedido.valor_liquido_brl,
        mensagem: format!(
            "PIX de R$ {:.2} enviado para {}",
            pedido.valor_liquido_brl,
            pedido.chave_pix
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calcular_valor_brl() {
        let usdc = dec!(100);
        let cambio = dec!(5.20);
        let brl = calcular_valor_brl(usdc, cambio);
        assert_eq!(brl, dec!(520));
    }

    #[test]
    fn test_calcular_taxa_vasp() {
        let brl = dec!(520);
        let taxa = calcular_taxa_vasp(brl);
        assert_eq!(taxa, dec!(2.60));
    }

    #[test]
    fn test_criar_pedido_pix() {
        let pedido = criar_pedido_pix(
            "pay-001",
            "merchant-001",
            "merchant@pix.com",
            dec!(100),
            dec!(5.20),
        );
        assert_eq!(pedido.valor_brl, dec!(520));
        assert_eq!(pedido.taxa_vasp, dec!(2.60));
        assert_eq!(pedido.valor_liquido_brl, dec!(517.40));
    }
}
