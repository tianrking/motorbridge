import React from 'react';
import { useI18n } from '../i18n';
import { ROBOT_ARM_MODELS } from '../lib/robotArm';
import { parseNum, toHex } from '../lib/utils';
import { ArmUrdfViewer } from './ArmUrdfViewer';

function numText(v, digits = 3) {
  return Number.isFinite(v) ? Number(v).toFixed(digits) : '-';
}

const REBOT_ARM_DAMIAO_DEFAULT_TEMPLATE = {
  1: { ctrlMode: '2', currentBw: '1000', velKp: '0.0125', velKi: '0.004', posKp: '150', posKi: '0.5' },
  2: { ctrlMode: '2', currentBw: '1000', velKp: '0.013', velKi: '0.004', posKp: '200', posKi: '10' },
  3: { ctrlMode: '2', currentBw: '1000', velKp: '0.013', velKi: '0.004', posKp: '200', posKi: '10' },
  4: { ctrlMode: '2', currentBw: '1000', velKp: '0.0008', velKi: '0.002', posKp: '50', posKi: '1' },
  5: { ctrlMode: '2', currentBw: '1000', velKp: '0.0008', velKi: '0.002', posKp: '50', posKi: '1' },
  6: { ctrlMode: '2', currentBw: '1000', velKp: '0.0008', velKi: '0.002', posKp: '50', posKi: '1' },
};

const REBOT_ARM_JOINT_LIMITS = {
  1: { min: -2.61, max: 2.61 },
  2: { min: -3.7, max: 0.0 },
  3: { min: -3.7, max: 0.0 },
  4: { min: -1.57, max: 1.57 },
  5: { min: -1.57, max: 1.57 },
  6: { min: -1.57, max: 1.57 },
  7: { min: -5.7, max: 0.0 },
};
const ZERO_SAFE_EPS_RAD = 0.08;

function jointLimit(joint) {
  return REBOT_ARM_JOINT_LIMITS[Number(joint)] || { min: -3.14, max: 3.14 };
}

function clampByLimit(value, lim) {
  return Math.max(lim.min, Math.min(lim.max, value));
}

