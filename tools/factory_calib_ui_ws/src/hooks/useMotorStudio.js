import { useEffect, useMemo, useRef, useState } from 'react';
import { DEFAULT_VENDOR_CONFIG } from '../lib/constants';
import { defaultControlsForHit, motorKey, toHex, ts } from '../lib/utils';
import { runScanOp } from '../lib/motorScanOps';
import {
  controlMotorOp,
  probeMotorOp,
  refreshMotorStateOp,
  setIdForOp,
  verifyHitOp,
  zeroMotorOp,
} from '../lib/motorStudioOps';
import { useGatewayBridge } from './useGatewayBridge';
import { useRobotArmStudio } from './useRobotArmStudio';
import { useI18n } from '../i18n';

const LS_HITS_KEY = 'factory_calib_ui_ws_hits_v1';
const LS_CONTROLS_KEY = 'factory_calib_ui_ws_controls_v1';
const LS_UI_PREFS_KEY = 'factory_calib_ui_ws_ui_prefs_v1';
const LS_ACTIVE_MOTOR_KEY = 'factory_calib_ui_ws_active_motor_v1';

const DEFAULT_UI_PREFS = {
  sectionConnectionCollapsed: false,
  sectionScanCollapsed: false,
  sectionManualCollapsed: false,
  sectionMotorsCollapsed: false,
  sectionStateCollapsed: true,
  sectionLogsCollapsed: true,
  armSliderLiveMove: false,
};

function loadJson(key, fallback) {
  try {
    if (typeof window === 'undefined') return fallback;
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw);
    return parsed ?? fallback;
  } catch {
    return fallback;
  }
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const DAMIAO_CTRL_PARAM_RID = {
  ctrlMode: 10,
  currentBw: 24,
  velKp: 25,
  velKi: 26,
  posKp: 27,
  posKi: 28,
};

