import { useRef, useState } from 'react';
import { WsGatewayClient } from '../wsGatewayClient';
import { useI18n } from '../i18n';

export function useGatewayBridge({ wsUrl, channel, pushLog, setStateSnapshot }) {
  const { t } = useI18n();
  const [connText, setConnText] = useState(t('conn_disconnected'));
  const [connected, setConnected] = useState(false);
  const clientRef = useRef(null);
  const reconnectTimerRef = useRef(null);
  const reconnectAttemptRef = useRef(0);
  const shouldAutoReconnectRef = useRef(false);
  const connectingRef = useRef(false);

  const clearReconnectTimer = () => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
  };

  const reconnectDelayMs = (attempt) => {
    const base = 600;
    const max = 5000;
    const exp = Math.min(max, base * (2 ** Math.max(0, attempt)));
    return exp + Math.floor(Math.random() * 300);
  };

  const connectNow = (client, statusText = '') => {
    if (connectingRef.current || connected || client?.isConnected?.()) return;
    connectingRef.current = true;
    setConnText(statusText || t('conn_connecting'));
    try {
      client.connect(wsUrl.trim());
    } catch (e) {
      connectingRef.current = false;
      throw e;
    }
  };

  const ensureClient = () => {
    if (!clientRef.current) {
      clientRef.current = new WsGatewayClient({
        onEvent: (msg, lvl = 'info') => {
          const text = String(msg || '');
          if (text.startsWith('ws msg:') || text.startsWith('async ws message:') || text.startsWith('tx:')) {
            return;
          }
          pushLog(msg, lvl);
        },
        onState: (st) => {
          setStateSnapshot(JSON.stringify(st, null, 2));
        },
        onMessage: (msg) => {
          if (msg?.type !== 'event' || msg?.event !== 'connected' || !msg?.data) return;
          const d = msg.data;
          const target = d.default_target || {};
          setStateSnapshot(
            JSON.stringify(
              {
                gateway: {
                  router_mode: d.router_mode || 'unknown',
                  connected_bus: d.connected_bus ?? null,
                },
                target_default: {
                  vendor: target.vendor ?? d.vendor ?? null,
                  transport: target.transport ?? d.transport ?? null,
                  channel: target.channel ?? d.channel ?? null,
                  model: target.model ?? d.model ?? null,
                  motor_id: target.motor_id ?? d.motor_id ?? null,
                  feedback_id: target.feedback_id ?? d.feedback_id ?? null,
                },
              },
              null,
              2,
            ),
          );
        },
        onOpen: () => {
          clearReconnectTimer();
          reconnectAttemptRef.current = 0;
          connectingRef.current = false;
          setConnected(true);
          setConnText(t('conn_connected'));
          pushLog(t('log_connected', { url: wsUrl }), 'ok');
        },
        onClose: () => {
          connectingRef.current = false;
          setConnected(false);
          setConnText(t('conn_disconnected'));
          pushLog(t('log_disconnected'), 'err');
          if (!shouldAutoReconnectRef.current) return;

          clearReconnectTimer();
          const attempt = reconnectAttemptRef.current + 1;
          reconnectAttemptRef.current = attempt;
          const delay = reconnectDelayMs(attempt - 1);
          setConnText(t('conn_reconnecting_in', { sec: Math.ceil(delay / 1000) }));
          pushLog(t('log_reconnect_attempt', { n: attempt, ms: delay }), 'info');

          reconnectTimerRef.current = setTimeout(() => {
            reconnectTimerRef.current = null;
            try {
              connectNow(clientRef.current, t('conn_reconnecting_n', { n: attempt }));
            } catch (e) {
              pushLog(t('log_reconnect_start_failed', { err: e.message || e }), 'err');
            }
          }, delay);
        },
        onError: () => {
          connectingRef.current = false;
          pushLog(t('log_error'), 'err');
        },
      });
    }
    return clientRef.current;
  };

  const sendCmd = async (op, payload = {}, timeoutMs = 8000) => {
    const client = ensureClient();
    return client.send(op, payload, timeoutMs);
  };

  const connectWs = () => {
    shouldAutoReconnectRef.current = true;
    clearReconnectTimer();
    connectNow(ensureClient());
  };

  const disconnectWs = () => {
    shouldAutoReconnectRef.current = false;
    reconnectAttemptRef.current = 0;
    clearReconnectTimer();
    connectingRef.current = false;
    ensureClient().disconnect();
    setConnText(t('conn_disconnected'));
  };

  const closeBusQuietly = async () => {
    try {
      const ret = await sendCmd('close_bus', {}, 3000);
      if (!ret.ok) pushLog(t('log_close_bus_warn', { err: ret.error || 'unknown' }), 'err');
    } catch (e) {
      pushLog(t('log_close_bus_warn', { err: e.message || e }), 'err');
    }
  };

  const setTargetFor = async (vendor, model, motorId, feedbackId) => {
    const ret = await sendCmd(
      'set_target',
      {
        vendor,
        channel,
        model,
        motor_id: motorId,
        feedback_id: feedbackId,
      },
      10000,
    );
    if (!ret.ok) throw new Error(ret.error || 'set_target failed');
  };

  return {
    connText,
    connected,
    connectWs,
    disconnectWs,
    sendCmd,
    closeBusQuietly,
    setTargetFor,
  };
}
