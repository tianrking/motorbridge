export class WsGatewayClient {
  constructor({ onEvent, onState, onOpen, onClose, onError, onMessage }) {
    this.ws = null;
    this.pending = [];
    this.onEvent = onEvent;
    this.onState = onState;
    this.onOpen = onOpen;
    this.onClose = onClose;
    this.onError = onError;
    this.onMessage = onMessage;
  }

  isConnected() {
    return this.ws && this.ws.readyState === WebSocket.OPEN;
  }

  connect(url) {
    this.disconnect();
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.onOpen?.();
    };

    this.ws.onclose = () => {
      while (this.pending.length) {
        const p = this.pending.shift();
        clearTimeout(p.timer);
        p.reject(new Error('ws closed'));
      }
      this.onClose?.();
    };

    this.ws.onerror = () => {
      this.onError?.(new Error('ws error'));
    };

    this.ws.onmessage = (ev) => {
      let msg;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        this.onEvent?.(`non-json: ${String(ev.data).slice(0, 120)}`, 'err');
        return;
      }

      this.onMessage?.(msg);

      if (msg?.type === 'state') {
        this.onState?.(msg.data);
        return;
      }

      if (typeof msg?.ok === 'boolean') {
        const p = this.pending.shift();
        if (!p) {
          this.onEvent?.(`async ws message: ${JSON.stringify(msg)}`, 'info');
          return;
        }
        clearTimeout(p.timer);
        p.resolve(msg);
        return;
      }

      this.onEvent?.(`ws msg: ${JSON.stringify(msg)}`, 'info');
    };
  }

  disconnect() {
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        // ignore
      }
    }
    this.ws = null;
  }

  send(op, payload = {}, timeoutMs = 8000) {
    return new Promise((resolve, reject) => {
      if (!this.isConnected()) {
        reject(new Error('ws not connected'));
        return;
      }

      const req = { op, ...payload };
      const pending = {
        resolve,
        reject,
        timer: setTimeout(() => {
          const idx = this.pending.indexOf(pending);
          if (idx >= 0) this.pending.splice(idx, 1);
          reject(new Error(`timeout waiting response for op=${op}`));
        }, timeoutMs),
      };
      this.pending.push(pending);

      try {
        this.ws.send(JSON.stringify(req));
      } catch (e) {
        clearTimeout(pending.timer);
        const idx = this.pending.indexOf(pending);
        if (idx >= 0) this.pending.splice(idx, 1);
        reject(e);
      }
    });
  }
}
