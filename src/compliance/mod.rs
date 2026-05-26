use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use tracing::info;

/// Limites operacionais BCB
pub const LIMITE_TRANSACAO_SEM_KYC: Decimal = dec!(1000);
pub const LIMITE_MENSAL_SEM_KYC: Decimal = dec!(3000);
pub const LIMITE_TRANSACAO_KYC_BASICO: Decimal = dec!(10000);
pub const LIMITE_MENSAL_KYC_BASICO: Decimal = dec!(50000);
pub const LIMITE_TRANSACAO_KYC_COMPLETO: Decimal = dec!(100000);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NivelKyc {
    Nenhum,
    Basico,
    Completo,
    Institucional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusCompliance {
    Aprovada,
    Pendente,
    Bloqueada,
    RequerKyc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroKyc {
    pub id: String,
    pub merchant_id: String,
    pub nivel: NivelKyc,
    pub documento: String,
    pub nome: String,
    pub pais: String,
    pub verificado: bool,
    pub criado_em: DateTime<Utc>,
    pub atualizado_em: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ResultadoCompliance {
    pub aprovada: bool,
    pub status: StatusCompliance,
    pub motivos: Vec<String>,
    pub requer_kyc: bool,
    pub nivel_kyc_requerido: NivelKyc,
    pub resolucao_bcb: String,
}

#[derive(Debug, Serialize)]
pub struct RelatorioTransacao {
    pub id: String,
    pub payment_id: String,
    pub merchant_id: String,
    pub valor_usdc: Decimal,
    pub valor_brl: Decimal,
    pub wallet_origem: String,
    pub wallet_destino: String,
    pub network: String,
    pub tx_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub flag_suspeita: bool,
    pub motivo_flag: Option<String>,
}

/// Resolução 519 — verifica limites sem KYC
fn verificar_res519(
    valor_usdc: Decimal,
    volume_mensal: Decimal,
    motivos: &mut Vec<String>,
) -> (bool, bool, NivelKyc) {
    let mut aprovada = true;
    let mut requer_kyc = false;

    if valor_usdc > LIMITE_TRANSACAO_SEM_KYC {
        aprovada = false;
        requer_kyc = true;
        motivos.push(format!(
            "Res. BCB 519: transação acima de USDC {} requer KYC básico",
            LIMITE_TRANSACAO_SEM_KYC
        ));
    }

    if volume_mensal + valor_usdc > LIMITE_MENSAL_SEM_KYC {
        aprovada = false;
        requer_kyc = true;
        motivos.push(format!(
            "Res. BCB 519: volume mensal acima de USDC {} requer KYC básico",
            LIMITE_MENSAL_SEM_KYC
        ));
    }

    (aprovada, requer_kyc, NivelKyc::Basico)
}

/// Resolução 520 — verifica limites com KYC básico
fn verificar_res520(
    valor_usdc: Decimal,
    volume_mensal: Decimal,
    kyc: &RegistroKyc,
    motivos: &mut Vec<String>,
) -> (bool, bool, NivelKyc) {
    let mut aprovada = true;
    let mut requer_kyc = false;

    if valor_usdc > LIMITE_TRANSACAO_KYC_BASICO {
        aprovada = false;
        requer_kyc = true;
        motivos.push(format!(
            "Res. BCB 520: transação acima de USDC {} requer KYC completo",
            LIMITE_TRANSACAO_KYC_BASICO
        ));
    }

    if volume_mensal + valor_usdc > LIMITE_MENSAL_KYC_BASICO {
        aprovada = false;
        requer_kyc = true;
        motivos.push(format!(
            "Res. BCB 520: volume mensal acima de USDC {} requer KYC completo",
            LIMITE_MENSAL_KYC_BASICO
        ));
    }

    if !kyc.verificado {
        aprovada = false;
        motivos.push("KYC básico não verificado".to_string());
    }

    (aprovada, requer_kyc, NivelKyc::Completo)
}

/// Resolução 521 — reporte COAF
fn verificar_res521(
    valor_usdc: Decimal,
    motivos: &mut Vec<String>,
) {
    if valor_usdc > dec!(10000) {
        motivos.push(
            "Res. BCB 521: transação acima de USDC 10.000 deve ser reportada ao COAF"
                .to_string()
        );
    }
}

/// Verifica países bloqueados por sanções
fn verificar_sancoes(
    kyc: &RegistroKyc,
    motivos: &mut Vec<String>,
) -> bool {
    let paises_bloqueados = ["IR", "KP", "CU", "SY", "RU"];
    if paises_bloqueados.contains(&kyc.pais.as_str()) {
        motivos.push(format!(
            "País {} bloqueado por sanções internacionais",
            kyc.pais
        ));
        return false;
    }
    true
}

/// Verifica compliance — subfunções extraídas (Orion)
pub fn verificar_compliance(
    valor_usdc: Decimal,
    _merchant_id: &str,
    kyc: Option<&RegistroKyc>,
    volume_mensal: Decimal,
) -> ResultadoCompliance {
    let mut motivos: Vec<String> = Vec::new();
    let mut aprovada = true;
    let mut requer_kyc = false;
    let mut nivel_requerido = NivelKyc::Nenhum;

    // Res. 519 — sem KYC
    if kyc.is_none() || kyc.map(|k| &k.nivel) == Some(&NivelKyc::Nenhum) {
        let (ap, rk, nivel) = verificar_res519(
            valor_usdc, volume_mensal, &mut motivos
        );
        if !ap { aprovada = false; }
        if rk { requer_kyc = true; nivel_requerido = nivel; }
    }

    // Res. 520 — KYC básico
    if let Some(k) = kyc {
        if k.nivel == NivelKyc::Basico {
            let (ap, rk, nivel) = verificar_res520(
                valor_usdc, volume_mensal, k, &mut motivos
            );
            if !ap { aprovada = false; }
            if rk { requer_kyc = true; nivel_requerido = nivel; }
        }
    }

    // Res. 521 — COAF
    verificar_res521(valor_usdc, &mut motivos);

    // Sanções internacionais
    if let Some(k) = kyc {
        if !verificar_sancoes(k, &mut motivos) {
            aprovada = false;
        }
    }

    let status = if aprovada {
        StatusCompliance::Aprovada
    } else if requer_kyc {
        StatusCompliance::RequerKyc
    } else {
        StatusCompliance::Bloqueada
    };

    info!("Compliance verificado: aprovada={} requer_kyc={}", aprovada, requer_kyc);

    ResultadoCompliance {
        aprovada,
        status,
        motivos,
        requer_kyc,
        nivel_kyc_requerido: nivel_requerido,
        resolucao_bcb: "519/520/521".to_string(),
    }
}

pub fn criar_kyc_basico(
    merchant_id: &str,
    documento: &str,
    nome: &str,
    pais: &str,
) -> RegistroKyc {
    RegistroKyc {
        id: Uuid::new_v4().to_string(),
        merchant_id: merchant_id.to_string(),
        nivel: NivelKyc::Basico,
        documento: documento.to_string(),
        nome: nome.to_string(),
        pais: pais.to_string(),
        verificado: false,
        criado_em: Utc::now(),
        atualizado_em: Utc::now(),
    }
}

pub fn gerar_relatorio_coaf(
    payment_id: &str,
    merchant_id: &str,
    valor_usdc: Decimal,
    valor_brl: Decimal,
    wallet_origem: &str,
    wallet_destino: &str,
    network: &str,
    tx_hash: Option<String>,
) -> RelatorioTransacao {
    let flag_suspeita = valor_usdc > dec!(10000);
    let motivo_flag = if flag_suspeita {
        Some("Transação acima de USDC 10.000 — Res. BCB 521".to_string())
    } else {
        None
    };

    info!("Relatório COAF gerado: payment_id={} flag={}", payment_id, flag_suspeita);

    RelatorioTransacao {
        id: Uuid::new_v4().to_string(),
        payment_id: payment_id.to_string(),
        merchant_id: merchant_id.to_string(),
        valor_usdc,
        valor_brl,
        wallet_origem: wallet_origem.to_string(),
        wallet_destino: wallet_destino.to_string(),
        network: network.to_string(),
        tx_hash,
        timestamp: Utc::now(),
        flag_suspeita,
        motivo_flag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_sem_kyc_valor_ok() {
        let resultado = verificar_compliance(
            dec!(500), "merchant-001", None, dec!(0),
        );
        assert!(resultado.aprovada);
    }

    #[test]
    fn test_compliance_sem_kyc_valor_alto() {
        let resultado = verificar_compliance(
            dec!(1500), "merchant-001", None, dec!(0),
        );
        assert!(!resultado.aprovada);
        assert!(resultado.requer_kyc);
    }

    #[test]
    fn test_compliance_kyc_basico_aprovado() {
        let kyc = RegistroKyc {
            id: "kyc-001".to_string(),
            merchant_id: "merchant-001".to_string(),
            nivel: NivelKyc::Basico,
            documento: "12345678901".to_string(),
            nome: "Marco Antônio".to_string(),
            pais: "BR".to_string(),
            verificado: true,
            criado_em: Utc::now(),
            atualizado_em: Utc::now(),
        };
        let resultado = verificar_compliance(
            dec!(5000), "merchant-001", Some(&kyc), dec!(0),
        );
        assert!(resultado.aprovada);
    }

    #[test]
    fn test_compliance_pais_bloqueado() {
        let kyc = RegistroKyc {
            id: "kyc-002".to_string(),
            merchant_id: "merchant-002".to_string(),
            nivel: NivelKyc::Completo,
            documento: "12345678901".to_string(),
            nome: "Test User".to_string(),
            pais: "KP".to_string(),
            verificado: true,
            criado_em: Utc::now(),
            atualizado_em: Utc::now(),
        };
        let resultado = verificar_compliance(
            dec!(100), "merchant-002", Some(&kyc), dec!(0),
        );
        assert!(!resultado.aprovada);
    }

    #[test]
    fn test_relatorio_coaf() {
        let relatorio = gerar_relatorio_coaf(
            "pay-001", "merchant-001",
            dec!(15000), dec!(78000),
            "wallet-origem", "wallet-destino",
            "solana", None,
        );
        assert!(relatorio.flag_suspeita);
        assert!(relatorio.motivo_flag.is_some());
    }
}
