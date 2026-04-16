import { useEffect, useRef, useState } from 'react';
import { DEFAULT_VENDOR_CONFIG } from '../lib/constants';
import { defaultControlsForHit, motorKey, toHex } from '../lib/utils';
import { runScanOp } from '../lib/motorScanOps';

export function useScanState({
  t,
  connected,
  scanTimeoutMs,
  activeMotorKey,
  setActiveMotorKey,
  setHits,
  setControls,
  pushLog,
  closeBusQuietly,
  setTargetFor,
  sendCmd,
}) {
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
  const [manualDraft, setManualDraft] = useState({
    vendor: 'damiao',
    model: '4310',
    escId: '0x05',
    mstId: '0x15',
  });
  const [newCardKeys, setNewCardKeys] = useState(new Set());
  const cardRefs = useRef({});

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

  const clearDevices = () => {
    const ok = typeof window === 'undefined' ? true : window.confirm(t('confirm_clear_all'));
    if (!ok) return;
    setHits([]);
    setControls({});
    setActiveMotorKey('');
    try {
      if (typeof window !== 'undefined') {
        window.localStorage.removeItem('factory_calib_ui_ws_hits_v1');
        window.localStorage.removeItem('factory_calib_ui_ws_controls_v1');
        window.localStorage.removeItem('factory_calib_ui_ws_active_motor_v1');
      }
    } catch {
      // ignore localStorage failures
    }
  };

  const clearOfflineMotors = (hits, setSelected) => {
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

  const removeMotorCard = (hit, setSelected) => {
    const ok =
      typeof window === 'undefined'
        ? true
        : window.confirm(
            t('confirm_delete_card', {
              vendor: hit.vendor,
              esc: toHex(hit.esc_id),
              mst: toHex(hit.mst_id),
            })
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
        : [...prev, hit]
    );
    setControls((prev) => ({ ...prev, [key]: prev[key] || defaultControlsForHit(hit) }));
    setActiveMotorKey(key);
    setNewCardKeys(new Set([key]));
    pushLog(
      t('log_manual_added', { vendor, esc: manualDraft.escId, mst: manualDraft.mstId }),
      'ok'
    );
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

  return {
    scanBusy,
    scanProgress,
    scanFoundFx,
    vendors,
    setVendors,
    manualDraft,
    setManualDraft,
    newCardKeys,
    cardRefs,
    runScan,
    clearDevices,
    clearOfflineMotors,
    removeMotorCard,
    moveMotorCard,
    addManualCard,
  };
}
