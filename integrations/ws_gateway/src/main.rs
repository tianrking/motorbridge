use futures_util::{SinkExt, StreamExt};
use motor_vendor_damiao::{ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor};
use motor_vendor_robstride::{
    ControlMode as RobstrideControlMode, ParameterValue as RobstrideParameterValue, RobstrideController,
    RobstrideMotor,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Vendor {
    Damiao,
    Robstride,
}

impl Vendor {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "damiao" => Ok(Self::Damiao),
            "robstride" => Ok(Self::Robstride),
            _ => Err(format!("unsupported vendor: {s}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Damiao => "damiao",
            Self::Robstride => "robstride",
        }
    }
}

#[derive(Clone, Debug)]
struct Target {
    vendor: Vendor,
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

enum ControllerHandle {
    Damiao(DamiaoController),
    Robstride(RobstrideController),
}

enum MotorHandle {
    Damiao(Arc<DamiaoMotor>),
    Robstride(Arc<RobstrideMotor>),
}

struct SessionCtx {
    target: Target,
    controller: Option<ControllerHandle>,
    motor: Option<MotorHandle>,
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
        match self.target.vendor {
            Vendor::Damiao => {
                let ctrl = DamiaoController::new_socketcan(&self.target.channel)
                    .map_err(|e| format!("open bus failed: {e}"))?;
                let motor = ctrl
                    .add_motor(self.target.motor_id, self.target.feedback_id, &self.target.model)
                    .map_err(|e| format!("add motor failed: {e}"))?;
                self.controller = Some(ControllerHandle::Damiao(ctrl));
                self.motor = Some(MotorHandle::Damiao(motor));
            }
            Vendor::Robstride => {
                let ctrl = RobstrideController::new_socketcan(&self.target.channel)
                    .map_err(|e| format!("open bus failed: {e}"))?;
                let motor = ctrl
                    .add_motor(self.target.motor_id, self.target.feedback_id, &self.target.model)
                    .map_err(|e| format!("add motor failed: {e}"))?;
                self.controller = Some(ControllerHandle::Robstride(ctrl));
                self.motor = Some(MotorHandle::Robstride(motor));
            }
        }
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
            match ctrl {
                ControllerHandle::Damiao(c) => {
                    if shutdown {
                        let _ = c.shutdown();
                    } else {
                        let _ = c.close_bus();
                    }
                }
                ControllerHandle::Robstride(c) => {
                    if shutdown {
                        let _ = c.shutdown();
                    } else {
                        let _ = c.close_bus();
                    }
                }
            }
        }
    }

    fn apply_active(&self) -> Result<(), String> {
        match self.motor.as_ref() {
            Some(MotorHandle::Damiao(motor)) => match self.active.as_ref() {
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
                Some(ActiveCommand::Vel { vel }) => motor.send_cmd_vel(*vel).map_err(|e| e.to_string()),
                Some(ActiveCommand::ForcePos { pos, vlim, ratio }) => motor
                    .send_cmd_force_pos(*pos, *vlim, *ratio)
                    .map_err(|e| e.to_string()),
                None => Ok(()),
            },
            Some(MotorHandle::Robstride(motor)) => match self.active.as_ref() {
                Some(ActiveCommand::Mit {
                    pos,
                    vel,
                    kp,
                    kd,
                    tau,
                }) => motor
                    .send_cmd_mit(*pos, *vel, *kp, *kd, *tau)
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::Vel { vel }) => motor
                    .set_velocity_target(*vel)
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::PosVel { .. }) | Some(ActiveCommand::ForcePos { .. }) => {
                    Err("pos_vel/force_pos are not supported for robstride".to_string())
                }
                None => Ok(()),
            },
            None => Err("motor not connected".to_string()),
        }
    }

    fn build_state_snapshot(&self) -> Result<Value, String> {
        match (&self.controller, &self.motor) {
            (Some(ControllerHandle::Damiao(_)), Some(MotorHandle::Damiao(motor))) => {
                let _ = motor.request_motor_feedback();
                if let Some(s) = motor.latest_state() {
                    Ok(json!({
                        "vendor": "damiao",
                        "has_value": true,
                        "can_id": s.can_id,
                        "arbitration_id": s.arbitration_id,
                        "status_code": s.status_code,
                        "status_name": s.status_name,
                        "pos": s.pos,
                        "vel": s.vel,
                        "torq": s.torq,
                        "t_mos": s.t_mos,
                        "t_rotor": s.t_rotor,
                    }))
                } else {
                    Ok(json!({"vendor":"damiao","has_value": false}))
                }
            }
            (Some(ControllerHandle::Robstride(ctrl)), Some(MotorHandle::Robstride(motor))) => {
                ctrl.poll_feedback_once().map_err(|e| e.to_string())?;
                if let Some(s) = motor.latest_state() {
                    Ok(json!({
                        "vendor": "robstride",
                        "has_value": true,
                        "arbitration_id": s.arbitration_id,
                        "device_id": s.device_id,
                        "status_code": 0,
                        "pos": s.position,
                        "vel": s.velocity,
                        "torq": s.torque,
                        "t_mos": s.temperature_c,
                        "flags": {
                            "uncalibrated": s.uncalibrated,
                            "stall": s.stall,
                            "magnetic_encoder_fault": s.magnetic_encoder_fault,
                            "overtemperature": s.overtemperature,
                            "overcurrent": s.overcurrent,
                            "undervoltage": s.undervoltage
                        }
                    }))
                } else {
                    Ok(json!({"vendor":"robstride","has_value": false}))
                }
            }
            _ => Err("motor not connected".to_string()),
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

fn parse_u32_hex_or_dec(s: &str) -> Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s}: {e}"))
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid integer {s}: {e}"))
    }
}

