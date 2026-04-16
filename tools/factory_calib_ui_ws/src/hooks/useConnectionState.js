import { useState } from 'react';
import { APP_DEFAULTS } from '../lib/appConfig';
import { useGatewayBridge } from './useGatewayBridge';

export function useConnectionState({ pushLog, setStateSnapshot }) {
  const [wsUrl, setWsUrl] = useState(APP_DEFAULTS.wsUrl);
  const [channel, setChannel] = useState(APP_DEFAULTS.channel);
  const [scanTimeoutMs, setScanTimeoutMs] = useState(APP_DEFAULTS.scanTimeoutMs);

  const bridge = useGatewayBridge({ wsUrl, channel, pushLog, setStateSnapshot });

  return {
    wsUrl,
    setWsUrl,
    channel,
    setChannel,
    scanTimeoutMs,
    setScanTimeoutMs,
    ...bridge,
  };
}
