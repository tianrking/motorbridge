export const ROBOT_ARM_MODELS = [
  { key: 'rebot-arm-damiao', label: 'rebot-arm-damiao' },
  { key: 'rebot-arm-robstride', label: 'rebot-arm-robstride' },
];

const ROBOT_ARM_MODEL_KEYS = new Set(ROBOT_ARM_MODELS.map((x) => x.key));
const ROBOT_ARM_MODEL_ALIASES = {
  'reBot-Arm 7DOF': 'rebot-arm-damiao',
  'reBot-Arm Lite': 'rebot-arm-robstride',
  'rebot-arm-7dof': 'rebot-arm-damiao',
  'rebot-arm-lite': 'rebot-arm-robstride',
  '7dof': 'rebot-arm-damiao',
  'lite': 'rebot-arm-robstride',
};

const PROFILE_VENDOR = {
  'rebot-arm-damiao': 'damiao',
  'rebot-arm-robstride': 'robstride',
};

const PROFILE_DEFAULT_MODEL = {
  'rebot-arm-damiao': '4310',
  'rebot-arm-robstride': 'rs-00',
};

const ROBSTRIDE_FEEDBACK_IDS = [0xFD, 0xFF, 0xFE];

export const ROBOT_ARM_JOINTS = [
  { joint: 1, esc_id: 0x01 },
  { joint: 2, esc_id: 0x02 },
  { joint: 3, esc_id: 0x03 },
  { joint: 4, esc_id: 0x04 },
  { joint: 5, esc_id: 0x05 },
  { joint: 6, esc_id: 0x06 },
  { joint: 7, esc_id: 0x07 },
];

export const REBOT_ARM_DAMIAO_DEFAULT_TEMPLATE = {
  1: { ctrlMode: '2', currentBw: '1000', velKp: '0.0125', velKi: '0.004', posKp: '150', posKi: '0.5' },
  2: { ctrlMode: '2', currentBw: '1000', velKp: '0.013', velKi: '0.004', posKp: '200', posKi: '10' },
  3: { ctrlMode: '2', currentBw: '1000', velKp: '0.013', velKi: '0.004', posKp: '200', posKi: '10' },
  4: { ctrlMode: '2', currentBw: '1000', velKp: '0.0008', velKi: '0.002', posKp: '50', posKi: '1' },
  5: { ctrlMode: '2', currentBw: '1000', velKp: '0.0008', velKi: '0.002', posKp: '50', posKi: '1' },
  6: { ctrlMode: '2', currentBw: '1000', velKp: '0.0008', velKi: '0.002', posKp: '50', posKi: '1' },
};

export const REBOT_ARM_JOINT_LIMITS = {
  1: { min: -2.61, max: 2.61 },
  2: { min: -3.7, max: 0.0 },
  3: { min: -3.7, max: 0.0 },
  4: { min: -1.57, max: 1.57 },
  5: { min: -1.57, max: 1.57 },
  6: { min: -1.57, max: 1.57 },
  7: { min: -5.7, max: 0.0 },
};

export const ZERO_SAFE_EPS_RAD = 0.08;

export function normalizeRobotArmModel(raw) {
  const text = String(raw ?? '').trim();
  if (!text) return ROBOT_ARM_MODELS[0].key;
  if (ROBOT_ARM_MODEL_KEYS.has(text)) return text;
  if (ROBOT_ARM_MODEL_ALIASES[text]) return ROBOT_ARM_MODEL_ALIASES[text];
  return ROBOT_ARM_MODELS[0].key;
}

export function armVendorForProfile(armModel) {
  const key = normalizeRobotArmModel(armModel);
  return PROFILE_VENDOR[key] || 'damiao';
}

export function armMotorModelForProfile(armModel) {
  const key = normalizeRobotArmModel(armModel);
  return PROFILE_DEFAULT_MODEL[key] || '4310';
}

export function defaultFeedbackIdForProfile(armModel, escId) {
  const vendor = armVendorForProfile(armModel);
  if (vendor === 'robstride') return 0xFD;
  return 0x10 + (Number(escId) & 0x0F);
}

export function isProfileJointHit(hit, armModel, jointCfg) {
  const vendor = armVendorForProfile(armModel);
  if (String(hit.vendor) !== vendor) return false;
  if (Number(hit.esc_id) !== Number(jointCfg.esc_id)) return false;
  const fid = Number(hit.mst_id);
  if (vendor === 'robstride') return ROBSTRIDE_FEEDBACK_IDS.includes(fid);
  return fid === defaultFeedbackIdForProfile(armModel, jointCfg.esc_id);
}

export function buildRobotArmHit(jointCfg, armModel) {
  const profile = normalizeRobotArmModel(armModel);
  const vendor = armVendorForProfile(profile);
  const motorModel = armMotorModelForProfile(profile);
  return {
    vendor,
    model: motorModel,
    model_guess: motorModel,
    arm_profile: profile,
    probe: jointCfg.esc_id,
    esc_id: jointCfg.esc_id,
    mst_id: defaultFeedbackIdForProfile(profile, jointCfg.esc_id),
    joint: jointCfg.joint,
    detected_by: 'arm-default',
    online: false,
    updated_at_ms: Date.now(),
    last_check_ms: Date.now(),
  };
}
