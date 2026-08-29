# solana_sandbox_counter (Solana Anchor Smart Contract)

Este repositório contém o código-fonte do Smart Contract em Rust/Anchor e o SDK cliente TypeScript, exportados diretamente do **Solana Architect IDE**.

## 🛡️ Relatório de Auditoria de Segurança AST
- **Score de Segurança Auditoria:** `95/100`
- **Program ID:** `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`
- **Framework:** Anchor v0.30.0
- **Rede Solana Alvo:** Localnet / Devnet / Mainnet-Beta

## 📁 Estrutura do Repositório
```text
.
├── Anchor.toml
├── Cargo.toml
├── README.md
├── client/
│   └── index.ts                 # SDK Cliente TypeScript para Interação
├── target/
│   └── idl/
│       └── solana_sandbox_counter.json   # IDL Anchor Gerado
└── programs/
    └── solana_sandbox_counter/
        ├── Cargo.toml
        └── src/
            └── lib.rs           # Smart Contract Rust Principal
```

## 🚀 Como Compilar e Testar Localmente

1. Certifique-se de ter o **Solana CLI** e o **Anchor v0.30** instalados.
2. Instale as dependências TypeScript:
   ```bash
   yarn install
   ```
3. Compile o programa Anchor:
   ```bash
   anchor build
   ```
4. Execute os testes em ambiente localnet:
   ```bash
   anchor test
   ```

---
*Gerado via **Solana Architect** - Ambiente de Auditoria AST e Sandbox Solana.*
