# ws_gateway (Scaffold Only)

Status: scaffold only, not implemented yet.

## Goal

Provide a binary WebSocket gateway over `motorbridge` so remote clients can send control/config requests and receive state streams.

## Planned capabilities

- Session lifecycle + channel/model/motor binding
- Control commands: enable/disable/mit/pos-vel/vel/force-pos
- Config commands: scan/set-id/verify/read-register/write-register
- State/event streaming and error reporting

## Proposed protocol

- Transport: WebSocket
- Payload: binary frame (versioned)
- Optional debug mode: JSON mirror frame for inspection

## Suggested future layout

- `src/main.rs`
- `src/protocol.rs`
- `src/service.rs`
- `src/session.rs`
- `src/state_stream.rs`

## Next step

Implement minimal server with:

1. one motor session
2. command opcode parser
3. periodic state push
