import React from 'react';
import { useI18n } from '../../i18n';
import { ProgressBar } from '../ProgressBar';

export function ParamTable({
  open,
  canAction,
  armToolbarBusy,
  paramBusy,
  paramInfo,
  paramProgress,
  paramRows,
  patchParam,
  readParams,
  writeParams,
  applyDefaultTemplate,
  onClose,
}) {
  const { t } = useI18n();
  if (!open) return null;

  return (
    <div className="armParamPanel">
      <div className="sectionTitle armPaneTitle">
        <h2>{t('arm_params_title')}</h2>
        <span className="tip">{t('arm_params_hint')}</span>
      </div>
      <div className="row toolbar compactToolbar">
        <button disabled={!canAction || armToolbarBusy} onClick={readParams}>
          {t('arm_read_params')}
        </button>
        <button className="primary" disabled={!canAction || armToolbarBusy} onClick={writeParams}>
          {t('arm_write_params')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={applyDefaultTemplate}>
          {t('arm_apply_default_template')}
        </button>
        <button className="ghostBtn" disabled={armToolbarBusy} onClick={onClose}>
          {t('close')}
        </button>
      </div>
      {paramInfo && <div className="tip">{paramInfo}</div>}
      <ProgressBar active={paramBusy || paramProgress?.active} progress={paramProgress} />
      <div className="armParamTableWrap">
        <table className="armParamTable">
          <thead>
            <tr>
              <th>{t('joint')}</th>
              <th>{t('arm_ctrl_mode')}</th>
              <th>{t('arm_current_bw')}</th>
              <th>{t('arm_vel_kp')}</th>
              <th>{t('arm_vel_ki')}</th>
              <th>{t('arm_pos_kp')}</th>
              <th>{t('arm_pos_ki')}</th>
              <th>{t('status')}</th>
            </tr>
          </thead>
          <tbody>
            {paramRows.map((row) => (
              <tr key={row.key}>
                <td>{row.joint}</td>
                <td>
                  <select
                    value={row.values.ctrlMode}
                    onChange={(e) => patchParam(row.key, 'ctrlMode', e.target.value)}
                    disabled={paramBusy}
                  >
                    <option value="1">1: MIT</option>
                    <option value="2">2: PosVel</option>
                    <option value="3">3: Vel</option>
                    <option value="4">4: ForcePos</option>
                  </select>
                </td>
                <td>
                  <input
                    value={row.values.currentBw}
                    onChange={(e) => patchParam(row.key, 'currentBw', e.target.value)}
                    disabled={paramBusy}
                  />
                </td>
                <td>
                  <input
                    value={row.values.velKp}
                    onChange={(e) => patchParam(row.key, 'velKp', e.target.value)}
                    disabled={paramBusy}
                  />
                </td>
                <td>
                  <input
                    value={row.values.velKi}
                    onChange={(e) => patchParam(row.key, 'velKi', e.target.value)}
                    disabled={paramBusy}
                  />
                </td>
                <td>
                  <input
                    value={row.values.posKp}
                    onChange={(e) => patchParam(row.key, 'posKp', e.target.value)}
                    disabled={paramBusy}
                  />
                </td>
                <td>
                  <input
                    value={row.values.posKi}
                    onChange={(e) => patchParam(row.key, 'posKi', e.target.value)}
                    disabled={paramBusy}
                  />
                </td>
                <td className={row.error ? 'errText' : ''}>
                  {row.error || (row.loaded ? t('ok') : '-')}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
