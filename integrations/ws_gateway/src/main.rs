use futures_util::{SinkExt, StreamExt};
use motor_vendor_damiao::{ControlMode as DamiaoControlMode, DamiaoController, DamiaoMotor};
use motor_vendor_hexfellow::{
    HexfellowController, HexfellowMotor, MitTarget as HexfellowMitTarget,
    PosVelTarget as HexfellowPosVelTarget,
};
use motor_vendor_myactuator::{MyActuatorController, MyActuatorMotor};
use motor_vendor_robstride::{
    ControlMode as RobstrideControlMode, ParameterValue as RobstrideParameterValue,
    RobstrideController, RobstrideMotor,
};
use motor_core::bus::{CanBus, CanFrame};
#[cfg(target_os = "windows")]
use motor_core::pcan::PcanBus;
#[cfg(target_os = "linux")]
use motor_core::socketcan::SocketCanBus;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::time;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

mod ops;
use ops::{
    as_bool, as_f32, as_u16, as_u64, cmd_scan, cmd_set_id, cmd_verify, handle_robstride_read_param,
    handle_robstride_write_param, parse_args, parse_damiao_mode, parse_robstride_mode,
    parse_transport_in_msg, parse_vendor_in_msg,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Vendor {
    Damiao,
    Hexfellow,
    Hightorque,
    Myactuator,
    Robstride,
}

impl Vendor {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "damiao" => Ok(Self::Damiao),
            "hexfellow" => Ok(Self::Hexfellow),
            "hightorque" => Ok(Self::Hightorque),
            "myactuator" => Ok(Self::Myactuator),
            "robstride" => Ok(Self::Robstride),
            _ => Err(format!("unsupported vendor: {s}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Damiao => "damiao",
            Self::Hexfellow => "hexfellow",
            Self::Hightorque => "hightorque",
            Self::Myactuator => "myactuator",
            Self::Robstride => "robstride",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Transport {
    Auto,
    SocketCan,
    SocketCanFd,
    DmSerial,
}

impl Transport {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "socketcan" => Ok(Self::SocketCan),
            "socketcanfd" => Ok(Self::SocketCanFd),
            "dm-serial" => Ok(Self::DmSerial),
            _ => Err(format!("unsupported transport: {s}")),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::SocketCan => "socketcan",
            Self::SocketCanFd => "socketcanfd",
            Self::DmSerial => "dm-serial",
        }
    }
}

#[derive(Clone, Debug)]
struct Target {
    vendor: Vendor,
    transport: Transport,
    channel: String,
    serial_port: String,
    serial_baud: u32,
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
    Hexfellow(HexfellowController),
    Hightorque(Box<dyn CanBus>),
    Myactuator(MyActuatorController),
    Robstride(RobstrideController),
}

enum MotorHandle {
    Damiao(Arc<DamiaoMotor>),
    Hexfellow(Arc<HexfellowMotor>),
    Hightorque(u16),
    Myactuator(Arc<MyActuatorMotor>),
    Robstride(Arc<RobstrideMotor>),
}

struct SessionCtx {
    target: Target,
    controller: Option<ControllerHandle>,
    motor: Option<MotorHandle>,
    active: Option<ActiveCommand>,
}

const TWO_PI: f32 = std::f32::consts::PI * 2.0;

fn myactuator_feedback_default(motor_id: u16) -> u16 {
    0x240u16.saturating_add(motor_id)
}

#[derive(Debug, Clone, Copy)]
struct HighTorqueStatus {
    motor_id: u16,
    pos_raw: i16,
    vel_raw: i16,
    tqe_raw: i16,
}

impl HighTorqueStatus {
    fn pos_rad(self) -> f32 {
        self.pos_raw as f32 * 0.0001 * TWO_PI
    }

    fn vel_rad_s(self) -> f32 {
        self.vel_raw as f32 * 0.00025 * TWO_PI
    }
}

fn can_ext_id_for_motor(motor_id: u16) -> u32 {
    u32::from(0x8000u16 | motor_id)
}

fn send_hightorque_ext(bus: &dyn CanBus, motor_id: u16, payload: &[u8]) -> Result<(), String> {
    if payload.len() > 8 {
        return Err("payload too long (max 8 bytes)".to_string());
    }
    let mut data = [0u8; 8];
    data[..payload.len()].copy_from_slice(payload);
    bus.send(CanFrame {
        arbitration_id: can_ext_id_for_motor(motor_id),
        data,
        dlc: payload.len() as u8,
        is_extended: true,
        is_rx: false,
    })
    .map_err(|e| e.to_string())
}

fn decode_hightorque_read_reply(frame: CanFrame) -> Option<HighTorqueStatus> {
    if frame.dlc < 8 {
        return None;
    }
    if frame.data[0] != 0x27 || frame.data[1] != 0x01 {
        return None;
    }
    let motor_id = if !frame.is_extended && (frame.arbitration_id & 0x00FF) == 0 {
        ((frame.arbitration_id >> 8) & 0x7F) as u16
    } else {
        (frame.arbitration_id & 0x7FF) as u16
    };
    Some(HighTorqueStatus {
        motor_id,
        pos_raw: i16::from_le_bytes([frame.data[2], frame.data[3]]),
        vel_raw: i16::from_le_bytes([frame.data[4], frame.data[5]]),
        tqe_raw: i16::from_le_bytes([frame.data[6], frame.data[7]]),
    })
}

fn wait_hightorque_status_for_motor(
    bus: &dyn CanBus,
    motor_id: u16,
    timeout: Duration,
) -> Result<Option<HighTorqueStatus>, String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if let Some(frame) = bus
            .recv(left.min(Duration::from_millis(20)))
            .map_err(|e| e.to_string())?
        {
            if let Some(status) = decode_hightorque_read_reply(frame) {
                if status.motor_id == motor_id {
                    return Ok(Some(status));
                }
            }
        }
    }
    Ok(None)
}

