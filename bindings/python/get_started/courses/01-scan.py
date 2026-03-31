#!/usr/bin/env python3
"""01: scan workflow in pure Python path.

No subprocess; uses motorbridge internal scan entry.
"""

from __future__ import annotations

from argparse import Namespace

from motorbridge.cli import _scan_command

# ===== USER CONFIG =====
TRANSPORT = "auto"  # auto/socketcan/dm-serial
CHANNEL = "can0"
VENDOR = "all"  # all/damiao/myactuator/robstride/hightorque/hexfellow
START_ID = "1"
END_ID = "255"
SERIAL_PORT = "/dev/ttyACM0"
SERIAL_BAUD = 921600
# RobStride tip:
# - keep VENDOR="robstride" for focused scan
# - common device id seen in field: 127
# =======================


def main() -> int:
    if TRANSPORT == "dm-serial" and VENDOR != "damiao":
        raise ValueError("dm-serial only supports damiao")

    args = Namespace(
        vendor=VENDOR,
        channel=CHANNEL,
        transport=TRANSPORT,
        serial_port=SERIAL_PORT,
        serial_baud=SERIAL_BAUD,
        model="4340P",
        start_id=START_ID,
        end_id=END_ID,
        feedback_ids="0xFF,0xFE,0x11,0x17,0x10",
        feedback_base="0x10",
        timeout_ms=80,
        param_id="0x7019",
        param_timeout_ms=120,
    )
    _scan_command(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
