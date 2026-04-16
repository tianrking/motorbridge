#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PY_BINDING_DIR="${ROOT_DIR}/bindings/python"

case "$(uname -s)" in
  Linux*)
    ABI_LIB="${ROOT_DIR}/target/release/libmotor_abi.so"
    GW_BIN="${ROOT_DIR}/target/release/ws_gateway"
    ;;
  Darwin*)
    ABI_LIB="${ROOT_DIR}/target/release/libmotor_abi.dylib"
    GW_BIN="${ROOT_DIR}/target/release/ws_gateway"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    ABI_LIB="${ROOT_DIR}/target/release/motor_abi.dll"
    GW_BIN="${ROOT_DIR}/target/release/ws_gateway.exe"
    ;;
  *)
    echo "Unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

echo "[1/6] Build Rust release artifacts (motor_abi + ws_gateway)"
cargo build -p motor_abi --release --manifest-path "${ROOT_DIR}/Cargo.toml"
cargo build -p ws_gateway --release --manifest-path "${ROOT_DIR}/Cargo.toml"

if [[ ! -f "${ABI_LIB}" ]]; then
  echo "ABI library not found: ${ABI_LIB}" >&2
  exit 1
fi
if [[ ! -f "${GW_BIN}" ]]; then
  echo "Gateway binary not found: ${GW_BIN}" >&2
  exit 1
fi

echo "[2/6] Build Python sdist + wheel with explicit artifact env"
cd "${PY_BINDING_DIR}"
rm -rf dist build src/motorbridge.egg-info
MOTORBRIDGE_LIB="${ABI_LIB}" \
MOTORBRIDGE_WS_GATEWAY_BIN="${GW_BIN}" \
python3 -m build

echo "[3/6] Validate built packages"
TWINE_DIR="$(mktemp -d /tmp/motorbridge-twine-check-XXXXXX)"
python3 -m venv "${TWINE_DIR}/venv"
"${TWINE_DIR}/venv/bin/python" -m pip install --upgrade pip twine >/dev/null
"${TWINE_DIR}/venv/bin/python" -m twine check dist/*
rm -rf "${TWINE_DIR}"

echo "[4/6] Create isolated smoke-test venv"
SMOKE_DIR="$(mktemp -d /tmp/motorbridge-release-smoke-XXXXXX)"
trap 'rm -rf "${SMOKE_DIR}"' EXIT
python3 -m venv "${SMOKE_DIR}/venv"
source "${SMOKE_DIR}/venv/bin/activate"
python -m pip install --upgrade pip >/dev/null

echo "[5/6] Install wheel and smoke test import + gateway launcher"
WHEEL_PATH="$(ls -1 dist/motorbridge-*.whl | head -n 1)"
python -m pip install "${WHEEL_PATH}" >/dev/null
python - <<'PY'
import motorbridge
from motorbridge import Controller, Mode
print("import_ok", motorbridge.__file__)
print("controller_ok", Controller is not None, Mode is not None)
PY
motorbridge-gateway --help >/dev/null

echo "[6/6] Done"
echo "Artifacts:"
ls -1 dist
echo
echo "Next:"
echo "  python3 -m twine upload -r testpypi dist/*"
echo "  python3 -m twine upload dist/*"
