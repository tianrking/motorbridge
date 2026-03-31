#!/usr/bin/env python3
"""Quickstart 01 (super simple): scan motors in pure Python.

This file is intentionally explicit and beginner-friendly.
Only edit the CONFIG area below.
"""

from __future__ import annotations

from argparse import Namespace

from motorbridge.cli import _scan_command

# ============================================================
# CONFIG (edit these lines)
# ============================================================
# Transport:
# - "auto": default, use normal CAN path
# - "socketcan": force normal CAN path
# - "dm-serial": Damiao serial bridge path (Damiao only)
TRANSPORT = "auto"

# Channel:
# - Linux SocketCAN/slcan: "can0", "can1", "slcan0"
# - Windows PCAN: "can0@1000000", "can1@1000000"
CHANNEL = "can0"

# Vendor:
# damiao / myactuator / robstride / hightorque / hexfellow / all
VENDOR = "all"

# ID range to probe
START_ID = "1"
END_ID = "255"

# dm-serial only (used only when TRANSPORT == "dm-serial")
SERIAL_PORT = "/dev/ttyACM0"
SERIAL_BAUD = 921600

# Timeouts (ms)
TIMEOUT_MS = 80
PARAM_TIMEOUT_MS = 120
# ============================================================


def main() -> int:
    if TRANSPORT == "dm-serial" and VENDOR != "damiao":
        raise ValueError("dm-serial only supports vendor='damiao'")

    args = Namespace(
        vendor=VENDOR,
        channel=CHANNEL,
        transport=TRANSPORT,
        serial_port=SERIAL_PORT,
        serial_baud=SERIAL_BAUD,
        model="4340",
        start_id=START_ID,
        end_id=END_ID,
        feedback_ids="0xFF,0xFE,0x00,0xAA",
        feedback_base="0x10",
        timeout_ms=TIMEOUT_MS,
        param_id="0x7019",
        param_timeout_ms=PARAM_TIMEOUT_MS,
    )
    print(
        "[scan] "
        f"transport={TRANSPORT} channel={CHANNEL} vendor={VENDOR} "
        f"id_range={START_ID}..{END_ID}"
    )
    _scan_command(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
