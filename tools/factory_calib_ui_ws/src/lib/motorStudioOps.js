import {
  defaultControlsForHit,
  mergeHitsByVendor,
  motorKey,
  normalizeHits,
  parseNum,
  toHex,
} from './utils';
import { SET_ID_VENDORS } from './constants';
import { CMD_TIMEOUTS } from './appConfig';
import { buildProbePayload, buildSetIdPayload } from './vendors';

export async function verifyHitOp({ h, vendors, setTargetFor, sendCmd, setHits, closeBusQuietly, pushLog }) {
  try {
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    const ret = await sendCmd(
      'verify',
      {
        vendor: h.vendor,
        motor_id: h.esc_id,
        feedback_id: h.mst_id,
        timeout_ms: 1200,
      },
      CMD_TIMEOUTS.verifyMs,
    );

    if (!ret.ok) {
      pushLog(`verify ${h.vendor} ${toHex(h.esc_id)} failed: ${ret.error || 'unknown'}`, 'err');
    } else {
      pushLog(`verify ${h.vendor} ${toHex(h.esc_id)} ok`, 'ok');
      setHits((prev) =>
        mergeHitsByVendor(prev, [
          {
            ...h,
            updated_at_ms: Date.now(),
            verify_ok: true,
            verified_esc_id: Number(ret?.data?.esc_id ?? Number.NaN),
            verified_mst_id: Number(ret?.data?.mst_id ?? Number.NaN),
          },
        ]),
      );
    }
    await closeBusQuietly();
  } catch (e) {
    pushLog(`verify error: ${e.message || e}`, 'err');
  }
}

export async function setIdForOp({ h, controls, vendors, setTargetFor, sendCmd, closeBusQuietly, pushLog }) {
  const c = controls[motorKey(h)] || defaultControlsForHit(h);
  const newEsc = parseNum(c.newEsc, h.esc_id);
  const newMst = parseNum(c.newMst, h.mst_id);

  if (!SET_ID_VENDORS.has(h.vendor)) {
    pushLog(`set-id skipped: ${h.vendor} not supported`, 'err');
    return;
  }

  try {
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    const payload = buildSetIdPayload(h.vendor, h, newEsc, newMst);
    const ret = await sendCmd('set_id', payload, CMD_TIMEOUTS.setIdMs);
    if (!ret.ok) {
      pushLog(`set-id ${h.vendor} failed: ${ret.error || 'unknown'}`, 'err');
    } else {
      pushLog(`set-id ${h.vendor} ok: ${toHex(newEsc)} / ${toHex(newMst)}`, 'ok');
    }
    await closeBusQuietly();
  } catch (e) {
    pushLog(`set-id error: ${e.message || e}`, 'err');
  }
}

export async function controlMotorOp({
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
}) {
  const c = { ...(controls[motorKey(h)] || defaultControlsForHit(h)), ...(controlOverride || {}) };
  const target = parseNum(c.target, 0);
  const vlim = parseNum(c.vlim, 1);
  const kp = parseNum(c.kp, 30);
  const kd = parseNum(c.kd, 1);
  const tau = parseNum(c.tau, 0);
  const ratio = parseNum(c.ratio, 0.1);

  try {
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);

    if (action === 'enable' || action === 'disable' || action === 'stop') {
      const ret = await sendCmd(action, { vendor: h.vendor }, CMD_TIMEOUTS.controlMs);
      if (!ret.ok) throw new Error(ret.error || `${action} failed`);
      pushLog(`${action} ${h.vendor} ${toHex(h.esc_id)} ok`, 'ok');
      if (action === 'enable' || action === 'disable') {
        const enabled = action === 'enable';
        setControls((prev) => ({ ...prev, [motorKey(h)]: { ...(prev[motorKey(h)] || c), enabled } }));
      }
      await closeBusQuietly();
      return true;
    }

    let op = 'pos_vel';
    let payload = { vendor: h.vendor, continuous: false, pos: target, vlim, ensure_timeout_ms: 2000 };

    if (c.mode === 'mit') {
      op = 'mit';
      payload = { vendor: h.vendor, continuous: false, pos: target, vel: 0, kp, kd, tau, ensure_timeout_ms: 2000 };
    } else if (c.mode === 'vel') {
      op = 'vel';
      payload = { vendor: h.vendor, continuous: false, vel: target, ensure_timeout_ms: 2000 };
    } else if (c.mode === 'force_pos') {
      op = 'force_pos';
      payload = { vendor: h.vendor, continuous: false, pos: target, vlim, ratio, ensure_timeout_ms: 2000 };
    }

    const ret = await sendCmd(op, payload, CMD_TIMEOUTS.verifyMs);
    if (!ret.ok) throw new Error(ret.error || `${op} failed`);

    pushLog(`move ${h.vendor} ${toHex(h.esc_id)} mode=${c.mode} target=${target.toFixed(3)} ok`, 'ok');
    setHits((prev) => mergeHitsByVendor(prev, [{ ...h, updated_at_ms: Date.now() }]));
    await closeBusQuietly();
    return true;
  } catch (e) {
    pushLog(`control ${action} error: ${e.message || e}`, 'err');
    return false;
  }
}

