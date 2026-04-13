export class WsGatewayClient {
  constructor({ onEvent, onState, onOpen, onClose, onError, onMessage }) {
    this.ws = null;
    this.pendingByReqId = new Map();
    this.pendingLegacyQueue = [];
    this.nextReqId = 1;
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
      for (const p of this.pendingByReqId.values()) {
        clearTimeout(p.timer);
        p.reject(new Error('ws closed'));
      }
      this.pendingByReqId.clear();

      while (this.pendingLegacyQueue.length) {
        const p = this.pendingLegacyQueue.shift();
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
        const reqId = msg?.req_id;
        const p =
          reqId != null && this.pendingByReqId.has(reqId)
            ? this.pendingByReqId.get(reqId)
            : this.pendingLegacyQueue.shift();
        if (!p) {
          this.onEvent?.(`async ws message: ${JSON.stringify(msg)}`, 'info');
          return;
        }
        if (reqId != null) {
          this.pendingByReqId.delete(reqId);
          const idx = this.pendingLegacyQueue.indexOf(p);
          if (idx >= 0) this.pendingLegacyQueue.splice(idx, 1);
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

      const reqId = this.nextReqId++;
      const req = { op, req_id: reqId, ...payload };
      const pending = {
        resolve,
        reject,
        reqId,
        timer: setTimeout(() => {
          this.pendingByReqId.delete(reqId);
          const idx = this.pendingLegacyQueue.indexOf(pending);
          if (idx >= 0) this.pendingLegacyQueue.splice(idx, 1);
          reject(new Error(`timeout waiting response for op=${op}`));
        }, timeoutMs),
      };
      this.pendingByReqId.set(reqId, pending);
      this.pendingLegacyQueue.push(pending);

      try {
        this.ws.send(JSON.stringify(req));
      } catch (e) {
        clearTimeout(pending.timer);
        this.pendingByReqId.delete(reqId);
        const idx = this.pendingLegacyQueue.indexOf(pending);
        if (idx >= 0) this.pendingLegacyQueue.splice(idx, 1);
        reject(e);
      }
    });
  }
}