fn parse_id_list_csv(s: &str) -> Vec<u16> {
    s.split(',')
        .filter_map(|x| {
            let t = x.trim();
            if t.is_empty() {
                None
            } else {
                parse_hex_or_dec(t).ok()
            }
        })
        .collect()
}

fn parse_args() -> Result<ServerConfig, String> {
    let mut bind = "0.0.0.0:9002".to_string();
    let mut vendor = Vendor::Damiao;
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
  cargo run -p ws_gateway --release -- \\\n    --bind 0.0.0.0:9002 --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 --dt-ms 20\n\
  cargo run -p ws_gateway --release -- \\\n    --bind 0.0.0.0:9002 --vendor robstride --channel can0 --model rs-06 --motor-id 127 --feedback-id 0xFF --dt-ms 20\n"
            );
            std::process::exit(0);
        }
        let next = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {k}"))?;
        match k.as_str() {
            "--bind" => bind = next.clone(),
            "--vendor" => vendor = Vendor::from_str(next)?,
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

    if vendor == Vendor::Robstride {
        if model == "4340P" || model == "4340" {
            model = "rs-00".to_string();
        }
        if feedback_id == 0x11 {
            feedback_id = 0xFF;
        }
    }

    Ok(ServerConfig {
        bind,
        target: Target {
            vendor,
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

fn parse_vendor_in_msg(v: &Value, default: Vendor) -> Result<Vendor, String> {
    match v.get("vendor").and_then(Value::as_str) {
        Some(s) => Vendor::from_str(s),
        None => Ok(default),
    }
}

fn parse_damiao_mode(v: &Value) -> Result<DamiaoControlMode, String> {
    if let Some(s) = v.get("mode").and_then(Value::as_str) {
        return match s.to_lowercase().as_str() {
            "mit" => Ok(DamiaoControlMode::Mit),
            "pos_vel" | "pos-vel" | "posvel" => Ok(DamiaoControlMode::PosVel),
            "vel" => Ok(DamiaoControlMode::Vel),
            "force_pos" | "force-pos" | "forcepos" => Ok(DamiaoControlMode::ForcePos),
            _ => Err(format!("unsupported mode string: {s}")),
        };
    }
    if let Some(n) = v.get("mode").and_then(Value::as_u64) {
        return match n {
            1 => Ok(DamiaoControlMode::Mit),
            2 => Ok(DamiaoControlMode::PosVel),
            3 => Ok(DamiaoControlMode::Vel),
            4 => Ok(DamiaoControlMode::ForcePos),
            _ => Err(format!("unsupported mode value: {n}")),
        };
    }
    Err("missing mode (string or numeric)".to_string())
}

fn parse_robstride_mode(v: &Value) -> Result<RobstrideControlMode, String> {
    if let Some(s) = v.get("mode").and_then(Value::as_str) {
        return match s.to_lowercase().as_str() {
            "mit" => Ok(RobstrideControlMode::Mit),
            "position" | "pos" => Ok(RobstrideControlMode::Position),
            "vel" | "velocity" => Ok(RobstrideControlMode::Velocity),
            _ => Err(format!("unsupported robstride mode string: {s}")),
        };
    }
    if let Some(n) = v.get("mode").and_then(Value::as_u64) {
        return match n {
            0 => Ok(RobstrideControlMode::Mit),
            1 => Ok(RobstrideControlMode::Position),
            2 => Ok(RobstrideControlMode::Velocity),
            _ => Err(format!("unsupported robstride mode value: {n}")),
        };
    }
    Err("missing mode (string or numeric)".to_string())
}

async fn send_json<S>(tx: &mut S, obj: Value) -> Result<(), String>
where
    S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    tx.send(Message::Text(obj.to_string().into()))
        .await
        .map_err(|e| e.to_string())
}

fn cmd_scan_damiao(v: &Value, base: &Target) -> Result<Value, String> {
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
        "vendor": "damiao",
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

fn cmd_scan_robstride(v: &Value, base: &Target) -> Result<Value, String> {
    let start_id = as_u16(v, "start_id", 1);
    let end_id = as_u16(v, "end_id", 255);
    let timeout_ms = as_u64(v, "timeout_ms", 120);
    let param_id = as_u16(v, "param_id", 0x7019);
    if end_id < start_id {
        return Err("end_id must be >= start_id".to_string());
    }

    let feedback_ids = match v.get("feedback_ids") {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|x| {
                x.as_u64()
                    .map(|n| n as u16)
                    .or_else(|| x.as_str().and_then(|s| parse_hex_or_dec(s).ok()))
            })
            .collect::<Vec<u16>>(),
        Some(Value::String(s)) => parse_id_list_csv(s),
        _ => vec![base.feedback_id, 0xFF, 0xFE, 0x00],
    };
    let feedback_ids = if feedback_ids.is_empty() {
        vec![base.feedback_id, 0xFF, 0xFE, 0x00]
    } else {
        feedback_ids
    };

    let mut hits = Vec::new();
    for mid in start_id..=end_id {
        let mut found = None;
        for fid in &feedback_ids {
            let ctrl = RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = match ctrl.add_motor(mid, *fid, &base.model) {
                Ok(m) => m,
                Err(_) => {
                    let _ = ctrl.close_bus();
                    continue;
                }
            };
            let ping = motor.ping(Duration::from_millis(timeout_ms));
            if let Ok(p) = ping {
                found = Some(json!({
                    "probe": mid,
                    "via": "ping",
                    "feedback_id": fid,
                    "device_id": p.device_id,
                    "responder_id": p.responder_id
                }));
                let _ = ctrl.close_bus();
                break;
            }
            if let Ok(val) = motor.get_parameter_f32(param_id, Duration::from_millis(timeout_ms)) {
                found = Some(json!({
                    "probe": mid,
                    "via": "read_param",
                    "feedback_id": fid,
                    "param_id": format!("0x{param_id:04X}"),
                    "value": val
                }));
                let _ = ctrl.close_bus();
                break;
            }
            let _ = ctrl.close_bus();
        }
        if let Some(hit) = found {
            hits.push(hit);
        }
    }

    Ok(json!({
        "vendor": "robstride",
        "count": hits.len(),
        "start_id": start_id,
        "end_id": end_id,
        "hits": hits,
    }))
}

fn cmd_scan(v: &Value, base: &Target) -> Result<Value, String> {
    match parse_vendor_in_msg(v, base.vendor)? {
        Vendor::Damiao => cmd_scan_damiao(v, base),
        Vendor::Robstride => cmd_scan_robstride(v, base),
    }
}

fn cmd_verify(v: &Value, base: &Target) -> Result<Value, String> {
    let vendor = parse_vendor_in_msg(v, base.vendor)?;
    let mid = as_u16(v, "motor_id", base.motor_id);
    let fid = as_u16(v, "feedback_id", base.feedback_id);
    let timeout_ms = as_u64(v, "timeout_ms", 1000);

    match vendor {
        Vendor::Damiao => {
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
                "vendor": "damiao",
                "motor_id": mid,
                "feedback_id": fid,
                "esc_id": esc,
                "mst_id": mst,
                "ok": esc == mid as u32 && mst == fid as u32,
            }))
        }
        Vendor::Robstride => {
            let ctrl = RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = ctrl
                .add_motor(mid, fid, &base.model)
                .map_err(|e| e.to_string())?;
            let ping = motor
                .ping(Duration::from_millis(timeout_ms))
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();
            Ok(json!({
                "vendor": "robstride",
                "motor_id": mid,
                "feedback_id": fid,
                "device_id": ping.device_id,
                "responder_id": ping.responder_id,
                "ok": ping.device_id == mid as u8,
            }))
        }
    }
}