export function RobotArmPage({
  connected,
  canAction,
  robotArmModel,
  armScanBusy,
  armScanProgress,
  armBulkBusy,
  armSelfCheckBusy,
  armSelfCheckProgress,
  armSelfCheckReport,
  setRobotArmModel,
  robotArmJointRows,
  ensureRobotArmCards,
  scanRobotArmJoint,
  scanRobotArmAll,
  runRobotArmSelfCheck,
  enableAllRobotArm,
  disableAllRobotArm,
  zeroAllRobotArm,
  resetPoseRobotArm,
  readRobotArmControlParams,
  writeRobotArmControlParams,
  patchControl,
  controlMotor,
  refreshMotorState,
  uiPrefs,
  setUiPref,
}) {
  const { t } = useI18n();
  const [activeJointKey, setActiveJointKey] = React.useState('');
  const [paramPanelOpen, setParamPanelOpen] = React.useState(false);
  const [paramBusy, setParamBusy] = React.useState(false);
  const [paramRows, setParamRows] = React.useState([]);
  const [paramInfo, setParamInfo] = React.useState('');
  const [paramProgress, setParamProgress] = React.useState({
    active: false,
    done: 0,
    total: 0,
    label: '',
    percent: 0,
  });
  const [limitWarn, setLimitWarn] = React.useState('');
  const [limitToast, setLimitToast] = React.useState({ visible: false, message: '', seq: 0 });
  const [urdfResetSeq, setUrdfResetSeq] = React.useState(0);
  const [firstUseOpen, setFirstUseOpen] = React.useState(false);
  const [zeroConfirm, setZeroConfirm] = React.useState({
    open: false,
    title: '',
    message: '',
    danger: false,
  });
  const zeroConfirmResolverRef = React.useRef(null);

  React.useEffect(() => {
    if (robotArmJointRows.length === 0) return;
    if (!activeJointKey) {
      setActiveJointKey(robotArmJointRows[0].key);
      return;
    }
    const exists = robotArmJointRows.some((x) => x.key === activeJointKey);
    if (!exists) setActiveJointKey(robotArmJointRows[0].key);
  }, [robotArmJointRows, activeJointKey]);

  React.useEffect(() => {
    setLimitWarn('');
  }, [activeJointKey]);

  React.useEffect(() => {
    if (!limitToast.visible) return undefined;
    const timer = setTimeout(() => {
      setLimitToast((prev) => ({ ...prev, visible: false }));
    }, 2600);
    return () => clearTimeout(timer);
  }, [limitToast]);

  const showLimitToast = React.useCallback((message) => {
    setLimitToast((prev) => ({ visible: true, message, seq: prev.seq + 1 }));
  }, []);

  const askZeroConfirm = React.useCallback((cfg) => {
    return new Promise((resolve) => {
      zeroConfirmResolverRef.current = resolve;
      setZeroConfirm({
        open: true,
        title: cfg?.title || '',
        message: cfg?.message || '',
        danger: Boolean(cfg?.danger),
      });
    });
  }, []);

  const closeZeroConfirm = React.useCallback((accepted) => {
    const done = zeroConfirmResolverRef.current;
    zeroConfirmResolverRef.current = null;
    setZeroConfirm((prev) => ({ ...prev, open: false }));
    if (done) done(Boolean(accepted));
  }, []);

  React.useEffect(() => {
    return () => {
      if (zeroConfirmResolverRef.current) {
        zeroConfirmResolverRef.current(false);
        zeroConfirmResolverRef.current = null;
      }
    };
  }, []);

  const activeRow = React.useMemo(
    () => robotArmJointRows.find((x) => x.key === activeJointKey) || robotArmJointRows[0] || null,
    [robotArmJointRows, activeJointKey],
  );

  const sliderValue = activeRow
    ? clampByLimit(parseNum(activeRow.control.target, 0), jointLimit(activeRow.joint))
    : 0;
  const jointTargets = React.useMemo(() => {
    const out = {};
    robotArmJointRows.forEach((row) => {
      out[`joint${row.joint}`] = clampByLimit(parseNum(row.control.target, 0), jointLimit(row.joint));
    });
    return out;
  }, [robotArmJointRows]);

  const zeroSafety = React.useMemo(() => {
    const notReady = robotArmJointRows
      .map((row) => {
        const p = Number(row?.hit?.pos);
        if (!Number.isFinite(p)) return { joint: row.joint, pos: null };
        return Math.abs(p) <= ZERO_SAFE_EPS_RAD ? null : { joint: row.joint, pos: p };
      })
      .filter(Boolean);

    return {
      ok: notReady.length === 0,
      notReady,
    };
  }, [robotArmJointRows]);

  const liveMove = Boolean(uiPrefs?.armSliderLiveMove);
  const liveMoveTimerRef = React.useRef(null);
  const pendingLiveMoveRef = React.useRef(null);

  React.useEffect(() => () => {
    if (liveMoveTimerRef.current) clearTimeout(liveMoveTimerRef.current);
  }, []);

  React.useEffect(() => {
    if (!armBulkBusy) return;
    pendingLiveMoveRef.current = null;
    if (liveMoveTimerRef.current) {
      clearTimeout(liveMoveTimerRef.current);
      liveMoveTimerRef.current = null;
    }
  }, [armBulkBusy]);

  const clampTargetForRow = React.useCallback(
    (row, rawText) => {
      const lim = jointLimit(row.joint);
      const raw = parseNum(rawText, 0);
      const clamped = clampByLimit(raw, lim);
      return { raw, clamped, clipped: Math.abs(raw - clamped) > 1e-9, lim };
    },
    [],
  );

  const scheduleLiveMove = React.useCallback(
    (row, targetText) => {
      if (!liveMove || !connected || armBulkBusy) return;
      const hit = row?.hit;
      if (!hit) return;
      pendingLiveMoveRef.current = { row, targetText };
      if (liveMoveTimerRef.current) return;
      liveMoveTimerRef.current = setTimeout(() => {
        liveMoveTimerRef.current = null;
        const pending = pendingLiveMoveRef.current;
        if (!pending) return;
        const checked = clampTargetForRow(pending.row, pending.targetText);
        if (checked.clipped) {
          const msg = t('arm_limit_blocked', {
            joint: pending.row.joint,
            req: checked.raw.toFixed(3),
            min: checked.lim.min.toFixed(3),
            max: checked.lim.max.toFixed(3),
            use: checked.clamped.toFixed(3),
          });
          setLimitWarn(msg);
          showLimitToast(msg);
          patchControl(pending.row.key, { target: String(checked.clamped) });
        } else {
          setLimitWarn('');
        }
        controlMotor(pending.row.hit, 'move', { target: String(checked.clamped) });
      }, 80);
    },
    [liveMove, connected, armBulkBusy, controlMotor, clampTargetForRow, t, patchControl, showLimitToast],
  );

  const onSliderTargetChange = React.useCallback(
    (targetText) => {
      if (!activeRow) return;
      patchControl(activeRow.key, { target: targetText });
      scheduleLiveMove(activeRow, targetText);
    },
    [activeRow, patchControl, scheduleLiveMove],
  );

  React.useEffect(() => {
    setParamRows((prev) =>
      robotArmJointRows.map((row) => {
        const old = prev.find((x) => x.key === row.key);
        return (
          old || {
            key: row.key,
            joint: row.joint,
            hit: row.hit,
            loaded: false,
            error: '',
            values: {
              ctrlMode: '2',
              currentBw: '1000',
              velKp: '0',
              velKi: '0',
              posKp: '0',
              posKi: '0',
            },
          }
        );
      }),
    );
  }, [robotArmJointRows]);

  const patchParam = React.useCallback((key, field, value) => {
    setParamRows((prev) =>
      prev.map((x) => (x.key === key ? { ...x, values: { ...x.values, [field]: value } } : x)),
    );
  }, []);

  const closeEnough = React.useCallback((a, b, eps = 1e-6) => {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return false;
    return Math.abs(a - b) <= eps;
  }, []);

  const applyReadResultToRows = React.useCallback((result) => {
    setParamRows((prev) =>
      prev.map((x) => {
        const r = result?.[x.key];
        if (!r) return x;
        if (!r.ok) return { ...x, loaded: false, error: r.error || 'read failed' };
        const v = r.values || {};
        return {
          ...x,
          loaded: true,
          error: '',
          values: {
            ctrlMode: String(v.ctrlMode ?? x.values.ctrlMode),
            currentBw: String(v.currentBw ?? x.values.currentBw),
            velKp: String(v.velKp ?? x.values.velKp),
            velKi: String(v.velKi ?? x.values.velKi),
            posKp: String(v.posKp ?? x.values.posKp),
            posKi: String(v.posKi ?? x.values.posKi),
          },
        };
      }),
    );
  }, []);

  const readParams = React.useCallback(async () => {
    setParamPanelOpen(true);
    setParamBusy(true);
    setParamInfo('');
    try {
      const result = await readRobotArmControlParams({ onProgress: setParamProgress });
      const matched = paramRows.filter((x) => Boolean(result?.[x.key])).length;
      applyReadResultToRows(result);
      setParamInfo(matched > 0 ? t('arm_params_read_done') : t('arm_params_damiao_only'));
    } catch (e) {
      setParamInfo(`${t('arm_params_read_failed')}: ${e.message || e}`);
    } finally {
      setParamBusy(false);
    }
  }, [applyReadResultToRows, paramRows, readRobotArmControlParams, t]);

  const writeParams = React.useCallback(async () => {
    setParamPanelOpen(true);
    setParamBusy(true);
    setParamInfo('');
    try {
      const rows = paramRows.map((x) => ({
        key: x.key,
        joint: x.joint,
        hit: x.hit,
        values: {
          ctrlMode: Math.max(1, Math.min(4, Math.round(parseNum(x.values.ctrlMode, 2)))),
          currentBw: parseNum(x.values.currentBw, 1000),
          velKp: parseNum(x.values.velKp, 0),
          velKi: parseNum(x.values.velKi, 0),
          posKp: parseNum(x.values.posKp, 0),
          posKi: parseNum(x.values.posKi, 0),
        },
      }));
      const writeResult = await writeRobotArmControlParams(rows, { onProgress: setParamProgress });
      const readBack = await readRobotArmControlParams({ onProgress: setParamProgress });
      applyReadResultToRows(readBack);

      const targetByKey = new Map(rows.map((x) => [x.key, x.values]));
      let mismatch = 0;
      let checked = 0;
      Object.entries(readBack || {}).forEach(([key, item]) => {
        const target = targetByKey.get(key);
        if (!target || !item?.ok) return;
        const actual = item.values || {};
        checked += 1;
        const same =
          Math.round(Number(actual.ctrlMode)) === Math.round(Number(target.ctrlMode)) &&
          closeEnough(Number(actual.currentBw), Number(target.currentBw), 1e-3) &&
          closeEnough(Number(actual.velKp), Number(target.velKp), 1e-6) &&
          closeEnough(Number(actual.velKi), Number(target.velKi), 1e-6) &&
          closeEnough(Number(actual.posKp), Number(target.posKp), 1e-6) &&
          closeEnough(Number(actual.posKi), Number(target.posKi), 1e-6);
        if (!same) mismatch += 1;
      });

      const writeFailed = Object.values(writeResult || {}).filter((x) => x?.ok === false).length;
      if (writeFailed > 0) {
        setParamInfo(`${t('arm_params_write_failed')}: ${writeFailed}`);
      } else if (checked > 0 && mismatch === 0) {
        setParamInfo(t('arm_params_verify_ok'));
      } else if (checked > 0) {
        setParamInfo(`${t('arm_params_verify_mismatch')}: ${mismatch}`);
      } else {
        setParamInfo(t('arm_params_write_done'));
      }
    } catch (e) {
      setParamInfo(`${t('arm_params_write_failed')}: ${e.message || e}`);
    } finally {
      setParamBusy(false);
    }
  }, [applyReadResultToRows, closeEnough, paramRows, readRobotArmControlParams, t, writeRobotArmControlParams]);

  const applyDefaultTemplate = React.useCallback(() => {
    setParamPanelOpen(true);
    setParamRows((prev) =>
      prev.map((row) => {
        const tpl = REBOT_ARM_DAMIAO_DEFAULT_TEMPLATE[row.joint];
        if (!tpl) return row;
        return {
          ...row,
          values: {
            ...row.values,
            ...tpl,
          },
        };
      }),
    );
    setParamInfo(t('arm_params_template_applied'));
  }, [t]);

  const onZeroAllSafe = React.useCallback(async () => {
    if (!zeroSafety.ok) {
      const short = zeroSafety.notReady
        .slice(0, 4)
        .map((x) => `J${x.joint}${x.pos == null ? '(?)' : `(${x.pos.toFixed(2)})`}`)
        .join(', ');
      const more = zeroSafety.notReady.length > 4 ? ` +${zeroSafety.notReady.length - 4}` : '';
      const msg = t('arm_zero_all_blocked', { joints: `${short}${more}`, eps: ZERO_SAFE_EPS_RAD.toFixed(2) });
      setLimitWarn(msg);
      showLimitToast(msg);
      const forceConfirm = await askZeroConfirm({
        title: t('arm_zero_all_force_title'),
        message: `${msg}\n\n${t('arm_zero_all_force_hint')}`,
        danger: true,
      });
      if (!forceConfirm) return;
    }

    const c1 = await askZeroConfirm({
      title: t('arm_zero_all_confirm_title'),
      message: t('arm_zero_all_confirm_1'),
      danger: true,
    });
    if (!c1) return;
    const c2 = await askZeroConfirm({
      title: t('arm_zero_all_confirm_title'),
      message: t('arm_zero_all_confirm_2'),
      danger: true,
    });
    if (!c2) return;
    await zeroAllRobotArm();
  }, [askZeroConfirm, showLimitToast, t, zeroAllRobotArm, zeroSafety]);

  const armToolbarBusy = armBulkBusy || armScanBusy || armSelfCheckBusy || paramBusy;
  const perJointBusy = armBulkBusy || paramBusy;

  return (
    <section className="card glass">
      {limitToast.visible && (
        <div key={limitToast.seq} className="armLimitToast" role="status" aria-live="polite">
          {limitToast.message}
        </div>
      )}
      {zeroConfirm.open && (
        <div className="armDialogMask" role="dialog" aria-modal="true" aria-live="assertive">
          <div className="armDialogCard">
            <h3>{zeroConfirm.title || t('arm_zero_all_confirm_title')}</h3>
            <p>{zeroConfirm.message}</p>
            <div className="row toolbar compactToolbar">
              <button className="ghostBtn" onClick={() => closeZeroConfirm(false)}>
                {t('cancel')}
              </button>
              <button
                className={zeroConfirm.danger ? 'dangerBtn' : 'primary'}
                onClick={() => closeZeroConfirm(true)}
              >
                {t('confirm')}
              </button>
            </div>
          </div>
        </div>
      )}
      {firstUseOpen && (
        <div className="armDialogMask" role="dialog" aria-modal="true">
          <div className="armDialogCard">
            <h3>{t('arm_first_use_title')}</h3>
            <p>{t('arm_first_use_intro')}</p>
            <ol className="armGuideList">
              <li>{t('arm_first_use_step_1')}</li>
              <li>{t('arm_first_use_step_2')}</li>
              <li>{t('arm_first_use_step_3')}</li>
              <li>{t('arm_first_use_step_4')}</li>
              <li>{t('arm_first_use_step_5')}</li>
            </ol>
            <div className="row toolbar compactToolbar">
              <button className="ghostBtn" onClick={() => setFirstUseOpen(false)}>
                {t('close')}
              </button>
            </div>
          </div>
        </div>
      )}
      <div className="sectionTitle">
        <h2>{t('robot_arm_title')}</h2>
        <span className="tip">{t('robot_arm_desc')}</span>
      </div>

      {!connected && <div className="offlineBanner">{t('ws_disconnected_motor')}</div>}

      <div className="row toolbar compactToolbar armTopToolbar">
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
        <button className="firstUseBtn" onClick={() => setFirstUseOpen(true)}>
          {t('arm_first_use_btn')}
        </button>
        <button onClick={ensureRobotArmCards}>{t('arm_prepare_cards')}</button>
        <button className="primary" disabled={!canAction || armToolbarBusy} onClick={scanRobotArmAll}>
          {t('arm_scan_all')}
        </button>
        <button
          className="ghostBtn"
          disabled={!canAction || armToolbarBusy}
          onClick={runRobotArmSelfCheck}
        >
          {t('arm_self_check')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={enableAllRobotArm}>
          {t('arm_enable_all')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={disableAllRobotArm}>
          {t('arm_disable_all')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={onZeroAllSafe} title={t('arm_zero_all_guard_hint')}>
          {t('arm_zero_all')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={resetPoseRobotArm}>
          {t('arm_reset_pose')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={readParams}>
          {t('arm_read_params')}
        </button>
        <button disabled={!canAction || armToolbarBusy || !paramPanelOpen} onClick={writeParams}>
          {t('arm_write_params')}
        </button>
        <button disabled={!canAction || armToolbarBusy} onClick={applyDefaultTemplate}>
          {t('arm_apply_default_template')}
        </button>
      </div>

      {paramPanelOpen && (
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
            <button className="ghostBtn" disabled={armToolbarBusy} onClick={() => setParamPanelOpen(false)}>
              {t('close')}
            </button>
          </div>
          {paramInfo && <div className="tip">{paramInfo}</div>}
          {(paramBusy || paramProgress?.active) && (
            <div className="scanProgressWrap">
              <div className="scanProgressText">
                <span>{paramProgress?.label || t('scanning')}</span>
                <span>
                  {paramProgress?.done || 0}/{Math.max(1, paramProgress?.total || 1)} ({paramProgress?.percent || 0}%)
                </span>
              </div>
              <div className="scanProgressTrack">
                <div className="scanProgressFill" style={{ width: `${paramProgress?.percent || 0}%` }} />
              </div>
            </div>
          )}
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
      )}

      {(armScanBusy || armScanProgress?.active) && (
        <div className="scanProgressWrap">
          <div className="scanProgressText">
            <span>{armScanProgress?.label || t('scanning')}</span>
            <span>
              {armScanProgress?.done || 0}/{Math.max(1, armScanProgress?.total || 7)} ({armScanProgress?.percent || 0}%)
            </span>
          </div>
          <div className="scanProgressTrack">
            <div className="scanProgressFill" style={{ width: `${armScanProgress?.percent || 0}%` }} />
          </div>
        </div>
      )}

      {armBulkBusy && <div className="tip">{t('arm_bulk_busy')}</div>}
      {(armSelfCheckBusy || armSelfCheckProgress?.active) && (
        <div className="scanProgressWrap">
          <div className="scanProgressText">
            <span>{armSelfCheckProgress?.label || t('arm_self_check_running')}</span>
            <span>
              {armSelfCheckProgress?.done || 0}/{Math.max(1, armSelfCheckProgress?.total || 4)} ({armSelfCheckProgress?.percent || 0}%)
            </span>
          </div>
          <div className="scanProgressTrack">
            <div className="scanProgressFill" style={{ width: `${armSelfCheckProgress?.percent || 0}%` }} />
          </div>
        </div>
      )}
      {armSelfCheckReport && (
        <div className={`armSelfCheckCard ${armSelfCheckReport.ok ? 'ok' : 'err'}`}>
          <div className="sectionTitle">
            <h2>{t('arm_self_check_result')}</h2>
            <span className="chip">{armSelfCheckReport.ok ? t('arm_self_check_pass') : t('arm_self_check_fail')}</span>
          </div>
          <div className="armMeta">
            <span>{t('arm_self_check_online')}: {armSelfCheckReport.onlineCount}/{armSelfCheckReport.total}</span>
            <span>{t('arm_self_check_param')}: ok={armSelfCheckReport.paramOkCount}, fail={armSelfCheckReport.paramFailCount}</span>
          </div>
          <div className="tip">{t('arm_self_check_reason')}: {armSelfCheckReport.reason}</div>
        </div>
      )}
      {!zeroSafety.ok && (
        <div className="tip warnText">
          {t('arm_zero_all_guard_hint')} · {t('arm_zero_all_blocked', {
            joints: zeroSafety.notReady.map((x) => `J${x.joint}`).join(', '),
            eps: ZERO_SAFE_EPS_RAD.toFixed(2),
          })}
        </div>
      )}

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
                <button disabled={!connected || perJointBusy} onClick={() => controlMotor(activeRow.hit, 'enable')}>
                  {t('enable')}
                </button>
                <button disabled={!connected || perJointBusy} onClick={() => controlMotor(activeRow.hit, 'disable')}>
                  {t('disable')}
                </button>
                <button
                  className="primary"
                  disabled={!connected || perJointBusy}
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
                <button disabled={!connected || perJointBusy} onClick={() => controlMotor(activeRow.hit, 'stop')}>
                  {t('stop')}
                </button>
                <button disabled={!connected || perJointBusy} onClick={() => refreshMotorState(activeRow.hit)}>
                  {t('refresh_state')}
                </button>
              </div>
            </div>
          )}

          <div className="armSimPanel">
            <div className="sectionTitle armPaneTitle">
              <h2>{t('arm_sim_title')}</h2>
              <div className="row compactToolbar">
                <span className="tip">{t('arm_ws_bridge_hint')}</span>
                <button className="ghostBtn small" onClick={() => setUrdfResetSeq((v) => v + 1)}>
                  {t('arm_reset_view')}
                </button>
              </div>
            </div>
            <p className="tip">{t('arm_sim_desc')}</p>
            <ArmUrdfViewer jointTargets={jointTargets} resetViewSeq={urdfResetSeq} />
          </div>
        </div>
      </div>
    </section>
  );
}
