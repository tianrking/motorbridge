import React from 'react';
import { useI18n } from '../i18n';
import { useMotorStudioContext } from '../hooks/useMotorStudioContext';

export function QuickMenu() {
  const { t } = useI18n();
  const { canAction, runScan, clearDevices, connectWs, disconnectWs } = useMotorStudioContext();
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
