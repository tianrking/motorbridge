import React from 'react';
import { useI18n } from '../i18n';

export function ConnectionPanel({
  wsUrl,
  setWsUrl,
  channel,
  setChannel,
  scanTimeoutMs,
  setScanTimeoutMs,
  connectWs,
  disconnectWs,
  collapsed,
  onToggleCollapsed,
}) {
  const { t } = useI18n();
  return (
    <section className="card glass">
      <div className="sectionTitle">
        <h2>{t('section_connection')}</h2>
        <button className="ghostBtn small" onClick={onToggleCollapsed}>
          {collapsed ? t('expand') : t('collapse')}
        </button>
      </div>

      {!collapsed && (
        <>
          <div className="grid3 denseGrid">
            <div className="field">
              <label>{t('ws_url')}</label>
              <input value={wsUrl} onChange={(e) => setWsUrl(e.target.value)} />
            </div>
            <div className="field">
              <label>{t('can_channel')}</label>
              <input value={channel} onChange={(e) => setChannel(e.target.value)} />
            </div>
            <div className="field">
              <label>{t('scan_timeout_ms')}</label>
              <input value={scanTimeoutMs} onChange={(e) => setScanTimeoutMs(e.target.value)} />
            </div>
          </div>

          <div className="row toolbar">
            <button className="primary strong" onClick={connectWs}>{t('connect')}</button>
            <button onClick={disconnectWs}>{t('disconnect')}</button>
            <span className="tip">{t('pure_frontend_tip')}</span>
          </div>
        </>
      )}

      {collapsed && <div className="tip">{t('websocket_can')}</div>}
    </section>
  );
}
