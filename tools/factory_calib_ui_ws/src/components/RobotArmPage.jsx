import React from 'react';
import { useI18n } from '../i18n';
import {
  REBOT_ARM_DAMIAO_DEFAULT_TEMPLATE,
  REBOT_ARM_JOINT_LIMITS,
  ROBOT_ARM_MODELS,
  ZERO_SAFE_EPS_RAD,
} from '../lib/robotArm';
import { sleep } from '../lib/async';
import { parseNum } from '../lib/utils';
import { ArmUrdfViewer } from './ArmUrdfViewer';
import { ProgressBar } from './ProgressBar';
import { useMotorStudioContext } from '../hooks/useMotorStudioContext';
import { usePersistedState } from '../hooks/usePersistedState';
import { JointList } from './robot-arm/JointList';
import { JointControlPanel } from './robot-arm/JointControlPanel';
import { ParamTable } from './robot-arm/ParamTable';
import { SelfCheckReport } from './robot-arm/SelfCheckReport';
import { ZeroConfirmDialog } from './robot-arm/ZeroConfirmDialog';

function jointLimit(joint) {
  return REBOT_ARM_JOINT_LIMITS[Number(joint)] || { min: -3.14, max: 3.14 };
}

function clampByLimit(value, lim) {
  return Math.max(lim.min, Math.min(lim.max, value));
}

const SAFE_DEMO_TARGETS = {
  1: 0.4,
  2: -0.3,
  3: -0.4,
  4: -0.2,
  5: -0.5,
  6: 0.5,
  7: -2.0,
};

