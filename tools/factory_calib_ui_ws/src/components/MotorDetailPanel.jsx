import React from 'react';
import { SET_ID_VENDORS, VENDOR_LABELS } from '../lib/constants';
import { formatLocal, motorKey, toHex } from '../lib/utils';
import { useI18n } from '../i18n';

function ModeSelect({ vendor, value, onChange }) {
  return (
    <select value={value} onChange={onChange}>
      {vendor === 'damiao' && (
        <>
          <option value="pos_vel">pos_vel</option>
          <option value="mit">mit</option>
          <option value="vel">vel</option>
          <option value="force_pos">force_pos</option>
        </>
      )}
      {vendor === 'robstride' && (
        <>
          <option value="mit">mit</option>
          <option value="vel">vel</option>
        </>
      )}
      {!['damiao', 'robstride'].includes(vendor) && <option value="vel">vel</option>}
    </select>
  );
}

function Field({ label, value, onChange }) {
  return (
    <div className="field">
      <label>{label}</label>
      <input value={value} onChange={onChange} />
    </div>
  );
}

function MetaItem({ label, value }) {
  return <div className="metaItem"><b>{label}</b> {value}</div>;
}

export function MotorDetailPanel({
  connected,
  activeMotor,
  activeControl,
  patchControl,
  controlMotor,
  probeMotor,
  setIdFor,
  verifyHit,
  refreshMotorState,
}) {
  const { t } = useI18n();
  if (!activeMotor || !activeControl) {
    return <div className="tip">{t('select_motor_tip')}</div>;
  }

  const key = motorKey(activeMotor);
  const patch = (field) => (e) => patchControl(key, { [field]: e.target.value });

  return (
    <>
      <div className="sectionTitle">
        <h2>{VENDOR_LABELS[activeMotor.vendor] || activeMotor.vendor} {toHex(activeMotor.esc_id)}</h2>
        <span>{t('mst')} {toHex(activeMotor.mst_id)} | {t('updated')} {formatLocal(activeMotor.updated_at_ms)}</span>
      </div>

      <div className="metaGrid">
        <MetaItem label={t('esc_id')} value={toHex(activeMotor.esc_id)} />
        <MetaItem label={t('mst_id')} value={toHex(activeMotor.mst_id)} />
        <MetaItem label={t('probe')} value={toHex(activeMotor.probe)} />
        <MetaItem label={t('detect')} value={activeMotor.detected_by || '-'} />
        <MetaItem label={t('online')} value={activeMotor.online === false ? t('offline') : t('online_unknown')} />
        <MetaItem label={t('model')} value={activeMotor.model_guess || activeMotor.model || '-'} />
        <MetaItem label={t('status')} value={Number.isFinite(activeMotor.status) ? String(activeMotor.status) : '-'} />
        <MetaItem label={t('pos')} value={Number.isFinite(activeMotor.pos) ? activeMotor.pos.toFixed(3) : '-'} />
        <MetaItem label={t('vel')} value={Number.isFinite(activeMotor.vel) ? activeMotor.vel.toFixed(3) : '-'} />
        <MetaItem label={t('torq')} value={Number.isFinite(activeMotor.torq) ? activeMotor.torq.toFixed(3) : '-'} />
        <MetaItem label={t('pmax')} value={Number.isFinite(activeMotor.pmax) ? activeMotor.pmax.toFixed(2) : '-'} />
        <MetaItem label={t('vmax')} value={Number.isFinite(activeMotor.vmax) ? activeMotor.vmax.toFixed(2) : '-'} />
        <MetaItem label={t('tmax')} value={Number.isFinite(activeMotor.tmax) ? activeMotor.tmax.toFixed(2) : '-'} />
      </div>

      <div className="grid3 denseGrid">
        <div className="field">
          <label>{t('mode')}</label>
          <ModeSelect vendor={activeMotor.vendor} value={activeControl.mode} onChange={patch('mode')} />
        </div>
        <Field label={t('target')} value={activeControl.target} onChange={patch('target')} />
        <Field label={t('vlim')} value={activeControl.vlim} onChange={patch('vlim')} />
        <Field label={t('kp')} value={activeControl.kp} onChange={patch('kp')} />
        <Field label={t('kd')} value={activeControl.kd} onChange={patch('kd')} />
        <Field label={t('tau')} value={activeControl.tau} onChange={patch('tau')} />
        <Field label={t('ratio')} value={activeControl.ratio} onChange={patch('ratio')} />
        <Field label={t('new_esc')} value={activeControl.newEsc} onChange={patch('newEsc')} />
        <Field label={t('new_mst')} value={activeControl.newMst} onChange={patch('newMst')} />
      </div>

      <div className="row toolbar">
        <button disabled={!connected} onClick={() => controlMotor(activeMotor, 'enable')}>{t('enable')}</button>
        <button disabled={!connected} onClick={() => controlMotor(activeMotor, 'disable')}>{t('disable')}</button>
        <button className="primary" disabled={!connected} onClick={() => controlMotor(activeMotor, 'move')}>{t('move')}</button>
        <button disabled={!connected} onClick={() => controlMotor(activeMotor, 'stop')}>{t('stop')}</button>
        <button disabled={!connected} onClick={() => probeMotor(activeMotor)}>{t('probe')}</button>
        <button disabled={!connected || !SET_ID_VENDORS.has(activeMotor.vendor)} onClick={() => setIdFor(activeMotor)}>{t('set_id')}</button>
        <button disabled={!connected} onClick={() => verifyHit(activeMotor)}>{t('verify')}</button>
        <button disabled={!connected} onClick={() => refreshMotorState(activeMotor)}>{t('refresh_state')}</button>
      </div>
      {!connected && <div className="tip">{t('connect_ws_first')}</div>}
    </>
  );
}
