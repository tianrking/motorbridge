import React from 'react';
import { useI18n } from '../i18n';

export function QuickMenu({ canAction, runScan, clearDevices, connectWs, disconnectWs }) {
  const { t } = useI18n();
  return (
    <section className="card glass actionMenu">
      <div className="menuGrid">
        <button className="primary strong" disabled={!canAction} onClick={() => runScan(['damiao'])}>
          {t('scan_damiao')}
        </button>
        <button className="primary strong" disabled={!canAction} onClick={() => runScan(['robstride'])}>
          {t('scan_robstride')}
        </button>
        <button disabled={!canAction} onClick={() => runScan()}>
          {t('scan_selected')}
        </button>
        <button onClick={clearDevices}>{t('clear_devices')}</button>
        <button onClick={connectWs}>{t('connect')}</button>
        <button onClick={disconnectWs}>{t('disconnect')}</button>
      </div>
    </section>
  );
}
