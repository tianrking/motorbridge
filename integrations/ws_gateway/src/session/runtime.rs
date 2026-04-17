use crate::vendors::hightorque_ws::{
    pos_raw_from_rad, send_hightorque_ext, tqe_raw_from_tau, vel_raw_from_rad_s,
    wait_hightorque_status_for_motor, TWO_PI,
};
use crate::model::{ActiveCommand, ControllerHandle, MotorHandle};
use motor_vendor_hexfellow::{MitTarget as HexfellowMitTarget, PosVelTarget as HexfellowPosVelTarget};
use motor_vendor_robstride::ParameterValue as RobstrideParameterValue;
use serde_json::{json, Value};
use std::time::Duration;

use super::SessionCtx;

impl SessionCtx {
    pub(crate) fn apply_active(&self) -> Result<(), String> {
        match self.motor.as_ref() {
            Some(MotorHandle::Damiao(motor)) => match self.active.as_ref() {
                Some(ActiveCommand::Mit { pos, vel, kp, kd, tau }) => motor
                    .send_cmd_mit(*pos, *vel, *kp, *kd, *tau)
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::PosVel { pos, vlim }) => {
                    motor.send_cmd_pos_vel(*pos, *vlim).map_err(|e| e.to_string())
                }
                Some(ActiveCommand::Vel { vel }) => {
                    motor.send_cmd_vel(*vel).map_err(|e| e.to_string())
                }
                Some(ActiveCommand::ForcePos { pos, vlim, ratio }) => motor
                    .send_cmd_force_pos(*pos, *vlim, *ratio)
                    .map_err(|e| e.to_string()),
                None => Ok(()),
            },
            Some(MotorHandle::Hexfellow(motor)) => match self.active.as_ref() {
                Some(ActiveCommand::Mit { pos, vel, kp, kd, tau }) => motor
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
                Some(ActiveCommand::Mit { pos, vel, tau, .. }) => {
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
                Some(ActiveCommand::Mit { pos, vel, kp, kd, tau }) => motor
                    .send_cmd_mit(*pos, *vel, *kp, *kd, *tau)
                    .map_err(|e| e.to_string()),
                Some(ActiveCommand::Vel { vel }) => {
                    motor.set_velocity_target(*vel).map_err(|e| e.to_string())
                }
                Some(ActiveCommand::PosVel { pos, vlim }) => {
                    let speed = vlim.abs();
                    if speed.is_finite() && speed > 0.0 {
                        motor
                            .write_parameter(0x7017, RobstrideParameterValue::F32(speed))
                            .map_err(|e| e.to_string())?;
                    }
                    motor
                        .write_parameter(0x7016, RobstrideParameterValue::F32(*pos))
                        .map_err(|e| e.to_string())
                }
                Some(ActiveCommand::ForcePos { .. }) => {
                    Err("force_pos is not supported for robstride".to_string())
                }
                None => Ok(()),
            },
            None => Err("motor not connected".to_string()),
        }
    }

    pub(crate) fn build_state_snapshot(&self) -> Result<Value, String> {
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
                let _ =
                    send_hightorque_ext(bus.as_ref(), *motor_id, &[0x17, 0x01, 0, 0, 0, 0, 0, 0]);
                match wait_hightorque_status_for_motor(
                    bus.as_ref(),
                    *motor_id,
                    Duration::from_millis(50),
                ) {
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
