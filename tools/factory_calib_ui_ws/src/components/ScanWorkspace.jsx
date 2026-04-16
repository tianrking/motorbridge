import React from 'react';
import { VENDORS, VENDOR_LABELS } from '../lib/constants';
import { useI18n } from '../i18n';
import { ProgressBar } from './ProgressBar';
import { CollapsibleSection } from './CollapsibleSection';

function setVendorField(setVendors, vendor, field, value) {
  setVendors((prev) => ({
    ...prev,
    [vendor]: { ...prev[vendor], [field]: value },
  }));
}

export function ScanWorkspace({
  vendors,
  setVendors,
  connected,
  canAction,
  scanBusy,
  scanProgress,
  scanFoundFx,
  runScan,
  clearDevices,
  manualDraft,
  setManualDraft,
  addManualCard,
  scanCollapsed,
  onToggleScanCollapsed,
  manualCollapsed,
  onToggleManualCollapsed,
}) {
  const { t } = useI18n();
  return (
    <CollapsibleSection
      title={t('section_scan')}
      collapsed={scanCollapsed}
      onToggleCollapsed={onToggleScanCollapsed}
    >

      {!connected && <div className="offlineBanner">{t('ws_disconnected_scan')}</div>}

      {!scanCollapsed && (
        <>
          <div className="vendorGrid">
            {VENDORS.map((vendor) => {
              const cfg = vendors[vendor];
              return (
                <div className={`vendorCard ${cfg.enabled ? 'active' : ''}`} key={vendor}>
                  <div className="vendorHead">
                    <label className="checkWrap">
                      <input
                        type="checkbox"
                        checked={cfg.enabled}
                        onChange={(e) => setVendorField(setVendors, vendor, 'enabled', e.target.checked)}
                      />
                      <span>{VENDOR_LABELS[vendor]}</span>
                    </label>
                    <span className="chip">{vendor}</span>
                  </div>

                  <div className="field">
                    <label>{t('model')}</label>
                    <input
                      value={cfg.model}
                      onChange={(e) => setVendorField(setVendors, vendor, 'model', e.target.value)}
                    />
                  </div>

                  <div className="grid2 tight">
                    <div className="field">
                      <label>{t('start')}</label>
                      <input
                        value={cfg.startId}
                        onChange={(e) => setVendorField(setVendors, vendor, 'startId', e.target.value)}
                      />
                    </div>
                    <div className="field">
                      <label>{t('end')}</label>
                      <input
                        value={cfg.endId}
                        onChange={(e) => setVendorField(setVendors, vendor, 'endId', e.target.value)}
                      />
                    </div>
                  </div>

                  <div className="field">
                    <label>{vendor === 'damiao' ? t('feedback_base') : vendor === 'robstride' ? t('feedback_id') : t('extra')}</label>
                    {vendor === 'damiao' && (
                      <input
                        value={cfg.feedbackBase}
                        onChange={(e) => setVendorField(setVendors, vendor, 'feedbackBase', e.target.value)}
                      />
                    )}
                    {vendor === 'robstride' && (
                      <input
                        value={cfg.feedbackId}
                        onChange={(e) => setVendorField(setVendors, vendor, 'feedbackId', e.target.value)}
                      />
                    )}
                    {vendor !== 'damiao' && vendor !== 'robstride' && <input value="-" readOnly />}
                  </div>
                </div>
              );
            })}
          </div>

          <div className="row toolbar">
            <button className="primary strong" disabled={!canAction} onClick={() => runScan()}>
              {t('scan_selected_vendors')}
            </button>
            <button disabled={!canAction} onClick={() => runScan(['damiao'])}>{t('scan_damiao')}</button>
            <button disabled={!canAction} onClick={() => runScan(['robstride'])}>{t('scan_robstride')}</button>
            <button onClick={clearDevices}>{t('clear_list')}</button>
          </div>

          <ProgressBar active={scanBusy || scanProgress.active} progress={scanProgress} />

          {scanFoundFx?.visible && (
            <>
              <div className="scanFoundFx" key={scanFoundFx.seq}>
                <span>{scanFoundFx.message || t('motor_found')}</span>
              </div>
              <div className="starBurstLayer" aria-hidden="true">
                <span className="star s1">✦</span>
                <span className="star s2">✧</span>
                <span className="star s3">✦</span>
                <span className="star s4">✧</span>
                <span className="star s5">✦</span>
                <span className="star s6">✧</span>
              </div>
            </>
          )}
        </>
      )}

      <div className="manualCard">
        <div className="sectionTitle">
          <h2>{t('section_manual')}</h2>
          <button className="ghostBtn small" onClick={onToggleManualCollapsed}>
            {manualCollapsed ? t('expand') : t('collapse')}
          </button>
        </div>

        {!manualCollapsed && (
          <>
            <div className="grid3 denseGrid">
              <div className="field">
                <label>{t('vendor')}</label>
                <select
                  value={manualDraft.vendor}
                  onChange={(e) => setManualDraft((prev) => ({ ...prev, vendor: e.target.value }))}
                >
                  <option value="damiao">damiao</option>
                  <option value="robstride">robstride</option>
                  <option value="myactuator">myactuator</option>
                  <option value="hightorque">hightorque</option>
                  <option value="hexfellow">hexfellow</option>
                </select>
              </div>
              <div className="field">
                <label>{t('model')}</label>
                <input
                  value={manualDraft.model}
                  onChange={(e) => setManualDraft((prev) => ({ ...prev, model: e.target.value }))}
                />
              </div>
              <div className="field">
                <label>{t('control_id_esc')}</label>
                <input
                  value={manualDraft.escId}
                  onChange={(e) => setManualDraft((prev) => ({ ...prev, escId: e.target.value }))}
                />
              </div>
              <div className="field">
                <label>{t('feedback_id_mst')}</label>
                <input
                  value={manualDraft.mstId}
                  onChange={(e) => setManualDraft((prev) => ({ ...prev, mstId: e.target.value }))}
                />
              </div>
            </div>
            <div className="row toolbar">
              <button className="primary" onClick={addManualCard}>{t('add_card')}</button>
            </div>
          </>
        )}
      </div>
    </CollapsibleSection>
  );
}
