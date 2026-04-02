# factory_calib_ui_ws (React)

WebSocket-based factory upper-computer UI (pure frontend).

- Frontend stack: React + Vite
- Backend control link: `integrations/ws_gateway`
- Browser only talks to WS API (`set_target` / `scan` / `set_id` / `verify` / `enable` / `disable` / `mit` / `pos_vel` / `vel` / `force_pos` / `stop`)

## Features

- Per-vendor scanning (`damiao`, `robstride`, `myactuator`, `hightorque`, `hexfellow`)
- One-click scan buttons for Damiao/RobStride
- Auto-append discovered motors into list
- Per-motor operations:
  - Enable / Disable / Move / Stop
  - Mode switch + target parameters
  - Set ID + Verify
- Damiao scan parameter visualization (`pmax/vmax/tmax`, `detected_by`, `model_guess`)

## 1) Start ws_gateway

From repo root:

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm
cargo run -p ws_gateway --release -- \
  --bind 0.0.0.0:9002 \
  --vendor damiao --channel can0 --model auto --motor-id 0x01 --feedback-id 0x11 --dt-ms 20
```

## 2) Frontend development mode

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm/tools/factory_calib_ui_ws
npm install
npm run dev
```

Open: `http://127.0.0.1:18110`

If you hit `vite: not found`, run one command:

```bash
npm run dev:ready
```

## 3) Build + local preview

```bash
cd /home/w0x7ce/Downloads/dm_candrive/rust_dm/tools/factory_calib_ui_ws
npm install
npm run build
npm run preview
```

Open: `http://127.0.0.1:18110/`

## Notes

- Linux SocketCAN channel format: `can0` (not `can0@1000000`).
- This UI is designed for factory/operator workflows and future extensibility (slider/trajectory pages can be added on top of current structure).
