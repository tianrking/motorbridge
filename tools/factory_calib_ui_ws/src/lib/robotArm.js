export const ROBOT_ARM_MODELS = [
  { key: 'rebot-arm-7dof', label: 'reBot-Arm 7DOF' },
  { key: 'rebot-arm-lite', label: 'reBot-Arm Lite' },
];

export const ROBOT_ARM_JOINTS = [
  { joint: 1, esc_id: 0x01, mst_id: 0x11 },
  { joint: 2, esc_id: 0x02, mst_id: 0x12 },
  { joint: 3, esc_id: 0x03, mst_id: 0x13 },
  { joint: 4, esc_id: 0x04, mst_id: 0x14 },
  { joint: 5, esc_id: 0x05, mst_id: 0x15 },
  { joint: 6, esc_id: 0x06, mst_id: 0x16 },
  { joint: 7, esc_id: 0x07, mst_id: 0x17 },
];

export function buildRobotArmHit(jointCfg, armModel) {
  return {
    vendor: 'damiao',
    model: armModel,
    model_guess: armModel,
    probe: jointCfg.esc_id,
    esc_id: jointCfg.esc_id,
    mst_id: jointCfg.mst_id,
    joint: jointCfg.joint,
    detected_by: 'arm-default',
    online: false,
    updated_at_ms: Date.now(),
    last_check_ms: Date.now(),
  };
}
