#!/usr/bin/env bash
set -euo pipefail

# Restart one or more SocketCAN interfaces.
# Usage:
#   scripts/can_restart.sh
#   scripts/can_restart.sh can0 can1
#   scripts/can_restart.sh --bitrate 1000000 --restart-ms 100 can0 can1

BITRATE=1000000
RESTART_MS=100
LOOPBACK=off
IFS_LIST=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bitrate)
      BITRATE="$2"
      shift 2
      ;;
    --restart-ms)
      RESTART_MS="$2"
      shift 2
      ;;
    --loopback)
      LOOPBACK="$2"
      shift 2
      ;;
    -h|--help)
      cat <<'EOF'
Restart SocketCAN interfaces.

Options:
  --bitrate <num>     CAN bitrate (default: 1000000)
  --restart-ms <num>  bus-off auto-restart (default: 100)
  --loopback <on|off> loopback mode (default: off)
  -h, --help          show help

Examples:
  scripts/can_restart.sh
  scripts/can_restart.sh can0 can1
  scripts/can_restart.sh --bitrate 500000 can0 can1
EOF
      exit 0
      ;;
    *)
      IFS_LIST+=("$1")
      shift
      ;;
  esac
done

if [[ ${#IFS_LIST[@]} -eq 0 ]]; then
  IFS_LIST=(can0 can1)
fi

restart_one() {
  local ifn="$1"
  if ! ip link show "$ifn" >/dev/null 2>&1; then
    echo "[can_restart] skip ${ifn}: interface not found"
    return 0
  fi

  echo "[can_restart] restarting ${ifn} bitrate=${BITRATE} restart-ms=${RESTART_MS} loopback=${LOOPBACK}"
  sudo ip link set "$ifn" down 2>/dev/null || true
  sudo ip link set "$ifn" type can bitrate "$BITRATE" restart-ms "$RESTART_MS" loopback "$LOOPBACK"
  sudo ip link set "$ifn" up
  ip -details link show "$ifn"
}

for ifn in "${IFS_LIST[@]}"; do
  restart_one "$ifn"
done

