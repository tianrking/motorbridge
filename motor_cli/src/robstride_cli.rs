use crate::args::{
    get_f32, get_opt_u16_hex_or_dec, get_str, get_u16_hex_or_dec, get_u64, parse_u16_hex_or_dec,
};
use motor_vendor_robstride::{
    model_limits as robstride_model_limits, ControlMode as RobstrideControlMode, ParameterDataType,
    ParameterValue, RobstrideController,
};
use std::collections::HashMap;
use std::time::Duration;

fn parse_robstride_param_value(param_id: u16, raw: &str) -> Result<ParameterValue, String> {
    let info = motor_vendor_robstride::parameter_info(param_id)
        .ok_or_else(|| format!("unknown RobStride parameter 0x{param_id:04X}"))?;
    match info.data_type {
        ParameterDataType::Int8 => raw
            .parse::<i8>()
            .map(ParameterValue::I8)
            .map_err(|e| format!("invalid --param-value: {e}")),
        ParameterDataType::UInt8 => raw
            .parse::<u8>()
            .map(ParameterValue::U8)
            .map_err(|e| format!("invalid --param-value: {e}")),
        ParameterDataType::UInt16 => {
            parse_u16_hex_or_dec(raw, "param-value").map(ParameterValue::U16)
        }
        ParameterDataType::UInt32 => {
            if let Some(hex) = raw.strip_prefix("0x") {
                u32::from_str_radix(hex, 16)
                    .map(ParameterValue::U32)
                    .map_err(|e| format!("invalid --param-value: {e}"))
            } else {
                raw.parse::<u32>()
                    .map(ParameterValue::U32)
                    .map_err(|e| format!("invalid --param-value: {e}"))
            }
        }
        ParameterDataType::Float32 => raw
            .parse::<f32>()
            .map(ParameterValue::F32)
            .map_err(|e| format!("invalid --param-value: {e}")),
    }
}

