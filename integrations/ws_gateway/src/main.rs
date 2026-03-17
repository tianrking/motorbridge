use futures_util::{SinkExt, StreamExt};
use motor_vendor_damiao::{ControlMode, DamiaoController, DamiaoMotor};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

#[derive(Clone, Debug)]
struct Target {
    channel: String,
    model: String,
    motor_id: u16,
    feedback_id: u16,
}

#[derive(Clone, Debug)]
struct ServerConfig {
    bind: String,
    target: Target,
    dt_ms: u64,
}

#[derive(Clone, Debug)]
enum ActiveCommand {
    Mit {
        pos: f32,
        vel: f32,
        kp: f32,
        kd: f32,
        tau: f32,
    },
    PosVel { pos: f32, vlim: f32 },
    Vel { vel: f32 },
    ForcePos { pos: f32, vlim: f32, ratio: f32 },
}

struct SessionCtx {
    target: Target,
    controller: Option<DamiaoController>,
    motor: Option<Arc<DamiaoMotor>>,
    active: Option<ActiveCommand>,
}

impl SessionCtx {
    fn new(target: Target) -> Self {
        Self {
            target,
            controller: None,
            motor: None,
            active: None,
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        self.disconnect(false);
        let ctrl = DamiaoController::new_socketcan(&self.target.channel)
            .map_err(|e| format!("open bus failed: {e}"))?;
        let motor = ctrl
            .add_motor(
                self.target.motor_id,
                self.target.feedback_id,
                &self.target.model,
            )
            .map_err(|e| format!("add motor failed: {e}"))?;
        self.controller = Some(ctrl);
        self.motor = Some(motor);
        Ok(())
    }

    fn ensure_connected(&mut self) -> Result<(), String> {
        if self.controller.is_none() || self.motor.is_none() {
            self.connect()?;
        }
        Ok(())
    }

    fn disconnect(&mut self, shutdown: bool) {
        self.active = None;
        self.motor = None;
        if let Some(ctrl) = self.controller.take() {
            if shutdown {
                let _ = ctrl.shutdown();
            } else {
                let _ = ctrl.close_bus();
            }
        }
    }

    fn apply_active(&self) -> Result<(), String> {
        let motor = self
            .motor
            .as_ref()
            .ok_or_else(|| "motor not connected".to_string())?;
        match self.active.as_ref() {
            Some(ActiveCommand::Mit {
                pos,
                vel,
                kp,
                kd,
                tau,
            }) => motor
                .send_cmd_mit(*pos, *vel, *kp, *kd, *tau)
                .map_err(|e| e.to_string()),
            Some(ActiveCommand::PosVel { pos, vlim }) => motor
                .send_cmd_pos_vel(*pos, *vlim)
                .map_err(|e| e.to_string()),
            Some(ActiveCommand::Vel { vel }) => {
                motor.send_cmd_vel(*vel).map_err(|e| e.to_string())
            }
            Some(ActiveCommand::ForcePos { pos, vlim, ratio }) => motor
                .send_cmd_force_pos(*pos, *vlim, *ratio)
                .map_err(|e| e.to_string()),
            None => Ok(()),
        }
    }
}

fn parse_hex_or_dec(s: &str) -> Result<u16, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s}: {e}"))
    } else {
        s.parse::<u16>()
            .map_err(|e| format!("invalid integer {s}: {e}"))
    }
}

fn parse_args() -> Result<ServerConfig, String> {
    let mut bind = "0.0.0.0:9002".to_string();
    let mut channel = "can0".to_string();
    let mut model = "4340P".to_string();
    let mut motor_id = 0x01u16;
    let mut feedback_id = 0x11u16;
    let mut dt_ms = 20u64;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0usize;
    while i < args.len() {
        let k = &args[i];
        if k == "--help" || k == "-h" {
            println!(
                "ws_gateway\n\
Usage:\n\
  cargo run -p ws_gateway --release -- \\\n    --bind 0.0.0.0:9002 --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20\n"
            );
            std::process::exit(0);
        }
        let next = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {k}"))?;
        match k.as_str() {
            "--bind" => bind = next.clone(),
            "--channel" => channel = next.clone(),
            "--model" => model = next.clone(),
            "--motor-id" => motor_id = parse_hex_or_dec(next)?,
            "--feedback-id" => feedback_id = parse_hex_or_dec(next)?,
            "--dt-ms" => {
                dt_ms = next
                    .parse::<u64>()
                    .map_err(|e| format!("invalid --dt-ms: {e}"))?;
            }
            _ => return Err(format!("unknown arg: {k}")),
        }
        i += 2;
    }

    Ok(ServerConfig {
        bind,
        target: Target {
            channel,
            model,
            motor_id,
            feedback_id,
        },
        dt_ms,
    })
}

