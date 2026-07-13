/**
 * SlipPay SDK - Gateway de pagamento USDC para e-commerce brasileiro
 * @version 1.1.0
 */

class SlipPay {
  constructor(config = {}) {
    this.apiKey = config.apiKey || '';
    this.apiUrl = config.apiUrl || 'http://localhost:3000';
    this.network = config.network || 'solana';
    this.merchantId = config.merchantId || '';
  }

  /**
   * Inicia um checkout SlipPay
   * @param {Object} options
   * @param {string} options.walletDestino - Endereço da wallet do merchant
   * @param {number} options.amount - Valor em USDC
   * @param {string} options.token - Token do merchant
   * @returns {Promise<Object>}
   */
  async criarCheckout(options) {
    try {
      const response = await fetch(`${this.apiUrl}/checkout`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Api-Key': this.apiKey,
        },
        body: JSON.stringify({
          merchant_id: this.merchantId,
          wallet_destino: options.walletDestino,
          token: options.token,
          network: this.network,
          amount: options.amount,
        }),
      });

      if (!response.ok) {
        throw new Error(`Erro ao criar checkout: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error('SlipPay - Erro ao criar checkout:', error);
      throw error;
    }
  }

  /**
   * Confirma um pagamento na blockchain
   * @param {Object} options
   * @param {string} options.paymentId - ID do payment
   * @param {string} options.txHash - Hash da transação
   * @param {string} options.payer - Endereço do pagador
   * @param {number} options.amount - Valor pago
   * @param {string} options.memo - Memo da transação
   * @returns {Promise<Object>}
   */
  async confirmarPagamento(options) {
    try {
      const response = await fetch(`${this.apiUrl}/webhook/confirm`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Api-Key': this.apiKey,
        },
        body: JSON.stringify({
          payment_id: options.paymentId,
          tx_hash: options.txHash,
          payer: options.payer,
          amount: options.amount,
          memo: options.memo,
        }),
      });

      if (!response.ok) {
        throw new Error(`Erro ao confirmar pagamento: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error('SlipPay - Erro ao confirmar pagamento:', error);
      throw error;
    }
  }

  /**
   * Consulta status de um payment
   * @param {string} paymentId - ID do payment
   * @returns {Promise<Object>}
   */
  async consultarPagamento(paymentId) {
    try {
      const response = await fetch(
        `${this.apiUrl}/payment/${paymentId}`,
        {
          method: 'GET',
          headers: {
            'X-Api-Key': this.apiKey,
          },
        }
      );

      if (!response.ok) {
        throw new Error(`Erro ao consultar pagamento: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error('SlipPay - Erro ao consultar pagamento:', error);
      throw error;
    }
  }

  /**
   * Cria off-ramp PIX (USDC → BRL)
   *
   * IMPORTANTE: o payment referenciado por paymentId precisa já estar
   * confirmado on-chain (status "paid") — o backend rejeita qualquer
   * payment_id que não tenha passado por confirmarPagamento() antes.
   * O valor em USDC é lido do payment confirmado no servidor, não é
   * mais enviado pelo cliente.
   *
   * @param {Object} options
   * @param {string} options.paymentId - ID do payment (já confirmado on-chain)
   * @param {string} options.chavePix - Chave PIX do merchant
   * @param {number} options.taxaCambio - Taxa de câmbio USDC/BRL
   * @returns {Promise<Object>}
   */
  async criarOffRampPix(options) {
    try {
      const response = await fetch(`${this.apiUrl}/pix/offramp`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Api-Key': this.apiKey,
        },
        body: JSON.stringify({
          payment_id: options.paymentId,
          chave_pix: options.chavePix,
          taxa_cambio: options.taxaCambio,
        }),
      });

      if (!response.ok) {
        throw new Error(`Erro ao criar PIX off-ramp: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error('SlipPay - Erro ao criar PIX off-ramp:', error);
      throw error;
    }
  }

  /**
   * Verifica saúde da API
   * @returns {Promise<boolean>}
   */
  async verificarSaude() {
    try {
      const response = await fetch(`${this.apiUrl}/health`);
      return response.ok;
    } catch (error) {
      console.error('SlipPay - API indisponível:', error);
      return false;
    }
  }
}

// Export para Node.js/npm
if (typeof module !== 'undefined' && module.exports) {
  module.exports = SlipPay;
}