export function useMotorStudio() {
  const { t } = useI18n();
  const [wsUrl, setWsUrl] = useState('ws://127.0.0.1:9002');
  const [channel, setChannel] = useState('can0');
  const [scanTimeoutMs, setScanTimeoutMs] = useState('500');
  const [scanBusy, setScanBusy] = useState(false);
  const [scanProgress, setScanProgress] = useState({
    active: false,
    done: 0,
    total: 0,
    label: '',
    percent: 0,
  });
  const [scanFoundFx, setScanFoundFx] = useState({ visible: false, message: '', seq: 0 });
  const [armBulkBusy, setArmBulkBusy] = useState(false);
  const [armSelfCheckBusy, setArmSelfCheckBusy] = useState(false);
  const [armSelfCheckProgress, setArmSelfCheckProgress] = useState({
    active: false,
    done: 0,
    total: 4,
    label: '',
    percent: 0,
  });
  const [armSelfCheckReport, setArmSelfCheckReport] = useState(null);

  const [vendors, setVendors] = useState(DEFAULT_VENDOR_CONFIG);
  const [hits, setHits] = useState(() => {
    const cached = loadJson(LS_HITS_KEY, []);
    return Array.isArray(cached) ? cached : [];
  });
  const [controls, setControls] = useState(() => {
    const cached = loadJson(LS_CONTROLS_KEY, {});
    return cached && typeof cached === 'object' ? cached : {};
  });
  const [selected, setSelected] = useState(new Set());
  const [activeMotorKey, setActiveMotorKey] = useState(() => loadJson(LS_ACTIVE_MOTOR_KEY, ''));
  const [uiPrefs, setUiPrefs] = useState(() => {
    const cached = loadJson(LS_UI_PREFS_KEY, DEFAULT_UI_PREFS);
    return { ...DEFAULT_UI_PREFS, ...(cached || {}) };
  });

  const [menuOpen, setMenuOpen] = useState(false);
  const [newCardKeys, setNewCardKeys] = useState(new Set());
  const [stateSnapshot, setStateSnapshot] = useState('(no state yet)');
  const [logs, setLogs] = useState([]);
  const [manualDraft, setManualDraft] = useState({
    vendor: 'damiao',
    model: '4310',
    escId: '0x05',
    mstId: '0x15',
  });

  const cardRefs = useRef({});

  const pushLog = (msg, level = 'info') => {
    setLogs((prev) => [...prev, { t: ts(), msg, level }].slice(-500));
  };

  const {
    connText,
    connected,
    targetTransport,
    targetSerialPort,
    connectWs,
    disconnectWs,
    sendCmd,
    closeBusQuietly,
    setTargetFor,
  } = useGatewayBridge({ wsUrl, channel, pushLog, setStateSnapshot });

  const canAction = connected && !scanBusy && !armBulkBusy;

  useEffect(() => {
    if (newCardKeys.size === 0) return undefined;
    const timer = setTimeout(() => setNewCardKeys(new Set()), 1400);
    return () => clearTimeout(timer);
  }, [newCardKeys]);

  useEffect(() => {
    if (!scanFoundFx.visible) return undefined;
    const timer = setTimeout(() => setScanFoundFx((prev) => ({ ...prev, visible: false })), 1400);
    return () => clearTimeout(timer);
  }, [scanFoundFx]);

  useEffect(() => {
    try {
      if (typeof window === 'undefined') return;
      window.localStorage.setItem(LS_HITS_KEY, JSON.stringify(hits));
    } catch {
      // ignore localStorage failures
    }
  }, [hits]);

  useEffect(() => {
    try {
      if (typeof window === 'undefined') return;
      window.localStorage.setItem(LS_CONTROLS_KEY, JSON.stringify(controls));
    } catch {
      // ignore localStorage failures
    }
  }, [controls]);

  useEffect(() => {
    try {
      if (typeof window === 'undefined') return;
      window.localStorage.setItem(LS_UI_PREFS_KEY, JSON.stringify(uiPrefs));
    } catch {
      // ignore localStorage failures
    }
  }, [uiPrefs]);

  useEffect(() => {
    try {
      if (typeof window === 'undefined') return;
      window.localStorage.setItem(LS_ACTIVE_MOTOR_KEY, JSON.stringify(activeMotorKey || ''));
    } catch {
      // ignore localStorage failures
    }
  }, [activeMotorKey]);

  useEffect(() => {
    if (!activeMotorKey) return;
    const exists = hits.some((h) => motorKey(h) === activeMotorKey);
    if (!exists) setActiveMotorKey('');
  }, [hits, activeMotorKey]);

  const patchControl = (k, patch) => {
    setControls((prev) => ({ ...prev, [k]: { ...(prev[k] || {}), ...patch } }));
  };

  const clearDevices = () => {
    const ok = typeof window === 'undefined' ? true : window.confirm(t('confirm_clear_all'));
    if (!ok) return;
    setHits([]);
    setControls({});
    setSelected(new Set());
    setActiveMotorKey('');
    try {
      if (typeof window !== 'undefined') {
        window.localStorage.removeItem(LS_HITS_KEY);
        window.localStorage.removeItem(LS_CONTROLS_KEY);
        window.localStorage.removeItem(LS_ACTIVE_MOTOR_KEY);
      }
    } catch {
      // ignore localStorage failures
    }
  };

  const clearOfflineMotors = () => {
    const offlineHits = hits.filter((h) => h.online === false);
    if (offlineHits.length === 0) {
      pushLog(t('log_no_offline_motors'), 'info');
      return;
    }

    const ok =
      typeof window === 'undefined'
        ? true
        : window.confirm(t('confirm_clear_offline', { count: offlineHits.length }));
    if (!ok) return;

    const offlineKeys = new Set(offlineHits.map((h) => motorKey(h)));
    setHits((prev) => prev.filter((h) => h.online !== false));
    setControls((prev) => {
      const next = { ...prev };
      for (const k of offlineKeys) delete next[k];
      return next;
    });
    setSelected((prev) => {
      const next = new Set(prev);
      for (const k of offlineKeys) next.delete(k);
      return next;
    });
    setActiveMotorKey((prev) => (prev && offlineKeys.has(prev) ? '' : prev));
    pushLog(t('log_offline_cleared', { count: offlineHits.length }), 'ok');
  };

  const removeMotorCard = (hit) => {
    const ok =
      typeof window === 'undefined'
        ? true
        : window.confirm(
            t('confirm_delete_card', { vendor: hit.vendor, esc: toHex(hit.esc_id), mst: toHex(hit.mst_id) }),
          );
    if (!ok) return;

    const key = motorKey(hit);
    setHits((prev) => prev.filter((x) => motorKey(x) !== key));
    setControls((prev) => {
      const next = { ...prev };
      delete next[key];
      return next;
    });
    setSelected((prev) => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
    setActiveMotorKey((prev) => (prev === key ? '' : prev));
    pushLog(t('log_card_removed', { vendor: hit.vendor, key }), 'ok');
  };

  const moveMotorCard = (fromKey, toKey) => {
    if (!fromKey || !toKey || fromKey === toKey) return;
    setHits((prev) => {
      const fromIndex = prev.findIndex((x) => motorKey(x) === fromKey);
      const toIndex = prev.findIndex((x) => motorKey(x) === toKey);
      if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return prev;
      const next = [...prev];
      const [item] = next.splice(fromIndex, 1);
      next.splice(toIndex, 0, item);
      return next;
    });
  };

  const addManualCard = () => {
    const vendor = String(manualDraft.vendor || 'damiao').trim() || 'damiao';
    const model = String(manualDraft.model || vendor).trim() || vendor;
    const esc = Number.parseInt(String(manualDraft.escId).replace(/^0x/i, ''), 16);
    const mst = Number.parseInt(String(manualDraft.mstId).replace(/^0x/i, ''), 16);
    if (!Number.isFinite(esc) || !Number.isFinite(mst)) {
      pushLog(t('log_manual_add_failed'), 'err');
      return;
    }

    const hit = {
      vendor,
      model,
      model_guess: model,
      esc_id: esc,
      mst_id: mst,
      probe: esc,
      detected_by: 'manual',
      online: false,
      updated_at_ms: Date.now(),
      last_check_ms: Date.now(),
    };
    const key = motorKey(hit);

    setHits((prev) =>
      prev.some((x) => motorKey(x) === key)
        ? prev.map((x) => (motorKey(x) === key ? { ...x, ...hit } : x))
        : [...prev, hit],
    );
    setControls((prev) => ({ ...prev, [key]: prev[key] || defaultControlsForHit(hit) }));
    setActiveMotorKey(key);
    setNewCardKeys(new Set([key]));
    pushLog(t('log_manual_added', { vendor, esc: manualDraft.escId, mst: manualDraft.mstId }), 'ok');
  };

  const runScan = (vendorList = null) =>
    runScanOp({
      connected,
      scanBusy,
      setScanBusy,
      vendors,
      scanTimeoutMs,
      activeMotorKey,
      setActiveMotorKey,
      setNewCardKeys,
      cardRefs,
      setHits,
      setControls,
      setScanProgress,
      onFound: ({ vendor, model, count }) => {
        setScanFoundFx((prev) => ({
          visible: true,
          message: t('found_count', { count, suffix: count > 1 ? 's' : '', vendor, model }),
          seq: prev.seq + 1,
        }));
      },
      pushLog,
      closeBusQuietly,
      setTargetFor,
      sendCmd,
      vendorList,
      t,
    });

  const verifyHit = (h) => verifyHitOp({ h, vendors, setTargetFor, sendCmd, setHits, closeBusQuietly, pushLog });

  const setIdFor = (h) => {
    const ok =
      typeof window === 'undefined'
        ? true
        : window.confirm(
            t('confirm_set_id', { vendor: h.vendor, esc: toHex(h.esc_id), mst: toHex(h.mst_id) }),
          );
    if (!ok) return Promise.resolve(false);
    return setIdForOp({ h, controls, vendors, setTargetFor, sendCmd, closeBusQuietly, pushLog });
  };

  const controlMotor = (h, action, controlOverride = null, options = {}) => {
    if (armBulkBusy && !options.allowDuringBulk) {
      pushLog(`control ${action} blocked: robot-arm bulk operation in progress`, 'info');
      return Promise.resolve(false);
    }
    return controlMotorOp({
      h,
      action,
      controls,
      controlOverride,
      vendors,
      setTargetFor,
      sendCmd,
      setHits,
      setControls,
      closeBusQuietly,
      pushLog,
    });
  };

  const zeroMotor = (h) =>
    zeroMotorOp({
      h,
      controls,
      vendors,
      setTargetFor,
      sendCmd,
      setHits,
      closeBusQuietly,
      pushLog,
    });

  const refreshMotorState = (h) =>
    refreshMotorStateOp({ h, vendors, setTargetFor, sendCmd, setHits, pushLog });

  const probeMotor = (h) =>
    probeMotorOp({ h, vendors, setTargetFor, sendCmd, setHits, closeBusQuietly, pushLog });

  const {
    robotArmModel,
    armScanBusy,
    armScanProgress,
    setRobotArmModel,
    robotArmJointRows,
    ensureRobotArmCards,
    scanRobotArmJoint,
    scanRobotArmAll,
  } = useRobotArmStudio({
    hits,
    setHits,
    controls,
    setControls,
    activeMotorKey,
    setActiveMotorKey,
    probeMotor,
    pushLog,
  });

  const runRobotArmBulk = async (name, fn) => {
    if (armBulkBusy) {
      pushLog(`robot-arm ${name} blocked: bulk operation in progress`, 'info');
      return false;
    }
    setArmBulkBusy(true);
    try {
      return await fn();
    } finally {
      setArmBulkBusy(false);
    }
  };

  const enableAllRobotArm = async () =>
    runRobotArmBulk('enable-all', async () => {
      pushLog('robot-arm enable-all start', 'info');
      let okCount = 0;
      for (const row of robotArmJointRows) {
        const ok = await controlMotor(row.hit, 'enable', null, { allowDuringBulk: true });
        if (ok) okCount += 1;
        await sleep(60);
      }
      const failCount = robotArmJointRows.length - okCount;
      pushLog(
        `robot-arm enable-all done ok=${okCount} fail=${failCount}`,
        failCount > 0 ? 'err' : 'ok',
      );
      return failCount === 0;
    });

  const disableAllRobotArm = async () =>
    runRobotArmBulk('disable-all', async () => {
      pushLog('robot-arm disable-all start', 'info');
      let okCount = 0;
      for (const row of robotArmJointRows) {
        const ok = await controlMotor(row.hit, 'disable', null, { allowDuringBulk: true });
        if (ok) okCount += 1;
        await sleep(60);
      }
      const failCount = robotArmJointRows.length - okCount;
      pushLog(
        `robot-arm disable-all done ok=${okCount} fail=${failCount}`,
        failCount > 0 ? 'err' : 'ok',
      );
      return failCount === 0;
    });

  const zeroAllRobotArm = async () =>
    runRobotArmBulk('zero-all', async () => {
      pushLog('robot-arm zero-all start', 'info');
      let okCount = 0;
      for (const row of robotArmJointRows) {
        const ok = await zeroMotor(row.hit);
        if (ok) okCount += 1;
        await sleep(70);
      }
      const failCount = robotArmJointRows.length - okCount;
      pushLog(`robot-arm zero-all done ok=${okCount} fail=${failCount}`, failCount > 0 ? 'err' : 'ok');
      return failCount === 0;
    });

  const resetPoseRobotArm = async () =>
    runRobotArmBulk('reset-pose', async () => {
      pushLog('robot-arm reset-pose start target=0.0rad', 'info');
      let okCount = 0;
      for (const row of robotArmJointRows) {
        const key = motorKey(row.hit);
        setControls((prev) => ({
          ...prev,
          [key]: {
            ...(prev[key] || defaultControlsForHit(row.hit)),
            mode: 'pos_vel',
            target: '0.0',
          },
        }));
        const ok = await controlMotor(
          row.hit,
          'move',
          { mode: 'pos_vel', target: '0.0' },
          { allowDuringBulk: true },
        );
        if (ok) okCount += 1;
        await sleep(60);
      }
      const failCount = robotArmJointRows.length - okCount;
      pushLog(`robot-arm reset-pose done ok=${okCount} fail=${failCount}`, failCount > 0 ? 'err' : 'ok');
      return failCount === 0;
    });

  const runRobotArmSelfCheck = async () => {
    if (armSelfCheckBusy) return;
    setArmSelfCheckBusy(true);
    setArmSelfCheckReport(null);
    const steps = [];
    const updateStep = (done, label) =>
      setArmSelfCheckProgress({
        active: true,
        done,
        total: 4,
        label,
        percent: Math.floor((done / 4) * 100),
      });

    try {
      updateStep(0, 'self-check: start');

      // 1) connection check
      const connOk = Boolean(connected);
      steps.push({ step: 'connection', ok: connOk, reason: connOk ? '' : 'ws disconnected' });
      if (!connOk) {
        setArmSelfCheckReport({
          ok: false,
          summary: 'FAILED',
          reason: 'ws disconnected',
          onlineCount: 0,
          total: 7,
          paramOkCount: 0,
          paramFailCount: 0,
          steps,
          at: Date.now(),
        });
        return;
      }
      updateStep(1, 'self-check: scan joints');

      // 2) scan 7 joints
      const scan = await scanRobotArmAll();
      const onlineCount = Number(scan?.onlineCount ?? 0);
      const total = Number(scan?.total ?? 7);
      const scanOk = onlineCount === total;
      steps.push({
        step: 'scan',
        ok: scanOk,
        reason: scanOk ? '' : `online ${onlineCount}/${total}`,
      });

      updateStep(2, 'self-check: summarize online');
      await sleep(80);

      // 3) online summary
      const onlineOk = onlineCount > 0;
      steps.push({
        step: 'online-summary',
        ok: onlineOk,
        reason: onlineOk ? `online ${onlineCount}/${total}` : 'no online joints',
      });

      // 4) read-back parameter verification
      updateStep(3, 'self-check: read params');
      const paramRet = await readRobotArmControlParams();
      let paramOkCount = 0;
      let paramFailCount = 0;
      Object.values(paramRet || {}).forEach((x) => {
        if (x?.ok) paramOkCount += 1;
        else paramFailCount += 1;
      });
      const paramOk = paramFailCount === 0 && paramOkCount > 0;
      steps.push({
        step: 'param-readback',
        ok: paramOk,
        reason: `ok=${paramOkCount}, fail=${paramFailCount}`,
      });

      updateStep(4, 'self-check: done');
      const allOk = steps.every((x) => x.ok);
      setArmSelfCheckReport({
        ok: allOk,
        summary: allOk ? 'PASSED' : 'FAILED',
        reason: allOk ? 'all checks passed' : steps.filter((x) => !x.ok).map((x) => x.reason).join('; '),
        onlineCount,
        total,
        paramOkCount,
        paramFailCount,
        steps,
        at: Date.now(),
      });
      pushLog(`robot-arm self-check ${allOk ? 'passed' : 'failed'} online=${onlineCount}/${total} params_ok=${paramOkCount} params_fail=${paramFailCount}`, allOk ? 'ok' : 'err');
    } finally {
      setArmSelfCheckBusy(false);
      setTimeout(() => {
        setArmSelfCheckProgress((prev) => ({ ...prev, active: false }));
      }, 700);
    }
  };

  const readDamiaoControlParams = async (h, timeoutMs = 1000, { closeBusAfter = true } = {}) => {
    if (!h || String(h.vendor) !== 'damiao') {
      throw new Error('read control params is damiao-only');
    }

    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    try {
      const ctrlModeRet = await sendCmd(
        'get_register_u32',
        { rid: DAMIAO_CTRL_PARAM_RID.ctrlMode, timeout_ms: timeoutMs },
        3000,
      );
      const currentBwRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.currentBw, timeout_ms: timeoutMs },
        3000,
      );
      const velKpRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.velKp, timeout_ms: timeoutMs },
        3000,
      );
      const velKiRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.velKi, timeout_ms: timeoutMs },
        3000,
      );
      const posKpRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.posKp, timeout_ms: timeoutMs },
        3000,
      );
      const posKiRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.posKi, timeout_ms: timeoutMs },
        3000,
      );

      const all = [ctrlModeRet, currentBwRet, velKpRet, velKiRet, posKpRet, posKiRet];
      const failed = all.find((x) => !x?.ok);
      if (failed) throw new Error(failed.error || 'read register failed');

      return {
        ctrlMode: Number(ctrlModeRet.data?.value ?? Number.NaN),
        currentBw: Number(currentBwRet.data?.value ?? Number.NaN),
        velKp: Number(velKpRet.data?.value ?? Number.NaN),
        velKi: Number(velKiRet.data?.value ?? Number.NaN),
        posKp: Number(posKpRet.data?.value ?? Number.NaN),
        posKi: Number(posKiRet.data?.value ?? Number.NaN),
      };
    } finally {
      if (closeBusAfter) await closeBusQuietly();
    }
  };

  const writeDamiaoControlParams = async (h, values, { store = true, closeBusAfter = true } = {}) => {
    if (!h || String(h.vendor) !== 'damiao') {
      throw new Error('write control params is damiao-only');
    }
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    try {
      const seq = [
        ['write_register_u32', { rid: DAMIAO_CTRL_PARAM_RID.ctrlMode, value: Number(values.ctrlMode) }],
        ['write_register_f32', { rid: DAMIAO_CTRL_PARAM_RID.currentBw, value: Number(values.currentBw) }],
        ['write_register_f32', { rid: DAMIAO_CTRL_PARAM_RID.velKp, value: Number(values.velKp) }],
        ['write_register_f32', { rid: DAMIAO_CTRL_PARAM_RID.velKi, value: Number(values.velKi) }],
        ['write_register_f32', { rid: DAMIAO_CTRL_PARAM_RID.posKp, value: Number(values.posKp) }],
        ['write_register_f32', { rid: DAMIAO_CTRL_PARAM_RID.posKi, value: Number(values.posKi) }],
      ];

      for (const [op, payload] of seq) {
        const ret = await sendCmd(op, payload, 3000);
        if (!ret?.ok) throw new Error(ret?.error || `${op} failed`);
      }

      if (store) {
        const stored = await sendCmd('store_parameters', { vendor: h.vendor }, 4000);
        if (!stored?.ok) throw new Error(stored?.error || 'store_parameters failed');
      }
    } finally {
      if (closeBusAfter) await closeBusQuietly();
    }
  };

  const readRobotArmControlParams = async ({ onProgress } = {}) => {
    const rows = robotArmJointRows.filter((x) => String(x.hit?.vendor) === 'damiao');
    if (rows.length === 0) {
      pushLog('robot-arm read params skipped: no damiao joints', 'err');
      return {};
    }
    onProgress?.({ active: true, done: 0, total: rows.length, label: 'reading params...', percent: 0 });
    const result = {};
    try {
      for (let i = 0; i < rows.length; i += 1) {
        const row = rows[i];
        try {
          const values = await readDamiaoControlParams(row.hit, 1000, { closeBusAfter: false });
          result[row.key] = { ok: true, values };
          pushLog(`robot-arm read params ok joint=${row.joint}`, 'ok');
        } catch (e) {
          result[row.key] = { ok: false, error: e.message || String(e) };
          pushLog(`robot-arm read params failed joint=${row.joint}: ${e.message || e}`, 'err');
        }
        const done = i + 1;
        onProgress?.({
          active: true,
          done,
          total: rows.length,
          label: `reading params joint ${row.joint} (${done}/${rows.length})`,
          percent: Math.floor((done / rows.length) * 100),
        });
        await sleep(10);
      }
      onProgress?.({ active: false, done: rows.length, total: rows.length, label: 'read done', percent: 100 });
      return result;
    } finally {
      await closeBusQuietly();
    }
  };

  const writeRobotArmControlParams = async (rowsWithValues = [], { onProgress } = {}) => {
    const rows = rowsWithValues.filter((x) => x?.hit && String(x.hit.vendor) === 'damiao');
    if (rows.length === 0) {
      pushLog('robot-arm write params skipped: no damiao joints', 'err');
      return {};
    }
    onProgress?.({ active: true, done: 0, total: rows.length, label: 'writing params...', percent: 0 });
    const result = {};
    try {
      for (let i = 0; i < rows.length; i += 1) {
        const row = rows[i];
        try {
          await writeDamiaoControlParams(row.hit, row.values, { store: true, closeBusAfter: false });
          pushLog(`robot-arm write params ok joint=${row.joint}`, 'ok');
          result[row.key] = { ok: true };
        } catch (e) {
          pushLog(`robot-arm write params failed joint=${row.joint}: ${e.message || e}`, 'err');
          result[row.key] = { ok: false, error: e.message || String(e) };
        }
        const done = i + 1;
        onProgress?.({
          active: true,
          done,
          total: rows.length,
          label: `writing params joint ${row.joint} (${done}/${rows.length})`,
          percent: Math.floor((done / rows.length) * 100),
        });
        await sleep(10);
      }
      onProgress?.({ active: false, done: rows.length, total: rows.length, label: 'write done', percent: 100 });
      return result;
    } finally {
      await closeBusQuietly();
    }
  };

  const selectedHits = useMemo(() => hits.filter((h) => selected.has(motorKey(h))), [hits, selected]);
  const activeMotor = useMemo(() => hits.find((h) => motorKey(h) === activeMotorKey) || null, [hits, activeMotorKey]);
  const activeControl = activeMotor ? controls[motorKey(activeMotor)] || defaultControlsForHit(activeMotor) : null;

  const toggleUiPref = (key) => setUiPrefs((prev) => ({ ...prev, [key]: !prev[key] }));
  const setUiPref = (key, value) => setUiPrefs((prev) => ({ ...prev, [key]: value }));
  const clearLogs = () => setLogs([]);

  return {
    wsUrl,
    setWsUrl,
    channel,
    setChannel,
    scanTimeoutMs,
    setScanTimeoutMs,
    connText,
    connected,
    targetTransport,
    targetSerialPort,
    scanBusy,
    scanProgress,
    scanFoundFx,
    canAction,
    vendors,
    setVendors,
    hits,
    controls,
    selectedHits,
    activeMotor,
    activeControl,
    activeMotorKey,
    setActiveMotorKey,
    newCardKeys,
    menuOpen,
    setMenuOpen,
    stateSnapshot,
    logs,
    clearLogs,
    uiPrefs,
    toggleUiPref,
    setUiPref,
    manualDraft,
    setManualDraft,
    robotArmModel,
    armScanBusy,
    armScanProgress,
    armBulkBusy,
    armSelfCheckBusy,
    armSelfCheckProgress,
    armSelfCheckReport,
    setRobotArmModel,
    robotArmJointRows,
    cardRefs,
    connectWs,
    disconnectWs,
    runScan,
    removeMotorCard,
    moveMotorCard,
    addManualCard,
    probeMotor,
    clearDevices,
    clearOfflineMotors,
    patchControl,
    controlMotor,
    zeroMotor,
    verifyHit,
    setIdFor,
    refreshMotorState,
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
  };
}
