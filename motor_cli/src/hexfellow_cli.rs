use crate::args::{get_f32, get_str, get_u16_hex_or_dec, get_u64};
use motor_vendor_hexfellow::{HexfellowController, MitTarget, PosVelTarget};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;

fn to_rev(rad: f32) -> f32 {
    rad / (2.0 * PI)
}

pub fn run_hexfellow(
    args: &HashMap<String, String>,
    channel: &str,
    model: &str,
    motor_id: u16,
    feedback_id: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let mode = get_str(args, "mode", "status");
    let transport = get_str(args, "transport", "auto");
    if transport != "auto" && transport != "socketcanfd" {
        return Err("hexfellow only supports --transport auto|socketcanfd".into());
    }

    let timeout_ms = get_u64(args, "timeout-ms", 200)?;
    let timeout = Duration::from_millis(timeout_ms);
    let controller = HexfellowController::new_socketcanfd(channel)?;

    if mode == "scan" {
        let start_id = get_u16_hex_or_dec(args, "start-id", 1)?;
        let end_id = get_u16_hex_or_dec(args, "end-id", 32)?;
        let hits = controller.scan_ids(start_id, end_id, timeout)?;
        for h in &hits {
            println!(
                "[hit] vendor=hexfellow node={} sw_ver={:?} peak_torque_raw={:?} kp_kd_factor_raw={:?} dev_type={:?}",
                h.node_id, h.sw_ver, h.peak_torque_raw, h.kp_kd_factor_raw, h.dev_type
            );
        }
        println!("[scan] done vendor=hexfellow hits={}", hits.len());
        controller.close_bus()?;
        return Ok(());
    }

    let motor = controller.add_motor(motor_id, feedback_id, model)?;
    match mode.as_str() {
        "enable" => {
            motor.enable_drive(timeout)?;
            println!("[ok] hexfellow enable sent");
        }
        "disable" => {
            motor.disable_drive(timeout)?;
            println!("[ok] hexfellow disable sent");
        }
        "status" => {
            let s = motor.query_status(timeout)?;
            println!(
                "[status] mode_display={} statusword={} pos_rev={:.6} vel_rev_s={:.6} torque_permille={} hb={:?}",
                s.mode_display,
                s.statusword,
                s.position_rev,
                s.velocity_rev_s,
                s.torque_permille,
                s.heartbeat_state
            );
        }
        "pos-vel" => {
            let pos_rad = get_f32(args, "pos", 0.0)?;
            let vel_rad_s = get_f32(args, "vlim", 2.0)?;
            motor.command_pos_vel(
                PosVelTarget {
                    position_rev: to_rev(pos_rad),
                    velocity_rev_s: to_rev(vel_rad_s),
                },
                timeout,
            )?;
            println!(
                "[ok] hexfellow pos-vel sent pos_rad={:.6} vlim_rad_s={:.6}",
                pos_rad, vel_rad_s
            );
        }
        "mit" => {
            let pos_rad = get_f32(args, "pos", 0.0)?;
            let vel_rad_s = get_f32(args, "vel", 0.0)?;
            let tau = get_f32(args, "tau", 0.0)?;
            let kp = get_f32(args, "kp", 1000.0)? as u16;
            let kd = get_f32(args, "kd", 100.0)? as u16;
            let limit_permille = get_u64(args, "limit-permille", 1000)? as u16;
            motor.command_mit(
                MitTarget {
                    position_rev: to_rev(pos_rad),
                    velocity_rev_s: to_rev(vel_rad_s),
                    torque_nm: tau,
                    kp,
                    kd,
                    limit_permille,
                },
                timeout,
            )?;
            println!(
                "[ok] hexfellow mit sent pos_rad={:.6} vel_rad_s={:.6} tau={:.6} kp={} kd={} limit_permille={}",
                pos_rad, vel_rad_s, tau, kp, kd, limit_permille
            );
        }
        _ => {
            return Err(
                "unknown hexfellow mode: expected scan|status|enable|disable|pos-vel|mit".into(),
            );
        }
    }

    controller.close_bus()?;
    Ok(())
}
