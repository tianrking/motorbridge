# MotorBridge Assistant (factory_calib_ui_ws, React)

WebSocket-based factory control UI (frontend only).

- Frontend: React + Vite
- Backend: `ws_gateway`
- Browser calls WS operations: `set_target` / `scan` / `set_id` / `verify` / `enable` / `disable` / `mit` / `pos_vel` / `vel` / `force_pos` / `stop`

## 1) Install Prerequisites

```bash
pip install -U motorbridge
cd tools/factory_calib_ui_ws
npm install
```

## 2) Choose Transport and Start `ws_gateway`

Default WS bind in all examples:

```bash
--bind 127.0.0.1:9002
```

Security note:

- Keep `127.0.0.1:9002` for local setup.
- If you must expose `ws_gateway` on LAN/WAN (`0.0.0.0` or a host IP), set `MOTORBRIDGE_WS_TOKEN` before starting gateway.
- The UI client must then send token header (`x-motorbridge-token` or `Authorization: Bearer ...`) during WS handshake.

### A. Damiao over `dm-serial`

Use this when the adapter is Damiao serial bridge.

Ubuntu:

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport dm-serial \
  --serial-port /dev/ttyACM0 --serial-baud 921600 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

macOS:

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport dm-serial \
  --serial-port /dev/cu.usbmodemXXXX --serial-baud 921600 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

If macOS reports dynamic-library loading errors, use package-local fallback:

```bash
GW="$(python3 -c "import motorbridge, pathlib; print(pathlib.Path(motorbridge.__file__).resolve().parent/'bin'/'ws_gateway')")"
PKG_DIR="$(python3 -c "import motorbridge, pathlib; print(pathlib.Path(motorbridge.__file__).resolve().parent)")"
DYLD_LIBRARY_PATH="$PKG_DIR/lib:${DYLD_LIBRARY_PATH:-}" "$GW" \
  --bind 127.0.0.1:9002 --vendor damiao --transport dm-serial \
  --serial-port /dev/cu.usbmodemXXXX --serial-baud 921600 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

Windows (PowerShell):

```powershell
motorbridge-gateway -- `
  --bind 127.0.0.1:9002 `
  --vendor damiao --transport dm-serial `
  --serial-port COM3 --serial-baud 921600 `
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

### B. PCAN / Standard CAN path (`socketcan`/`pcan`)

Use this when the adapter is PCAN or other standard CAN interface.

Ubuntu (SocketCAN):

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport auto \
  --channel can0 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

macOS (PCBUSB runtime):

```bash
motorbridge-gateway -- \
  --bind 127.0.0.1:9002 \
  --vendor damiao --transport auto \
  --channel can0 \
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

Windows (PCAN backend):

```powershell
motorbridge-gateway -- `
  --bind 127.0.0.1:9002 `
  --vendor damiao --transport auto `
  --channel can0@1000000 `
  --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

## 3) Start Frontend

```bash
cd tools/factory_calib_ui_ws
npm run dev
```

Open: `http://127.0.0.1:18110`

In UI connection panel:

- WS URL: `ws://127.0.0.1:9002`
- Channel: same as gateway channel (for example `can0`)

## 4) Verify Quickly

- General page: run `Scan Damiao`.
- Robot Arm page: select profile, click `Prepare 7 Cards`, then `Scan All Joints`.

## 5) Notes

- Linux SocketCAN channel should be `can0` (do not use `can0@1000000`).
- `dm-serial` is Damiao-only transport.
- `ws disconnected` in logs is often browser reconnect behavior; check whether gateway is still listening.

For detailed CAN troubleshooting, see:

- `docs/en/can_debugging.md`
