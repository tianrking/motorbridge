import { useState } from 'react';
import { bulkOp, sleep } from '../lib/async';
import { DAMIAO_CTRL_PARAM_RID } from '../lib/appConfig';
import { defaultControlsForHit, motorKey } from '../lib/utils';
import { useRobotArmStudio } from './useRobotArmStudio';

export function useRobotArmOps({
  connected,
  vendors,
  hits,
  setHits,
  controls,
  setControls,
  activeMotorKey,
  setActiveMotorKey,
  pushLog,
  controlMotor,
  zeroMotor,
  probeMotor,
  setTargetFor,
  sendCmd,
  closeBusQuietly,
}) {
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

  const readDamiaoControlParams = async (h, timeoutMs = 1000, { closeBusAfter = true } = {}) => {
    if (!h || String(h.vendor) !== 'damiao') {
      throw new Error('read control params is damiao-only');
    }

    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    try {
      const ctrlModeRet = await sendCmd(
        'get_register_u32',
        { rid: DAMIAO_CTRL_PARAM_RID.ctrlMode, timeout_ms: timeoutMs },
        3000
      );
      const currentBwRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.currentBw, timeout_ms: timeoutMs },
        3000
      );
      const velKpRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.velKp, timeout_ms: timeoutMs },
        3000
      );
      const velKiRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.velKi, timeout_ms: timeoutMs },
        3000
      );
      const posKpRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.posKp, timeout_ms: timeoutMs },
        3000
      );
      const posKiRet = await sendCmd(
        'get_register_f32',
        { rid: DAMIAO_CTRL_PARAM_RID.posKi, timeout_ms: timeoutMs },
        3000
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

  const writeDamiaoControlParams = async (
    h,
    values,
    { store = true, closeBusAfter = true } = {}
  ) => {
    if (!h || String(h.vendor) !== 'damiao') {
      throw new Error('write control params is damiao-only');
    }
    await setTargetFor(h.vendor, h.model || vendors[h.vendor].model, h.esc_id, h.mst_id);
    try {
      const seq = [
        [
          'write_register_u32',
          { rid: DAMIAO_CTRL_PARAM_RID.ctrlMode, value: Number(values.ctrlMode) },
        ],
        [
          'write_register_f32',
          { rid: DAMIAO_CTRL_PARAM_RID.currentBw, value: Number(values.currentBw) },
        ],
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
    onProgress?.({
      active: true,
      done: 0,
      total: rows.length,
      label: 'reading params...',
      percent: 0,
    });
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
      onProgress?.({
        active: false,
        done: rows.length,
        total: rows.length,
        label: 'read done',
        percent: 100,
      });
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
    onProgress?.({
      active: true,
      done: 0,
      total: rows.length,
      label: 'writing params...',
      percent: 0,
    });
    const result = {};
    try {
      for (let i = 0; i < rows.length; i += 1) {
        const row = rows[i];
        try {
          await writeDamiaoControlParams(row.hit, row.values, {
            store: true,
            closeBusAfter: false,
          });
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
      onProgress?.({
        active: false,
        done: rows.length,
        total: rows.length,
        label: 'write done',
        percent: 100,
      });
      return result;
    } finally {
      await closeBusQuietly();
    }
  };

  const enableAllRobotArm = async () =>
    runRobotArmBulk('enable-all', async () => {
      pushLog('robot-arm enable-all start', 'info');
      const { okCount, failCount } = await bulkOp(
        robotArmJointRows,
        (row) => controlMotor(row.hit, 'enable', null, { allowDuringBulk: true }),
        60
      );
      pushLog(
        `robot-arm enable-all done ok=${okCount} fail=${failCount}`,
        failCount > 0 ? 'err' : 'ok'
      );
      return failCount === 0;
    });

  const disableAllRobotArm = async () =>
    runRobotArmBulk('disable-all', async () => {
      pushLog('robot-arm disable-all start', 'info');
      const { okCount, failCount } = await bulkOp(
        robotArmJointRows,
        (row) => controlMotor(row.hit, 'disable', null, { allowDuringBulk: true }),
        60
      );
      pushLog(
        `robot-arm disable-all done ok=${okCount} fail=${failCount}`,
        failCount > 0 ? 'err' : 'ok'
      );
      return failCount === 0;
    });

  const zeroAllRobotArm = async () =>
    runRobotArmBulk('zero-all', async () => {
      pushLog('robot-arm zero-all start', 'info');
      const { okCount, failCount } = await bulkOp(
        robotArmJointRows,
        (row) => zeroMotor(row.hit),
        70
      );
      pushLog(
        `robot-arm zero-all done ok=${okCount} fail=${failCount}`,
        failCount > 0 ? 'err' : 'ok'
      );
      return failCount === 0;
    });

  const resetPoseRobotArm = async () =>
    runRobotArmBulk('reset-pose', async () => {
      pushLog('robot-arm reset-pose start target=0.0rad', 'info');
      let okCount = 0;
      for (const row of robotArmJointRows) {
        const key = motorKey(row.hit);
        const mode = 'pos_vel';
        setControls((prev) => ({
          ...prev,
          [key]: {
            ...(prev[key] || defaultControlsForHit(row.hit)),
            mode,
            target: '0.0',
          },
        }));
        const ok = await controlMotor(
          row.hit,
          'move',
          { mode, target: '0.0' },
          { allowDuringBulk: true }
        );
        if (ok) okCount += 1;
        await sleep(60);
      }
      const failCount = robotArmJointRows.length - okCount;
      pushLog(
        `robot-arm reset-pose done ok=${okCount} fail=${failCount}`,
        failCount > 0 ? 'err' : 'ok'
      );
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
      const onlineOk = onlineCount > 0;
      steps.push({
        step: 'online-summary',
        ok: onlineOk,
        reason: onlineOk ? `online ${onlineCount}/${total}` : 'no online joints',
      });
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
        reason: allOk
          ? 'all checks passed'
          : steps
              .filter((x) => !x.ok)
              .map((x) => x.reason)
              .join('; '),
        onlineCount,
        total,
        paramOkCount,
        paramFailCount,
        steps,
        at: Date.now(),
      });
      pushLog(
        `robot-arm self-check ${allOk ? 'passed' : 'failed'} online=${onlineCount}/${total} params_ok=${paramOkCount} params_fail=${paramFailCount}`,
        allOk ? 'ok' : 'err'
      );
    } finally {
      setArmSelfCheckBusy(false);
      setTimeout(() => {
        setArmSelfCheckProgress((prev) => ({ ...prev, active: false }));
      }, 700);
    }
  };

  return {
    robotArmModel,
    armScanBusy,
    armScanProgress,
    setRobotArmModel,
    robotArmJointRows,
    ensureRobotArmCards,
    scanRobotArmJoint,
    scanRobotArmAll,
    armBulkBusy,
    armSelfCheckBusy,
    armSelfCheckProgress,
    armSelfCheckReport,
    runRobotArmSelfCheck,
    enableAllRobotArm,
    disableAllRobotArm,
    zeroAllRobotArm,
    resetPoseRobotArm,
    readRobotArmControlParams,
    writeRobotArmControlParams,
  };
}
