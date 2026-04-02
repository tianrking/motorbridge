import { useEffect, useMemo, useRef, useState } from 'react';
import { DEFAULT_VENDOR_CONFIG } from '../lib/constants';
import { defaultControlsForHit, motorKey, ts } from '../lib/utils';
import { runScanOp } from '../lib/motorScanOps';
import {
  controlMotorOp,
  probeMotorOp,
  refreshMotorStateOp,
  setIdForOp,
  verifyHitOp,
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
    connectWs,
    disconnectWs,
    sendCmd,
    closeBusQuietly,
    setTargetFor,
  } = useGatewayBridge({ wsUrl, channel, pushLog, setStateSnapshot });

  const canAction = connected && !scanBusy;

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

  const removeMotorCard = (hit) => {
    const ok =
      typeof window === 'undefined'
        ? true
        : window.confirm(t('confirm_delete_card', { vendor: hit.vendor, esc: hit.esc_id, mst: hit.mst_id }));
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
        : window.confirm(t('confirm_set_id', { vendor: h.vendor, esc: h.esc_id, mst: h.mst_id }));
    if (!ok) return Promise.resolve(false);
    return setIdForOp({ h, controls, vendors, setTargetFor, sendCmd, closeBusQuietly, pushLog });
  };

  const controlMotor = (h, action) =>
    controlMotorOp({ h, action, controls, vendors, setTargetFor, sendCmd, setHits, closeBusQuietly, pushLog });

  const refreshMotorState = (h) =>
    refreshMotorStateOp({ h, vendors, setTargetFor, sendCmd, setHits, pushLog });

  const probeMotor = (h) =>
    probeMotorOp({ h, vendors, setTargetFor, sendCmd, setHits, closeBusQuietly, pushLog });

  const {
    robotArmModel,
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
    patchControl,
    controlMotor,
    verifyHit,
    setIdFor,
    refreshMotorState,
    ensureRobotArmCards,
    scanRobotArmJoint,
    scanRobotArmAll,
  };
}
