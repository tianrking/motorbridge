import { useMemo, useState } from 'react';
import { defaultControlsForHit, motorKey } from '../lib/utils';
import { sleep } from '../lib/async';
import {
  ROBOT_ARM_JOINTS,
  ROBOT_ARM_MODELS,
  armMotorModelForProfile,
  buildRobotArmHit,
  isProfileJointHit,
  normalizeRobotArmModel,
} from '../lib/robotArm';
import { usePersistedState } from './usePersistedState';

const LS_ROBOT_ARM_MODEL_KEY = 'factory_calib_ui_ws_robot_arm_model_v1';

function normalizeControlForHit(hit, rawControl) {
  const defaults = defaultControlsForHit(hit);
  const merged = {
    ...defaults,
    ...(rawControl && typeof rawControl === 'object' ? rawControl : {}),
  };
  const required = ['mode', 'target', 'vlim', 'kp', 'kd', 'tau', 'ratio', 'newEsc', 'newMst'];
  required.forEach((k) => {
    const v = merged[k];
    if (v == null || String(v).trim() === '') merged[k] = defaults[k];
  });
  return merged;
}

export function useRobotArmStudio({
  hits,
  setHits,
  controls,
  setControls,
  activeMotorKey,
  setActiveMotorKey,
  probeMotor,
  pushLog,
}) {
  const [armScanBusy, setArmScanBusy] = useState(false);
  const [armScanProgress, setArmScanProgress] = useState({
    active: false,
    done: 0,
    total: 7,
    label: '',
    percent: 0,
  });
  const [robotArmModel, setRobotArmModelState] = usePersistedState(
    LS_ROBOT_ARM_MODEL_KEY,
    ROBOT_ARM_MODELS[0].key,
    (cached, fallback) => normalizeRobotArmModel(cached || fallback)
  );

  const setRobotArmModel = (nextRaw) => {
    const next = normalizeRobotArmModel(nextRaw);
    setRobotArmModelState(next);
    setHits((prev) =>
      prev.map((h) => {
        const j = ROBOT_ARM_JOINTS.find((x) => isProfileJointHit(h, next, x));
        if (!j) return h;
        return { ...h, arm_profile: next, joint: j.joint };
      })
    );
  };

  const ensureRobotArmCards = () => {
    const profile = normalizeRobotArmModel(robotArmModel);
    const defaultMotorModel = armMotorModelForProfile(profile);
    const armHits = ROBOT_ARM_JOINTS.map((j) => buildRobotArmHit(j, profile));

    setHits((prev) => {
      const merged = [...prev];
      for (const h of armHits) {
        const jointCfg = ROBOT_ARM_JOINTS.find((j) => Number(j.esc_id) === Number(h.esc_id));
        const idx = merged.findIndex((x) => jointCfg && isProfileJointHit(x, profile, jointCfg));
        if (idx >= 0) {
          const old = merged[idx];
          const keepModel = String(old.model || '').trim();
          const resolvedModel = keepModel && keepModel !== 'auto' ? keepModel : defaultMotorModel;
          merged[idx] = {
            ...old,
            ...h,
            model: resolvedModel,
            model_guess: old.model_guess || resolvedModel,
            arm_profile: profile,
            online: old.online ?? false,
          };
        } else {
          merged.push({
            ...h,
            model: defaultMotorModel,
            model_guess: defaultMotorModel,
            arm_profile: profile,
          });
        }
      }
      return merged;
    });

    setControls((prev) => {
      const next = { ...prev };
      for (const h of armHits) {
        const key = motorKey(h);
        next[key] = normalizeControlForHit(h, next[key]);
      }
      return next;
    });

    if (!activeMotorKey && armHits[0]) setActiveMotorKey(motorKey(armHits[0]));
  };

  const robotArmJointRows = useMemo(
    () =>
      ROBOT_ARM_JOINTS.map((j) => {
        const found = hits.find((h) => isProfileJointHit(h, robotArmModel, j));
        const hit = found || buildRobotArmHit(j, robotArmModel);
        const key = motorKey(hit);
        const control = normalizeControlForHit(hit, controls[key]);
        return { joint: j.joint, hit, control, key };
      }),
    [hits, controls, robotArmModel]
  );

  const scanRobotArmJoint = async (jointNumber) => {
    const row = robotArmJointRows.find((x) => x.joint === jointNumber);
    if (!row) return false;
    return probeMotor(row.hit);
  };

  const scanRobotArmAll = async () => {
    if (armScanBusy) {
      pushLog('robot-arm scan ignored: previous scan still running', 'err');
      return null;
    }

    setArmScanBusy(true);
    setArmScanProgress({
      active: true,
      done: 0,
      total: ROBOT_ARM_JOINTS.length,
      label: 'robot-arm scanning...',
      percent: 0,
    });

    ensureRobotArmCards();
    const profile = normalizeRobotArmModel(robotArmModel);
    pushLog(`robot-arm scan start profile=${profile} joints=1..7`, 'info');
    let onlineCount = 0;
    try {
      for (let i = 0; i < ROBOT_ARM_JOINTS.length; i += 1) {
        const j = ROBOT_ARM_JOINTS[i];
        const step = i + 1;
        const hit = buildRobotArmHit(j, profile);

        let tick = 0;
        const basePercent = Math.floor((i / ROBOT_ARM_JOINTS.length) * 100);
        const progressTimer = setInterval(() => {
          tick += 1;
          const inStep = Math.min(12, tick);
          setArmScanProgress({
            active: true,
            done: i,
            total: ROBOT_ARM_JOINTS.length,
            label: `robot-arm scanning joint ${j.joint} (${step}/7)`,
            percent: Math.min(99, basePercent + inStep),
          });
        }, 120);

        const ok = await probeMotor(hit);
        if (ok) onlineCount += 1;
        clearInterval(progressTimer);

        setArmScanProgress({
          active: true,
          done: step,
          total: ROBOT_ARM_JOINTS.length,
          label: `robot-arm scanning joint ${j.joint} (${step}/7)`,
          percent: Math.floor((step / ROBOT_ARM_JOINTS.length) * 100),
        });
        await sleep(40);
      }

      pushLog(`robot-arm scan done online=${onlineCount}/7`, 'ok');
      setArmScanProgress({
        active: true,
        done: ROBOT_ARM_JOINTS.length,
        total: ROBOT_ARM_JOINTS.length,
        label: 'robot-arm scan done',
        percent: 100,
      });
      return { total: ROBOT_ARM_JOINTS.length, onlineCount };
    } finally {
      setArmScanBusy(false);
      setTimeout(() => {
        setArmScanProgress((prev) => ({ ...prev, active: false }));
      }, 500);
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
  };
}
