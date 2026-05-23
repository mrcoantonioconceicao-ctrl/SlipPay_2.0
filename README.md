# SlipPay 2.0

Non-custodial USDC payment gateway para e-commerce brasileiro. Construído em Rust com Solana + Stellar.

## 🚀 Status

**MVP Funcional** — 20/20 testes passando ✅

## 📋 Features

### Pagamentos
- ✅ Checkout USDC não-custodial
- ✅ Verificação on-chain de transações
- ✅ Taxa automática 1.5%
- ✅ Expiração de pagamentos (15 min)

### Segurança
- ✅ Autenticação por API Key
- ✅ Ed25519 + AES-256-GCM
- ✅ Antifraude com score multi-fator
- ✅ Validação de memo + valor

### Off-Ramp PIX
- ✅ Conversão USDC → BRL
- ✅ Integração com VASP parceiro
- ✅ Taxa 0.5% do VASP

### SDK JavaScript
- ✅ `@slippay/sdk-js` para merchants
- ✅ 4 métodos principais
- ✅ Exemplo de uso incluído

## 🏗️ Arquitetura
src/
├── main.rs              # Entry point
├── finance/             # Cálculos financeiros (taxa, breakdown)
├── security/            # Criptografia, autenticação, API keys
├── services/            # RPC Solana, transferências USDC, verificação TX
├── governance/          # PostgreSQL, auditoria, payments
├── interface/           # API REST (Axum)
├── ai/                  # Antifraude com score de risco
├── pix/                 # Off-ramp PIX para VASP
└── ast/                 # Parser/Lexer (future: smart contracts)
sdk/js/
├── slippay.js           # SDK JavaScript
├── package.json         # NPM metadata
└── README.md            # Documentação
## 🛠️ Requisitos

- Rust 1.70+
- PostgreSQL 12+
- Node.js 16+ (para SDK)

## 🚀 Como Rodar

### Backend
```bash
cd ~/slippay_2.0
cargo build --release
cargo run
API rodará em http://localhost:3000
Testes
cargo test

📚 API REST
POST /checkout
Inicia um pagamento

curl -X POST http://localhost:3000/checkout \
  -H "Content-Type: application/json" \
  -H "X-Api-Key: slippay-dev-key-2026" \
  -d '{
    "merchant_id": "merchant-001",
    "wallet_destino": "GBxyz...",
    "token": "usdc",
    "network": "solana",
    "amount": 100
  }'

POST /webhook/confirm
Confirma pagamento após TX on-chain

curl -X POST http://localhost:3000/webhook/confirm \
  -H "Content-Type: application/json" \
  -H "X-Api-Key: slippay-dev-key-2026" \
  -d '{
    "payment_id": "pay-123",
    "tx_hash": "5zT...",
    "payer": "wallet-pagador",
    "amount": 100,
    "memo": "uuid-memo"
  }'

🔐 Segurança
Non-custodial: SlipPay nunca toca nas chaves ou fundos
On-chain verification: Todas as TXs são verificadas na blockchain
Atomic settlement: Pagamento + fee em uma transação
Rate limiting: API throttling por chave
Encryption: Dados sensíveis com AES-256-GCM
📊 Roadmap
[ ] Integração com Stripe/Shopify
[ ] Dashboard para merchants
[ ] Webhooks para notificações
[ ] Suporte a múltiplas stablecoins
[ ] Liquidação automática de taxas
[ ] Mobile app
👥 Autor
Marco Antônio — @mrcoantonioconceicao-ctrl
📄 Licença
MIT
