use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::env;
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
    pub vasp_tx_id: Option<String>,
    pub eta_segundos: Option<u32>,
}

/// Configuração do VASP
#[derive(Debug, Clone)]
pub struct ConfigVasp {
    pub nome: String,
    pub api_url: String,
    pub api_key: String,
    pub taxa: Decimal,
    pub modo: ModoVasp,
}

#[derive(Debug, Clone)]
pub enum ModoVasp {
    Simulado,
    Producao,
}

/// Taxa do VASP parceiro: 0.5%
pub const TAXA_VASP: Decimal = dec!(0.5);

/// Limite máximo por off-ramp
pub const LIMITE_OFFRAMP_USDC: Decimal = dec!(50000);

/// Carrega configuração do VASP do .env
pub fn carregar_config_vasp() -> ConfigVasp {
    let modo = match env::var("VASP_MODO")
        .unwrap_or_else(|_| "simulado".to_string())
        .as_str()
    {
        "producao" => ModoVasp::Producao,
        _ => ModoVasp::Simulado,
    };

    ConfigVasp {
        nome: env::var("VASP_NOME").unwrap_or_else(|_| "SlipPay VASP Simulado".to_string()),
        api_url: env::var("VASP_API_URL")
            .unwrap_or_else(|_| "https://api.vasp.simulado.io".to_string()),
        api_key: env::var("VASP_API_KEY").unwrap_or_else(|_| "vasp-key-simulado".to_string()),
        taxa: TAXA_VASP,
        modo,
    }
}

/// Calcula o valor em BRL dado USDC e taxa de câmbio
pub fn calcular_valor_brl(valor_usdc: Decimal, taxa_cambio: Decimal) -> Decimal {
    (valor_usdc * taxa_cambio).round_dp(2)
}

/// Calcula taxa do VASP
pub fn calcular_taxa_vasp(valor_brl: Decimal) -> Decimal {
    (valor_brl * (TAXA_VASP / dec!(100))).round_dp(2)
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

    let valor_liquido_brl = (valor_brl - taxa_vasp).round_dp(2);

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

/// Valida chave PIX
pub fn validar_chave_pix(chave: &str) -> (bool, &'static str) {
    if chave.is_empty() {
        return (false, "Chave PIX vazia");
    }

    let apenas_numeros: String = chave.chars().filter(|c| c.is_numeric()).collect();

    if apenas_numeros.len() == 11 {
        return (true, "cpf");
    }

    if apenas_numeros.len() == 14 {
        return (true, "cnpj");
    }

    if chave.starts_with("+55") && chave.len() >= 13 {
        return (true, "telefone");
    }

    if chave.contains('@') && chave.contains('.') {
        return (true, "email");
    }

    if Uuid::parse_str(chave).is_ok() {
        return (true, "aleatoria");
    }

    (false, "formato inválido")
}

/// Envia pedido ao VASP
pub async fn enviar_para_vasp(pedido: &PedidoPix) -> ResultadoPix {
    let config = carregar_config_vasp();

    let (pix_valido, tipo_chave) = validar_chave_pix(&pedido.chave_pix);

    if !pix_valido {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: format!("Chave PIX inválida: {}", tipo_chave),
            vasp_tx_id: None,
            eta_segundos: None,
        };
    }

    if pedido.taxa_cambio <= dec!(0) {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: "Taxa de câmbio inválida".to_string(),
            vasp_tx_id: None,
            eta_segundos: None,
        };
    }

    if pedido.valor_usdc <= dec!(0) {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: "Valor USDC inválido".to_string(),
            vasp_tx_id: None,
            eta_segundos: None,
        };
    }

    if pedido.valor_usdc < dec!(1) {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: "Valor mínimo é 1 USDC".to_string(),
            vasp_tx_id: None,
            eta_segundos: None,
        };
    }

    if pedido.valor_usdc > LIMITE_OFFRAMP_USDC {
        return ResultadoPix {
            sucesso: false,
            pedido_id: pedido.id.clone(),
            valor_brl: dec!(0),
            valor_liquido_brl: dec!(0),
            mensagem: format!("Limite máximo de off-ramp é {} USDC", LIMITE_OFFRAMP_USDC),
            vasp_tx_id: None,
            eta_segundos: None,
        };
    }

    match config.modo {
        ModoVasp::Simulado => enviar_simulado(pedido, tipo_chave).await,

        ModoVasp::Producao => enviar_producao(pedido, &config).await,
    }
}

/// Modo simulado
async fn enviar_simulado(pedido: &PedidoPix, tipo_chave: &str) -> ResultadoPix {
    let vasp_tx_id = format!("VASP-SIM-{}", &pedido.id[..8]);

    ResultadoPix {
        sucesso: true,
        pedido_id: pedido.id.clone(),
        valor_brl: pedido.valor_brl,
        valor_liquido_brl: pedido.valor_liquido_brl,
        mensagem: format!(
            "PIX simulado de R$ {:.2} enviado para {} ({}). TX: {}",
            pedido.valor_liquido_brl, pedido.chave_pix, tipo_chave, vasp_tx_id,
        ),
        vasp_tx_id: Some(vasp_tx_id),
        eta_segundos: Some(30),
    }
}

/// Produção (placeholder)
async fn enviar_producao(pedido: &PedidoPix, config: &ConfigVasp) -> ResultadoPix {
    ResultadoPix {
        sucesso: false,
        pedido_id: pedido.id.clone(),
        valor_brl: dec!(0),
        valor_liquido_brl: dec!(0),
        mensagem: format!("VASP {} em produção — integração pendente", config.nome),
        vasp_tx_id: None,
        eta_segundos: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calcular_valor_brl() {
        let usdc = dec!(100);
        let cambio = dec!(5.20);

        assert_eq!(calcular_valor_brl(usdc, cambio), dec!(520));
    }

    #[test]
    fn test_calcular_taxa_vasp() {
        let brl = dec!(520);

        assert_eq!(calcular_taxa_vasp(brl), dec!(2.60));
    }

    #[test]
    fn test_criar_pedido_pix() {
        let pedido = criar_pedido_pix(
            "pay-001",
            "merchant-001",
            "merchant@email.com",
            dec!(100),
            dec!(5.20),
        );

        assert_eq!(pedido.valor_brl, dec!(520));
        assert_eq!(pedido.taxa_vasp, dec!(2.60));
        assert_eq!(pedido.valor_liquido_brl, dec!(517.40));
    }

    #[test]
    fn test_validar_chave_pix_email() {
        let (valido, tipo) = validar_chave_pix("merchant@email.com");

        assert!(valido);
        assert_eq!(tipo, "email");
    }

    #[test]
    fn test_validar_chave_pix_cpf() {
        let (valido, tipo) = validar_chave_pix("12345678901");

        assert!(valido);
        assert_eq!(tipo, "cpf");
    }

    #[test]
    fn test_validar_chave_pix_invalida() {
        let (valido, _) = validar_chave_pix("chave-invalida");

        assert!(!valido);
    }

    #[test]
    fn test_validar_chave_pix_telefone() {
        let (valido, tipo) = validar_chave_pix("+5511999999999");

        assert!(valido);
        assert_eq!(tipo, "telefone");
    }
}
