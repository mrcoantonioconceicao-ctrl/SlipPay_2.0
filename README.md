# SlipPay 2.0

> Non-custodial USDC payment gateway para e-commerce brasileiro.
> Construído em Rust • Solana • PostgreSQL



![Status](https://img.shields.io/badge/status-em%20desenvolvimento-yellow)




![Testes](https://img.shields.io/badge/testes-29%2F29%20passando-brightgreen)




![Versão](https://img.shields.io/badge/versão-2.0.0-blue)




![Licença](https://img.shields.io/badge/licença-MIT-green)



---

## 📋 Índice

- [O que é o SlipPay](#o-que-é-o-slippay)
- [Arquitetura](#arquitetura)
- [Módulos](#módulos)
- [API REST](#api-rest)
- [SDK JavaScript](#sdk-javascript)
- [Dashboard](#dashboard)
- [Compliance BCB](#compliance-bcb)
- [Como rodar](#como-rodar)
- [Testes](#testes)
- [Deploy](#deploy)
- [Roadmap](#roadmap)

---

## O que é o SlipPay

O SlipPay é um gateway de pagamento **não-custodial** em USDC para o
e-commerce brasileiro. Compradores pagam em USDC diretamente da sua
própria wallet. Merchants recebem USDC ou BRL via PIX. A plataforma
**nunca toca nos fundos**.

### Problema que resolve
- Holders de USDC no Brasil não conseguem gastar em lojas
- Merchants não têm SDK para aceitar stablecoins no VTEX/Shopify/Nuvemshop
- Processadoras de cartão cobram 3-5% e demoram dias para liquidar

### Solução
- Checkout USDC com liquidação atômica em menos de 5 segundos
- Taxa de 1.5% (vs 3-5% do cartão)
- Off-ramp PIX automático via VASP parceiro
- SDK JavaScript drop-in para qualquer e-commerce

---

## Arquitetura
┌─────────────────────────────────────────────────────┐
│                    COMPRADOR                         │
│              (Phantom / Solflare)                    │
└──────────────────────┬──────────────────────────────┘
│ USDC
▼
┌─────────────────────────────────────────────────────┐
│              SOLANA BLOCKCHAIN                       │
│         (Liquidação atômica < 5s)                   │
└──────────────────────┬──────────────────────────────┘
│
▼
┌─────────────────────────────────────────────────────┐
│              SLIPPAY API (Rust)                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐            │
│  │ Finance  │ │ Security │ │    AI    │            │
│  │  1.5%   │ │ Ed25519  │ │Antifraude│            │
│  └──────────┘ └──────────┘ └──────────┘            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐            │
│  │Governance│ │   PIX    │ │Compliance│            │
│  │PostgreSQL│ │  VASP    │ │BCB519/520│            │
│  └──────────┘ └──────────┘ └──────────┘            │
└──────────────────────┬──────────────────────────────┘
│
┌────────────┴────────────┐
▼                         ▼
┌──────────────────┐    ┌──────────────────────┐
│    MERCHANT      │    │     VASP PARCEIRO    │
│  USDC direto     │    │   USDC → BRL (PIX)  │
└──────────────────┘    └──────────────────────┘

---

## Módulos

### `src/finance/` — Cálculos Financeiros
Responsável por todos os cálculos monetários do sistema.

```rust
// Taxa SlipPay: 1.5%
pub const TAXA_SLIPPAY: Decimal = dec!(1.5);

// Exemplo de uso
let breakdown = calcular_breakdown(dec!(299));
// breakdown.taxa_slippay   → 4.485 USDC
// breakdown.valor_merchant → 294.515 USDC

Funções:
calcular_breakdown(valor) — breakdown completo com taxa 1.5%
calcular_taxa(valor, taxa) — calcula taxa sobre valor
converter_moeda(valor, cambio) — conversão USDC/BRL
aplicar_desconto(valor, desconto) — aplica desconto percentual
src/security/ — Segurança
Criptografia, autenticação e geração de API Keys.
Algoritmos:
Ed25519 — assinatura de transações
AES-256-GCM — criptografia de dados sensíveis
SHA-256 — hashing de mensagens
Funções:
validar_api_key(key) — valida API Key no header
gerar_api_key(merchant_id, secret) — gera API Key via hash
assinar_mensagem(msg, chave) — assina com Ed25519
verificar_mensagem(msg, assinatura, chave) — verifica assinatura
criptografar(dados, chave, nonce) — AES-256-GCM encrypt
descriptografar(dados, chave, nonce) — AES-256-GCM decrypt
src/services/ — Blockchain Solana
Integração com a rede Solana via RPC.
Funções:
inicializar_cliente(url) — conecta ao RPC Solana
consultar_saldo(cliente, conta) — saldo SOL
consultar_saldo_usdc(cliente, conta) — saldo USDC (SPL Token)
enviar_usdc(cliente, remetente, destino, valor) — transferência USDC
verificar_transacao(cliente, tx_hash) — verifica TX on-chain
Constantes:

Funções:
conectar_db(url) — conecta ao PostgreSQL
salvar_payment(pool, payment) — persiste payment
buscar_payment(pool, id) — busca por ID
atualizar_status_payment(pool, id, status) — atualiza status
registrar_transacao(pool, log) — registra auditoria
src/ai/ — Antifraude
Score de risco multi-fator para detecção de fraudes.
Fatores analisados:
Valor absoluto da transação
Desvio em relação ao histórico do merchant
Formato e reputação da wallet
Network utilizada
Volume nas últimas 5 transações
Score de risco (0-100):
0-20 → Baixo — aprovado automaticamente
21-50 → Médio — aprovado com monitoramento
51-80 → Alto — revisão manual recomendada
81-100 → Crítico — bloqueado automaticamente

let risco = analisar_risco(
    valor,
    &historico,
    &wallet,
    &network,
);
// risco.score    → 0-100
// risco.nivel    → "baixo" | "medio" | "alto" | "critico"
// risco.aprovada → true | false
// risco.motivos  → Vec<String>

src/pix/ — Off-Ramp PIX
Conversão USDC → BRL via VASP parceiro.
Fluxo:
USDC → VASP → BRL → PIX → Merchant

Taxas:
Taxa VASP: 0.5% sobre o valor em BRL
Exemplo: 100 USDC × 5.20 = R$ 520.00 - R$ 2.60 = R$ 517.40

Tipos de chave PIX suportados:
CPF (11 dígitos)
CNPJ (14 dígitos)
Email
Telefone (+55...)
Chave aleatória (UUID)
Modos de operação:
VASP_MODO=simulado — para desenvolvimento e testes
VASP_MODO=producao — integração real com VASP parceiro
src/compliance/ — Compliance BCB
Implementação das Resoluções BCB 519/520/521 (Nov 2025).
Limites operacionais:
Nível KYC
Limite por TX
Limite Mensal
Nenhum
USDC 1.000
USDC 3.000
Básico (CPF+nome)
USDC 10.000
USDC 50.000
Completo (CPF+RG)
USDC 100.000
Sem limite
Funcionalidades:
Verificação automática de KYC por transação
Bloqueio de países sancionados (IR, KP, CU, SY, RU)
Geração de relatório COAF para transações acima de USDC 10.000
Adequação à Resolução 521 (FX crypto = câmbio estrangeiro)
src/interface/ — API REST
Servidor HTTP construído com Axum.
Configuração:
CORS habilitado para qualquer origem
Autenticação via header X-Api-Key
Porta configurável via .env
API REST
Base URL: http://localhost:3000
Autenticação: Header X-Api-Key: sua-api-key
GET /health
Verifica saúde da API.

{
  "status": "ok",
  "version": "2.0",
  "network": "devnet"
}

POST /checkout
Cria um novo checkout de pagamento.
Headers:

X-Api-Key: slippay-dev-key-2026
Content-Type: application/json

Body:
{
  "merchant_id": "merchant-001",
  "wallet_destino": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
  "token": "usdc",
  "network": "solana",
  "amount": 299.00
}

Resposta:

{
  "payment_id": "8094b6fc-2ee7-45...",
  "merchant_id": "merchant-001",
  "wallet_destino": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
  "token": "usdc",
  "network": "solana",
  "amount": "299",
  "taxa_slippay": "4.485",
  "valor_merchant": "294.515",
  "memo": "9019260f-14db-42c2-b1e9-f12a2f4a1d3e",
  "expires_at": "2026-05-23T21:10:20Z",
  "status": "pending"
}

POST /webhook/confirm
Confirma um pagamento após a TX on-chain.
Body:
{
  "payment_id": "8094b6fc-2ee7-45...",
  "tx_hash": "5zT9abc...",
  "payer": "wallet-do-comprador",
  "amount": 299.00,
  "memo": "9019260f-14db-42c2-b1e9-f12a2f4a1d3e"
}

Resposta:

{
  "status": "paid",
  "risk_score": 10,
  "tx_hash": "5zT9abc...",
  "confirmacoes": 32
}

GET /payment/:id
Consulta status de um payment.
Resposta:

{
  "payment_id": "8094b6fc-2ee7-45...",
  "status": "paid",
  "amount": "299",
  "merchant_id": "merchant-001",
  "expires_at": "2026-05-23T21:10:20Z"
}

POST /pix/offramp
Converte USDC em BRL e envia via PIX.
Body:

{
  "payment_id": "8094b6fc-2ee7-45...",
  "merchant_id": "merchant-001",
  "chave_pix": "merchant@email.com",
  "valor_usdc": 294.515,
  "taxa_cambio": 5.20
}

Resposta:

{
  "sucesso": true,
  "pedido_id": "VASP-SIM-8094b6fc",
  "valor_brl": "1531.48",
  "valor_liquido_brl": "1523.82",
  "mensagem": "PIX de R$ 1523.82 enviado para merchant@email.com",
  "vasp_tx_id": "VASP-SIM-8094b6f",
  "eta_segundos": 30
}

Dashboard
Interface web para merchants gerenciarem pagamentos.
Acesso: http://localhost:8080
Credenciais de teste:
Merchant ID: merchant-001
API Key: slippay-dev-key-2026
Funcionalidades:
Cards com métricas em tempo real
Tabela de pagamentos recentes
Criar checkout diretamente do dashboard
PIX off-ramp integrado
Responsivo para mobile e desktop

Compliance BCB
Resoluções implementadas
Resolução BCB 519 — Limites sem KYC
Transações até USDC 1.000 sem identificação
Volume mensal até USDC 3.000
Resolução BCB 520 — KYC básico
CPF + nome completo
Transações até USDC 10.000
Volume mensal até USDC 50.000
Resolução BCB 521 — FX crypto
USDC categorizado como câmbio estrangeiro
Reporte obrigatório ao COAF acima de USDC 10.000
Vigência: Maio 2026
Timeline regulatório
Data
Evento
Nov 2025
BCB publica Resoluções 519/520/521
Fev 2026
Framework entra em vigor
Mai 2026
Regras FX para crypto ativadas
Out 2026
Fim do período de graça

Testes

running 29 tests
test ai::tests::test_risco_baixo ... ok
test ai::tests::test_risco_alto_valor ... ok
test compliance::tests::test_compliance_kyc_basico_aprovado ... ok
test compliance::tests::test_compliance_pais_bloqueado ... ok
test compliance::tests::test_compliance_sem_kyc_valor_alto ... ok
test compliance::tests::test_compliance_sem_kyc_valor_ok ... ok
test compliance::tests::test_relatorio_coaf ... ok
test finance::tests::test_breakdown_1000_usdc ... ok
test finance::tests::test_taxa_slippay ... ok
test pix::tests::test_criar_pedido_pix ... ok
test pix::tests::test_validar_chave_pix_email ... ok
test pix::tests::test_validar_chave_pix_cpf ... ok
test pix::tests::test_validar_chave_pix_telefone ... ok
test security::tests::test_api_key_valida ... ok
test services::tests::test_consultar_saldo ... ok
...
test result: ok. 29 passed; 0 failed

Railway
Conecte seu GitHub ao Railway
Importe o repositório SlipPay_2.0
Configure as variáveis de ambiente do .env
Deploy automático via railway.toml

Variáveis de ambiente necessárias

DATABASE_URL=postgres://user:pass@host/slippay
HOST=0.0.0.0
PORT=3000
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet
API_KEYS=sua-api-key-aqui
JWT_SECRET=seu-secret-aqui
VASP_MODO=simulado
VASP_NOME=SlipPay VASP
VASP_API_URL=https://api.vasp.io
VASP_API_KEY=sua-vasp-key

Estrutura do Projeto

SlipPay_2.0/
├── src/
│   ├── main.rs              # Entry point
│   ├── finance/mod.rs       # Cálculos financeiros
│   ├── security/mod.rs      # Criptografia e autenticação
│   ├── services/mod.rs      # RPC Solana e USDC
│   ├── governance/mod.rs    # PostgreSQL e auditoria
│   ├── interface/mod.rs     # API REST (Axum)
│   ├── ai/mod.rs            # Antifraude com score
│   ├── pix/mod.rs           # Off-ramp PIX via VASP
│   ├── compliance/mod.rs    # BCB 519/520/521
│   └── ast/                 # Parser (futuro: smart contracts)
├── dashboard/
│   └── index.html           # Dashboard responsivo
├── sdk/
│   ├── js/
│   │   ├── slippay.js       # SDK JavaScript
│   │   ├── package.json     # NPM metadata
│   │   └── README.md        # Docs do SDK
│   └── plugins/
│       ├── shopify.js       # Plugin Shopify/VTEX
│       └── exemplo-shopify.html  # Loja demo
├── Cargo.toml               # Dependências Rust
├── Dockerfile               # Container Docker
├── docker-compose.yml       # Stack completa
├── railway.toml             # Config Railway
└── .env                     # Variáveis de ambiente

Roadmap
✅ Concluído
[x] Backend Rust modular
[x] API REST com 7 endpoints
[x] Verificação on-chain Solana
[x] Taxa 1.5% automática
[x] Antifraude multi-fator
[x] Off-ramp PIX (modo simulado)
[x] Autenticação API Key
[x] Dashboard responsivo
[x] SDK JavaScript
[x] Plugin Shopify/VTEX
[x] Variáveis de ambiente
[x] Docker e Railway
[x] Compliance BCB 519/520/521
[x] 29/29 testes passando
🔜 Próximos passos
[ ] Parceria com VASP licenciado BCB
[ ] Deploy em servidor real
[ ] HTTPS e domínio slippay.io
[ ] Registro no BCB como VASP
[ ] Dashboard com gráficos
[ ] Webhooks de notificação
[ ] Plugin Nuvemshop oficial
[ ] App mobile para merchants
Autor
Marco Antônio
@mrcoantonioconceicao-ctrl
Entusiasta de IA e engenharia de prompt • Autodidata
Licença
MIT © 2026 SlipPay
"The Stripe of stablecoin payments for Brazil"
— SlipPay Pitch Deck, 2026
