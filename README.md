# SlipPay 2.0

> Non-custodial USDC payment gateway para e-commerce brasileiro.
> Construído em Rust • Solana • PostgreSQL

![Status](https://img.shields.io/badge/status-em%20desenvolvimento-yellow)
![Testes](https://img.shields.io/badge/testes-32%2F32%20passando-brightgreen)
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
- [Dívida técnica conhecida](#dívida-técnica-conhecida)
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

```
┌─────────────────────────────────────────────────────┐
│                    COMPRADOR                         │
│              (Phantom / Solflare)                    │
└──────────────────────┬──────────────────────────────┘
                        │ USDC
                        ▼
┌─────────────────────────────────────────────────────┐
│              SOLANA BLOCKCHAIN                       │
│         (Liquidação atômica < 5s)                    │
└──────────────────────┬──────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────┐
│              SLIPPAY API (Rust)                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐             │
│  │ Finance  │ │ Security │ │    AI    │             │
│  │  1.5%    │ │ Ed25519  │ │Antifraude│             │
│  └──────────┘ └──────────┘ └──────────┘             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐             │
│  │Governance│ │   PIX    │ │Compliance│             │
│  │PostgreSQL│ │  VASP    │ │BCB519/520│             │
│  └──────────┘ └──────────┘ └──────────┘             │
└──────────────────────┬──────────────────────────────┘
                        │
           ┌────────────┴────────────┐
           ▼                         ▼
┌──────────────────┐    ┌──────────────────────┐
│    MERCHANT      │    │     VASP PARCEIRO     │
│  USDC direto     │    │   USDC → BRL (PIX)    │
└──────────────────┘    └──────────────────────┘
```

**Fluxo de segurança do off-ramp PIX:** o valor convertido em BRL
nunca vem do cliente da API — só é liberado depois que
`webhook_confirm` valida a transação on-chain e marca o payment como
`paid`. O endpoint `/pix/offramp` lê o valor confirmado direto do
banco (`governance`), não aceita `valor_usdc` no payload.

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
```

Funções:
- `calcular_breakdown(valor)` — breakdown completo com taxa 1.5%
- `calcular_taxa(valor, taxa)` — calcula taxa sobre valor
- `converter_moeda(valor, cambio)` — conversão USDC/BRL
- `aplicar_desconto(valor, desconto)` — aplica desconto percentual

### `src/security/` — Segurança
Criptografia, autenticação e geração de API Keys.

Algoritmos:
- Ed25519 — assinatura de transações
- AES-256-GCM — criptografia de dados sensíveis
- SHA-256 — hashing de mensagens

Funções:
- `validar_api_key(key)` — valida API Key no header
- `gerar_api_key(merchant_id, secret)` — gera API Key via hash
- `assinar_mensagem(msg, chave)` — assina com Ed25519
- `verificar_mensagem(msg, assinatura, chave)` — verifica assinatura
- `criptografar(dados, chave, nonce)` — AES-256-GCM encrypt
- `descriptografar(dados, chave, nonce)` — AES-256-GCM decrypt

### `src/services/` — Blockchain Solana
Integração com a rede Solana via RPC.

Funções:
- `inicializar_cliente(url)` — conecta ao RPC Solana
- `consultar_saldo(cliente, conta)` — saldo SOL
- `consultar_saldo_usdc(cliente, conta)` — saldo USDC (SPL Token)
- `enviar_usdc(cliente, remetente, destino, valor)` — transferência USDC
- `verificar_transacao(cliente, tx_hash)` — verifica TX on-chain

### `src/governance/` — Persistência e Auditoria
PostgreSQL via sqlx.

Funções:
- `conectar_db(url)` — conecta ao PostgreSQL
- `salvar_payment(pool, payment)` — persiste payment
- `buscar_payment(pool, id)` — busca por ID
- `atualizar_status_payment(pool, id, status)` — atualiza status
- `registrar_transacao(pool, log)` — registra auditoria

### `src/ai/` — Antifraude
Score de risco multi-fator para detecção de fraudes.

Fatores analisados:
- Valor absoluto da transação
- Desvio em relação ao histórico do merchant
- Formato e reputação da wallet
- Network utilizada
- Volume nas últimas 5 transações

Score de risco (0-100):
- 0-20 → Baixo — aprovado automaticamente
- 21-50 → Médio — aprovado com monitoramento
- 51-80 → Alto — revisão manual recomendada
- 81-100 → Crítico — bloqueado automaticamente

```rust
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
```

### `src/pix/` — Off-Ramp PIX
Conversão USDC → BRL via VASP parceiro.

**Fluxo:**
```
USDC (confirmado on-chain) → VASP → BRL → PIX → Merchant
```

Taxas:
- Taxa VASP: 0.5% sobre o valor em BRL
- Exemplo: 100 USDC × 5.20 = R$ 520.00 − R$ 2.60 = R$ 517.40

Tipos de chave PIX suportados:
- CPF (11 dígitos)
- CNPJ (14 dígitos)
- Email
- Telefone (+55...)
- Chave aleatória (UUID)

Modos de operação (`VASP_MODO`):
- `simulado` — para desenvolvimento e testes, sem chamada HTTP real
- `producao` — chamada HTTP real via `reqwest` para `VASP_API_URL`,
  autenticada com `VASP_API_KEY` (Bearer token)

⚠️ O contrato de resposta do parceiro (`tx_id`, `status`,
`eta_segundos`) ainda é uma suposição — precisa ser validado contra a
documentação real do VASP escolhido (BRLA / Bitso / Mercado Bitcoin)
antes de ir para produção.

### `src/compliance/` — Compliance BCB
Implementação das Resoluções BCB 519/520/521 (Nov 2025, vigentes desde
fev/2026).

| Nível KYC | Limite por TX | Limite Mensal |
|---|---|---|
| Nenhum | USDC 1.000 | USDC 3.000 |
| Básico (CPF+nome) | USDC 10.000 | USDC 50.000 |
| Completo (CPF+RG) | USDC 100.000 | Sem limite |

Funcionalidades:
- Verificação automática de KYC por transação
- Bloqueio de países sancionados (IR, KP, CU, SY, RU)
- Geração de relatório COAF para transações acima de USDC 10.000
- Adequação à Resolução 521 (FX crypto = câmbio estrangeiro)

### `src/interface/` — API REST
Servidor HTTP construído com Axum.

Configuração:
- CORS habilitado para qualquer origem
- Autenticação via header `X-Api-Key`
- Porta configurável via `.env`

---

## API REST

Base URL: `https://slippay-2-0.onrender.com`
Autenticação: Header `X-Api-Key: sua-api-key`

### `GET /health`
Verifica saúde da API.

```json
{
  "status": "ok",
  "version": "2.0",
  "network": "devnet"
}
```

### `POST /checkout`
Cria um novo checkout de pagamento.

Headers:
```
X-Api-Key: slippay-dev-key-2026
Content-Type: application/json
```

Body:
```json
{
  "merchant_id": "merchant-001",
  "wallet_destino": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
  "token": "usdc",
  "network": "solana",
  "amount": 299.00
}
```

Resposta:
```json
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
```

### `POST /webhook/confirm`
Confirma um pagamento após a TX on-chain. Este é o passo obrigatório
antes de qualquer off-ramp PIX.

Body:
```json
{
  "payment_id": "8094b6fc-2ee7-45...",
  "tx_hash": "5zT9abc...",
  "payer": "wallet-do-comprador",
  "amount": 299.00,
  "memo": "9019260f-14db-42c2-b1e9-f12a2f4a1d3e"
}
```

Resposta:
```json
{
  "status": "paid",
  "risk_score": 10,
  "tx_hash": "5zT9abc...",
  "confirmacoes": 32
}
```

### `GET /payment/:id`
Consulta status de um payment.

Resposta:
```json
{
  "payment_id": "8094b6fc-2ee7-45...",
  "status": "paid",
  "amount": "299",
  "merchant_id": "merchant-001",
  "expires_at": "2026-05-23T21:10:20Z"
}
```

### `POST /pix/offramp`
Converte USDC em BRL e envia via PIX.

⚠️ **Só aceita `payment_id` de payments já confirmados on-chain**
(status `"paid"`). O valor em USDC é lido do banco, não do payload —
isso evita que alguém force um off-ramp com valor inventado.

Body:
```json
{
  "payment_id": "8094b6fc-2ee7-45...",
  "chave_pix": "merchant@email.com",
  "taxa_cambio": 5.20
}
```

Resposta:
```json
{
  "sucesso": true,
  "pedido_id": "VASP-SIM-8094b6fc",
  "valor_brl": "1531.48",
  "valor_liquido_brl": "1523.82",
  "mensagem": "PIX simulado de R$ 1523.82 enviado para merchant@email.com",
  "vasp_tx_id": "VASP-SIM-8094b6f",
  "eta_segundos": 30
}
```

---

## SDK JavaScript

```bash
npm install @slippay/sdk-js
```

```javascript
const SlipPay = require('@slippay/sdk-js');

const slippay = new SlipPay({
  apiKey: 'sua-api-key',
  apiUrl: 'https://slippay-2-0.onrender.com',
  merchantId: 'merchant-001',
});

// Criar checkout
const checkout = await slippay.criarCheckout({
  walletDestino: 'sua-wallet-solana',
  amount: 299.00,
  token: 'usdc',
});

// Confirmar pagamento (após TX on-chain)
await slippay.confirmarPagamento({
  paymentId: checkout.payment_id,
  txHash: 'hash-da-transacao',
  payer: 'wallet-do-comprador',
  amount: 299.00,
  memo: checkout.memo,
});

// Off-ramp PIX — só funciona com payment já confirmado (status "paid")
await slippay.criarOffRampPix({
  paymentId: checkout.payment_id,
  chavePix: 'merchant@email.com',
  taxaCambio: 5.20,
});
```

---

## Dashboard

Interface web para merchants gerenciarem pagamentos.

Acesso: `dashboard/index.html`
Credenciais de teste:
- Merchant ID: `merchant-001`
- API Key: `slippay-dev-key-2026`

Funcionalidades:
- Cards com métricas em tempo real
- Tabela de pagamentos recentes
- Criar checkout diretamente do dashboard
- PIX off-ramp integrado — o seletor só lista payments com status
  `"paid"` (confirmados on-chain), em linha com a regra do backend
- Responsivo para mobile e desktop

⚠️ Limitação conhecida: a lista de pagamentos só existe na sessão do
navegador (não há endpoint `GET /payments?merchant_id=X` ainda).
Recarregar a página perde o histórico local, mesmo que os pagamentos
continuem confirmados no banco.

---

## Compliance BCB

### Resoluções implementadas

**Resolução BCB 519** — Limites sem KYC
- Transações até USDC 1.000 sem identificação
- Volume mensal até USDC 3.000

**Resolução BCB 520** — KYC básico
- CPF + nome completo
- Transações até USDC 10.000
- Volume mensal até USDC 50.000

**Resolução BCB 521** — FX crypto
- USDC categorizado como câmbio estrangeiro
- Reporte obrigatório ao COAF acima de USDC 10.000

### Timeline regulatório

| Data | Evento |
|---|---|
| Nov 2025 | BCB publica Resoluções 519/520/521 |
| Fev 2026 | Framework entra em vigor |
| Mai 2026 | Regras FX para crypto ativadas (DeCripto) |
| Out 2026 | Prazo final para VASPs protocolarem autorização junto ao BCB |

---

## Como rodar

```bash
# Clonar o repositório
git clone https://github.com/mrcoantonioconceicao-ctrl/SlipPay_2.0
cd slippay_2.0

# Instalar dependências e compilar
cargo build

# Rodar testes
cargo test

# Rodar o servidor
cargo run
```

---

## Testes

```
running 32 tests
test ai::tests::test_risco_baixo ... ok
test ai::tests::test_risco_alto_valor ... ok
test ai::tests::test_risco_network_desconhecida ... ok
test ai::tests::test_risco_desvio_historico ... ok
test ast::lexer::tests::test_tokenize_rule ... ok
test compliance::tests::test_compliance_kyc_basico_aprovado ... ok
test compliance::tests::test_compliance_pais_bloqueado ... ok
test compliance::tests::test_compliance_sem_kyc_valor_alto ... ok
test compliance::tests::test_compliance_sem_kyc_valor_ok ... ok
test compliance::tests::test_relatorio_coaf ... ok
test database::tests::test_conectar ... ok
test finance::tests::test_aplicar_desconto ... ok
test finance::tests::test_breakdown_1000_usdc ... ok
test finance::tests::test_converter_moeda ... ok
test finance::tests::test_taxa_slippay ... ok
test finance::tests::test_valor_total ... ok
test pix::tests::test_calcular_taxa_vasp ... ok
test pix::tests::test_calcular_valor_brl ... ok
test pix::tests::test_criar_pedido_pix ... ok
test pix::tests::test_validar_chave_pix_cpf ... ok
test pix::tests::test_validar_chave_pix_email ... ok
test pix::tests::test_validar_chave_pix_invalida ... ok
test pix::tests::test_validar_chave_pix_telefone ... ok
test security::tests::test_api_key_invalida ... ok
test security::tests::test_api_key_valida ... ok
test security::tests::test_assinatura_valida ... ok
test security::tests::test_criptografia ... ok
test security::tests::test_gerar_api_key ... ok
test security::tests::test_hash ... ok
test security::tests::test_secure_compare ... ok
test services::tests::test_mint_usdc ... ok
test services::tests::test_tx_hash_invalido ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Deploy

Deploy atual em produção: **Render** (`https://slippay-2-0.onrender.com`).

> O `railway.toml` presente no repositório é resquício de uma tentativa
> anterior com Railway — o deploy ativo é no Render. Vale remover esse
> arquivo para não confundir configuração futura.

### Variáveis de ambiente necessárias

```env
DATABASE_URL=postgres://user:pass@host/slippay
HOST=0.0.0.0
PORT=3000
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet
API_KEYS=sua-api-key-aqui
VASP_MODO=simulado
VASP_NOME=SlipPay VASP
VASP_API_URL=https://api.vasp.io
VASP_API_KEY=sua-vasp-key
```

### Pendências de deploy
- [ ] Domínio customizado (hoje em `slippay-2-0.onrender.com`)
- [ ] Plano pago do Render, para eliminar cold start (instância dorme
      sem tráfego no plano free — primeira requisição pode levar
      15-30s)
- [ ] Hardening do pipeline CI/CD (GitHub Actions: lint, test,
      security audit, SBOM, build Docker, deploy)

---

## Dívida técnica conhecida

- **`ed25519-dalek 1.0`** trava a versão do `zeroize` em `<1.4`, o que
  entrou em conflito com o `reqwest` (que precisa de `zeroize ^1.6`
  via `rustls`). Contornado usando `default-tls` (OpenSSL) em vez de
  `rustls-tls` no `reqwest`. Solução definitiva: migrar
  `ed25519-dalek` para v2, que usa `curve25519-dalek 4.x` — isso muda
  a API de assinatura em `src/security/mod.rs` e precisa de revisão
  dedicada.
- `solana-client 1.18.26` e `sqlx-core 0.6.3` geram aviso de
  "future incompatibility" — vão quebrar em versão futura do Rust.
  Rodar `cargo report future-incompatibilities --id 1` para detalhes
  antes que vire bloqueio de build.
- Diretório `src/ast/` (lexer, parser, evaluator) não é usado por
  nenhum dos 8 módulos documentados — investigar se é experimento
  órfão ou dependência real de algum fluxo (ex: regras do `ai`).
- `src/governance/mod.rs` e `src/governance/mods.rs` coexistem com
  nomes quase idênticos — checar se `mods.rs` está órfão.
- `src/database/mod.rs` sobrepõe responsabilidade com
  `src/governance/` (ambos mexem com persistência) — avaliar unificar.
- Binário `ngrok` e `ngrok-v3-stable-linux-arm64.tgz` estão versionados
  no repositório — devem ir para `.gitignore`, nunca commitados.
- Falta endpoint `GET /payments?merchant_id=X&status=paid` — o
  dashboard depende de estado local do navegador para listar
  pagamentos confirmados.

---

## Roadmap

### ✅ Concluído
- [x] Backend Rust modular
- [x] API REST com 7 endpoints
- [x] Verificação on-chain Solana
- [x] Taxa 1.5% automática
- [x] Antifraude multi-fator
- [x] Off-ramp PIX (modo simulado + client HTTP real para produção)
- [x] Autenticação API Key
- [x] Dashboard responsivo
- [x] SDK JavaScript
- [x] Plugin Shopify/VTEX
- [x] Variáveis de ambiente
- [x] Docker
- [x] Deploy em produção (Render)
- [x] Compliance BCB 519/520/521
- [x] Correção de segurança: off-ramp PIX exige payment confirmado
      on-chain, valor não vem mais do cliente
- [x] 32/32 testes passando

### 🔜 Próximos passos
- [ ] Parceria com VASP licenciado BCB (BRLA / Bitso / Mercado Bitcoin)
- [ ] Validar contrato de resposta real da API do VASP escolhido
- [ ] Domínio próprio + plano pago (eliminar cold start)
- [ ] Hardening do CI/CD (GitHub Actions)
- [ ] Endpoint de listagem de payments por merchant
- [ ] Resolver dívida técnica do `ed25519-dalek` (migrar para v2)
- [ ] Dashboard com gráficos
- [ ] Webhooks de notificação
- [ ] Plugin Nuvemshop oficial
- [ ] App mobile para merchants

---

## Autor

**Marco Antônio**
[@mrcoantonioconceicao-ctrl](https://github.com/mrcoantonioconceicao-ctrl)
Desenvolvedor autodidata e fundador • Blumenau, SC

## Licença

MIT © 2026 SlipPay

---

*"The Stripe of stablecoin payments for Brazil"*
