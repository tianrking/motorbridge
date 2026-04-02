import {
  damiaoModelCandidates,
  dedupHitsByVendor,
  defaultControlsForHit,
  mergeHitsByVendor,
  motorKey,
  normalizeHits,
  parseNum,
  toHex,
} from './utils';
import { VENDORS } from './constants';

export async function runScanOp({
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
  onFound,
  pushLog,
  closeBusQuietly,
  setTargetFor,
  sendCmd,
  vendorList = null,
  t = null,
}) {
  const tr = (k, fallback) => (typeof t === 'function' ? t(k) : fallback);
  if (!connected) {
    pushLog('scan aborted: ws not connected', 'err');
    return;
  }
  if (scanBusy) {
    pushLog('scan ignored: previous scan still running', 'err');
    return;
  }

  const activeVendors = vendorList || VENDORS.filter((v) => vendors[v].enabled);
  if (activeVendors.length === 0) {
    pushLog('scan aborted: no vendor selected', 'err');
    return;
  }

  setScanBusy(true);
    setScanProgress({ active: true, done: 0, total: 0, label: tr('scanning', 'scanning...'), percent: 0 });

  try {
    const allFound = [];
    const phases = [];

    for (const vendor of activeVendors) {
      const cfg = vendors[vendor];
      const startId = parseNum(cfg.startId, 1);
      const endId = parseNum(cfg.endId, 16);
      const count = Math.max(1, endId - startId + 1);
      const models = vendor === 'damiao' ? damiaoModelCandidates(cfg.model) : [cfg.model || vendor];
      for (const model of models) phases.push({ vendor, model, count });
    }

    const totalSteps = phases.reduce((acc, p) => acc + p.count, 0);
    let completedSteps = 0;
    setScanProgress({ active: true, done: 0, total: totalSteps, label: tr('scanning', 'scanning...'), percent: 0 });

    for (const vendor of activeVendors) {
      const cfg = vendors[vendor];
      const startId = parseNum(cfg.startId, 1);
      const endId = parseNum(cfg.endId, 16);
      const timeout = Math.max(120, parseNum(scanTimeoutMs, 500));
      pushLog(`scan start ${vendor}:${toHex(startId)}..${toHex(endId)}`);

      const models = vendor === 'damiao' ? damiaoModelCandidates(cfg.model) : [cfg.model || vendor];
      let vendorHits = [];

      for (const model of models) {
        let progressTimer = null;
        const progressStartMs = Date.now();
        const estimateMs = Math.max(800, Math.min(30000, Math.max(1, endId - startId + 1) * timeout));
        const phaseCount = Math.max(1, endId - startId + 1);

        const updateProgress = (isFinal = false) => {
          const elapsed = Date.now() - progressStartMs;
          const estimatedInPhase = isFinal
            ? phaseCount
            : Math.min(phaseCount - 1, Math.max(0, Math.floor((elapsed / estimateMs) * phaseCount)));
          const done = completedSteps + estimatedInPhase;
          const percent = totalSteps > 0 ? Math.min(100, Math.floor((done / totalSteps) * 100)) : 0;
          setScanProgress({
            active: true,
            done,
            total: totalSteps,
            label: `scanning ${vendor}(${model}) ${toHex(startId)}..${toHex(endId)}`,
            percent,
          });
        };

        updateProgress(false);
        progressTimer = setInterval(() => updateProgress(false), 160);

        const defaultFid =
          vendor === 'robstride'
            ? parseNum(cfg.feedbackId, 0xFF)
            : vendor === 'damiao'
              ? parseNum(cfg.feedbackBase, 0x10) + (startId & 0x0f)
              : 0;

        await setTargetFor(vendor, model, startId, defaultFid);

        const scanPayload = {
          vendor,
          start_id: startId,
          end_id: endId,
          timeout_ms: timeout,
        };
        if (vendor === 'damiao') scanPayload.feedback_base = parseNum(cfg.feedbackBase, 0x10);
        if (vendor === 'robstride') scanPayload.feedback_ids = [parseNum(cfg.feedbackId, 0xFF)];

        const rangeCount = Math.max(1, endId - startId + 1);
        const scanWaitMs = Math.min(180000, Math.max(30000, rangeCount * timeout * 4));
        const ret = await sendCmd('scan', scanPayload, scanWaitMs);

        if (progressTimer) clearInterval(progressTimer);
        completedSteps += phaseCount;
        updateProgress(true);

        if (!ret.ok) {
          pushLog(`scan ${vendor} model=${model} failed: ${ret.error || 'unknown'}`, 'err');
          continue;
        }

        const normalized = normalizeHits(vendor, ret.data, model);
        vendorHits.push(...normalized);
        pushLog(`scan ${vendor} model=${model} ok: ${normalized.length} hit(s)`, 'ok');
        if (normalized.length > 0 && typeof onFound === 'function') {
          onFound({ vendor, model, count: normalized.length });
        }

        if (normalized.length > 0) {
          setHits((prev) => {
            const prevKeys = new Set(prev.map((x) => motorKey(x)));
            const merged = mergeHitsByVendor(prev, dedupHitsByVendor(normalized));
            const incomingNew = merged.map((x) => motorKey(x)).filter((k) => !prevKeys.has(k));

            setControls((prevControls) => {
              const next = { ...prevControls };
              for (const h of merged) {
                const k = motorKey(h);
                if (!next[k]) next[k] = defaultControlsForHit(h);
              }
              return next;
            });

            if (!activeMotorKey && merged.length > 0) {
              setActiveMotorKey(motorKey(merged[0]));
            }

            if (incomingNew.length > 0) {
              const first = incomingNew[0];
              setNewCardKeys(new Set(incomingNew));
              setTimeout(() => {
                const el = cardRefs.current[first];
                if (el) el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
              }, 30);
            }
            return merged;
          });
        }

        if (vendor === 'damiao') {
          for (const h of normalized) {
            if (h.detected_by === 'registers') {
              pushLog(
                `hit esc=${toHex(h.esc_id)} mst=${toHex(h.mst_id)} pmax=${Number(h.pmax).toFixed(2)} vmax=${Number(h.vmax).toFixed(2)} tmax=${Number(h.tmax).toFixed(2)} model=${h.model_guess || '-'}`,
                'ok',
              );
            } else if (h.detected_by === 'feedback') {
              pushLog(
                `hit esc=${toHex(h.esc_id)} mst=${toHex(h.mst_id)} status=${h.status} pos=${Number(h.pos).toFixed(3)} vel=${Number(h.vel).toFixed(3)} torq=${Number(h.torq).toFixed(3)}`,
                'ok',
              );
            }
          }
        }
      }

      vendorHits = dedupHitsByVendor(vendorHits);
      await closeBusQuietly();
      allFound.push(...vendorHits);
    }

    setHits((prev) => {
      const merged = mergeHitsByVendor(prev, dedupHitsByVendor(allFound));
      setControls((prevControls) => {
        const next = { ...prevControls };
        for (const h of merged) {
          const k = motorKey(h);
          if (!next[k]) next[k] = defaultControlsForHit(h);
        }
        return next;
      });
      return merged;
    });

    setScanProgress({
      active: true,
      done: totalSteps,
      total: totalSteps,
      label: tr('scan_done', 'scan done'),
      percent: 100,
    });
    pushLog(`scan done: added ${allFound.length} hit(s)`, 'ok');
  } catch (e) {
    pushLog(`scan error: ${e.message || e}`, 'err');
  } finally {
    setScanBusy(false);
    setTimeout(() => {
      setScanProgress((prev) => ({ ...prev, active: false }));
    }, 500);
  }
}