fn as_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn as_u64(v: &Value, key: &str, default: u64) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn as_f32(v: &Value, key: &str, default: f32) -> f32 {
    v.get(key)
        .and_then(Value::as_f64)
        .map(|x| x as f32)
        .unwrap_or(default)
}

fn as_u16(v: &Value, key: &str, default: u16) -> u16 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_u64().map(|x| x as u16).unwrap_or(default),
        Some(Value::String(s)) => parse_hex_or_dec(s).unwrap_or(default),
        _ => default,
    }
}

fn parse_mode(v: &Value) -> Result<ControlMode, String> {
    if let Some(s) = v.get("mode").and_then(Value::as_str) {
        return match s.to_lowercase().as_str() {
            "mit" => Ok(ControlMode::Mit),
            "pos_vel" | "pos-vel" | "posvel" => Ok(ControlMode::PosVel),
            "vel" => Ok(ControlMode::Vel),
            "force_pos" | "force-pos" | "forcepos" => Ok(ControlMode::ForcePos),
            _ => Err(format!("unsupported mode string: {s}")),
        };
    }
    if let Some(n) = v.get("mode").and_then(Value::as_u64) {
        return match n {
            1 => Ok(ControlMode::Mit),
            2 => Ok(ControlMode::PosVel),
            3 => Ok(ControlMode::Vel),
            4 => Ok(ControlMode::ForcePos),
            _ => Err(format!("unsupported mode value: {n}")),
        };
    }
    Err("missing mode (string or numeric)".to_string())
}

async fn send_json<S>(tx: &mut S, obj: Value) -> Result<(), String>
where
    S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    tx.send(Message::Text(obj.to_string()))
        .await
        .map_err(|e| e.to_string())
}