fn cmd_set_id(v: &Value, base: &Target) -> Result<Value, String> {
    let vendor = parse_vendor_in_msg(v, base.vendor)?;
    match vendor {
        Vendor::Damiao => {
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
                "vendor": "damiao",
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
                        "vendor":"damiao",
                        "motor_id": new_mid,
                        "feedback_id": new_fid,
                        "timeout_ms": timeout_ms
                    }),
                    base,
                )?;
                out["verify"] = verify_result;
            }
            Ok(out)
        }
        Vendor::Robstride => {
            let old_mid = as_u16(v, "old_motor_id", base.motor_id);
            let fid = as_u16(v, "feedback_id", base.feedback_id);
            let new_mid = as_u16(v, "new_motor_id", old_mid);
            if new_mid == 0 || new_mid > 0xFF {
                return Err("robstride new_motor_id must be 1..255".to_string());
            }
            let verify = as_bool(v, "verify", true);
            let timeout_ms = as_u64(v, "timeout_ms", 1000);
            let ctrl = RobstrideController::new_socketcan(&base.channel).map_err(|e| e.to_string())?;
            let motor = ctrl
                .add_motor(old_mid, fid, &base.model)
                .map_err(|e| e.to_string())?;
            motor
                .set_device_id(new_mid as u8)
                .map_err(|e| e.to_string())?;
            let _ = ctrl.close_bus();

            let mut out = json!({
                "vendor": "robstride",
                "old_motor_id": old_mid,
                "new_motor_id": new_mid,
                "feedback_id": fid
            });
            if verify {
                std::thread::sleep(Duration::from_millis(120));
                let verify_result = cmd_verify(
                    &json!({
                        "vendor":"robstride",
                        "motor_id": new_mid,
                        "feedback_id": fid,
                        "timeout_ms": timeout_ms
                    }),
                    base,
                )?;
                out["verify"] = verify_result;
            }
            Ok(out)
        }
    }
}

