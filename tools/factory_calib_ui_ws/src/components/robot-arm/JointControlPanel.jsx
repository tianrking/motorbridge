import React from 'react';
import { useI18n } from '../../i18n';
import { toHex } from '../../lib/utils';

export function JointControlPanel({
  activeRow,
  perJointBusy,
  liveMove,
  sliderValue,
  limitWarn,
  patchControl,
  onSliderTargetChange,
  jointLimit,
  setUiPref,
  controlMotor,
  refreshMotorState,
  clampTargetForRow,
  setLimitWarn,
  showLimitToast,
}) {
  const { t } = useI18n();
  if (!activeRow) return null;
  return (
    <div className="armControlPanel">
      <div className="sectionTitle armPaneTitle">
        <h2>
          {t('arm_right_control')} · {t('joint')} {activeRow.joint}
        </h2>
        <span className="tip">
          ESC {toHex(activeRow.hit.esc_id)} / MST {toHex(activeRow.hit.mst_id)}
        </span>
      </div>

      <div className="grid3 tight armFields">
        <div className="field">
          <label>{t('mode')}</label>
          <select
            value={activeRow.control.mode}
            onChange={(e) => patchControl(activeRow.key, { mode: e.target.value })}
          >
            <option value="pos_vel">pos_vel</option>
            <option value="mit">mit</option>
            <option value="vel">vel</option>
            <option value="force_pos">force_pos</option>
          </select>
        </div>
        <div className="field">
          <label>{t('vlim')}</label>
          <input
            value={activeRow.control.vlim}
            onChange={(e) => patchControl(activeRow.key, { vlim: e.target.value })}
          />
        </div>
        <div className="field">
          <label>{t('tau')}</label>
          <input
            value={activeRow.control.tau}
            onChange={(e) => patchControl(activeRow.key, { tau: e.target.value })}
          />
        </div>
        <div className="field">
          <label>{t('kp')}</label>
          <input
            value={activeRow.control.kp}
            onChange={(e) => patchControl(activeRow.key, { kp: e.target.value })}
          />
        </div>
        <div className="field">
          <label>{t('kd')}</label>
          <input
            value={activeRow.control.kd}
            onChange={(e) => patchControl(activeRow.key, { kd: e.target.value })}
          />
        </div>
        <div className="field">
          <label>{t('target')}</label>
          <input
            value={activeRow.control.target}
            onChange={(e) => onSliderTargetChange(e.target.value)}
          />
        </div>
      </div>

      <div className="field armSliderWrap">
        <label>
          {t('arm_pos_slider')}: {sliderValue.toFixed(3)}
        </label>
        <input
          type="range"
          min={String(jointLimit(activeRow.joint).min)}
          max={String(jointLimit(activeRow.joint).max)}
          step="0.01"
          value={sliderValue}
          onChange={(e) => onSliderTargetChange(e.target.value)}
        />
        <div className="armSliderMeta">
          <label className="armLiveToggle">
            <input
              type="checkbox"
              checked={liveMove}
              disabled={perJointBusy}
              onChange={(e) => setUiPref('armSliderLiveMove', e.target.checked)}
            />
            <span>{t('arm_live_move')}</span>
          </label>
          <span>{liveMove ? t('arm_live_move_on') : t('arm_live_move_off')}</span>
        </div>
        <div className="armSliderMeta">
          <span>
            {t('arm_pos_range_hint')}: {jointLimit(activeRow.joint).min.toFixed(2)} ..{' '}
            {jointLimit(activeRow.joint).max.toFixed(2)}
          </span>
          <input
            className="armPosInput"
            value={activeRow.control.target}
            disabled={perJointBusy}
            onChange={(e) => onSliderTargetChange(e.target.value)}
          />
        </div>
        {limitWarn && <div className="tip warnText">{limitWarn}</div>}
      </div>

      <div className="row toolbar compactToolbar">
        <button disabled={perJointBusy} onClick={() => controlMotor(activeRow.hit, 'enable')}>
          {t('enable')}
        </button>
        <button disabled={perJointBusy} onClick={() => controlMotor(activeRow.hit, 'disable')}>
          {t('disable')}
        </button>
        <button
          className="primary"
          disabled={perJointBusy}
          onClick={() => {
            const checked = clampTargetForRow(activeRow, activeRow.control.target);
            if (checked.clipped) {
              const msg = t('arm_limit_blocked', {
                joint: activeRow.joint,
                req: checked.raw.toFixed(3),
                min: checked.lim.min.toFixed(3),
                max: checked.lim.max.toFixed(3),
                use: checked.clamped.toFixed(3),
              });
              setLimitWarn(msg);
              showLimitToast(msg);
              patchControl(activeRow.key, { target: String(checked.clamped) });
            } else {
              setLimitWarn('');
            }
            controlMotor(activeRow.hit, 'move', { target: String(checked.clamped) });
          }}
        >
          {t('move')}
        </button>
        <button disabled={perJointBusy} onClick={() => controlMotor(activeRow.hit, 'stop')}>
          {t('stop')}
        </button>
        <button disabled={perJointBusy} onClick={() => refreshMotorState(activeRow.hit)}>
          {t('refresh_state')}
        </button>
      </div>
    </div>
  );
}