export function RobotArmPage() {
  const { t } = useI18n();
  const {
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
  } = useMotorStudioContext();
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
  const [demoToast, setDemoToast] = React.useState({
    visible: false,
    seq: 0,
    tone: 'info',
    title: '',
    detail: '',
  });
  const [urdfResetSeq, setUrdfResetSeq] = React.useState(0);
  const [urdfClearTrailSeq, setUrdfClearTrailSeq] = React.useState(0);
  const [urdfExportTrailSeq, setUrdfExportTrailSeq] = React.useState(0);
  const [urdfReplaySeq, setUrdfReplaySeq] = React.useState(0);
  const [urdfReplayStopSeq, setUrdfReplayStopSeq] = React.useState(0);
  const [urdfReplayFinishSeq, setUrdfReplayFinishSeq] = React.useState(0);
  const [urdfReplayBusy, setUrdfReplayBusy] = React.useState(false);
  const [urdfReplaySpeed, setUrdfReplaySpeed] = React.useState(1);
  const urdfSimMode = 'trajectory';
  const [urdfTrailStyle, setUrdfTrailStyle] = React.useState('multi');
  const [urdfTrailColor, setUrdfTrailColor] = React.useState('#ff2d55');
  const [urdfTrailVisible, setUrdfTrailVisible] = React.useState(true);
  const [urdfImportedTrail, setUrdfImportedTrail] = React.useState(null);
  const [urdfImportInfo, setUrdfImportInfo] = React.useState('');
  const [urdfSeqLibrary, setUrdfSeqLibrary] = usePersistedState(
    'factory_calib_ui_ws_arm_seq_library_v1',
    [],
    (cached) => (Array.isArray(cached) ? cached.filter((x) => x && Array.isArray(x.points) && x.points.length >= 2) : []),
  );
  const [urdfSeqPick, setUrdfSeqPick] = React.useState('');
  const importTrailInputRef = React.useRef(null);
  const [firstUseOpen, setFirstUseOpen] = React.useState(false);
  const [demoAction, setDemoAction] = React.useState('safe_seq');
  const [demoBusy, setDemoBusy] = React.useState(false);
  const [zeroConfirm, setZeroConfirm] = React.useState({
    open: false,
    title: '',
    message: '',
    danger: false,
  });
  const zeroConfirmResolverRef = React.useRef(null);
  const rowsRef = React.useRef(robotArmJointRows);
  const initControlSyncDoneRef = React.useRef(false);

  React.useEffect(() => {
    rowsRef.current = robotArmJointRows;
  }, [robotArmJointRows]);

  React.useEffect(() => {
    if (initControlSyncDoneRef.current) return;
    if (!robotArmJointRows.length) return;
    robotArmJointRows.forEach((row) => {
      const lim = jointLimit(row.joint);
      const rawPos = Number(row?.hit?.pos);
      const synced = row?.hit?.online && Number.isFinite(rawPos) ? clampByLimit(rawPos, lim) : 0;
      patchControl(row.key, {
        mode: 'pos_vel',
        vlim: '1.0',
        tau: '0.0',
        kp: '30.0',
        kd: '1.0',
        target: String(synced),
      });
    });
    initControlSyncDoneRef.current = true;
  }, [robotArmJointRows, patchControl]);

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

  React.useEffect(() => {
    if (!demoToast.visible || demoBusy) return undefined;
    const timer = setTimeout(() => {
      setDemoToast((prev) => ({ ...prev, visible: false }));
    }, 2600);
    return () => clearTimeout(timer);
  }, [demoToast, demoBusy]);

  const showLimitToast = React.useCallback((message) => {
    setLimitToast((prev) => ({ visible: true, message, seq: prev.seq + 1 }));
  }, []);

  const showDemoToast = React.useCallback((tone, title, detail = '') => {
    setDemoToast((prev) => ({
      visible: true,
      seq: prev.seq + 1,
      tone,
      title,
      detail,
    }));
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

  const onlineCount = React.useMemo(
    () => robotArmJointRows.filter((row) => Boolean(row?.hit?.online)).length,
    [robotArmJointRows],
  );

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

  const runDemo = React.useCallback(async () => {
    if (demoBusy) return;

    const getOnlineRows = () => {
      const rows = rowsRef.current || [];
      const onlineRows = rows.filter((row) => Boolean(row?.hit?.online));
      return onlineRows.length > 0 ? onlineRows : [];
    };

    setDemoBusy(true);
    try {
      if (demoAction === 'safe_seq_scan') {
        showDemoToast('info', t('arm_demo_running', { name: t('arm_demo_safe_seq_scan') }), t('arm_demo_scan'));
        await scanRobotArmAll();
        await sleep(120);
        await enableAllRobotArm();
      }

      const onlineRows = getOnlineRows();
      if (onlineRows.length === 0) {
        showDemoToast('warn', t('arm_demo_failed'), t('arm_demo_no_online'));
        return;
      }
      const targets = [...onlineRows].sort((a, b) => Number(a.joint) - Number(b.joint)).slice(0, 7);
      const seq = [];
      targets.forEach((row) => {
        seq.push({
          row,
          target: Number(SAFE_DEMO_TARGETS[row.joint] ?? 0.25),
          phase: 'forward',
        });
      });
      [...targets].reverse().forEach((row) => {
        seq.push({ row, target: 0, phase: 'reset' });
      });

      const namedSeq = seq.map((step, idx) => ({
        ...step,
        note: t(`arm_demo_phase_${step.phase}`),
        index: idx + 1,
      }));
      let okCount = 0;
      const demoName = demoAction === 'safe_seq_scan' ? t('arm_demo_safe_seq_scan') : t('arm_demo_safe_seq');
      for (const step of namedSeq) {
        const lim = jointLimit(step.row.joint);
        const target = clampByLimit(Number(step.target), lim);
        patchControl(step.row.key, { mode: 'pos_vel', target: String(target) });
        showDemoToast(
          'info',
          t('arm_demo_running', { name: demoName }),
          t('arm_demo_step_phase', {
            step: step.index,
            total: namedSeq.length,
            joint: `J${step.row.joint}`,
            phase: step.note,
            target: target.toFixed(3),
          }),
        );
        const ok = await controlMotor(step.row.hit, 'move', {
          mode: 'pos_vel',
          target: String(target),
        });
        if (ok) okCount += 1;
        await sleep(300);
      }

      showDemoToast(
        okCount === namedSeq.length ? 'ok' : 'warn',
        okCount === namedSeq.length ? t('arm_demo_done') : t('arm_demo_failed'),
        t('arm_demo_result', { ok: okCount, total: namedSeq.length }),
      );
    } catch (e) {
      showDemoToast('warn', t('arm_demo_failed'), e?.message || String(e));
    } finally {
      setDemoBusy(false);
    }
  }, [
    demoBusy,
    demoAction,
    patchControl,
    showDemoToast,
    t,
    controlMotor,
    scanRobotArmAll,
    enableAllRobotArm,
  ]);

  const armToolbarBusy = armBulkBusy || armScanBusy || armSelfCheckBusy || paramBusy || demoBusy || urdfReplayBusy;
  const perJointBusy = armBulkBusy || paramBusy || urdfReplayBusy;

  const openImportTrailDialog = React.useCallback(() => {
    importTrailInputRef.current?.click();
  }, []);

  const onImportTrailFile = React.useCallback(
    async (e) => {
      const file = e.target?.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const json = JSON.parse(text);
        const rawPoints = Array.isArray(json?.points)
          ? json.points
          : Array.isArray(json?.sequence)
            ? json.sequence
            : Array.isArray(json?.waypoints)
              ? json.waypoints
              : [];
        const points = rawPoints
          .map((p) => {
            if (Array.isArray(p) && p.length >= 3) {
              return { x: Number(p[0]), y: Number(p[1]), z: Number(p[2]) };
            }
            if (p && typeof p === 'object') {
              const joints = p.joints && typeof p.joints === 'object' ? p.joints : undefined;
              if (p.pos && typeof p.pos === 'object') {
                return { x: Number(p.pos.x), y: Number(p.pos.y), z: Number(p.pos.z), ...(joints ? { joints } : {}) };
              }
              return { x: Number(p.x), y: Number(p.y), z: Number(p.z), ...(joints ? { joints } : {}) };
            }
            return null;
          })
          .filter((p) => p && Number.isFinite(p.x) && Number.isFinite(p.y) && Number.isFinite(p.z));
        if (points.length < 2) throw new Error('invalid trajectory points');
        setUrdfImportedTrail({
          name: file.name,
          points,
          at: Date.now(),
        });
        setUrdfImportInfo(t('arm_traj_import_ok', { count: points.length }));
        setUrdfTrailVisible(true);
      } catch (err) {
        setUrdfImportInfo(`${t('arm_traj_import_fail')}: ${err?.message || String(err)}`);
      } finally {
        if (e.target) e.target.value = '';
      }
    },
    [t],
  );

  const replayImportedTrail = React.useCallback(() => {
    if (!urdfImportedTrail?.points?.length) {
      setUrdfImportInfo(t('arm_traj_replay_need_import'));
      return;
    }
    setUrdfReplaySeq((v) => v + 1);
  }, [urdfImportedTrail, t]);

  React.useEffect(() => {
    if (!urdfSeqLibrary.length) {
      if (urdfSeqPick) setUrdfSeqPick('');
      return;
    }
    const exists = urdfSeqLibrary.some((x) => x.id === urdfSeqPick);
    if (!exists) setUrdfSeqPick(urdfSeqLibrary[0].id);
  }, [urdfSeqLibrary, urdfSeqPick]);

  const saveCurrentSequenceToLibrary = React.useCallback(() => {
    if (!urdfImportedTrail?.points?.length) {
      setUrdfImportInfo(t('arm_seq_save_need_import'));
      return;
    }
    const now = Date.now();
    const base = String(urdfImportedTrail.name || '').replace(/\.json$/i, '').trim();
    const name = base || `seq_${new Date(now).toLocaleTimeString()}`;
    const item = {
      id: `seq_${now}_${Math.random().toString(36).slice(2, 8)}`,
      name,
      at: now,
      points: urdfImportedTrail.points,
    };
    setUrdfSeqLibrary((prev) => [item, ...prev].slice(0, 64));
    setUrdfSeqPick(item.id);
    setUrdfImportInfo(t('arm_seq_saved', { name: item.name }));
  }, [setUrdfSeqLibrary, t, urdfImportedTrail]);

  const loadSelectedSequence = React.useCallback(
    (opts = { replay: false }) => {
      const item = urdfSeqLibrary.find((x) => x.id === urdfSeqPick);
      if (!item) {
        setUrdfImportInfo(t('arm_seq_load_need_select'));
        return;
      }
      setUrdfImportedTrail({
        name: `${item.name}.json`,
        points: item.points,
        at: Date.now(),
      });
      setUrdfTrailVisible(true);
      setUrdfImportInfo(t('arm_seq_loaded', { name: item.name, count: item.points.length }));
      if (opts?.replay) setUrdfReplaySeq((v) => v + 1);
    },
    [t, urdfSeqLibrary, urdfSeqPick],
  );

  const deleteSelectedSequence = React.useCallback(() => {
    const item = urdfSeqLibrary.find((x) => x.id === urdfSeqPick);
    if (!item) return;
    setUrdfSeqLibrary((prev) => prev.filter((x) => x.id !== item.id));
    setUrdfImportInfo(t('arm_seq_deleted', { name: item.name }));
  }, [setUrdfSeqLibrary, t, urdfSeqLibrary, urdfSeqPick]);

  return (
    <section className="card glass">
      {limitToast.visible && (
        <div key={limitToast.seq} className="armLimitToast" role="status" aria-live="polite">
          {limitToast.message}
        </div>
      )}
      {demoToast.visible && (
        <div
          key={demoToast.seq}
          className={`armDemoToast ${demoToast.tone}`}
          role="status"
          aria-live="polite"
        >
          <div className="armDemoToastTitle">{demoToast.title}</div>
          {demoToast.detail && <div className="armDemoToastDetail">{demoToast.detail}</div>}
        </div>
      )}
      <ZeroConfirmDialog
        open={zeroConfirm.open}
        title={zeroConfirm.title}
        message={zeroConfirm.message}
        danger={zeroConfirm.danger}
        onCancel={() => closeZeroConfirm(false)}
        onConfirm={() => closeZeroConfirm(true)}
      />
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
        <button disabled={!canAction || armToolbarBusy} onClick={runDemo}>
          {t('arm_demo_btn')}
        </button>
        <div className="field miniField">
          <label>{t('arm_demo_list')}</label>
          <select
            value={demoAction}
            disabled={!canAction || armToolbarBusy}
            onChange={(e) => setDemoAction(e.target.value)}
          >
            <option value="safe_seq">{t('arm_demo_safe_seq')}</option>
            <option value="safe_seq_scan">{t('arm_demo_safe_seq_scan')}</option>
          </select>
        </div>
      </div>
      <div className="tip warnText">{t('arm_demo_beta_warn')}</div>

      <ParamTable
        open={paramPanelOpen}
        canAction={canAction}
        armToolbarBusy={armToolbarBusy}
        paramBusy={paramBusy}
        paramInfo={paramInfo}
        paramProgress={paramProgress}
        paramRows={paramRows}
        patchParam={patchParam}
        readParams={readParams}
        writeParams={writeParams}
        applyDefaultTemplate={applyDefaultTemplate}
        onClose={() => setParamPanelOpen(false)}
      />

      <ProgressBar active={armScanBusy || armScanProgress?.active} progress={armScanProgress} />

      {armBulkBusy && <div className="tip">{t('arm_bulk_busy')}</div>}
      <ProgressBar
        active={armSelfCheckBusy || armSelfCheckProgress?.active}
        progress={armSelfCheckProgress}
        fallbackLabel={t('arm_self_check_running')}
      />
      <SelfCheckReport report={armSelfCheckReport} />
      {onlineCount > 0 && onlineCount < robotArmJointRows.length && (
        <div className="tip">
          {t('arm_demo_online_hint', { online: onlineCount, total: robotArmJointRows.length })}
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
        <JointList
          robotArmJointRows={robotArmJointRows}
          activeRowKey={activeRow?.key}
          onSelect={setActiveJointKey}
          connected={connected}
          scanRobotArmJoint={scanRobotArmJoint}
          refreshMotorState={refreshMotorState}
        />

        <div className="armRightPane">
          <JointControlPanel
            activeRow={activeRow}
            perJointBusy={perJointBusy}
            liveMove={liveMove}
            sliderValue={sliderValue}
            limitWarn={limitWarn}
            patchControl={patchControl}
            onSliderTargetChange={onSliderTargetChange}
            jointLimit={jointLimit}
            setUiPref={setUiPref}
            controlMotor={controlMotor}
            refreshMotorState={refreshMotorState}
            clampTargetForRow={clampTargetForRow}
            setLimitWarn={setLimitWarn}
            showLimitToast={showLimitToast}
          />

          <div className="armSimPanel">
            <div className="sectionTitle armPaneTitle">
              <h2>{t('arm_sim_title')}</h2>
            </div>
            <div className="armSimControls">
              <div className="armSimStatusRow">
                <span className="armModeChip">
                  {t('arm_sim_mode_current', {
                    mode: t('arm_sim_mode_trajectory'),
                  })}
                </span>
                <div className="row compactToolbar">
                  <button className="ghostBtn small" disabled={urdfReplayBusy} onClick={() => setUrdfClearTrailSeq((v) => v + 1)}>
                    {t('arm_clear_traj')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy} onClick={() => setUrdfExportTrailSeq((v) => v + 1)}>
                    {t('arm_export_traj')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy} onClick={openImportTrailDialog}>
                    {t('arm_import_traj')}
                  </button>
                  <button
                    className="ghostBtn small"
                    disabled={!urdfImportedTrail?.points?.length || urdfReplayBusy}
                    onClick={replayImportedTrail}
                  >
                    {t('arm_replay_traj')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy} onClick={saveCurrentSequenceToLibrary}>
                    {t('arm_seq_save_current')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy || !urdfSeqLibrary.length} onClick={() => loadSelectedSequence({ replay: false })}>
                    {t('arm_seq_load')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy || !urdfSeqLibrary.length} onClick={() => loadSelectedSequence({ replay: true })}>
                    {t('arm_seq_replay_selected')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy || !urdfSeqLibrary.length} onClick={deleteSelectedSequence}>
                    {t('arm_seq_delete')}
                  </button>
                  <button
                    className="ghostBtn small"
                    disabled={!urdfReplayBusy}
                    onClick={() => setUrdfReplayStopSeq((v) => v + 1)}
                  >
                    {t('arm_replay_stop')}
                  </button>
                  <button
                    className="ghostBtn small"
                    disabled={!urdfReplayBusy}
                    onClick={() => setUrdfReplayFinishSeq((v) => v + 1)}
                  >
                    {t('arm_replay_finish')}
                  </button>
                  <button className="ghostBtn small" disabled={urdfReplayBusy} onClick={() => setUrdfResetSeq((v) => v + 1)}>
                    {t('arm_reset_view')}
                  </button>
                </div>
              </div>
              <div className="armSimFieldRow">
                <label className="armSimField">
                  <span>{t('arm_traj_visible')}</span>
                  <select
                    disabled={urdfReplayBusy}
                    value={urdfTrailVisible ? 'show' : 'hide'}
                    onChange={(e) => setUrdfTrailVisible(e.target.value === 'show')}
                  >
                    <option value="show">{t('arm_traj_show')}</option>
                    <option value="hide">{t('arm_traj_hide')}</option>
                  </select>
                </label>
                <label className="armSimField">
                  <span>{t('arm_seq_library')}</span>
                  <select
                    disabled={urdfReplayBusy || !urdfSeqLibrary.length}
                    value={urdfSeqPick}
                    onChange={(e) => setUrdfSeqPick(e.target.value)}
                  >
                    {!urdfSeqLibrary.length && <option value="">{t('arm_seq_none')}</option>}
                    {urdfSeqLibrary.map((x) => (
                      <option key={x.id} value={x.id}>
                        {x.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="armSimField">
                  <span>{t('arm_traj_style')}</span>
                  <select disabled={urdfReplayBusy} value={urdfTrailStyle} onChange={(e) => setUrdfTrailStyle(e.target.value)}>
                    <option value="multi">{t('arm_traj_style_multi')}</option>
                    <option value="mono">{t('arm_traj_style_mono')}</option>
                  </select>
                </label>
                <label className="armSimField">
                  <span>{t('arm_traj_color')}</span>
                  <div className="armColorField">
                    <input
                      type="color"
                      disabled={urdfReplayBusy}
                      value={urdfTrailColor}
                      title={t('arm_traj_color')}
                      onChange={(e) => setUrdfTrailColor(e.target.value)}
                    />
                    <code>{String(urdfTrailColor || '').toUpperCase()}</code>
                  </div>
                </label>
                <label className="armSimField">
                  <span>{t('arm_replay_speed')}</span>
                  <div className="armColorField">
                    <input
                      type="range"
                      min="0.2"
                      max="3"
                      step="0.1"
                      value={urdfReplaySpeed}
                      onChange={(e) => setUrdfReplaySpeed(Number(e.target.value) || 1)}
                    />
                    <code>{urdfReplaySpeed.toFixed(1)}x</code>
                  </div>
                </label>
              </div>
            </div>
            <p className="tip armSimDesc">{t('arm_sim_desc')}</p>
            {urdfImportInfo && <p className="tip">{urdfImportInfo}</p>}
            <input
              ref={importTrailInputRef}
              type="file"
              accept="application/json,.json"
              style={{ display: 'none' }}
              onChange={onImportTrailFile}
            />
            <ArmUrdfViewer
              jointTargets={jointTargets}
              resetViewSeq={urdfResetSeq}
              clearTrailSeq={urdfClearTrailSeq}
              exportTrailSeq={urdfExportTrailSeq}
              replaySeq={urdfReplaySeq}
              replayStopSeq={urdfReplayStopSeq}
              replayFinishSeq={urdfReplayFinishSeq}
              replaySpeed={urdfReplaySpeed}
              importedTrail={urdfImportedTrail}
              simMode={urdfSimMode}
              trailStyle={urdfTrailStyle}
              trailColor={urdfTrailColor}
              trailVisible={urdfTrailVisible}
              onReplayStateChange={setUrdfReplayBusy}
            />
          </div>
        </div>
      </div>
    </section>
  );
}
