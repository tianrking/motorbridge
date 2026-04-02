export function ts() {
  return new Date().toLocaleTimeString();
}

export function parseNum(raw, fallback = 0) {
  const s = String(raw ?? '').trim();
  if (!s) return fallback;
  if (s.startsWith('0x') || s.startsWith('0X')) {
    const n = Number.parseInt(s, 16);
    return Number.isFinite(n) ? n : fallback;
  }
  const n = Number.parseFloat(s);
  return Number.isFinite(n) ? n : fallback;
}

export function toHex(n) {
  return `0x${Number(n).toString(16).toUpperCase()}`;
}

export function motorKey(h) {
  return `${h.vendor}:${h.esc_id}:${h.mst_id}`;
}

export function mergeHitsByVendor(prev, incoming) {
  const map = new Map();
  for (const h of prev) map.set(motorKey(h), h);
  for (const h of incoming) {
    const k = motorKey(h);
    const old = map.get(k);
    map.set(k, { ...(old || {}), ...h });
  }
  return [...map.values()];
}

export function dedupHitsByVendor(list) {
  const seen = new Set();
  const out = [];
  for (const h of list) {
    const k = motorKey(h);
    if (seen.has(k)) continue;
    seen.add(k);
    out.push(h);
  }
  return out;
}

export function defaultControlsForHit(hit) {
  const mode = hit.vendor === 'damiao' ? 'pos_vel' : (hit.vendor === 'robstride' ? 'mit' : 'vel');
  return {
    mode,
    target: '0.0',
    vlim: '1.0',
    kp: '30.0',
    kd: '1.0',
    tau: '0.0',
    ratio: '0.10',
    newEsc: toHex(hit.esc_id),
    newMst: toHex(hit.mst_id),
  };
}

export function isAutoModel(raw) {
  const s = String(raw || '').trim().toLowerCase();
  return s === '' || s === 'auto' || s === 'all' || s === '*';
}

export function damiaoModelCandidates(raw) {
  const s = String(raw || '').trim();
  if (isAutoModel(s)) return ['4310', '4340P', '4340'];
  if (s.includes(',')) {
    const arr = s.split(',').map((x) => x.trim()).filter(Boolean);
    return arr.length ? arr : ['4310', '4340P', '4340'];
  }
  return [s];
}

export function formatLocal(ms) {
  if (!ms) return '-';
  try {
    return new Date(ms).toLocaleTimeString();
  } catch {
    return '-';
  }
}

export function scanSummary(h) {
  if (h.vendor !== 'damiao') return `${toHex(h.esc_id)} / ${toHex(h.mst_id)}`;
  if (h.detected_by === 'registers') {
    return `reg p=${Number.isFinite(h.pmax) ? h.pmax.toFixed(2) : '-'} v=${Number.isFinite(h.vmax) ? h.vmax.toFixed(2) : '-'} t=${Number.isFinite(h.tmax) ? h.tmax.toFixed(2) : '-'}`;
  }
  if (h.detected_by === 'feedback') {
    return `fb pos=${Number.isFinite(h.pos) ? h.pos.toFixed(2) : '-'} vel=${Number.isFinite(h.vel) ? h.vel.toFixed(2) : '-'} torq=${Number.isFinite(h.torq) ? h.torq.toFixed(2) : '-'}`;
  }
  return '-';
}

export function normalizeHits(vendor, data, model) {
  const arr = Array.isArray(data?.hits) ? data.hits : [];
  const now = Date.now();
  const out = [];
  for (const h of arr) {
    if (vendor === 'damiao') {
      out.push({
        vendor,
        model,
        probe: Number(h.probe ?? h.esc_id ?? 0),
        esc_id: Number(h.esc_id ?? h.probe ?? 0),
        mst_id: Number(h.mst_id ?? h.probe_feedback_id ?? 0),
        detected_by: String(h.detected_by ?? ''),
        model_guess: String(h.model_guess ?? ''),
        pmax: Number(h.pmax ?? Number.NaN),
        vmax: Number(h.vmax ?? Number.NaN),
        tmax: Number(h.tmax ?? Number.NaN),
        status: Number(h.status ?? Number.NaN),
        pos: Number(h.pos ?? Number.NaN),
        vel: Number(h.vel ?? Number.NaN),
        torq: Number(h.torq ?? Number.NaN),
        updated_at_ms: now,
      });
      continue;
    }
    if (vendor === 'robstride') {
      out.push({
        vendor,
        model,
        probe: Number(h.probe ?? h.device_id ?? 0),
        esc_id: Number(h.device_id ?? h.probe ?? 0),
        mst_id: Number(h.feedback_id ?? 0xFF),
        updated_at_ms: now,
      });
      continue;
    }
    out.push({
      vendor,
      model,
      probe: Number(h.probe ?? h.motor_id ?? h.node_id ?? 0),
      esc_id: Number(h.motor_id ?? h.node_id ?? h.probe ?? 0),
      mst_id: Number(h.feedback_id ?? 0),
      updated_at_ms: now,
    });
  }
  return out;
}
