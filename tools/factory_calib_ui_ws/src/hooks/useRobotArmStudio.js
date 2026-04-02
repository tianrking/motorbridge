import { useEffect, useMemo, useState } from 'react';
import { defaultControlsForHit, motorKey } from '../lib/utils';
import { ROBOT_ARM_JOINTS, ROBOT_ARM_MODELS, buildRobotArmHit } from '../lib/robotArm';

const LS_ROBOT_ARM_MODEL_KEY = 'factory_calib_ui_ws_robot_arm_model_v1';

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
  const [robotArmModel, setRobotArmModelState] = useState(() =>
    loadJson(LS_ROBOT_ARM_MODEL_KEY, ROBOT_ARM_MODELS[0].key),
  );

  useEffect(() => {
    try {
      if (typeof window === 'undefined') return;
      window.localStorage.setItem(LS_ROBOT_ARM_MODEL_KEY, JSON.stringify(robotArmModel));
    } catch {
      // ignore localStorage failures
    }
  }, [robotArmModel]);

  const setRobotArmModel = (next) => {
    setRobotArmModelState(next);
    setHits((prev) =>
      prev.map((h) => {
        const j = ROBOT_ARM_JOINTS.find(
          (x) => x.esc_id === Number(h.esc_id) && x.mst_id === Number(h.mst_id),
        );
        if (!j || h.vendor !== 'damiao') return h;
        return { ...h, model: next, model_guess: next, joint: j.joint };
      }),
    );
  };

  const ensureRobotArmCards = () => {
    const armHits = ROBOT_ARM_JOINTS.map((j) => buildRobotArmHit(j, robotArmModel));
    setHits((prev) => {
      const merged = [...prev];
      for (const h of armHits) {
        const idx = merged.findIndex(
          (x) => x.vendor === 'damiao' && Number(x.esc_id) === h.esc_id && Number(x.mst_id) === h.mst_id,
        );
        if (idx >= 0) merged[idx] = { ...merged[idx], ...h, online: merged[idx].online ?? false };
        else merged.push(h);
      }
      return merged;
    });
    setControls((prev) => {
      const next = { ...prev };
      for (const h of armHits) {
        const key = motorKey(h);
        if (!next[key]) next[key] = defaultControlsForHit(h);
      }
      return next;
    });
    if (!activeMotorKey && armHits[0]) setActiveMotorKey(motorKey(armHits[0]));
  };

  const robotArmJointRows = useMemo(
    () =>
      ROBOT_ARM_JOINTS.map((j) => {
        const found = hits.find(
          (h) => h.vendor === 'damiao' && Number(h.esc_id) === j.esc_id && Number(h.mst_id) === j.mst_id,
        );
        const hit = found || buildRobotArmHit(j, robotArmModel);
        const key = motorKey(hit);
        const control = controls[key] || defaultControlsForHit(hit);
        return { joint: j.joint, hit, control, key };
      }),
    [hits, controls, robotArmModel],
  );

  const scanRobotArmJoint = async (jointNumber) => {
    const row = robotArmJointRows.find((x) => x.joint === jointNumber);
    if (!row) return false;
    return probeMotor(row.hit);
  };

  const scanRobotArmAll = async () => {
    ensureRobotArmCards();
    pushLog(`robot-arm scan start model=${robotArmModel} joints=1..7`, 'info');
    let onlineCount = 0;
    for (const row of robotArmJointRows) {
      const ok = await probeMotor(row.hit);
      if (ok) onlineCount += 1;
      await sleep(40);
    }
    pushLog(`robot-arm scan done online=${onlineCount}/7`, 'ok');
  };

  return {
    robotArmModel,
    setRobotArmModel,
    robotArmJointRows,
    ensureRobotArmCards,
    scanRobotArmJoint,
    scanRobotArmAll,
  };
}