fn parse_param_type(v: &Value) -> String {
    v.get("type")
        .and_then(Value::as_str)
        .unwrap_or("f32")
        .to_lowercase()
}

fn parse_param_value(v: &Value) -> Option<String> {
    v.get("value").map(|x| match x {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        _ => "".to_string(),
    })
}

fn handle_robstride_read_param(motor: &Arc<RobstrideMotor>, v: &Value) -> Result<Value, String> {
    let param_id = as_u16(v, "param_id", 0x7019);
    let timeout_ms = as_u64(v, "timeout_ms", 500);
    let ty = parse_param_type(v);
    let timeout = Duration::from_millis(timeout_ms);
    match ty.as_str() {
        "i8" => Ok(json!({"param_id": param_id, "type":"i8", "value": motor.get_parameter_i8(param_id, timeout).map_err(|e| e.to_string())? })),
        "u8" => match motor.get_parameter(param_id, timeout).map_err(|e| e.to_string())? {
            RobstrideParameterValue::U8(x) => Ok(json!({"param_id": param_id, "type":"u8", "value": x })),
            _ => Err(format!("parameter 0x{param_id:04X} is not u8")),
        },
        "u16" => match motor.get_parameter(param_id, timeout).map_err(|e| e.to_string())? {
            RobstrideParameterValue::U16(x) => Ok(json!({"param_id": param_id, "type":"u16", "value": x })),
            _ => Err(format!("parameter 0x{param_id:04X} is not u16")),
        },
        "u32" => match motor.get_parameter(param_id, timeout).map_err(|e| e.to_string())? {
            RobstrideParameterValue::U32(x) => Ok(json!({"param_id": param_id, "type":"u32", "value": x })),
            _ => Err(format!("parameter 0x{param_id:04X} is not u32")),
        },
        _ => Ok(json!({"param_id": param_id, "type":"f32", "value": motor.get_parameter_f32(param_id, timeout).map_err(|e| e.to_string())? })),
    }
}

