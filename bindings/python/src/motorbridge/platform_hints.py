from __future__ import annotations

import ctypes
import sys
from pathlib import Path


_CAN_TRANSPORTS = {"auto", "socketcan", "socketcanfd"}


def parse_transport_arg(argv: list[str], default: str = "auto") -> str:
    transport = default
    i = 0
    while i < len(argv):
        tok = argv[i]
        if tok == "--transport":
            if i + 1 < len(argv):
                transport = argv[i + 1]
                i += 2
                continue
            break
        if tok.startswith("--transport="):
            transport = tok.split("=", 1)[1]
            i += 1
            continue
        i += 1
    return str(transport or default).strip().lower()


def transport_needs_can_runtime(transport: str) -> bool:
    return str(transport or "auto").strip().lower() in _CAN_TRANSPORTS


def is_macos() -> bool:
    return sys.platform == "darwin"


def is_windows() -> bool:
    return sys.platform.startswith("win")


def is_linux() -> bool:
    return sys.platform.startswith("linux")


def parse_channel_arg(argv: list[str], default: str = "can0") -> str:
    channel = default
    i = 0
    while i < len(argv):
        tok = argv[i]
        if tok == "--channel":
            if i + 1 < len(argv):
                channel = argv[i + 1]
                i += 2
                continue
            break
        if tok.startswith("--channel="):
            channel = tok.split("=", 1)[1]
            i += 1
            continue
        i += 1
    return str(channel or default).strip()


def can_load_pcbusb() -> bool:
    if not is_macos():
        return True

    candidate_names = [
        "libPCBUSB.dylib",
        "/usr/local/lib/libPCBUSB.dylib",
        "/opt/homebrew/lib/libPCBUSB.dylib",
        str(Path.home() / ".local/lib/libPCBUSB.dylib"),
    ]
    for name in candidate_names:
        try:
            ctypes.CDLL(name)
            return True
        except OSError:
            continue
    return False


def macos_pcbusb_hint(tool_name: str) -> str:
    return (
        f"[{tool_name}] macOS CAN runtime not found: libPCBUSB.dylib\n"
        "CAN transport on macOS requires MacCAN PCBUSB runtime.\n"
        "Install PCBUSB first, then retry.\n"
        "Quick checks:\n"
        "  ls /usr/local/lib/libPCBUSB.dylib\n"
        "  ls ~/.local/lib/libPCBUSB.dylib\n"
        "Reference: motorbridge README.zh-CN.md -> \"macOS PCAN 运行时（PCBUSB）\""
    )


def can_load_pcanbasic_windows() -> bool:
    if not is_windows():
        return True
    try:
        ctypes.CDLL("PCANBasic.dll")
        return True
    except OSError:
        return False


def windows_pcan_hint(tool_name: str) -> str:
    return (
        f"[{tool_name}] Windows CAN runtime not found: PCANBasic.dll\n"
        "CAN transport on Windows requires PEAK PCAN driver + PCAN-Basic runtime.\n"
        "Install PEAK driver/PCAN-Basic, then reopen terminal and retry.\n"
        "Channel examples:\n"
        "  can0@1000000  (maps to PCAN_USBBUS1)\n"
        "  can1@1000000  (maps to PCAN_USBBUS2)"
    )


def _linux_iface_state(iface: str) -> tuple[bool, str]:
    base = Path("/sys/class/net") / iface
    if not base.exists():
        return False, "missing"
    state_file = base / "operstate"
    try:
        state = state_file.read_text(encoding="utf-8").strip().lower()
    except OSError:
        return True, "unknown"
    return True, state


def linux_socketcan_hint(tool_name: str, channel: str) -> str:
    raw = str(channel or "can0").strip()
    iface = raw.split("@", 1)[0].strip() or "can0"
    suffix_tip = ""
    if "@" in raw:
        suffix_tip = (
            f"\nDetected channel '{raw}'. Linux SocketCAN should not include '@bitrate'. "
            f"Use plain interface name like '{iface}'."
        )

    return (
        f"[{tool_name}] Linux CAN interface not ready: {raw}\n"
        "For CAN transport, bring up SocketCAN/slcan first, then retry.\n"
        "Quick checks:\n"
        f"  ip link show {iface}\n"
        f"  sudo ip link set {iface} up\n"
        "If using USB serial CAN adapter (slcan), create interface first:\n"
        "  sudo slcand -o -c -s8 /dev/ttyUSB0 slcan0\n"
        "  sudo ip link set slcan0 up"
        f"{suffix_tip}"
    )


def preflight_can_runtime(tool_name: str, transport: str, channel: str) -> str | None:
    if not transport_needs_can_runtime(transport):
        return None

    if is_macos():
        if not can_load_pcbusb():
            return macos_pcbusb_hint(tool_name)
        return None

    if is_windows():
        if not can_load_pcanbasic_windows():
            return windows_pcan_hint(tool_name)
        return None

    if is_linux():
        iface = str(channel or "can0").strip().split("@", 1)[0].strip() or "can0"
        exists, state = _linux_iface_state(iface)
        if (not exists) or state in {"down", "dormant", "lowerlayerdown", "notpresent"}:
            return linux_socketcan_hint(tool_name, channel)
        return None

    return None