export async function zeroMotorOp({
  h,
  controls,
  vendors,
  setTargetFor,
  sendCmd,
  setHits,
  closeBusQuietly,
  pushLog,
}) {
  const c = controls[motorKey(h)] || defaultControlsForHit(h);
  if (!c.enabled) {
    pushLog(`zero blocked ${h.vendor} ${toHex(h.esc_id)}: enable first`, 'err');
    return false;
  }

  try {
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);

    const zeroRet = await sendCmd('set_zero_position', { vendor: h.vendor }, CMD_TIMEOUTS.controlMs);
    if (!zeroRet.ok) throw new Error(zeroRet.error || 'set_zero_position failed');

    const storeRet = await sendCmd('store_parameters', { vendor: h.vendor }, CMD_TIMEOUTS.controlMs);
    if (!storeRet.ok) {
      pushLog(
        `zero ${h.vendor} ${toHex(h.esc_id)} ok, but store failed: ${storeRet.error || 'unknown'}`,
        'err',
      );
    } else {
      pushLog(`zero+store ${h.vendor} ${toHex(h.esc_id)} ok`, 'ok');
    }

    setHits((prev) => mergeHitsByVendor(prev, [{ ...h, updated_at_ms: Date.now() }]));
    await closeBusQuietly();
    return true;
  } catch (e) {
    pushLog(`zero error: ${e.message || e}`, 'err');
    return false;
  }
}

export async function refreshMotorStateOp({ h, vendors, setTargetFor, sendCmd, setHits, pushLog }) {
  try {
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    const ret = await sendCmd('state_once', {}, CMD_TIMEOUTS.stateMs);
    if (!ret.ok) throw new Error(ret.error || 'state_once failed');

    const d = ret.data || {};
    setHits((prev) =>
      mergeHitsByVendor(prev, [
        {
          ...h,
          status: Number(d.status_code ?? h.status ?? Number.NaN),
          pos: Number(d.pos ?? h.pos ?? Number.NaN),
          vel: Number(d.vel ?? h.vel ?? Number.NaN),
          torq: Number(d.torq ?? h.torq ?? Number.NaN),
          online: true,
          last_check_ms: Date.now(),
          updated_at_ms: Date.now(),
        },
      ]),
    );

    pushLog(`state refreshed ${h.vendor} ${toHex(h.esc_id)}`, 'ok');
  } catch (e) {
    setHits((prev) =>
      mergeHitsByVendor(prev, [
        {
          ...h,
          online: false,
          last_check_ms: Date.now(),
          updated_at_ms: Date.now(),
        },
      ]),
    );
    pushLog(`state refresh failed: ${e.message || e}`, 'err');
  }
}

export async function checkOnlineOnceOp({
  h,
  vendors,
  setTargetFor,
  sendCmd,
  setHits,
  pushLog,
  silent = true,
}) {
  try {
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    const ret = await sendCmd('state_once', {}, 1200);
    if (!ret.ok) throw new Error(ret.error || 'state_once failed');

    const d = ret.data || {};
    setHits((prev) =>
      mergeHitsByVendor(prev, [
        {
          ...h,
          status: Number(d.status_code ?? h.status ?? Number.NaN),
          pos: Number(d.pos ?? h.pos ?? Number.NaN),
          vel: Number(d.vel ?? h.vel ?? Number.NaN),
          torq: Number(d.torq ?? h.torq ?? Number.NaN),
          online: true,
          last_check_ms: Date.now(),
          updated_at_ms: Date.now(),
        },
      ]),
    );

    if (!silent) pushLog(`online check ${h.vendor} ${toHex(h.esc_id)} online`, 'ok');
    return true;
  } catch {
    setHits((prev) =>
      mergeHitsByVendor(prev, [
        {
          ...h,
          online: false,
          last_check_ms: Date.now(),
          updated_at_ms: Date.now(),
        },
      ]),
    );
    if (!silent) pushLog(`online check ${h.vendor} ${toHex(h.esc_id)} offline`, 'err');
    return false;
  }
}

export async function probeMotorOp({
  h,
  vendors,
  setTargetFor,
  sendCmd,
  setHits,
  closeBusQuietly,
  pushLog,
}) {
  try {
    const model = h.model || vendors[h.vendor]?.model || h.vendor;
    await setTargetFor(h.vendor, model, h.esc_id, h.mst_id);

    const payload = buildProbePayload(h.vendor, h.esc_id, h.mst_id);
    const ret = await sendCmd('scan', payload, CMD_TIMEOUTS.verifyMs);
    if (!ret.ok) throw new Error(ret.error || 'probe scan failed');

    const list = normalizeHits(h.vendor, ret.data, model);
    const found = list.find((x) => Number(x.esc_id) === Number(h.esc_id));
    const online = Boolean(found);

    setHits((prev) =>
      mergeHitsByVendor(prev, [
        {
          ...h,
          ...(found || {}),
          model,
          online,
          last_check_ms: Date.now(),
          updated_at_ms: Date.now(),
        },
      ]),
    );

    pushLog(
      `probe ${h.vendor} ${toHex(h.esc_id)} ${online ? 'online' : 'offline'}`,
      online ? 'ok' : 'err',
    );
    await closeBusQuietly();
    return online;
  } catch (e) {
    setHits((prev) =>
      mergeHitsByVendor(prev, [
        {
          ...h,
          online: false,
          last_check_ms: Date.now(),
        },
      ]),
    );
    pushLog(`probe failed: ${e.message || e}`, 'err');
    return false;
  }
}
