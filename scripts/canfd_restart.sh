#!/usr/bin/env bash
set -euo pipefail

# Restart one or more SocketCAN interfaces in CAN-FD mode.
# Usage:
#   scripts/canfd_restart.sh
#   scripts/canfd_restart.sh can0 can1
#   scripts/canfd_restart.sh --bitrate 1000000 --dbitrate 5000000 can0

BITRATE=1000000
DBITRATE=5000000
RESTART_MS=100
LOOPBACK=off
FD=on
CAN_SP=""
DATA_SP=""
SJW=""
DSJW=""
IFS_LIST=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bitrate)
      BITRATE="$2"
      shift 2
      ;;
    --dbitrate)
      DBITRATE="$2"
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
    --fd)
      FD="$2"
      shift 2
      ;;
    --sample-point)
      CAN_SP="$2"
      shift 2
      ;;
    --dsample-point)
      DATA_SP="$2"
      shift 2
      ;;
    --sjw)
      SJW="$2"
      shift 2
      ;;
    --dsjw)
      DSJW="$2"
      shift 2
      ;;
    -h|--help)
      cat <<'EOF'
Restart SocketCAN interfaces (CAN-FD capable mode).

Options:
  --bitrate <num>        arbitration bitrate (default: 1000000)
  --dbitrate <num>       data bitrate (default: 5000000)
  --restart-ms <num>     bus-off auto-restart (default: 100)
  --loopback <on|off>    loopback mode (default: off)
  --fd <on|off>          CAN-FD on/off (default: on)
  --sample-point <v>     arbitration sample-point (e.g. 0.8)
  --dsample-point <v>    data sample-point (e.g. 0.75)
  --sjw <num>            arbitration SJW
  --dsjw <num>           data SJW
  -h, --help             show help

Examples:
  scripts/canfd_restart.sh
  scripts/canfd_restart.sh can0
  scripts/canfd_restart.sh --bitrate 1000000 --dbitrate 5000000 --sample-point 0.8 --dsample-point 0.75 --sjw 5 --dsjw 3 can0
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
    echo "[canfd_restart] skip ${ifn}: interface not found"
    return 0
  fi

  local -a cmd=(
    sudo ip link set "$ifn" type can
    bitrate "$BITRATE"
    loopback "$LOOPBACK"
    fd "$FD"
  )
  local -a cmd_no_restart=(
    sudo ip link set "$ifn" type can
    bitrate "$BITRATE"
    loopback "$LOOPBACK"
    fd "$FD"
  )

  if [[ "$FD" == "on" ]]; then
    cmd+=(dbitrate "$DBITRATE")
    cmd_no_restart+=(dbitrate "$DBITRATE")
  fi
  if [[ -n "${RESTART_MS}" ]]; then
    cmd+=(restart-ms "$RESTART_MS")
  fi
  if [[ -n "$CAN_SP" ]]; then
    cmd+=(sample-point "$CAN_SP")
    cmd_no_restart+=(sample-point "$CAN_SP")
  fi
  if [[ -n "$DATA_SP" ]]; then
    cmd+=(dsample-point "$DATA_SP")
    cmd_no_restart+=(dsample-point "$DATA_SP")
  fi
  if [[ -n "$SJW" ]]; then
    cmd+=(sjw "$SJW")
    cmd_no_restart+=(sjw "$SJW")
  fi
  if [[ -n "$DSJW" ]]; then
    cmd+=(dsjw "$DSJW")
    cmd_no_restart+=(dsjw "$DSJW")
  fi

  echo "[canfd_restart] restarting ${ifn} bitrate=${BITRATE} dbitrate=${DBITRATE} fd=${FD} restart-ms=${RESTART_MS} loopback=${LOOPBACK}"
  sudo ip link set "$ifn" down 2>/dev/null || true
  echo "[canfd_restart] cmd: ${cmd[*]}"
  local cmd_err=""
  if ! cmd_err="$("${cmd[@]}" 2>&1)"; then
    echo "$cmd_err" >&2
    if grep -qi "doesn't support restart from Bus Off" <<<"$cmd_err"; then
      echo "[canfd_restart] retry without restart-ms ..." >&2
      echo "[canfd_restart] cmd: ${cmd_no_restart[*]}"
      "${cmd_no_restart[@]}"
    else
      return 1
    fi
  fi
  sudo ip link set "$ifn" up
  local details
  details="$(ip -details link show "$ifn")"
  echo "$details"

  if [[ "$FD" == "on" ]]; then
    if ! grep -Eq "fd on|<FD>" <<<"$details"; then
      echo "[canfd_restart] error: ${ifn} is not in CAN-FD mode (missing 'fd on')" >&2
      return 1
    fi
  fi
}

for ifn in "${IFS_LIST[@]}"; do
  restart_one "$ifn"
done
