#!/usr/bin/env bash
set -euo pipefail

ROOT="/home/w0x7ce/Downloads/dm_candrive/rust_dm"
WS_URL="${1:-ws://127.0.0.1:9002}"
CHANNEL="${2:-can0}"
START_ID="${3:-1}"
END_ID="${4:-16}"
TIMEOUT_MS="${5:-500}"
FEEDBACK_BASE="${6:-16}"

cd "${ROOT}"

echo "[1/2] motor_cli scan"
CLI_OUT=$(target/release/motor_cli --vendor damiao --channel "${CHANNEL}" --mode scan --start-id "${START_ID}" --end-id "${END_ID}" --feedback-id 0x11 2>&1 || true)
echo "${CLI_OUT}"
CLI_HITS=$(echo "${CLI_OUT}" | sed -n 's/.*hits=\([0-9]\+\).*/\1/p' | tail -n1)
CLI_HITS=${CLI_HITS:-NA}

echo
echo "[2/2] ws scan"
WS_OUT=$({
  printf '%s\n' "{\"op\":\"set_target\",\"vendor\":\"damiao\",\"channel\":\"${CHANNEL}\",\"model\":\"4310\",\"motor_id\":1,\"feedback_id\":17}"
  printf '%s\n' "{\"op\":\"scan\",\"vendor\":\"damiao\",\"start_id\":${START_ID},\"end_id\":${END_ID},\"feedback_base\":${FEEDBACK_BASE},\"timeout_ms\":${TIMEOUT_MS}}"
  sleep 2
} | websocat "${WS_URL}" 2>&1 || true)
echo "${WS_OUT}"
WS_HITS=$(echo "${WS_OUT}" | sed -n 's/.*"count":\([0-9]\+\).*/\1/p' | tail -n1)
WS_HITS=${WS_HITS:-NA}

echo
echo "[result] cli_hits=${CLI_HITS} ws_hits=${WS_HITS}"
if [[ "${CLI_HITS}" != "NA" && "${WS_HITS}" != "NA" && "${CLI_HITS}" == "${WS_HITS}" ]]; then
  echo "[result] CONSISTENT"
else
  echo "[result] MISMATCH or INCONCLUSIVE"
fi
