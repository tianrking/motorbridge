import { describe, expect, it } from 'vitest';
import { REBOT_ARM_JOINT_LIMITS, normalizeRobotArmModel } from './robotArm';

describe('robotArm config', () => {
  it('normalizes aliases', () => {
    expect(normalizeRobotArmModel('reBot-Arm Lite')).toBe('rebot-arm-robstride');
    expect(normalizeRobotArmModel('7dof')).toBe('rebot-arm-damiao');
  });

  it('contains joint limits for all joints', () => {
    for (let joint = 1; joint <= 7; joint += 1) {
      expect(REBOT_ARM_JOINT_LIMITS[joint]).toBeTruthy();
      expect(REBOT_ARM_JOINT_LIMITS[joint].min).toBeLessThanOrEqual(
        REBOT_ARM_JOINT_LIMITS[joint].max
      );
    }
  });
});
