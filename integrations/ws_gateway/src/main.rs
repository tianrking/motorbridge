use futures_util::{SinkExt, StreamExt};
use motor_vendor_damiao::{ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor};
use motor_vendor_robstride::{
    ControlMode as RobstrideControlMode, ParameterValue as RobstrideParameterValue,
    RobstrideController, RobstrideMotor,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

mod ops;
use ops::{
    as_bool, as_f32, as_u16, as_u64, cmd_scan, cmd_set_id, cmd_verify, handle_robstride_read_param,
    handle_robstride_write_param, parse_args, parse_damiao_mode, parse_robstride_mode,
    parse_vendor_in_msg,
};

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
    PosVel {
        pos: f32,
        vlim: f32,
    },
    Vel {
        vel: f32,
    },
    ForcePos {
        pos: f32,
        vlim: f32,
        ratio: f32,
    },
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
                    .add_motor(
                        self.target.motor_id,
                        self.target.feedback_id,
                        &self.target.model,
                    )
                    .map_err(|e| format!("add motor failed: {e}"))?;
                self.controller = Some(ControllerHandle::Damiao(ctrl));
                self.motor = Some(MotorHandle::Damiao(motor));
            }
            Vendor::Robstride => {
                let ctrl = RobstrideController::new_socketcan(&self.target.channel)
                    .map_err(|e| format!("open bus failed: {e}"))?;
                let motor = ctrl
                    .add_motor(
                        self.target.motor_id,
                        self.target.feedback_id,
                        &self.target.model,
                    )
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
                Some(ActiveCommand::Vel { vel }) => {
                    motor.send_cmd_vel(*vel).map_err(|e| e.to_string())
                }
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
                Some(ActiveCommand::Vel { vel }) => {
                    motor.set_velocity_target(*vel).map_err(|e| e.to_string())
                }
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

async fn send_json<S>(tx: &mut S, obj: Value) -> Result<(), String>
where
    S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    tx.send(Message::Text(obj.to_string().into()))
        .await
        .map_err(|e| e.to_string())
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
