use motor_vendor_damiao::{ControlMode, DamiaoController};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let controller = DamiaoController::new_socketcan("can0")?;
    let motor = controller.add_motor(0x01, 0x11, "4340P")?;

    println!("[1] enable all");
    controller.enable_all()?;
    std::thread::sleep(Duration::from_millis(500));

    println!("[2] ensure MIT mode");
    if let Err(e) = motor.ensure_control_mode(ControlMode::Mit, Duration::from_secs(1)) {
        eprintln!(
            "[warn] ensure MIT mode failed: {e}. Continue anyway (some firmwares do not reply register read reliably)."
        );
        eprintln!("[warn] trying direct write register 10 = 1 (MIT)...");
        if let Err(e2) = motor.write_register_u32(10, 1) {
            eprintln!("[warn] direct write reg10 failed too: {e2}. Continue with MIT command test.");
        }
    }

    println!("[3] hold zero for 2s");
    let hold_start = Instant::now();
    while hold_start.elapsed() < Duration::from_secs(2) {
        motor.send_cmd_mit(0.0, 0.0, 30.0, 1.0, 0.0)?;
        if let Some(s) = motor.latest_state() {
            println!("hold pos={:+.3} vel={:+.3} torq={:+.3}", s.pos, s.vel, s.torq);
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    println!("[4] sine motion (+/-3 rad)");
    let start = Instant::now();
    loop {
        let t = start.elapsed().as_secs_f32();
        let target = (0.5 * t).sin() * 3.0;
        motor.send_cmd_mit(target, 0.0, 50.0, 1.0, 0.0)?;
        if let Some(s) = motor.latest_state() {
            println!(
                "target={:+.2} actual={:+.3} vel={:+.3} torq={:+.3} status={}",
                target, s.pos, s.vel, s.torq, s.status_code
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
