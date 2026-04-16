import React from 'react';
import { useI18n } from '../i18n';
import { CollapsibleSection } from './CollapsibleSection';
import { useMotorStudioContext } from '../hooks/useMotorStudioContext';

export function ConnectionPanel() {
  const { t } = useI18n();
  const {
    wsUrl,
    setWsUrl,
    channel,
    setChannel,
    targetTransport,
    targetSerialPort,
    scanTimeoutMs,
    setScanTimeoutMs,
    connectWs,
    disconnectWs,
    uiPrefs,
    toggleUiPref,
  } = useMotorStudioContext();
  const collapsed = uiPrefs.sectionConnectionCollapsed;
  const onToggleCollapsed = () => toggleUiPref('sectionConnectionCollapsed');
  const isDmSerial = String(targetTransport || '').trim().toLowerCase() === 'dm-serial';
  const transportText = String(targetTransport || 'auto').trim() || 'auto';
  const serialText = String(targetSerialPort || '').trim() || t('serial_port_gateway_managed');
  return (
    <CollapsibleSection
      title={t('section_connection')}
      collapsed={collapsed}
      onToggleCollapsed={onToggleCollapsed}
      collapsedHint={t('websocket_can')}
    >

      {!collapsed && (
        <>
          <div className="connCompactGrid">
            <div className="field compactField">
              <label>{t('ws_url')}</label>
              <input value={wsUrl} onChange={(e) => setWsUrl(e.target.value)} />
            </div>
            <div className="field compactField">
              <label>{t('transport')}</label>
              <input value={transportText} readOnly />
            </div>
            <div className="field compactField">
              <label>{isDmSerial ? t('serial_port') : t('can_channel')}</label>
              {isDmSerial ? (
                <input value={serialText} readOnly disabled title={t('serial_port_gateway_managed')} />
              ) : (
                <input value={channel} onChange={(e) => setChannel(e.target.value)} />
              )}
            </div>
            <div className="field compactField">
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
    </CollapsibleSection>
  );
}