fn pos_raw_from_rad(rad: f32) -> i16 {
    (rad / TWO_PI * 10_000.0).round() as i16
}

fn vel_raw_from_rad_s(rad_s: f32) -> i16 {
    (rad_s / TWO_PI / 0.00025).round() as i16
}

fn tqe_raw_from_tau(tau: f32) -> i16 {
    (tau * 100.0).round() as i16
}

fn open_hightorque_bus(target: &Target) -> Result<Box<dyn CanBus>, String> {
    match target.transport {
        Transport::Auto | Transport::SocketCan => {
            #[cfg(target_os = "linux")]
            {
                return Ok(Box::new(
                    SocketCanBus::open(&target.channel).map_err(|e| format!("open bus failed: {e}"))?,
                ));
            }
            #[cfg(target_os = "windows")]
            {
                return Ok(Box::new(
                    PcanBus::open(&target.channel).map_err(|e| format!("open bus failed: {e}"))?,
                ));
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                Err("No CAN backend for current platform".to_string())
            }
        }
        Transport::SocketCanFd => {
            Err("hightorque currently uses standard CAN transport only".to_string())
        }
        Transport::DmSerial => Err("dm-serial transport is damiao-only".to_string()),
    }
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
                let ctrl = match self.target.transport {
                    Transport::Auto | Transport::SocketCan => {
                        DamiaoController::new_socketcan(&self.target.channel)
                    }
                    Transport::SocketCanFd => DamiaoController::new_socketcanfd(&self.target.channel),
                    Transport::DmSerial => {
                        DamiaoController::new_dm_serial(&self.target.serial_port, self.target.serial_baud)
                    }
                }
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
            Vendor::Hexfellow => {
                if !matches!(self.target.transport, Transport::Auto | Transport::SocketCanFd) {
                    return Err("hexfellow requires transport socketcanfd (or auto)".to_string());
                }
                let ctrl = HexfellowController::new_socketcanfd(&self.target.channel)
                    .map_err(|e| format!("open bus failed: {e}"))?;
                let motor = ctrl
                    .add_motor(
                        self.target.motor_id,
                        self.target.feedback_id,
                        &self.target.model,
                    )
                    .map_err(|e| format!("add motor failed: {e}"))?;
                self.controller = Some(ControllerHandle::Hexfellow(ctrl));
                self.motor = Some(MotorHandle::Hexfellow(motor));
            }
            Vendor::Hightorque => {
                let bus = open_hightorque_bus(&self.target)?;
                self.controller = Some(ControllerHandle::Hightorque(bus));
                self.motor = Some(MotorHandle::Hightorque(self.target.motor_id));
            }
            Vendor::Myactuator => {
                let ctrl = match self.target.transport {
                    Transport::Auto | Transport::SocketCan => {
                        MyActuatorController::new_socketcan(&self.target.channel)
                    }
                    Transport::SocketCanFd => {
                        MyActuatorController::new_socketcanfd(&self.target.channel)
                    }
                    Transport::DmSerial => Err(motor_core::error::MotorError::InvalidArgument(
                        "dm-serial transport is damiao-only".to_string(),
                    )),
                }
                .map_err(|e| format!("open bus failed: {e}"))?;
                let fid = if self.target.feedback_id == 0 {
                    myactuator_feedback_default(self.target.motor_id)
                } else {
                    self.target.feedback_id
                };
                let motor = ctrl
                    .add_motor(self.target.motor_id, fid, &self.target.model)
                    .map_err(|e| format!("add motor failed: {e}"))?;
                self.controller = Some(ControllerHandle::Myactuator(ctrl));
                self.motor = Some(MotorHandle::Myactuator(motor));
            }
            Vendor::Robstride => {
                let ctrl = match self.target.transport {
                    Transport::Auto | Transport::SocketCan => {
                        RobstrideController::new_socketcan(&self.target.channel)
                    }
                    Transport::SocketCanFd => {
                        RobstrideController::new_socketcanfd(&self.target.channel)
                    }
                    Transport::DmSerial => Err(motor_core::error::MotorError::InvalidArgument(
                        "dm-serial transport is damiao-only".to_string(),
                    )),
                }
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
                ControllerHandle::Hexfellow(c) => {
                    if shutdown {
                        let _ = c.shutdown();
                    } else {
                        let _ = c.close_bus();
                    }
                }
                ControllerHandle::Hightorque(bus) => {
                    let _ = bus.shutdown();
                }
                ControllerHandle::Myactuator(c) => {
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
            Some(MotorHandle::Hexfellow(motor)) => match self.active.as_ref() {
                Some(ActiveCommand::Mit {
                    pos,
                    vel,
                    kp,
                    kd,
                    tau,
                }) => motor
                    .command_mit(
                        HexfellowMitTarget {
                            position_rev: *pos / TWO_PI,
                            velocity_rev_s: *vel / TWO_PI,
                            torque_nm: *tau,
                            kp: kp.clamp(0.0, u16::MAX as f32).round() as u16,
                            kd: kd.clamp(0.0, u16::MAX as f32).round() as u16,
                            limit_permille: 1000,
                        },
                        Duration::from_millis(300),
                    )
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::PosVel { pos, vlim }) => motor
                    .command_pos_vel(
                        HexfellowPosVelTarget {
                            position_rev: *pos / TWO_PI,
                            velocity_rev_s: *vlim / TWO_PI,
                        },
                        Duration::from_millis(300),
                    )
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::Vel { .. }) | Some(ActiveCommand::ForcePos { .. }) => {
                    Err("vel/force_pos are not supported for hexfellow".to_string())
                }
                None => Ok(()),
            },
            Some(MotorHandle::Hightorque(motor_id)) => match self.active.as_ref() {
                Some(ActiveCommand::Mit {
                    pos, vel, tau, ..
                }) => {
                    let pos_raw = pos_raw_from_rad(*pos);
                    let vel_raw = vel_raw_from_rad_s(*vel);
                    let tqe_raw = tqe_raw_from_tau(*tau);
                    let mut data = [0x07, 0x35, 0, 0, 0, 0, 0, 0];
                    data[2..4].copy_from_slice(&vel_raw.to_le_bytes());
                    data[4..6].copy_from_slice(&tqe_raw.to_le_bytes());
                    data[6..8].copy_from_slice(&pos_raw.to_le_bytes());
                    match self.controller.as_ref() {
                        Some(ControllerHandle::Hightorque(bus)) => {
                            send_hightorque_ext(bus.as_ref(), *motor_id, &data)
                        }
                        _ => Err("motor not connected".to_string()),
                    }
                }
                Some(ActiveCommand::Vel { vel }) => {
                    let vel_raw = vel_raw_from_rad_s(*vel);
                    let tqe_raw = 0i16;
                    let mut data = [0x07, 0x07, 0x00, 0x80, 0x20, 0x00, 0x80, 0x00];
                    data[4..6].copy_from_slice(&vel_raw.to_le_bytes());
                    data[6..8].copy_from_slice(&tqe_raw.to_le_bytes());
                    match self.controller.as_ref() {
                        Some(ControllerHandle::Hightorque(bus)) => {
                            send_hightorque_ext(bus.as_ref(), *motor_id, &data)
                        }
                        _ => Err("motor not connected".to_string()),
                    }
                }
                Some(ActiveCommand::PosVel { .. }) | Some(ActiveCommand::ForcePos { .. }) => {
                    Err("pos_vel/force_pos are not supported for hightorque".to_string())
                }
                None => Ok(()),
            },
            Some(MotorHandle::Myactuator(motor)) => match self.active.as_ref() {
                Some(ActiveCommand::Vel { vel }) => motor
                    .send_velocity_setpoint(vel.to_degrees())
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::Mit { .. })
                | Some(ActiveCommand::PosVel { .. })
                | Some(ActiveCommand::ForcePos { .. }) => {
                    Err("active command not supported for myactuator".to_string())
                }
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
            (Some(ControllerHandle::Hexfellow(_)), Some(MotorHandle::Hexfellow(motor))) => {
                match motor.query_status(Duration::from_millis(200)) {
                    Ok(s) => Ok(json!({
                        "vendor": "hexfellow",
                        "has_value": true,
                        "mode_display": s.mode_display,
                        "statusword": s.statusword,
                        "pos": s.position_rev * TWO_PI,
                        "vel": s.velocity_rev_s * TWO_PI,
                        "torq": s.torque_permille as f32 / 1000.0,
                        "status_code": s.heartbeat_state.unwrap_or(0),
                    })),
                    Err(_) => Ok(json!({"vendor":"hexfellow","has_value": false})),
                }
            }
            (Some(ControllerHandle::Hightorque(bus)), Some(MotorHandle::Hightorque(motor_id))) => {
                let _ = send_hightorque_ext(bus.as_ref(), *motor_id, &[0x17, 0x01, 0, 0, 0, 0, 0, 0]);
                match wait_hightorque_status_for_motor(bus.as_ref(), *motor_id, Duration::from_millis(50)) {
                    Ok(Some(s)) => Ok(json!({
                        "vendor":"hightorque",
                        "has_value": true,
                        "motor_id": s.motor_id,
                        "pos_raw": s.pos_raw,
                        "vel_raw": s.vel_raw,
                        "tqe_raw": s.tqe_raw,
                        "pos": s.pos_rad(),
                        "vel": s.vel_rad_s(),
                        "torq": s.tqe_raw as f32 / 100.0,
                        "status_code": 0
                    })),
                    _ => Ok(json!({"vendor":"hightorque","has_value": false})),
                }
            }
            (Some(ControllerHandle::Myactuator(ctrl)), Some(MotorHandle::Myactuator(motor))) => {
                let _ = motor.request_status();
                let _ = motor.request_multi_turn_angle();
                let _ = ctrl.poll_feedback_once();
                if let Some(s) = motor.latest_state() {
                    Ok(json!({
                        "vendor":"myactuator",
                        "has_value": true,
                        "arbitration_id": s.arbitration_id,
                        "status_code": s.command,
                        "pos": s.shaft_angle_deg.to_radians(),
                        "vel": s.speed_dps.to_radians(),
                        "torq": s.current_a,
                        "t_mos": s.temperature_c,
                    }))
                } else {
                    Ok(json!({"vendor":"myactuator","has_value": false}))
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
                    "transport": ctx.target.transport.as_str(),
                    "channel": ctx.target.channel,
                    "serial_port": ctx.target.serial_port,
                    "serial_baud": ctx.target.serial_baud,
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
                                    Vendor::Hexfellow => Ok(json!({"pong": true, "vendor":"hexfellow"})),
                                    Vendor::Myactuator => Ok(json!({"pong": true, "vendor":"myactuator"})),
                                    Vendor::Hightorque => Ok(json!({"pong": true, "vendor":"hightorque"})),
                                }
                            }
                            "set_target" => {
                                let mut next = ctx.target.clone();
                                next.vendor = parse_vendor_in_msg(&v, next.vendor)?;
                                next.transport = parse_transport_in_msg(&v, next.transport)?;
                                next.channel = v.get("channel").and_then(Value::as_str).unwrap_or(&next.channel).to_string();
                                next.serial_port = v.get("serial_port").and_then(Value::as_str).unwrap_or(&next.serial_port).to_string();
                                next.serial_baud = as_u64(&v, "serial_baud", next.serial_baud as u64) as u32;
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
                                } else if next.vendor == Vendor::Myactuator {
                                    if next.model == "4340" || next.model == "4340P" {
                                        next.model = "X8".to_string();
                                    }
                                    if next.feedback_id == 0x11 {
                                        next.feedback_id = 0x241;
                                    }
                                } else if next.vendor == Vendor::Hexfellow {
                                    if next.model == "4340" || next.model == "4340P" {
                                        next.model = "hexfellow".to_string();
                                    }
                                    if next.feedback_id == 0x11 {
                                        next.feedback_id = 0;
                                    }
                                } else if next.vendor == Vendor::Hightorque {
                                    if next.model == "4340" || next.model == "4340P" {
                                        next.model = "hightorque".to_string();
                                    }
                                    if next.feedback_id == 0x11 {
                                        next.feedback_id = 0x01;
                                    }
                                }
                                ctx.target = next;
                                ctx.active = None;
                                ctx.connect()?;
                                Ok(json!({
                                    "vendor": ctx.target.vendor.as_str(),
                                    "transport": ctx.target.transport.as_str(),
                                    "channel": ctx.target.channel,
                                    "serial_port": ctx.target.serial_port,
                                    "serial_baud": ctx.target.serial_baud,
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
                                        ControllerHandle::Hexfellow(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
                                        ControllerHandle::Hightorque(_) => {}
                                        ControllerHandle::Myactuator(ctrl) => ctrl.enable_all().map_err(|e| e.to_string())?,
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
                                        ControllerHandle::Hexfellow(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
                                        ControllerHandle::Hightorque(_) => {}
                                        ControllerHandle::Myactuator(ctrl) => ctrl.disable_all().map_err(|e| e.to_string())?,
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
                                    Some(MotorHandle::Hexfellow(m)) => {
                                        if let ActiveCommand::Mit{pos,vel,kp,kd,tau} = cmd {
                                            m.command_mit(
                                                HexfellowMitTarget {
                                                    position_rev: pos / TWO_PI,
                                                    velocity_rev_s: vel / TWO_PI,
                                                    torque_nm: tau,
                                                    kp: kp.clamp(0.0, u16::MAX as f32).round() as u16,
                                                    kd: kd.clamp(0.0, u16::MAX as f32).round() as u16,
                                                    limit_permille: 1000,
                                                },
                                                Duration::from_millis(300),
                                            ).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    Some(MotorHandle::Hightorque(mid)) => {
                                        if let ActiveCommand::Mit{pos,vel,tau,..} = cmd {
                                            let pos_raw = pos_raw_from_rad(pos);
                                            let vel_raw = vel_raw_from_rad_s(vel);
                                            let tqe_raw = tqe_raw_from_tau(tau);
                                            let mut data = [0x07, 0x35, 0, 0, 0, 0, 0, 0];
                                            data[2..4].copy_from_slice(&vel_raw.to_le_bytes());
                                            data[4..6].copy_from_slice(&tqe_raw.to_le_bytes());
                                            data[6..8].copy_from_slice(&pos_raw.to_le_bytes());
                                            if let Some(ControllerHandle::Hightorque(bus)) = ctx.controller.as_ref() {
                                                send_hightorque_ext(bus.as_ref(), *mid, &data)?;
                                            }
                                        }
                                    }
                                    Some(MotorHandle::Myactuator(_)) => {
                                        return Err("mit is not supported for myactuator".to_string());
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
                                    Some(MotorHandle::Hexfellow(m)) => {
                                        if let ActiveCommand::PosVel{pos,vlim} = cmd {
                                            m.command_pos_vel(
                                                HexfellowPosVelTarget {
                                                    position_rev: pos / TWO_PI,
                                                    velocity_rev_s: vlim / TWO_PI,
                                                },
                                                Duration::from_millis(300),
                                            ).map_err(|e| e.to_string())?;
                                        }
                                        ctx.active = if as_bool(&v, "continuous", false) { Some(cmd) } else { None };
                                        Ok(json!({"op":"pos_vel","continuous": as_bool(&v, "continuous", false)}))
                                    }
                                    Some(MotorHandle::Robstride(_)) => Err("pos_vel is not supported for robstride".to_string()),
                                    Some(MotorHandle::Hightorque(_)) => Err("pos_vel is not supported for hightorque".to_string()),
                                    Some(MotorHandle::Myactuator(_)) => Err("pos_vel is not supported for myactuator".to_string()),
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
                                    Some(MotorHandle::Myactuator(m)) => {
                                        if let ActiveCommand::Vel{vel} = cmd {
                                            m.send_velocity_setpoint(vel.to_degrees()).map_err(|e| e.to_string())?;
                                        }
                                    }
                                    Some(MotorHandle::Hightorque(mid)) => {
                                        if let ActiveCommand::Vel{vel} = cmd {
                                            let vel_raw = vel_raw_from_rad_s(vel);
                                            let tqe_raw = 0i16;
                                            let mut data = [0x07, 0x07, 0x00, 0x80, 0x20, 0x00, 0x80, 0x00];
                                            data[4..6].copy_from_slice(&vel_raw.to_le_bytes());
                                            data[6..8].copy_from_slice(&tqe_raw.to_le_bytes());
                                            if let Some(ControllerHandle::Hightorque(bus)) = ctx.controller.as_ref() {
                                                send_hightorque_ext(bus.as_ref(), *mid, &data)?;
                                            }
                                        }
                                    }
                                    Some(MotorHandle::Hexfellow(_)) => {
                                        return Err("vel is not supported for hexfellow; use pos_vel or mit".to_string());
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
                                    Some(MotorHandle::Hexfellow(_)) => Err("force_pos is not supported for hexfellow".to_string()),
                                    Some(MotorHandle::Hightorque(_)) => Err("force_pos is not supported for hightorque".to_string()),
                                    Some(MotorHandle::Myactuator(_)) => Err("force_pos is not supported for myactuator".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "stop" => {
                                ctx.active = None;
                                if let Some(m) = ctx.motor.as_ref() {
                                    match m {
                                        MotorHandle::Damiao(mm) => {
                                            mm.send_cmd_vel(0.0).map_err(|e| e.to_string())?;
                                        }
                                        MotorHandle::Hexfellow(mm) => {
                                            mm.command_mit(
                                                HexfellowMitTarget {
                                                    position_rev: 0.0,
                                                    velocity_rev_s: 0.0,
                                                    torque_nm: 0.0,
                                                    kp: 0,
                                                    kd: 0,
                                                    limit_permille: 1000,
                                                },
                                                Duration::from_millis(200),
                                            ).map_err(|e| e.to_string())?;
                                        }
                                        MotorHandle::Hightorque(mid) => {
                                            if let Some(ControllerHandle::Hightorque(bus)) = ctx.controller.as_ref() {
                                                send_hightorque_ext(bus.as_ref(), *mid, &[0x01, 0x00, 0x00])?;
                                            }
                                        }
                                        MotorHandle::Myactuator(mm) => {
                                            mm.stop_motor().map_err(|e| e.to_string())?;
                                        }
                                        MotorHandle::Robstride(mm) => {
                                            mm.set_velocity_target(0.0).map_err(|e| e.to_string())?;
                                        }
                                    }
                                }
                                Ok(json!({"stopped": true}))
                            }
                            "state_once" => {
                                ctx.ensure_connected()?;
                                Ok(json!({"state": ctx.build_state_snapshot()?}))
                            }
                            "status" => {
                                ctx.ensure_connected()?;
                                match (&ctx.controller, &ctx.motor) {
                                    (Some(ControllerHandle::Myactuator(c)), Some(MotorHandle::Myactuator(m))) => {
                                        m.request_status().map_err(|e| e.to_string())?;
                                        m.request_multi_turn_angle().map_err(|e| e.to_string())?;
                                        c.poll_feedback_once().map_err(|e| e.to_string())?;
                                    }
                                    (Some(ControllerHandle::Hexfellow(_)), Some(MotorHandle::Hexfellow(_)))
                                    | (Some(ControllerHandle::Damiao(_)), Some(MotorHandle::Damiao(_)))
                                    | (Some(ControllerHandle::Robstride(_)), Some(MotorHandle::Robstride(_)))
                                    | (Some(ControllerHandle::Hightorque(_)), Some(MotorHandle::Hightorque(_))) => {}
                                    _ => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"state": ctx.build_state_snapshot()?}))
                            }
                            "current" => {
                                ctx.ensure_connected()?;
                                let current = as_f32(&v, "current", 0.0);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Myactuator(m)) => {
                                        m.send_current_setpoint(current).map_err(|e| e.to_string())?;
                                        Ok(json!({"op":"current", "current": current}))
                                    }
                                    Some(_) => Err("current is supported for myactuator only".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "pos" => {
                                ctx.ensure_connected()?;
                                let pos = as_f32(&v, "pos", 0.0);
                                let max_speed = as_f32(&v, "max_speed", 8.726646);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Myactuator(m)) => {
                                        m.send_position_absolute_setpoint(pos.to_degrees(), max_speed.to_degrees())
                                            .map_err(|e| e.to_string())?;
                                        Ok(json!({"op":"pos", "pos": pos, "max_speed": max_speed}))
                                    }
                                    Some(_) => Err("pos is supported for myactuator only".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "version" => {
                                ctx.ensure_connected()?;
                                let timeout_ms = as_u64(&v, "timeout_ms", 500);
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Myactuator(m)) => {
                                        m.request_version_date().map_err(|e| e.to_string())?;
                                        let version = m.await_version_date(Duration::from_millis(timeout_ms)).map_err(|e| e.to_string())?;
                                        Ok(json!({"version": version}))
                                    }
                                    Some(_) => Err("version is supported for myactuator only".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "mode_query" | "mode-query" => {
                                ctx.ensure_connected()?;
                                match (&ctx.controller, &ctx.motor) {
                                    (Some(ControllerHandle::Myactuator(c)), Some(MotorHandle::Myactuator(m))) => {
                                        m.request_control_mode().map_err(|e| e.to_string())?;
                                        c.poll_feedback_once().map_err(|e| e.to_string())?;
                                        Ok(json!({"mode": m.latest_control_mode()}))
                                    }
                                    (Some(_), Some(_)) => Err("mode_query is supported for myactuator only".to_string()),
                                    _ => Err("motor not connected".to_string()),
                                }
                            }
                            "read" => {
                                ctx.ensure_connected()?;
                                match (&ctx.controller, &ctx.motor) {
                                    (Some(ControllerHandle::Hightorque(bus)), Some(MotorHandle::Hightorque(mid))) => {
                                        send_hightorque_ext(bus.as_ref(), *mid, &[0x17, 0x01, 0, 0, 0, 0, 0, 0])?;
                                        if let Some(s) = wait_hightorque_status_for_motor(bus.as_ref(), *mid, Duration::from_millis(as_u64(&v, "timeout_ms", 500)))? {
                                            Ok(json!({
                                                "motor_id": s.motor_id,
                                                "pos_raw": s.pos_raw,
                                                "vel_raw": s.vel_raw,
                                                "tqe_raw": s.tqe_raw,
                                                "pos": s.pos_rad(),
                                                "vel": s.vel_rad_s(),
                                                "torq": s.tqe_raw as f32 / 100.0
                                            }))
                                        } else {
                                            Err("hightorque read timeout".to_string())
                                        }
                                    }
                                    (Some(_), Some(_)) => Err("read op is reserved for hightorque".to_string()),
                                    _ => Err("motor not connected".to_string()),
                                }
                            }
                            "clear_error" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => m.clear_error().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Robstride(_)) => return Err("clear_error is not supported for robstride".to_string()),
                                    Some(MotorHandle::Hexfellow(_)) => return Err("clear_error is not supported for hexfellow".to_string()),
                                    Some(MotorHandle::Hightorque(_)) => return Err("clear_error is not supported for hightorque".to_string()),
                                    Some(MotorHandle::Myactuator(_)) => return Err("clear_error is not supported for myactuator".to_string()),
                                    None => return Err("motor not connected".to_string()),
                                }
                                Ok(json!({"cleared": true}))
                            }
                            "set_zero_position" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Damiao(m)) => m.set_zero_position().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Robstride(m)) => m.set_zero_position().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Myactuator(m)) => m.set_current_position_as_zero().map_err(|e| e.to_string())?,
                                    Some(MotorHandle::Hexfellow(_)) => return Err("set_zero_position is not supported for hexfellow".to_string()),
                                    Some(MotorHandle::Hightorque(_)) => return Err("set_zero_position is not supported for hightorque".to_string()),
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
                                    Some(MotorHandle::Hexfellow(m)) => {
                                        let mode = v.get("mode").and_then(Value::as_str).unwrap_or("mit").to_lowercase();
                                        let raw_mode = if mode == "mit" || mode == "1" { 5 } else if mode == "pos_vel" || mode == "pos-vel" || mode == "2" { 1 } else { return Err("hexfellow mode must be mit|pos_vel".to_string()) };
                                        m.ensure_mode_enabled(raw_mode, Duration::from_millis(timeout_ms)).map_err(|e| e.to_string())?;
                                    }
                                    Some(MotorHandle::Myactuator(_)) => {
                                        return Err("ensure_mode is not supported for myactuator".to_string());
                                    }
                                    Some(MotorHandle::Hightorque(_)) => {
                                        return Err("ensure_mode is not supported for hightorque".to_string());
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
                                    (Some(ControllerHandle::Hexfellow(c)), Some(MotorHandle::Hexfellow(_))) => {
                                        c.poll_feedback_once().map_err(|e| e.to_string())?;
                                    }
                                    (Some(ControllerHandle::Myactuator(c)), Some(MotorHandle::Myactuator(m))) => {
                                        m.request_status().map_err(|e| e.to_string())?;
                                        c.poll_feedback_once().map_err(|e| e.to_string())?;
                                    }
                                    (Some(ControllerHandle::Hightorque(bus)), Some(MotorHandle::Hightorque(mid))) => {
                                        send_hightorque_ext(bus.as_ref(), *mid, &[0x17, 0x01, 0, 0, 0, 0, 0, 0])?;
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
                                    Some(MotorHandle::Hexfellow(_)) => return Err("store_parameters is not supported for hexfellow".to_string()),
                                    Some(MotorHandle::Hightorque(_)) => return Err("store_parameters is not supported for hightorque".to_string()),
                                    Some(MotorHandle::Myactuator(_)) => return Err("store_parameters is not supported for myactuator".to_string()),
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
                                    Some(MotorHandle::Hexfellow(_)) => Err("set_can_timeout_ms is not supported for hexfellow".to_string()),
                                    Some(MotorHandle::Hightorque(_)) => Err("set_can_timeout_ms is not supported for hightorque".to_string()),
                                    Some(MotorHandle::Myactuator(_)) => Err("set_can_timeout_ms is not supported for myactuator".to_string()),
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
                                    Some(_) => Err("write_register_u32 is damiao-only".to_string()),
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
                                    Some(_) => Err("write_register_f32 is damiao-only".to_string()),
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
                                    Some(_) => Err("get_register_u32 is damiao-only".to_string()),
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
                                    Some(_) => Err("get_register_f32 is damiao-only".to_string()),
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
                                    Some(_) => Err("robstride_ping requires vendor=robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "robstride_read_param" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Robstride(m)) => handle_robstride_read_param(m, &v),
                                    Some(MotorHandle::Damiao(_)) => Err("robstride_read_param requires vendor=robstride".to_string()),
                                    Some(_) => Err("robstride_read_param requires vendor=robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "robstride_write_param" => {
                                ctx.ensure_connected()?;
                                match ctx.motor.as_ref() {
                                    Some(MotorHandle::Robstride(m)) => handle_robstride_write_param(m, &v),
                                    Some(MotorHandle::Damiao(_)) => Err("robstride_write_param requires vendor=robstride".to_string()),
                                    Some(_) => Err("robstride_write_param requires vendor=robstride".to_string()),
                                    None => Err("motor not connected".to_string()),
                                }
                            }
                            "poll_feedback_once" => {
                                ctx.ensure_connected()?;
                                if let Some(c) = ctx.controller.as_ref() {
                                    match c {
                                        ControllerHandle::Damiao(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
                                        ControllerHandle::Hexfellow(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
                                        ControllerHandle::Hightorque(_) => {}
                                        ControllerHandle::Myactuator(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
                                        ControllerHandle::Robstride(ctrl) => ctrl.poll_feedback_once().map_err(|e| e.to_string())?,
                                    }
                                }
                                Ok(json!({"polled": true}))
                            }
                            "shutdown" => {
                                if let Some(c) = ctx.controller.as_ref() {
                                    match c {
                                        ControllerHandle::Damiao(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
                                        ControllerHandle::Hexfellow(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
                                        ControllerHandle::Hightorque(bus) => bus.shutdown().map_err(|e| e.to_string())?,
                                        ControllerHandle::Myactuator(ctrl) => ctrl.shutdown().map_err(|e| e.to_string())?,
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
        "ws_gateway listening on ws://{} (vendor={}, transport={}, channel={}, serial_port={}, serial_baud={}, model={}, motor_id=0x{:X}, feedback_id=0x{:X}, dt_ms={})",
        cfg.bind,
        cfg.target.vendor.as_str(),
        cfg.target.transport.as_str(),
        cfg.target.channel,
        cfg.target.serial_port,
        cfg.target.serial_baud,
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
