export function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function bulkOp(rows, fn, gapMs = 0) {
  let okCount = 0;
  let failCount = 0;
  const total = Array.isArray(rows) ? rows.length : 0;

  for (let i = 0; i < total; i += 1) {
    const row = rows[i];
    try {
      const ok = await fn(row, i);
      if (ok) okCount += 1;
      else failCount += 1;
    } catch {
      failCount += 1;
    }
    if (gapMs > 0 && i < total - 1) {
      await sleep(gapMs);
    }
  }

  return { total, okCount, failCount };
}