fn handle_robstride_write_param(motor: &Arc<RobstrideMotor>, v: &Value) -> Result<Value, String> {
    let param_id = as_u16(v, "param_id", 0x700A);
    let timeout_ms = as_u64(v, "timeout_ms", 500);
    let verify = as_bool(v, "verify", true);
    let ty = parse_param_type(v);
    let raw = parse_param_value(v).ok_or_else(|| "missing value".to_string())?;
    let pval = match ty.as_str() {
        "i8" => RobstrideParameterValue::I8(
            parse_u32_hex_or_dec(&raw)
                .map_err(|e| format!("invalid i8 value: {e}"))? as i8,
        ),
        "u8" => RobstrideParameterValue::U8(
            parse_u32_hex_or_dec(&raw)
                .map_err(|e| format!("invalid u8 value: {e}"))? as u8,
        ),
        "u16" => RobstrideParameterValue::U16(
            parse_u32_hex_or_dec(&raw)
                .map_err(|e| format!("invalid u16 value: {e}"))? as u16,
        ),
        "u32" => RobstrideParameterValue::U32(
            parse_u32_hex_or_dec(&raw)
                .map_err(|e| format!("invalid u32 value: {e}"))?,
        ),
        _ => RobstrideParameterValue::F32(
            raw.parse::<f32>()
                .map_err(|e| format!("invalid f32 value: {e}"))?,
        ),
    };
    motor
        .write_parameter(param_id, pval)
        .map_err(|e| e.to_string())?;
    let verify_data = if verify {
        Some(handle_robstride_read_param(
            motor,
            &json!({"param_id": param_id, "type": ty, "timeout_ms": timeout_ms}),
        )?)
    } else {
        None
    };
    Ok(json!({
        "param_id": param_id,
        "type": ty,
        "value": raw,
        "verify": verify_data
    }))
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
                    "vendor": ctx.target.vendor.as_str(),
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
                            "ping" => {
                                match ctx.target.vendor {
                                    Vendor::Robstride => {
                                        ctx.ensure_connected()?;
                                        if let Some(MotorHandle::Robstride(m)) = ctx.motor.as_ref() {
                                            let p = m.ping(Duration::from_millis(as_u64(&v, "timeout_ms", 200))).map_err(|e| e.to_string())?;
                                            Ok(json!({"pong":true,"vendor":"robstride","device_id":p.device_id,"responder_id":p.responder_id}))
                                        } else {
                                            Err("motor not connected".to_string())
                                        }
                                    }
                                    Vendor::Damiao => Ok(json!({"pong": true, "vendor":"damiao"})),
                                }
                            }
                            "set_target" => {
                                let mut next = ctx.target.clone();
                                next.vendor = parse_vendor_in_msg(&v, next.vendor)?;
                                next.channel = v.get("channel").and_then(Value::as_str).unwrap_or(&next.channel).to_string();
                                next.model = v.get("model").and_then(Value::as_str).unwrap_or(&next.model).to_string();
                                next.motor_id = as_u16(&v, "motor_id", next.motor_id);
                                next.feedback_id = as_u16(&v, "feedback_id", next.feedback_id);
                                if next.vendor == Vendor::Robstride {
                                    if next.model == "4340" || next.model == "4340P" {
                                        next.model = "rs-00".to_string();
                                    }
                                    if next.feedback_id == 0x11 {
                                        next.feedback_id = 0xFF;
                                    }
                                }
                                ctx.target = next;
                                ctx.active = None;
                                ctx.connect()?;
                                Ok(json!({
                                    "vendor": ctx.target.vendor.as_str(),
                                    "channel": ctx.target.channel,
                                    "model": ctx.target.model,
                                    "motor_id": ctx.target.motor_id,
                                    "feedback_id": ctx.target.feedback_id,
                                }))
                            }
                            "enable" => {
                                ctx.ensure_connected()?;
                                if let Some(c) = ctx.controller.as_ref() {
                                    match c {
                                        ControllerHandle::Damiao(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
                                        ControllerHandle::Robstride(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
                                    }
                                }
                                ctx.active = None;
                                Ok(json!({"enabled": true}))
                            }
                            "disable" => {
                                ctx.ensure_connected()?;
                                if let Some(c) = ctx.controller.as_ref() {
                                    match c {
                                        ControllerHandle::Damiao(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
                                        ControllerHandle::Robstride(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
                                    }
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
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        m.ensure_control_mode(DamiaoControlMode::Mit, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                            .map_err(|e| e.to_string())?;
                                        if let ActiveCommand::Mit{pos,vel,kp,kd,tau} = cmd {
                                            m.send_cmd_mit(pos,vel,kp,kd,tau).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    Some(MotorHandle::Robstride(m)) => {
                                        m.set_mode(RobstrideControlMode::Mit).map_err(|e| e.to_string())?;
                                        if let ActiveCommand::Mit{pos,vel,kp,kd,tau} = cmd {
                                            m.send_cmd_mit(pos,vel,kp,kd,tau).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    None => return Err("motor not connected".to_string()),
                                }
                                ctx.active = if as_bool(&v, "continuous", false) { Some(cmd) } else { None };
                                Ok(json!({"op":"mit","continuous": as_bool(&v, "continuous", false)}))
                            }
                            "pos_vel" | "pos-vel" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::PosVel { pos: as_f32(&v, "pos", 0.0), vlim: as_f32(&v, "vlim", 1.0)};
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        m.ensure_control_mode(DamiaoControlMode::PosVel, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                            .map_err(|e| e.to_string())?;
                                        if let ActiveCommand::PosVel{pos,vlim} = cmd {
                                            m.send_cmd_pos_vel(pos,vlim).map_err(|e| e.to_string())?;
                                        }
                                        ctx.active = if as_bool(&v, "continuous", false) { Some(cmd) } else { None };
                                        Ok(json!({"op":"pos_vel","continuous": as_bool(&v, "continuous", false)}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("pos_vel is not supported for robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "vel" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::Vel { vel: as_f32(&v, "vel", 0.0)};
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        m.ensure_control_mode(DamiaoControlMode::Vel, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                            .map_err(|e| e.to_string())?;
                                        if let ActiveCommand::Vel{vel} = cmd {
                                            m.send_cmd_vel(vel).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    Some(MotorHandle::Robstride(m)) => {
                                        m.set_mode(RobstrideControlMode::Velocity).map_err(|e| e.to_string())?;
                                        if let ActiveCommand::Vel{vel} = cmd {
                                            m.set_velocity_target(vel).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    None => return Err("motor not connected".to_string()),
                                }
                                ctx.active = if as_bool(&v, "continuous", false) { Some(cmd) } else { None };
                                Ok(json!({"op":"vel","continuous": as_bool(&v, "continuous", false)}))
                            }
                            "force_pos" | "force-pos" => {
                                ctx.ensure_connected()?;
                                let cmd = ActiveCommand::ForcePos {
                                    pos: as_f32(&v, "pos", 0.0),
                                    vlim: as_f32(&v, "vlim", 1.0),
                                    ratio: as_f32(&v, "ratio", 0.3),
                                };
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        m.ensure_control_mode(DamiaoControlMode::ForcePos, Duration::from_millis(as_u64(&v,"ensure_timeout_ms",1000)))
                                            .map_err(|e| e.to_string())?;
                                        if let ActiveCommand::ForcePos{pos,vlim,ratio} = cmd {
                                            m.send_cmd_force_pos(pos,vlim,ratio).map_err(|e| e.to_string())?;
                                        }
                                        ctx.active = if as_bool(&v, "continuous", false) { Some(cmd) } else { None };
                                        Ok(json!({"op":"force_pos","continuous": as_bool(&v, "continuous", false)}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("force_pos is not supported for robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "stop" => {
                                ctx.active = None;
                                Ok(json!({"stopped": true}))
                            }
                            "state_once" => {
                                ctx.ensure_connected()?;
                                Ok(json!({"state": ctx.build_state_snapshot()?}))
                            }
                            "clear_error" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => m.clear_error().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Robstride(_)) => return Err("clear_error is not supported for robstride".to_string()),
                                    None => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"cleared": true}))
                            }
                            "set_zero_position" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => m.set_zero_position().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Robstride(m)) => m.set_zero_position().map_err(|e| e.to_string())?,
                                    None => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"zero_set": true}))
                            }
                            "ensure_mode" => {
                                ctx.ensure_connected()?;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        let mode = parse_damiao_mode(&v)?;
                                        m.ensure_control_mode(mode, Duration::from_millis(timeout_ms))
                                            .map_err(|e| e.to_string())?;
                                    }
                                    Some(MotorHandle::Robstride(m)) => {
                                        let mode = parse_robstride_mode(&v)?;
                                        m.set_mode(mode).map_err(|e| e.to_string())?;
                                    }
                                    None => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"ensured": true}))
                            }
                            "request_feedback" => {
                                ctx.ensure_connected()?;
                                match (&ctx.controller, &ctx.motor) {
                                    (Some(ControllerHandle::Damiao(_)), Some(MotorHandle::Damiao(m))) => {
                                        m.request_motor_feedback().map_err(|e| e.to_string())?;
                                    }
                                    (Some(ControllerHandle::Robstride(c)), Some(MotorHandle::Robstride(_))) => {
                                        c.poll_feedback_once().map_err(|e| e.to_string())?;
                                    }
                                    _ => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"requested": true}))
                            }
                            "store_parameters" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => m.store_parameters().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Robstride(m)) => m.save_parameters().map_err(|e| e.to_string())?,
                                    None => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"stored": true}))
                            }
                            "set_can_timeout_ms" => {
                                ctx.ensure_connected()?;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        let reg_value = (timeout_ms as u32).saturating_mul(20);
                                        m.write_register_u32(9, reg_value).map_err(|e| e.to_string())?;
                                        Ok(json!({"timeout_ms": timeout_ms, "reg9_value": reg_value}))
                                    }
                                    Some(MotorHandle::Robstride(m)) => {
                                        m.write_parameter(0x7028, RobstrideParameterValue::U32(timeout_ms as u32)).map_err(|e| e.to_string())?;
                                        Ok(json!({"timeout_ms": timeout_ms, "param_id":"0x7028"}))
                                    }
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "write_register_u32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let value = as_u64(&v, "value", 0) as u32;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        m.write_register_u32(rid, value).map_err(|e| e.to_string())?;
                                        Ok(json!({"rid": rid, "value": value}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("write_register_u32 is damiao-only; use robstride_write_param".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "write_register_f32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let value = as_f32(&v, "value", 0.0);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        m.write_register_f32(rid, value).map_err(|e| e.to_string())?;
                                        Ok(json!({"rid": rid, "value": value}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("write_register_f32 is damiao-only; use robstride_write_param".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "get_register_u32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        let val = m
                                            .get_register_u32(rid, Duration::from_millis(timeout_ms))
                                            .map_err(|e| e.to_string())?;
                                        Ok(json!({"rid": rid, "value": val}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("get_register_u32 is damiao-only; use robstride_read_param".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "get_register_f32" => {
                                ctx.ensure_connected()?;
                                let rid = as_u16(&v, "rid", 0) as u8;
                                let timeout_ms = as_u64(&v, "timeout_ms", 1000);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => {
                                        let val = m
                                            .get_register_f32(rid, Duration::from_millis(timeout_ms))
                                            .map_err(|e| e.to_string())?;
                                        Ok(json!({"rid": rid, "value": val}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("get_register_f32 is damiao-only; use robstride_read_param".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "robstride_ping" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Robstride(m)) => {
                                        let p = m.ping(Duration::from_millis(as_u64(&v, "timeout_ms", 200))).map_err(|e| e.to_string())?;
                                        Ok(json!({"device_id": p.device_id, "responder_id": p.responder_id}))
                                    }
                                    Some(MotorHandle::Damiao(_)) => Err("robstride_ping requires vendor=robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "robstride_read_param" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Robstride(m)) => handle_robstride_read_param(m, &v),
                                    Some(MotorHandle::Damiao(_)) => Err("robstride_read_param requires vendor=robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "robstride_write_param" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Robstride(m)) => handle_robstride_write_param(m, &v),
                                    Some(MotorHandle::Damiao(_)) => Err("robstride_write_param requires vendor=robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "poll_feedback_once" => {
                                ctx.ensure_connected()?;
                                if let Some(c) = ctx.controller.as_ref() {
                                    match c {
                                        ControllerHandle::Damiao(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
                                        ControllerHandle::Robstride(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
                                    }
                                }
                                Ok(json!({"polled": true}))
                            }
                            "shutdown" => {
                                if let Some(c) = ctx.controller.as_ref() {
                                    match c {
                                        ControllerHandle::Damiao(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
                                        ControllerHandle::Robstride(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
                                    }
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
                if ctx.motor.is_some() {
                    match ctx.build_state_snapshot() {
                        Ok(st) => send_json(&mut tx, json!({"type":"state", "data": st})).await?,
                        Err(err) => send_json(&mut tx, json!({"ok": false, "op":"state_tick","error": err})).await?,
                    }
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
        "ws_gateway listening on ws://{} (vendor={}, channel={}, model={}, motor_id=0x{:X}, feedback_id=0x{:X}, dt_ms={})",
        cfg.bind,
        cfg.target.vendor.as_str(),
        cfg.target.channel,
        cfg.target.model,
        cfg.target.motor_id,
        cfg.target.feedback_id,
        cfg.dt_ms
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