fn print_robstride_param_value(param_id: u16, value: ParameterValue) {
    let name = motor_vendor_robstride::parameter_info(param_id)
        .map(|info| info.name)
        .unwrap_or("unknown");
    match value {
        ParameterValue::I8(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::U8(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::U16(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::U32(v) => println!("param 0x{param_id:04X} ({name}) = {v}"),
        ParameterValue::F32(v) => println!("param 0x{param_id:04X} ({name}) = {v:.6}"),
    }
}

pub fn run_robstride(
    args: &HashMap<String, String>,
    channel: &str,
    model: &str,
    motor_id: u16,
    feedback_id: u16,
    vendor_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let model_input = model;
    let model = if vendor_name == "hightorque" {
        let m = model_input.trim().to_ascii_lowercase();
        if m.is_empty() || m == "hightorque" || m == "ht" || m == "auto" || m == "default" {
            println!(
                "[info] vendor=hightorque model={} -> compat model=rs-00",
                model_input
            );
            "rs-00"
        } else if m.starts_with("rs-") {
            model_input
        } else {
            println!(
                "[warn] vendor=hightorque received unsupported compat model '{}'; fallback to rs-00",
                model_input
            );
            "rs-00"
        }
    } else {
        model_input
    };

    let mode = get_str(args, "mode", "ping");
    let loop_n = get_u64(args, "loop", 1)?;
    let dt_ms = get_u64(args, "dt-ms", 20)?;
    let ensure_mode = get_u64(args, "ensure-mode", 1)? != 0;
    let set_motor_id = get_opt_u16_hex_or_dec(args, "set-motor-id")?;
    let store_after_set = get_u64(args, "store", 1)? != 0;
    let controller = RobstrideController::new_socketcan(channel)?;

    if let Some((pmax, vmax, tmax)) = robstride_model_limits(model) {
        println!(
            "[info] {}(robstride-compatible) model {} limits pmax={:.3} vmax={:.3} tmax={:.3}",
            vendor_name, model, pmax, vmax, tmax
        );
    }

    let query_ping = |m: &std::sync::Arc<motor_vendor_robstride::RobstrideMotor>| -> Option<u16> {
        if m.get_parameter(0x7005, Duration::from_millis(120)).is_ok() {
            return Some(0x7005);
        }
        if m.get_parameter(0x7019, Duration::from_millis(120)).is_ok() {
            return Some(0x7019);
        }
        None
    };

    if mode == "scan" {
        let raw_start_id = get_u16_hex_or_dec(args, "start-id", 1)?;
        let raw_end_id = get_u16_hex_or_dec(args, "end-id", 255)?;
        if raw_start_id == 0
            || raw_end_id == 0
            || raw_start_id > 255
            || raw_end_id > 255
            || raw_start_id > raw_end_id
        {
            return Err("invalid scan range: expected 1..255 and start<=end".into());
        }
        let (start_id, end_id) = if vendor_name == "hightorque" {
            (raw_start_id.clamp(1, 32), raw_end_id.clamp(1, 32))
        } else {
            (raw_start_id, raw_end_id)
        };
        if start_id > end_id {
            return Err("invalid scan range after clamping".into());
        }

        println!(
            "[scan] probing {} IDs {}..{} on {}",
            vendor_name, start_id, end_id, channel
        );
        if vendor_name == "hightorque" && (raw_start_id != start_id || raw_end_id != end_id) {
            println!(
                "[scan] vendor=hightorque range clamped to {}..{} (requested {}..{})",
                start_id, end_id, raw_start_id, raw_end_id
            );
        }
        let mut hits = 0usize;
        for id in start_id..=end_id {
            let probe_ctrl = RobstrideController::new_socketcan(channel)?;
            let candidate = probe_ctrl.add_motor(id, feedback_id, model)?;
            let mut hit = false;
            if let Ok(reply) =
                candidate.ping(Duration::from_millis(if vendor_name == "hightorque" {
                    40
                } else {
                    80
                }))
            {
                println!(
                    "[hit] vendor={} id={} responder_id={} model_hint={} payload={:02x?}",
                    vendor_name, reply.device_id, reply.responder_id, model, reply.payload
                );
                hit = true;
            } else if vendor_name != "hightorque" {
                if let Some(pid) = query_ping(&candidate) {
                    println!(
                        "[hit] vendor={} id={} by=query-param(0x{:04X}) model_hint={}",
                        vendor_name, id, pid, model
                    );
                    hit = true;
                }
            }
            if hit {
                hits += 1;
            }
            probe_ctrl.close_bus()?;
            std::thread::sleep(Duration::from_millis(2));
        }
        if hits == 0 {
            controller.close_bus()?;
            let fallback = RobstrideController::new_socketcan(channel)?;
            let manual_vel = get_f32(args, "manual-vel", 0.2)?;
            let manual_ms = get_u64(args, "manual-ms", 200)?;
            let manual_gap_ms = get_u64(args, "manual-gap-ms", 200)?;
            println!(
                "[scan] no ping replies for {}; fallback to blind pulse probing (observe motor motion)",
                vendor_name
            );
            println!(
                "[scan] pulse: vel={:.3} for {}ms, gap={}ms",
                manual_vel, manual_ms, manual_gap_ms
            );
            for id in start_id..=end_id {
                let candidate = fallback.add_motor(id, feedback_id, model)?;
                let _ = candidate.enable();
                let _ = candidate.set_mode(RobstrideControlMode::Velocity);
                let mut state_seen = false;
                let t_end = std::time::Instant::now() + Duration::from_millis(manual_ms);
                while std::time::Instant::now() < t_end {
                    let _ = candidate.set_velocity_target(manual_vel);
                    if candidate.latest_state().is_some() {
                        state_seen = true;
                    }
                    std::thread::sleep(Duration::from_millis(40));
                }
                for _ in 0..3 {
                    let _ = candidate.set_velocity_target(0.0);
                    if candidate.latest_state().is_some() {
                        state_seen = true;
                    }
                    std::thread::sleep(Duration::from_millis(30));
                }
                let _ = candidate.disable();
                if state_seen {
                    hits += 1;
                    if let Some(s) = candidate.latest_state() {
                        println!(
                            "[hit] vendor={} id={} by=status pos={:+.3} vel={:+.3} torq={:+.3}",
                            vendor_name, id, s.position, s.velocity, s.torque
                        );
                    } else {
                        println!("[hit] vendor={} id={} by=status", vendor_name, id);
                    }
                } else {
                    println!(
                        "[probe] vendor={} id={} model_hint={} (if this ID moved, note it)",
                        vendor_name, id, model
                    );
                }
                std::thread::sleep(Duration::from_millis(manual_gap_ms));
            }
            fallback.close_bus()?;
            println!("[scan] done vendor={} hits={hits}", vendor_name);
            return Ok(());
        }
        println!("[scan] done vendor={} hits={hits}", vendor_name);
        controller.close_bus()?;
        return Ok(());
    }
    let motor = controller.add_motor(motor_id, feedback_id, model)?;

    match mode.as_str() {
        "ping" => {
            if let Ok(reply) = motor.ping(Duration::from_millis(500)) {
                println!(
                    "[ok] ping device_id={} responder_id={} payload={:02x?}",
                    reply.device_id, reply.responder_id, reply.payload
                );
            } else if let Some(pid) = query_ping(&motor) {
                println!("[ok] ping(by query) param 0x{pid:04X} responded");
            } else {
                return Err(format!(
                    "{} ping failed: no response to GET_DEVICE_ID or query parameters",
                    vendor_name
                )
                .into());
            }
            controller.close_bus()?;
            return Ok(());
        }
        "read-param" => {
            let param_id = get_u16_hex_or_dec(args, "param-id", 0)?;
            let value = motor.get_parameter(param_id, Duration::from_millis(500))?;
            print_robstride_param_value(param_id, value);
            controller.close_bus()?;
            return Ok(());
        }
        "write-param" => {
            let param_id = get_u16_hex_or_dec(args, "param-id", 0)?;
            let raw = args
                .get("param-value")
                .ok_or_else(|| "missing --param-value".to_string())?;
            let value = parse_robstride_param_value(param_id, raw)?;
            motor.write_parameter(param_id, value)?;
            std::thread::sleep(Duration::from_millis(50));
            let verify = motor.get_parameter(param_id, Duration::from_millis(500))?;
            print_robstride_param_value(param_id, verify);
            controller.close_bus()?;
            return Ok(());
        }
        _ => {}
    }

    if let Some(new_motor_id) = set_motor_id {
        motor.set_device_id(new_motor_id as u8)?;
        println!(
            "[id-set] {} device id update requested: {} -> {}",
            vendor_name, motor_id, new_motor_id
        );
        if store_after_set {
            motor.save_parameters()?;
            println!("[id-set] {} save-parameters sent", vendor_name);
        }
        controller.close_bus()?;
        return Ok(());
    }

    if mode != "disable" {
        controller.enable_all()?;
        std::thread::sleep(Duration::from_millis(100));
    }

    if ensure_mode {
        match mode.as_str() {
            "mit" => motor.set_mode(RobstrideControlMode::Mit)?,
            "vel" => motor.set_mode(RobstrideControlMode::Velocity)?,
            _ => {}
        }
    }

    for i in 0..loop_n {
        match mode.as_str() {
            "enable" => motor.enable()?,
            "disable" => motor.disable()?,
            "mit" => {
                motor.send_cmd_mit(
                    get_f32(args, "pos", 0.0)?,
                    get_f32(args, "vel", 0.0)?,
                    get_f32(args, "kp", 8.0)?,
                    get_f32(args, "kd", 0.2)?,
                    get_f32(args, "tau", 0.0)?,
                )?;
            }
            "vel" => {
                motor.set_velocity_target(get_f32(args, "vel", 0.0)?)?;
            }
            _ => return Err(format!("unknown {vendor_name} mode: {mode}").into()),
        }

        if let Some(s) = motor.latest_state() {
            println!(
                "#{i} pos={:+.3} vel={:+.3} torq={:+.3} temp={:.1} flags[u={} stall={} enc={} ot={} oc={} uv={}]",
                s.position,
                s.velocity,
                s.torque,
                s.temperature_c,
                s.uncalibrated,
                s.stall,
                s.magnetic_encoder_fault,
                s.overtemperature,
                s.overcurrent,
                s.undervoltage
            );
        }
        std::thread::sleep(Duration::from_millis(dt_ms));
    }

    if mode == "enable" || mode == "disable" {
        controller.close_bus()?;
    } else {
        controller.shutdown()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_robstride_param_value_uses_parameter_type() {
        let mode = parse_robstride_param_value(0x7005, "2").expect("int8 mode");
        let timeout = parse_robstride_param_value(0x7028, "123").expect("u32 timeout");
        let mech = parse_robstride_param_value(0x7019, "1.5").expect("f32 mech pos");

        match mode {
            ParameterValue::I8(v) => assert_eq!(v, 2),
            _ => panic!("expected I8"),
        }
        match timeout {
            ParameterValue::U32(v) => assert_eq!(v, 123),
            _ => panic!("expected U32"),
        }
        match mech {
            ParameterValue::F32(v) => assert!((v - 1.5).abs() < 1e-6),
            _ => panic!("expected F32"),
        }
    }
}
