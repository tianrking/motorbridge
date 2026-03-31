#!/usr/bin/env python3
"""02: register read/write template.

Vendor register map differs; this file demonstrates safe call pattern.
"""

from __future__ import annotations

from motorbridge import Controller

# ===== USER CONFIG =====
CHANNEL = "can0"
VENDOR = "damiao"
MODEL = "4340P"
MOTOR_ID = 0x01
FEEDBACK_ID = 0x11
TIMEOUT_MS = 800

# Use your real register IDs after checking vendor docs.
REG_F32_ID = 0x0000
REG_U32_ID = 0x0000
# =======================


def add_motor(ctrl: Controller):
    if VENDOR == "damiao":
        return ctrl.add_damiao_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "myactuator":
        return ctrl.add_myactuator_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "robstride":
        return ctrl.add_robstride_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "hightorque":
        return ctrl.add_hightorque_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    if VENDOR == "hexfellow":
        return ctrl.add_hexfellow_motor(MOTOR_ID, FEEDBACK_ID, MODEL)
    raise ValueError(f"unsupported vendor: {VENDOR}")


def main() -> int:
    with Controller(CHANNEL) as ctrl:
        motor = add_motor(ctrl)
        motor.set_can_timeout_ms(TIMEOUT_MS)

        # Read examples (will fail if reg id is unsupported).
        try:
            v_f32 = motor.get_register_f32(REG_F32_ID, TIMEOUT_MS)
            print(f"f32 reg 0x{REG_F32_ID:04X} = {v_f32}")
        except Exception as e:  # noqa: BLE001
            print("get_register_f32 skipped/failed:", e)

        try:
            v_u32 = motor.get_register_u32(REG_U32_ID, TIMEOUT_MS)
            print(f"u32 reg 0x{REG_U32_ID:04X} = {v_u32}")
        except Exception as e:  # noqa: BLE001
            print("get_register_u32 skipped/failed:", e)

        # Write examples: uncomment only when register map is confirmed.
        # motor.write_register_f32(REG_F32_ID, 1.0)
        # motor.write_register_u32(REG_U32_ID, 1)
        # motor.store_parameters()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
