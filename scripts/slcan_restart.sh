#!/usr/bin/env bash
set -euo pipefail

# Restart SLCAN and bring up SocketCAN interface
# Usage:
#   slcan_restart.sh [/dev/ttyACM0] [can0] [1000000]

TTY_DEV="${1:-/dev/ttyACM0}"
CAN_IF="${2:-can0}"
BITRATE="${3:-1000000}"

if ! command -v slcand >/dev/null 2>&1; then
  echo "Error: slcand not found. Install can-utils first." >&2
  exit 1
fi

if ! command -v ip >/dev/null 2>&1; then
  echo "Error: ip command not found." >&2
  exit 1
fi

if [[ ! -e "$TTY_DEV" ]]; then
  echo "Error: tty device not found: $TTY_DEV" >&2
  exit 1
fi

case "$BITRATE" in
  10000) S_OPT="0" ;;
  20000) S_OPT="1" ;;
  50000) S_OPT="2" ;;
  100000) S_OPT="3" ;;
  125000) S_OPT="4" ;;
  250000) S_OPT="5" ;;
  500000) S_OPT="6" ;;
  750000) S_OPT="7" ;;
  1000000) S_OPT="8" ;;
  *)
    echo "Error: unsupported bitrate '$BITRATE'. Use one of: 10000 20000 50000 100000 125000 250000 500000 750000 1000000" >&2
    exit 1
    ;;
esac

echo "[slcan_restart] device=$TTY_DEV iface=$CAN_IF bitrate=$BITRATE"

sudo pkill slcand 2>/dev/null || true
sudo ip link set slcan0 down 2>/dev/null || true
sudo ip link set "$CAN_IF" down 2>/dev/null || true

start_err=""
if ! start_err="$(sudo slcand -o -c -f -s"$S_OPT" "$TTY_DEV" "$CAN_IF" 2>&1)"; then
  echo "[slcan_restart] first start failed, retrying once..." >&2
  sudo pkill slcand 2>/dev/null || true
  sudo ip link set "$CAN_IF" down 2>/dev/null || true
  sleep 0.2
  if ! start_err="$(sudo slcand -o -c -f -s"$S_OPT" "$TTY_DEV" "$CAN_IF" 2>&1)"; then
    echo "Error: failed to start slcand: $start_err" >&2
    exit 1
  fi
fi

sudo ip link set "$CAN_IF" up

echo "[slcan_restart] done"
ip -details link show "$CAN_IF"
