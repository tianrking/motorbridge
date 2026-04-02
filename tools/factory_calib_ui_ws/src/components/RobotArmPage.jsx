import React from 'react';
import { useI18n } from '../i18n';
import { ROBOT_ARM_MODELS } from '../lib/robotArm';
import { parseNum, toHex } from '../lib/utils';
import { ArmUrdfViewer } from './ArmUrdfViewer';

function numText(v, digits = 3) {
  return Number.isFinite(v) ? Number(v).toFixed(digits) : '-';
}

export function RobotArmPage({
  connected,
  canAction,
  robotArmModel,
  setRobotArmModel,
  robotArmJointRows,
  ensureRobotArmCards,
  scanRobotArmJoint,
  scanRobotArmAll,
  patchControl,
  controlMotor,
  refreshMotorState,
}) {
  const { t } = useI18n();
  const [activeJointKey, setActiveJointKey] = React.useState('');

  React.useEffect(() => {
    if (robotArmJointRows.length === 0) return;
    if (!activeJointKey) {
      setActiveJointKey(robotArmJointRows[0].key);
      return;
    }
    const exists = robotArmJointRows.some((x) => x.key === activeJointKey);
    if (!exists) setActiveJointKey(robotArmJointRows[0].key);
  }, [robotArmJointRows, activeJointKey]);

  const activeRow = React.useMemo(
    () => robotArmJointRows.find((x) => x.key === activeJointKey) || robotArmJointRows[0] || null,
    [robotArmJointRows, activeJointKey],
  );

  const sliderValue = activeRow
    ? Math.max(-3.14, Math.min(3.14, parseNum(activeRow.control.target, 0)))
    : 0;
  const jointTargets = React.useMemo(() => {
    const out = {};
    robotArmJointRows.forEach((row) => {
      out[`joint${row.joint}`] = Math.max(-3.14, Math.min(3.14, parseNum(row.control.target, 0)));
    });
    return out;
  }, [robotArmJointRows]);

  return (
    <section className="card glass">
      <div className="sectionTitle">
        <h2>{t('robot_arm_title')}</h2>
        <span className="tip">{t('robot_arm_desc')}</span>
      </div>

      {!connected && <div className="offlineBanner">{t('ws_disconnected_motor')}</div>}

      <div className="row toolbar compactToolbar">
        <div className="field miniField">
          <label>{t('arm_model')}</label>
          <select value={robotArmModel} onChange={(e) => setRobotArmModel(e.target.value)}>
            {ROBOT_ARM_MODELS.map((m) => (
              <option key={m.key} value={m.key}>
                {m.label}
              </option>
            ))}
          </select>
        </div>
        <button onClick={ensureRobotArmCards}>{t('arm_prepare_cards')}</button>
        <button className="primary" disabled={!canAction} onClick={scanRobotArmAll}>
          {t('arm_scan_all')}
        </button>
      </div>

      <div className="armStudio">
        <div className="armLeftPane">
          <div className="sectionTitle armPaneTitle">
            <h2>{t('arm_left_joints')}</h2>
            <span className="tip">{robotArmJointRows.length}/7</span>
          </div>
          <div className="armJointList">
            {robotArmJointRows.map((row) => (
              <div
                key={row.key}
                className={`armJointCard ${activeRow?.key === row.key ? 'active' : ''}`}
                onClick={() => setActiveJointKey(row.key)}
              >
                <div className="armCardHead">
                  <strong>
                    {t('joint')} {row.joint}
                  </strong>
                  <span className={`chip ${row.hit.online === false ? '' : 'chipOk'}`}>
                    {row.hit.online === false ? t('offline') : t('online_unknown')}
                  </span>
                </div>
                <div className="armMeta">
                  <span>ESC {toHex(row.hit.esc_id)}</span>
                  <span>MST {toHex(row.hit.mst_id)}</span>
                </div>
                <div className="armMeta">
                  <span>{t('pos')} {numText(row.hit.pos)}</span>
                  <span>{t('vel')} {numText(row.hit.vel)}</span>
                  <span>{t('torq')} {numText(row.hit.torq)}</span>
                </div>
                <div className="row compactToolbar">
                  <button
                    className="small ghostBtn"
                    disabled={!connected}
                    onClick={(e) => {
                      e.stopPropagation();
                      scanRobotArmJoint(row.joint);
                    }}
                  >
                    {t('arm_scan_joint')}
                  </button>
                  <button
                    className="small ghostBtn"
                    disabled={!connected}
                    onClick={(e) => {
                      e.stopPropagation();
                      refreshMotorState(row.hit);
                    }}
                  >
                    {t('refresh_state')}
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="armRightPane">
          {activeRow && (
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
                    onChange={(e) => patchControl(activeRow.key, { target: e.target.value })}
                  />
                </div>
              </div>

              <div className="field armSliderWrap">
                <label>
                  {t('arm_pos_slider')}: {sliderValue.toFixed(3)}
                </label>
                <input
                  type="range"
                  min="-3.14"
                  max="3.14"
                  step="0.01"
                  value={sliderValue}
                  onChange={(e) => patchControl(activeRow.key, { target: e.target.value })}
                />
                <div className="armSliderMeta">
                  <span>{t('arm_pos_range_hint')}</span>
                  <input
                    className="armPosInput"
                    value={activeRow.control.target}
                    onChange={(e) => patchControl(activeRow.key, { target: e.target.value })}
                  />
                </div>
              </div>

              <div className="row toolbar compactToolbar">
                <button disabled={!connected} onClick={() => controlMotor(activeRow.hit, 'enable')}>
                  {t('enable')}
                </button>
                <button disabled={!connected} onClick={() => controlMotor(activeRow.hit, 'disable')}>
                  {t('disable')}
                </button>
                <button className="primary" disabled={!connected} onClick={() => controlMotor(activeRow.hit, 'move')}>
                  {t('move')}
                </button>
                <button disabled={!connected} onClick={() => controlMotor(activeRow.hit, 'stop')}>
                  {t('stop')}
                </button>
                <button disabled={!connected} onClick={() => refreshMotorState(activeRow.hit)}>
                  {t('refresh_state')}
                </button>
              </div>
            </div>
          )}

          <div className="armSimPanel">
            <div className="sectionTitle armPaneTitle">
              <h2>{t('arm_sim_title')}</h2>
              <span className="tip">{t('arm_ws_bridge_hint')}</span>
            </div>
            <p className="tip">{t('arm_sim_desc')}</p>
            <ArmUrdfViewer jointTargets={jointTargets} />
          </div>
        </div>
      </div>
    </section>
  );
}
