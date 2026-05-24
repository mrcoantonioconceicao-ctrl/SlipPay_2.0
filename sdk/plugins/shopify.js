/**
 * SlipPay - Plugin Shopify/VTEX/Nuvemshop
 * Adiciona botão de pagamento USDC no checkout
 * @version 1.0.0
 */

(function () {
  'use strict';

  // ============================================
  // CONFIGURAÇÃO
  // ============================================
  const SLIPPAY_CONFIG = {
    apiUrl: 'http://127.0.0.1:3000',
    apiKey: '',
    merchantId: '',
    walletDestino: '',
    network: 'solana',
    taxaCambio: 5.20,
    chavePix: '',
    moeda: 'USDC',
    tema: 'dark',
  };

  // ============================================
  // ESTILOS
  // ============================================
  const styles = `
    #slippay-overlay {
      position: fixed;
      inset: 0;
      background: rgba(0,0,0,0.7);
      z-index: 99999;
      display: flex;
      align-items: center;
      justify-content: center;
      backdrop-filter: blur(4px);
    }

    #slippay-modal {
      background: #0d1226;
      border: 1px solid #1e2d4a;
      border-radius: 16px;
      padding: 36px;
      width: 100%;
      max-width: 480px;
      margin: 16px;
      box-shadow: 0 32px 64px rgba(0,0,0,0.6);
      font-family: 'Segoe UI', sans-serif;
      color: #e2e8f0;
      animation: slippay-slide-in 0.3s ease;
    }

    @keyframes slippay-slide-in {
      from { transform: translateY(20px); opacity: 0; }
      to { transform: translateY(0); opacity: 1; }
    }

    #slippay-modal .sp-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 24px;
    }

    #slippay-modal .sp-logo {
      font-size: 22px;
      font-weight: 700;
      color: #a3e635;
    }

    #slippay-modal .sp-logo span { color: #fff; }

    #slippay-modal .sp-close {
      background: #1a2744;
      border: none;
      color: #9ca3af;
      width: 32px;
      height: 32px;
      border-radius: 50%;
      cursor: pointer;
      font-size: 18px;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    #slippay-modal .sp-amount-box {
      background: #0a0e1a;
      border: 1px solid #1e2d4a;
      border-radius: 12px;
      padding: 20px;
      margin-bottom: 24px;
      text-align: center;
    }

    #slippay-modal .sp-amount-label {
      font-size: 12px;
      color: #64748b;
      text-transform: uppercase;
      letter-spacing: 1px;
      margin-bottom: 8px;
    }

    #slippay-modal .sp-amount-value {
      font-size: 36px;
      font-weight: 700;
      color: #a3e635;
    }

    #slippay-modal .sp-amount-sub {
      font-size: 12px;
      color: #64748b;
      margin-top: 4px;
    }

    #slippay-modal .sp-breakdown {
      background: #0a0e1a;
      border: 1px solid #1e2d4a;
      border-radius: 8px;
      padding: 14px 16px;
      margin-bottom: 24px;
      font-size: 13px;
    }

    #slippay-modal .sp-breakdown-row {
      display: flex;
      justify-content: space-between;
      padding: 4px 0;
      color: #94a3b8;
    }

    #slippay-modal .sp-breakdown-row.total {
      color: #f1f5f9;
      font-weight: 600;
      border-top: 1px solid #1e2d4a;
      margin-top: 8px;
      padding-top: 10px;
    }

    #slippay-modal .sp-steps {
      margin-bottom: 24px;
    }

    #slippay-modal .sp-step {
      display: flex;
      align-items: flex-start;
      gap: 12px;
      padding: 10px 0;
      border-bottom: 1px solid #0f1629;
    }

    #slippay-modal .sp-step:last-child {
      border-bottom: none;
    }

    #slippay-modal .sp-step-num {
      width: 24px;
      height: 24px;
      border-radius: 50%;
      background: #1a2744;
      border: 1px solid #2d4a7a;
      color: #60a5fa;
      font-size: 12px;
      font-weight: 700;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }

    #slippay-modal .sp-step-num.done {
      background: #14532d;
      border-color: #4ade80;
      color: #4ade80;
    }

    #slippay-modal .sp-step-text {
      font-size: 13px;
      color: #94a3b8;
      line-height: 1.5;
    }

    #slippay-modal .sp-step-text strong {
      color: #f1f5f9;
      display: block;
      margin-bottom: 2px;
    }

    #slippay-modal .sp-memo {
      background: #0a0e1a;
      border: 1px solid #2d4a7a;
      border-radius: 8px;
      padding: 12px;
      font-family: monospace;
      font-size: 12px;
      color: #60a5fa;
      word-break: break-all;
      margin-top: 8px;
    }

    #slippay-modal .sp-wallet {
      background: #0a0e1a;
      border: 1px solid #2d4a7a;
      border-radius: 8px;
      padding: 12px;
      font-family: monospace;
      font-size: 11px;
      color: #a3e635;
      word-break: break-all;
      margin-top: 8px;
    }

    #slippay-modal .sp-btn {
      width: 100%;
      background: #a3e635;
      color: #0a0e1a;
      border: none;
      border-radius: 8px;
      padding: 14px;
      font-size: 15px;
      font-weight: 700;
      cursor: pointer;
      margin-top: 8px;
      transition: background 0.2s;
    }

    #slippay-modal .sp-btn:hover { background: #84cc16; }

    #slippay-modal .sp-btn:disabled {
      background: #374151;
      color: #64748b;
      cursor: not-allowed;
    }

    #slippay-modal .sp-btn-secondary {
      width: 100%;
      background: transparent;
      border: 1px solid #1e2d4a;
      color: #9ca3af;
      border-radius: 8px;
      padding: 12px;
      font-size: 14px;
      cursor: pointer;
      margin-top: 8px;
    }

    #slippay-modal .sp-status {
      text-align: center;
      padding: 24px;
    }

    #slippay-modal .sp-status-icon {
      font-size: 48px;
      margin-bottom: 12px;
    }

    #slippay-modal .sp-status-title {
      font-size: 20px;
      font-weight: 700;
      margin-bottom: 8px;
    }

    #slippay-modal .sp-status-sub {
      font-size: 14px;
      color: #64748b;
    }

    #slippay-modal .sp-timer {
      text-align: center;
      font-size: 12px;
      color: #fbbf24;
      margin-bottom: 16px;
    }

    #slippay-modal .sp-network-badge {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      background: #1a2744;
      border: 1px solid #2d4a7a;
      color: #60a5fa;
      padding: 4px 10px;
      border-radius: 20px;
      font-size: 11px;
      font-weight: 600;
    }

    .slippay-btn {
      background: #a3e635;
      color: #0a0e1a;
      border: none;
      border-radius: 8px;
      padding: 14px 24px;
      font-size: 15px;
      font-weight: 700;
      cursor: pointer;
      width: 100%;
      margin-top: 12px;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      transition: background 0.2s;
      font-family: 'Segoe UI', sans-serif;
    }

    .slippay-btn:hover { background: #84cc16; }
  `;

  // ============================================
  // ESTADO
  // ============================================
  let estado = {
    fase: 'inicio',
    paymentId: null,
    memo: null,
    valor: 0,
    expiresAt: null,
    timer: null,
  };

  // ============================================
  // UTILITÁRIOS
  // ============================================
  function injetarEstilos() {
    const style = document.createElement('style');
    style.textContent = styles;
    document.head.appendChild(style);
  }

  function formatarValor(v) {
    return parseFloat(v).toFixed(2);
  }

  function calcularTaxa(valor) {
    return (parseFloat(valor) * 0.015).toFixed(3);
  }

  function calcularMerchant(valor) {
    return (parseFloat(valor) * 0.985).toFixed(3);
  }

  // ============================================
  // MODAL
  // ============================================
  function abrirModal(valor) {
    estado.valor = parseFloat(valor);
    estado.fase = 'criando';

    const overlay = document.createElement('div');
    overlay.id = 'slippay-overlay';
    overlay.innerHTML = `
      <div id="slippay-modal">
        <div class="sp-header">
          <div class="sp-logo">Slip<span>Pay</span></div>
          <button class="sp-close" onclick="SlipPay.fecharModal()">✕</button>
        </div>
        <div style="text-align:center;padding:32px">
          <div style="font-size:32px;margin-bottom:12px">⏳</div>
          <div style="color:#60a5fa">Criando checkout...</div>
        </div>
      </div>
    `;

    document.body.appendChild(overlay);
    criarCheckout(valor);
  }

  function fecharModal() {
    const overlay = document.getElementById('slippay-overlay');
    if (overlay) overlay.remove();
    if (estado.timer) clearInterval(estado.timer);
    estado = { fase: 'inicio', paymentId: null, memo: null, valor: 0 };
  }

  function renderizarCheckout(data) {
    const taxa = calcularTaxa(data.amount);
    const merchant = calcularMerchant(data.amount);
    const expires = new Date(data.expires_at);

    estado.paymentId = data.payment_id;
    estado.memo = data.memo;
    estado.expiresAt = expires;

    const modal = document.getElementById('slippay-modal');
    modal.innerHTML = `
      <div class="sp-header">
        <div class="sp-logo">Slip<span>Pay</span></div>
        <div style="display:flex;align-items:center;gap:8px">
          <span class="sp-network-badge">⬡ Solana</span>
          <button class="sp-close" onclick="SlipPay.fecharModal()">✕</button>
        </div>
      </div>

      <div class="sp-amount-box">
        <div class="sp-amount-label">Valor a pagar</div>
        <div class="sp-amount-value">
          ${formatarValor(data.amount)} USDC
        </div>
        <div class="sp-amount-sub">
          ≈ R$ ${formatarValor(data.amount * SLIPPAY_CONFIG.taxaCambio)}
        </div>
      </div>

      <div class="sp-breakdown">
        <div class="sp-breakdown-row">
          <span>Subtotal</span>
          <span>USDC ${formatarValor(data.amount)}</span>
        </div>
        <div class="sp-breakdown-row">
          <span>Taxa SlipPay (1.5%)</span>
          <span style="color:#f87171">USDC ${taxa}</span>
        </div>
        <div class="sp-breakdown-row">
          <span>Merchant recebe</span>
          <span style="color:#4ade80">USDC ${merchant}</span>
        </div>
      </div>

      <div class="sp-timer" id="sp-timer">⏱ Expira em 15:00</div>

      <div class="sp-steps">
        <div class="sp-step">
          <div class="sp-step-num done">✓</div>
          <div class="sp-step-text">
            <strong>Checkout criado</strong>
            ID: ${data.payment_id.substring(0, 16)}...
          </div>
        </div>
        <div class="sp-step">
          <div class="sp-step-num">2</div>
          <div class="sp-step-text">
            <strong>Envie USDC para a wallet</strong>
            Use exatamente o memo abaixo na transação
            <div class="sp-wallet">${data.wallet_destino}</div>
            <div class="sp-memo">MEMO: ${data.memo}</div>
          </div>
        </div>
        <div class="sp-step">
          <div class="sp-step-num">3</div>
          <div class="sp-step-text">
            <strong>Cole o hash da transação</strong>
            Após enviar o USDC
          </div>
        </div>
      </div>

      <input
        type="text"
        id="sp-tx-hash"
        placeholder="Hash da transação (ex: 5zT9...)"
        style="width:100%;background:#0a0e1a;border:1px solid #1e2d4a;
               border-radius:8px;padding:12px;color:#f1f5f9;
               font-size:14px;outline:none;margin-bottom:8px;
               font-family:monospace"
      />

      <button class="sp-btn" onclick="SlipPay.confirmarPagamento()">
        ✓ Confirmar Pagamento
      </button>
      <button class="sp-btn-secondary" onclick="SlipPay.fecharModal()">
        Cancelar
      </button>
    `;

    iniciarTimer(expires);
  }

  function iniciarTimer(expires) {
    if (estado.timer) clearInterval(estado.timer);

    estado.timer = setInterval(() => {
      const agora = new Date();
      const diff = Math.max(0, expires - agora);
      const min = Math.floor(diff / 60000);
      const seg = Math.floor((diff % 60000) / 1000);
      const el = document.getElementById('sp-timer');
      if (el) {
        el.textContent = `⏱ Expira em ${min}:${seg.toString().padStart(2,'0')}`;
        if (diff < 60000) el.style.color = '#f87171';
      }
      if (diff === 0) clearInterval(estado.timer);
    }, 1000);
  }

  function renderizarSucesso(data) {
    const modal = document.getElementById('slippay-modal');
    modal.innerHTML = `
      <div class="sp-status">
        <div class="sp-status-icon">✅</div>
        <div class="sp-status-title" style="color:#4ade80">
          Pagamento Confirmado!
        </div>
        <div class="sp-status-sub">
          Sua transação foi validada na blockchain Solana.
        </div>
        <div style="margin:20px 0;background:#0a0e1a;border:1px solid #14532d;
                    border-radius:8px;padding:16px;font-size:13px">
          <div style="color:#64748b;margin-bottom:4px">TX Hash</div>
          <div style="color:#4ade80;font-family:monospace;font-size:11px;
                      word-break:break-all">${data.tx_hash}</div>
          <div style="color:#64748b;margin-top:12px;margin-bottom:4px">
            Confirmações
          </div>
          <div style="color:#f1f5f9;font-weight:700">${data.confirmacoes}</div>
        </div>
        <button class="sp-btn" onclick="SlipPay.fecharModal()">
          Fechar
        </button>
      </div>
    `;
  }

  function renderizarErro(msg) {
    const modal = document.getElementById('slippay-modal');
    modal.innerHTML = `
      <div class="sp-status">
        <div class="sp-status-icon">❌</div>
        <div class="sp-status-title" style="color:#f87171">
          Erro no Pagamento
        </div>
        <div class="sp-status-sub">${msg}</div>
        <button class="sp-btn" onclick="SlipPay.fecharModal()"
          style="margin-top:24px">
          Fechar
        </button>
      </div>
    `;
  }

  // ============================================
  // API CALLS
  // ============================================
  async function criarCheckout(valor) {
    try {
      const res = await fetch(`${SLIPPAY_CONFIG.apiUrl}/checkout`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Api-Key': SLIPPAY_CONFIG.apiKey,
        },
        body: JSON.stringify({
          merchant_id: SLIPPAY_CONFIG.merchantId,
          wallet_destino: SLIPPAY_CONFIG.walletDestino,
          token: 'usdc',
          network: SLIPPAY_CONFIG.network,
          amount: parseFloat(valor),
        }),
      });

      const data = await res.json();

      if (data.error) {
        renderizarErro(data.error);
        return;
      }

      renderizarCheckout(data);

    } catch (e) {
      renderizarErro('Erro ao conectar ao SlipPay: ' + e.message);
    }
  }

  async function confirmarPagamento() {
    const txHash = document.getElementById('sp-tx-hash').value.trim();

    if (!txHash) {
      alert('Cole o hash da transação');
      return;
    }

    const btn = document.querySelector('#slippay-modal .sp-btn');
    btn.disabled = true;
    btn.textContent = '⏳ Verificando on-chain...';

    try {
      const res = await fetch(
        `${SLIPPAY_CONFIG.apiUrl}/webhook/confirm`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Api-Key': SLIPPAY_CONFIG.apiKey,
          },
          body: JSON.stringify({
            payment_id: estado.paymentId,
            tx_hash: txHash,
            payer: 'buyer-wallet',
            amount: estado.valor,
            memo: estado.memo,
          }),
        }
      );

      const data = await res.json();

      if (data.error) {
        renderizarErro(data.error);
        return;
      }

      if (estado.timer) clearInterval(estado.timer);
      renderizarSucesso(data);

    } catch (e) {
      renderizarErro('Erro ao confirmar: ' + e.message);
    }
  }

  // ============================================
  // BOTÃO
  // ============================================
  function criarBotao(container, valor) {
    const btn = document.createElement('button');
    btn.className = 'slippay-btn';
    btn.innerHTML = `
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="#0a0e1a" stroke-width="2"/>
        <path d="M8 12l3 3 5-5" stroke="#0a0e1a" stroke-width="2"
          stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      Pagar ${formatarValor(valor)} USDC com SlipPay
    `;
    btn.onclick = () => abrirModal(valor);
    container.appendChild(btn);
  }

  // ============================================
  // INIT
  // ============================================
  function init(config) {
    Object.assign(SLIPPAY_CONFIG, config);
    injetarEstilos();

    const containers = document.querySelectorAll(
      '[data-slippay-checkout]'
    );

    containers.forEach(el => {
      const valor = el.getAttribute('data-slippay-valor')
        || config.valor
        || 0;
      criarBotao(el, valor);
    });
  }

  // ============================================
  // EXPORT
  // ============================================
  window.SlipPay = {
    init,
    abrirModal,
    fecharModal,
    confirmarPagamento,
  };

})();
