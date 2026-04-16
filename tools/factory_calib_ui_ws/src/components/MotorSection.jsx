import React, { useMemo, useState } from 'react';
import { MotorCards } from './MotorCards';
import { MotorDetailPanel } from './MotorDetailPanel';
import { useI18n } from '../i18n';
import { CollapsibleSection } from './CollapsibleSection';

export function MotorSection({
  hits,
  selectedHits,
  connected,
  activeMotorKey,
  setActiveMotorKey,
  newCardKeys,
  cardRefs,
  removeMotorCard,
  clearAllMotors,
  clearOfflineMotors,
  moveMotorCard,
  activeMotor,
  activeControl,
  patchControl,
  controlMotor,
  zeroMotor,
  probeMotor,
  setIdFor,
  verifyHit,
  refreshMotorState,
  collapsed,
  onToggleCollapsed,
}) {
  const { t } = useI18n();
  const [search, setSearch] = useState('');
  const [vendorFilter, setVendorFilter] = useState('all');
  const [onlineFilter, setOnlineFilter] = useState('all');

  const vendors = useMemo(() => {
    const set = new Set(hits.map((h) => h.vendor));
    return ['all', ...set];
  }, [hits]);

  const filteredHits = useMemo(() => {
    const q = search.trim().toLowerCase();
    return hits.filter((h) => {
      if (vendorFilter !== 'all' && h.vendor !== vendorFilter) return false;
      if (onlineFilter === 'online' && h.online === false) return false;
      if (onlineFilter === 'offline' && h.online !== false) return false;
      if (!q) return true;
      const text = `${h.vendor} ${h.model || ''} ${h.model_guess || ''} ${h.esc_id} ${h.mst_id}`.toLowerCase();
      return text.includes(q);
    });
  }, [hits, search, vendorFilter, onlineFilter]);

  return (
    <CollapsibleSection
      title={t('section_motors')}
      collapsed={collapsed}
      onToggleCollapsed={onToggleCollapsed}
      collapsedHint={`${hits.length} ${t('detected')}`}
    >

      {!collapsed && (
        <>
          <div className="row toolbar compactToolbar">
            <span className="tip">{hits.length} {t('detected')} | {selectedHits.length} {t('selected')} | {filteredHits.length} {t('shown')}</span>
            <input
              className="inlineInput"
              placeholder={t('search_placeholder')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <select className="inlineSelect" value={vendorFilter} onChange={(e) => setVendorFilter(e.target.value)}>
              {vendors.map((v) => (
                <option key={v} value={v}>{v === 'all' ? t('all') : v}</option>
              ))}
            </select>
            <select className="inlineSelect" value={onlineFilter} onChange={(e) => setOnlineFilter(e.target.value)}>
              <option value="all">{t('all')}</option>
              <option value="online">{t('online_unknown')}</option>
              <option value="offline">{t('offline')}</option>
            </select>
            <button onClick={clearOfflineMotors}>{t('clear_offline')}</button>
            <button onClick={clearAllMotors}>{t('clear_all_motors')}</button>
          </div>

          {!connected && <div className="offlineBanner">{t('ws_disconnected_motor')}</div>}

          <div className="motorLayout">
            <MotorCards
              hits={filteredHits}
              connected={connected}
              activeMotorKey={activeMotorKey}
              setActiveMotorKey={setActiveMotorKey}
              newCardKeys={newCardKeys}
              cardRefs={cardRefs}
              removeMotorCard={removeMotorCard}
              moveMotorCard={moveMotorCard}
              probeMotor={probeMotor}
            />

            <div className="motorPanel">
              <MotorDetailPanel
                connected={connected}
                activeMotor={activeMotor}
                activeControl={activeControl}
                patchControl={patchControl}
                controlMotor={controlMotor}
                zeroMotor={zeroMotor}
                probeMotor={probeMotor}
                setIdFor={setIdFor}
                verifyHit={verifyHit}
                refreshMotorState={refreshMotorState}
              />
            </div>
          </div>
        </>
      )}
    </CollapsibleSection>
  );
}
