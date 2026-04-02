import React from 'react';
import { VENDOR_LABELS } from '../lib/constants';
import { formatLocal, motorKey, scanSummary, toHex } from '../lib/utils';
import { useI18n } from '../i18n';

export function MotorCards({
  hits,
  connected,
  activeMotorKey,
  setActiveMotorKey,
  newCardKeys,
  cardRefs,
  removeMotorCard,
  moveMotorCard,
  probeMotor,
}) {
  const { t } = useI18n();
  const onDragStart = (e, key) => {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', key);
  };

  const onDrop = (e, targetKey) => {
    e.preventDefault();
    const fromKey = e.dataTransfer.getData('text/plain');
    moveMotorCard(fromKey, targetKey);
  };

  return (
    <div className="motorCards">
      {hits.length === 0 && <div className="tip">{t('no_motors')}</div>}
      {hits.map((hit) => {
        const key = motorKey(hit);
        const isActive = key === activeMotorKey;
        return (
          <button
            key={key}
            className={`motorCard ${isActive ? 'active' : ''} ${newCardKeys.has(key) ? 'new' : ''}`}
            onClick={() => setActiveMotorKey(key)}
            draggable
            onDragStart={(e) => onDragStart(e, key)}
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => onDrop(e, key)}
            ref={(el) => {
              cardRefs.current[key] = el;
            }}
          >
            <div className="motorCardTop motorCardTopNew">
              <span className="chip chipVendor">{VENDOR_LABELS[hit.vendor] || hit.vendor}</span>
              <div className="cardActions">
                <span className="chip chipId">ESC {toHex(hit.esc_id)}</span>
                <button
                  className="iconBtn ping"
                  title={t('ping_motor')}
                  disabled={!connected}
                  onClick={(e) => {
                    e.stopPropagation();
                    probeMotor(hit);
                  }}
                >
                  ●
                </button>
                <button
                  className="iconBtn"
                  title={t('move_by_drag')}
                  onClick={(e) => e.stopPropagation()}
                >
                  ↕
                </button>
                <button
                  className="iconBtn danger"
                  title={t('delete_card')}
                  onClick={(e) => {
                    e.stopPropagation();
                    removeMotorCard(hit);
                  }}
                >
                  ×
                </button>
              </div>
            </div>
            <div className="motorSignal">
              <span className={`dotLive ${hit.online === false ? 'off' : ''}`} />
              <span className="signalText">{hit.online === false ? t('offline') : t('online_unknown')}</span>
            </div>
            <div className="motorMeta"><strong>{t('mst')}</strong> {toHex(hit.mst_id)} | <strong>{t('probe')}</strong> {toHex(hit.probe)}</div>
            <div className="motorMeta"><strong>{t('model')}</strong> {hit.model_guess || hit.model || '-'}</div>
            <div className="motorMeta"><strong>{t('detect')}</strong> {hit.detected_by || '-'}</div>
            <div className="motorMeta motorSummary">{scanSummary(hit)}</div>
            <div className="motorMeta"><strong>{t('updated')}</strong> {formatLocal(hit.updated_at_ms)}</div>
            <div className="motorMeta"><strong>{t('last_check')}</strong> {formatLocal(hit.last_check_ms)}</div>
          </button>
        );
      })}
    </div>
  );
}
