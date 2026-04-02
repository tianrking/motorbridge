#!/usr/bin/env bash
set -euo pipefail

WS_URL="${1:-ws://127.0.0.1:9002}"
CHANNEL="${2:-can0}"
START_ID="${3:-1}"
END_ID="${4:-16}"
TIMEOUT_MS="${5:-500}"

if ! command -v websocat >/dev/null 2>&1; then
  echo "[ERR] websocat not found. install: cargo install websocat" >&2
  exit 1
fi

run_ws() {
  local name="$1"
  local payload="$2"
  echo ""
  echo "===== ${name} ====="
  { printf '%s\n' "${payload}"; sleep 1; } | websocat "${WS_URL}" | sed -n '1,8p'
}

run_ws_multi() {
  local name="$1"
  shift
  echo ""
  echo "===== ${name} ====="
  {
    for msg in "$@"; do
      printf '%s\n' "${msg}"
    done
    sleep 2
  } | websocat "${WS_URL}" | sed -n '1,20p'
}

echo "[info] ws_url=${WS_URL}, channel=${CHANNEL}, range=${START_ID}..${END_ID}, timeout_ms=${TIMEOUT_MS}"

run_ws "ping" '{"op":"ping"}'

for model in 4310 4340P 4340; do
  run_ws_multi "damiao scan model=${model}" \
    "{\"op\":\"set_target\",\"vendor\":\"damiao\",\"channel\":\"${CHANNEL}\",\"model\":\"${model}\",\"motor_id\":1,\"feedback_id\":17}" \
    "{\"op\":\"scan\",\"vendor\":\"damiao\",\"start_id\":${START_ID},\"end_id\":${END_ID},\"feedback_base\":16,\"timeout_ms\":${TIMEOUT_MS}}"
done

run_ws_multi "damiao verify sample" \
  '{"op":"verify","vendor":"damiao","motor_id":5,"feedback_id":21,"timeout_ms":1000}' \
  '{"op":"verify","vendor":"damiao","motor_id":7,"feedback_id":23,"timeout_ms":1000}'

echo ""
echo "[done] smoke test finished"