fn cmd_scan(v: &Value, base: &Target) -> Result<Value, String> {
    let start_id = as_u16(v, "start_id", 1);
    let end_id = as_u16(v, "end_id", 16);
    let feedback_base = as_u16(v, "feedback_base", 16);
    let timeout_ms = as_u64(v, "timeout_ms", 100);

    if end_id < start_id {
        return Err("end_id must be >= start_id".to_string());
    }

    let mut hits = Vec::new();
    for mid in start_id..=end_id {
        let fid = feedback_base + (mid & 0x0F);
        let ctrl = DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
        let motor = match ctrl.add_motor(mid, fid, &base.model) {
            Ok(m) => m,
            Err(_) => {
                let _ = ctrl.close_bus();
                continue;
            }
        };
        let esc = motor.get_register_u32(8, Duration::from_millis(timeout_ms));
        let mst = motor.get_register_u32(7, Duration::from_millis(timeout_ms));
        if let (Ok(esc_id), Ok(mst_id)) = (esc, mst) {
            hits.push(json!({"probe": mid, "esc_id": esc_id, "mst_id": mst_id, "probe_feedback_id": fid}));
        }
        let _ = ctrl.close_bus();
    }

    Ok(json!({
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

fn cmd_verify(v: &Value, base: &Target) -> Result<Value, String> {
    let mid = as_u16(v, "motor_id", base.motor_id);
    let fid = as_u16(v, "feedback_id", base.feedback_id);
    let timeout_ms = as_u64(v, "timeout_ms", 1000);

    let ctrl = DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
    let motor = ctrl
        .add_motor(mid, fid, &base.model)
        .map_err(|e| e.to_string())?;

    let esc = motor
        .get_register_u32(8, Duration::from_millis(timeout_ms))
        .map_err(|e| e.to_string())?;
    let mst = motor
        .get_register_u32(7, Duration::from_millis(timeout_ms))
        .map_err(|e| e.to_string())?;
    let _ = ctrl.close_bus();

    Ok(json!({
        "motor_id": mid,
        "feedback_id": fid,
        "esc_id": esc,
        "mst_id": mst,
        "ok": esc == mid as u32 && mst == fid as u32,
    }))
}

fn cmd_set_id(v: &Value, base: &Target) -> Result<Value, String> {
    let old_mid = as_u16(v, "old_motor_id", base.motor_id);
    let old_fid = as_u16(v, "old_feedback_id", base.feedback_id);
    let new_mid = as_u16(v, "new_motor_id", old_mid);
    let new_fid = as_u16(v, "new_feedback_id", old_fid);
    let store = as_bool(v, "store", true);
    let verify = as_bool(v, "verify", true);
    let timeout_ms = as_u64(v, "timeout_ms", 1000);

    let ctrl = DamiaoController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
    let motor = ctrl
        .add_motor(old_mid, old_fid, &base.model)
        .map_err(|e| e.to_string())?;

    // Robust update order: MST_ID first, then ESC_ID.
    if new_fid != old_fid {
        motor
            .write_register_u32(7, new_fid as u32)
            .map_err(|e| e.to_string())?;
    }
    if new_mid != old_mid {
        motor
            .write_register_u32(8, new_mid as u32)
            .map_err(|e| e.to_string())?;
    }
    if store {
        motor.store_parameters().map_err(|e| e.to_string())?;
    }
    let _ = ctrl.close_bus();

    let mut out = json!({
        "old_motor_id": old_mid,
        "old_feedback_id": old_fid,
        "new_motor_id": new_mid,
        "new_feedback_id": new_fid,
        "store": store,
    });

    if verify {
        std::thread::sleep(Duration::from_millis(120));
        let verify_result = cmd_verify(
            &json!({
                "motor_id": new_mid,
                "feedback_id": new_fid,
                "timeout_ms": timeout_ms,
            }),
            base,
        )?;
        out["verify"] = verify_result;
    }

    Ok(out)
}

fn build_state(motor: &Arc<DamiaoMotor>) -> Value {
    if let Some(s) = motor.latest_state() {
        json!({
            "has_value": true,
            "can_id": s.can_id,
            "arbitration_id": s.arbitration_id,
            "status_code": s.status_code,
            "pos": s.pos,
            "vel": s.vel,
            "torq": s.torq,
            "t_mos": s.t_mos,
            "t_rotor": s.t_rotor,
        })
    } else {
        json!({"has_value": false})
    }
}

async fn handle_socket(stream: TcpStream, cfg: ServerConfig) -> Result<(), String> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let ws = accept_async(stream).await.map_err(|e| e.to_string())?;
    let (mut tx, mut rx) = ws.split();

    let mut ctx = SessionCtx::new(cfg.target.clone());
    if let Err(e) = ctx.connect() {
        let _ = send_json(
            &mut tx,
            json!({"type":"event","event":"connect_failed","error": e}),
        )
        .await;
    } else {
        let _ = send_json(
            &mut tx,
            json!({
                "type":"event",
                "event":"connected",
                "data": {
                    "channel": ctx.target.channel,
                    "model": ctx.target.model,
                    "motor_id": ctx.target.motor_id,
                    "feedback_id": ctx.target.feedback_id,
                    "peer": peer,
                }
            }),
        )
        .await;
    }

    let mut ticker = time::interval(Duration::from_millis(cfg.dt_ms));
    loop {
        tokio::select! {
            maybe_msg = rx.next() => {
                let msg = match maybe_msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => return Err(format!("ws recv error: {e}")),
                    None => break,
                };

                match msg {
                    Message::Text(text) => {
                        let v: Value = match serde_json::from_str(&text) {
                            Ok(x) => x,
                            Err(e) => {
                                send_json(&mut tx, json!({"ok":false, "error": format!("invalid json: {e}")})).await?;
                                continue;
                            }
                        };
                        let op = v.get("op").and_then(Value::as_str).unwrap_or("").to_lowercase();

                        let result: Result<Value, String> = match op.as_str() {
                            "ping" => Ok(json!({"pong": true})),
                            "set_target" => {
                                let mut next = ctx.target.clone();
                                next.channel = v.get("channel").and_then(Value::as_str).unwrap_or(&next.channel).to_string();
                                next.model = v.get("model").and_then(Value::as_str).unwrap_or(&next.model).to_string();
                                next.motor_id = as_u16(&v, "motor_id", next.motor_id);
                                next.feedback_id = as_u16(&v, "feedback_id", next.feedback_id);
                                ctx.target = next;
                                ctx.active = None;
                                ctx.connect()?;
                                Ok(json!({
                                    "channel": ctx.target.channel,
                                    "model": ctx.target.model,
                                    "motor_id": ctx.target.motor_id,
                                    "feedback_id": ctx.target.feedback_id,
                                }))
                            }
                            "enable" => {
                                ctx.ensure_connected()?;
                                if let Some(ctrl) = ctx.controller.as_ref() {
                                    ctrl.enable_all().map_err(|e| e.to_string())?;
                                }
                                if let Some(m) = ctx.motor.as_ref() {
                                    let _ = m.request_motor_feedback();
                                }
                                ctx.active = None;
                                Ok(json!({"enabled": true}))
                            }
                            "disable" => {
                                ctx.ensure_connected()?;
                                if let Some(ctrl) = ctx.controller.as_ref() {
                                    ctrl.disable_all().map_err(|e| e.to_string())?;
                                }
                                if let Some(m) = ctx.motor.as_ref() {
                                    let _ = m.request_motor_feedback();
                                }
                                ctx.active = None;
                                Ok(json!({"disabled": true}))
                            }
                            "mit" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::Mit {
                                    pos: as_f32(&v, "pos", 0.0),
                                    vel: as_f32(&v, "vel", 0.0),
                                    kp: as_f32(&v, "kp", 30.0),
                                    kd: as_f32(&v, "kd", 1.0),
                                    tau: as_f32(&v, "tau", 0.0),
                                };
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.ensure_control_mode(ControlMode::Mit, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                        .map_err(|e| e.to_string())?;
                                }
                                ctx.active = if as_bool(&v, "continuous", false) { Some(cmd.clone()) } else { None };
                                if let Some(m) = ctx.motor.as_ref() {
                                    match cmd {
                                        ActiveCommand::Mit{pos,vel,kp,kd,tau} => m.send_cmd_mit(pos,vel,kp,kd,tau).map_err(|e| e.to_string())?,
                                        _ => {}
                                    }
                                }
                                Ok(json!({"op":"mit","continuous": as_bool(&v, "continuous", false)}))
                            }
                            "pos_vel" | "pos-vel" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::PosVel { pos: as_f32(&v, "pos", 0.0), vlim: as_f32(&v, "vlim", 1.0)};
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.ensure_control_mode(ControlMode::PosVel, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                        .map_err(|e| e.to_string())?;
                                }
                                ctx.active = if as_bool(&v, "continuous", false) { Some(cmd.clone()) } else { None };
                                if let Some(m) = ctx.motor.as_ref() {
                                    match cmd { ActiveCommand::PosVel{pos,vlim} => m.send_cmd_pos_vel(pos,vlim).map_err(|e| e.to_string())?, _=>{} }
                                }
                                Ok(json!({"op":"pos_vel","continuous": as_bool(&v, "continuous", false)}))
                            }
                            "vel" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::Vel { vel: as_f32(&v, "vel", 0.0)};
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.ensure_control_mode(ControlMode::Vel, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                        .map_err(|e| e.to_string())?;
                                }
                                ctx.active = if as_bool(&v, "continuous", false) { Some(cmd.clone()) } else { None };
                                if let Some(m) = ctx.motor.as_ref() {
                                    match cmd { ActiveCommand::Vel{vel} => m.send_cmd_vel(vel).map_err(|e| e.to_string())?, _=>{} }
                                }
                                Ok(json!({"op":"vel","continuous": as_bool(&v, "continuous", false)}))
                            }
                            "force_pos" | "force-pos" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::ForcePos {
                                    pos: as_f32(&v, "pos", 0.0),
                                    vlim: as_f32(&v, "vlim", 1.0),
                                    ratio: as_f32(&v, "ratio", 0.3),
                                };
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.ensure_control_mode(ControlMode::ForcePos, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                        .map_err(|e| e.to_string())?;
                                }
                                ctx.active = if as_bool(&v, "continuous", false) { Some(cmd.clone()) } else { None };
                                if let Some(m) = ctx.motor.as_ref() {
                                    match cmd { ActiveCommand::ForcePos{pos,vlim,ratio} => m.send_cmd_force_pos(pos,vlim,ratio).map_err(|e| e.to_string())?, _=>{} }
                                }
                                Ok(json!({"op":"force_pos","continuous": as_bool(&v, "continuous", false)}))
                            }
                            "stop" => {
                                ctx.active = None;
                                Ok(json!({"stopped": true}))
                            }
                            "state_once" => {
                                ctx.ensure_connected()?;
                                if let Some(m) = ctx.motor.as_ref() {
                                    let _ = m.request_motor_feedback();
                                    Ok(json!({"state": build_state(m)}))
                                } else {
                                    Err("motor not connected".to_string())
                                }
                            }
                            "clear_error" => {
                                ctx.ensure_connected()?;
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.clear_error().map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"cleared": true}))
                            }
                            "set_zero_position" => {
                                ctx.ensure_connected()?;
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.set_zero_position().map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"zero_set": true}))
                            }
                            "ensure_mode" => {
                                ctx.ensure_connected()?;
                                let mode = parse_mode(&v)?;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.ensure_control_mode(mode, Duration::from_millis(timeout_ms))
                                        .map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"ensured": true}))
                            }
                            "request_feedback" => {
                                ctx.ensure_connected()?;
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.request_motor_feedback().map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"requested": true}))
                            }
                            "store_parameters" => {
                                ctx.ensure_connected()?;
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.store_parameters().map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"stored": true}))
                            }
                            "set_can_timeout_ms" => {
                                ctx.ensure_connected()?;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                let reg_value = (timeout_ms as u32).saturating_mul(20);
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.write_register_u32(9, reg_value).map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"timeout_ms": timeout_ms, "reg9_value": reg_value}))
                            }
                            "write_register_u32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let value = as_u64(&v, "value", 0) as u32;
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.write_register_u32(rid, value).map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"rid": rid, "value": value}))
                            }
                            "write_register_f32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let value = as_f32(&v, "value", 0.0);
                                if let Some(m) = ctx.motor.as_ref() {
                                    m.write_register_f32(rid, value).map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"rid": rid, "value": value}))
                            }
                            "get_register_u32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                if let Some(m) = ctx.motor.as_ref() {
                                    let val = m
                                        .get_register_u32(rid, Duration::from_millis(timeout_ms))
                                        .map_err(|e| e.to_string())?;
                                    Ok(json!({"rid": rid, "value": val}))
                                } else {
                                    Err("motor not connected".to_string())
                                }
                            }
                            "get_register_f32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                if let Some(m) = ctx.motor.as_ref() {
                                    let val = m
                                        .get_register_f32(rid, Duration::from_millis(timeout_ms))
                                        .map_err(|e| e.to_string())?;
                                    Ok(json!({"rid": rid, "value": val}))
                                } else {
                                    Err("motor not connected".to_string())
                                }
                            }
                            "poll_feedback_once" => {
                                ctx.ensure_connected()?;
                                if let Some(c) = ctx.controller.as_ref() {
                                    c.poll_feedback_once().map_err(|e| e.to_string())?;
                                }
                                Ok(json!({"polled": true}))
                            }
                            "shutdown" => {
                                if let Some(c) = ctx.controller.as_ref() {
                                    c.shutdown().map_err(|e| e.to_string())?;
                                }
                                ctx.active = None;
                                Ok(json!({"shutdown": true}))
                            }
                            "close_bus" => {
                                ctx.disconnect(false);
                                Ok(json!({"closed": true}))
                            }
                            "scan" => cmd_scan(&v, &ctx.target),
                            "set_id" => cmd_set_id(&v, &ctx.target),
                            "verify" => cmd_verify(&v, &ctx.target),
                            _ => Err(format!("unsupported op: {op}")),
                        };

                        match result {
                            Ok(data) => send_json(&mut tx, json!({"ok": true, "op": op, "data": data})).await?,
                            Err(err) => send_json(&mut tx, json!({"ok": false, "op": op, "error": err})).await?,
                        }
                    }
                    Message::Ping(payload) => {
                        tx.send(Message::Pong(payload)).await.map_err(|e| e.to_string())?;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            _ = ticker.tick() => {
                if ctx.active.is_some() {
                    if let Err(e) = ctx.apply_active() {
                        ctx.active = None;
                        send_json(&mut tx, json!({"ok": false, "op": "active_tick", "error": e})).await?;
                    }
                }
                if let Some(m) = ctx.motor.as_ref() {
                    let _ = m.request_motor_feedback();
                    send_json(&mut tx, json!({"type":"state", "data": build_state(m)})).await?;
                }
            }
        }
    }

    ctx.disconnect(false);
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = parse_args().map_err(|e| format!("arg parse error: {e}"))?;
    let listener = TcpListener::bind(&cfg.bind).await?;

    println!(
        "ws_gateway listening on ws://{} (channel={}, model={}, motor_id=0x{:X}, feedback_id=0x{:X}, dt_ms={})",
        cfg.bind, cfg.target.channel, cfg.target.model, cfg.target.motor_id, cfg.target.feedback_id, cfg.dt_ms
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let cfg_cloned = cfg.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_socket(stream, cfg_cloned).await {
                eprintln!("[ws_gateway] session error: {e}");
            }
        });
    }
}
