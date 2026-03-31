#!/usr/bin/env python3
"""09: multi-motor one-shot query lesson (config-driven).

This script is designed for factory-style scaling:
- add/remove Damiao motors by editing one list
- add/remove MyActuator motors by editing one list
- add/remove RobStride motors by editing one list
"""

from __future__ import annotations

import time

from motorbridge import Controller

CHANNEL = "can0"

# ----------------------------------------------------------------------
# 1) Damiao motor table (same controller/vendor family)
# Add or remove rows freely.
# name: output label in terminal
# id/fid/model: motor-id, feedback-id, model string
# ----------------------------------------------------------------------
DAMIAO_MOTORS = [
    {"name": "dm1", "id": 0x04, "fid": 0x14, "model": "4340P"},
    {"name": "dm2", "id": 0x07, "fid": 0x17, "model": "4310"},
]

# ----------------------------------------------------------------------
# 2) Optional MyActuator motor table (separate controller/vendor family)
# ----------------------------------------------------------------------
USE_MYACTUATOR = False
MYACTUATOR_MOTORS = [
    {"name": "my1", "id": 1, "fid": 0x241, "model": "X8"},
]

# ----------------------------------------------------------------------
# 3) Optional RobStride motor table (separate controller/vendor family)
# RobStride common setting in field:
#   id=127, feedback/responder=0xFE, model=rs-00
# ----------------------------------------------------------------------
USE_ROBSTRIDE = True
ROBSTRIDE_MOTORS = [
    {"name": "rs1", "id": 127, "fid": 0xFE, "model": "rs-00"},
]

# Query behavior switch:
# - True: retry request+poll until state is ready (recommended)
# - False: strict one-shot query
RETRY_ENABLED = True
MAX_RETRIES = 12
RETRY_DT_MS = 50


def query_states_batch_once(ctrl: Controller, motors):
    for m in motors:
        m.request_feedback()
    ctrl.poll_feedback_once()
    return [m.get_state() for m in motors]


def query_states_with_retry(ctrl: Controller, motors):
    if not RETRY_ENABLED:
        return query_states_batch_once(ctrl, motors)

    states = [None for _ in motors]
    for _ in range(MAX_RETRIES):
        states = query_states_batch_once(ctrl, motors)
        if all(s is not None for s in states):
            return states
        time.sleep(max(RETRY_DT_MS, 1) / 1000.0)
    return states


def attach_damiao(ctrl: Controller, cfgs):
    names = []
    motors = []
    for cfg in cfgs:
        names.append(str(cfg["name"]))
        motors.append(ctrl.add_damiao_motor(int(cfg["id"]), int(cfg["fid"]), str(cfg["model"])))
    return names, motors


def attach_myactuator(ctrl: Controller, cfgs):
    names = []
    motors = []
    for cfg in cfgs:
        names.append(str(cfg["name"]))
        motors.append(ctrl.add_myactuator_motor(int(cfg["id"]), int(cfg["fid"]), str(cfg["model"])))
    return names, motors


def attach_robstride(ctrl: Controller, cfgs):
    names = []
    motors = []
    for cfg in cfgs:
        names.append(str(cfg["name"]))
        motors.append(ctrl.add_robstride_motor(int(cfg["id"]), int(cfg["fid"]), str(cfg["model"])))
    return names, motors


def append_states(out: list[str], names, states):
    for name, state in zip(names, states):
        out.append(f"{name}={state}")


def main() -> int:
    with Controller(CHANNEL) as dm_ctrl:
        dm_names, dm_motors = attach_damiao(dm_ctrl, DAMIAO_MOTORS)
        dm_ctrl.enable_all()
        dm_states = query_states_with_retry(dm_ctrl, dm_motors)
        out: list[str] = []
        append_states(out, dm_names, dm_states)

        if USE_MYACTUATOR:
            with Controller(CHANNEL) as my_ctrl:
                my_names, my_motors = attach_myactuator(my_ctrl, MYACTUATOR_MOTORS)
                my_ctrl.enable_all()
                my_states = query_states_with_retry(my_ctrl, my_motors)
                append_states(out, my_names, my_states)

        if USE_ROBSTRIDE:
            with Controller(CHANNEL) as rs_ctrl:
                rs_names, rs_motors = attach_robstride(rs_ctrl, ROBSTRIDE_MOTORS)
                rs_ctrl.enable_all()
                rs_states = query_states_with_retry(rs_ctrl, rs_motors)
                append_states(out, rs_names, rs_states)

        print(" ".join(out))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
