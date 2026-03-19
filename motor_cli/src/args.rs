use std::collections::HashMap;

pub fn parse_args() -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut it = std::env::args().skip(1).peekable();
    while let Some(k) = it.next() {
        if !k.starts_with("--") {
            continue;
        }
        if k == "--help" {
            out.insert("help".to_string(), "1".to_string());
            continue;
        }
        let key = k.trim_start_matches("--").to_string();
        match it.peek() {
            Some(v) if !v.starts_with("--") => {
                if let Some(val) = it.next() {
                    out.insert(key, val);
                }
            }
            _ => {
                out.insert(key, "1".to_string());
            }
        }
    }
    out
}

pub fn get_str(args: &HashMap<String, String>, key: &str, default: &str) -> String {
    args.get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

pub fn get_f32(args: &HashMap<String, String>, key: &str, default: f32) -> Result<f32, String> {
    match args.get(key) {
        Some(v) => v
            .parse::<f32>()
            .map_err(|e| format!("invalid --{key}: {e}")),
        None => Ok(default),
    }
}

pub fn get_u64(args: &HashMap<String, String>, key: &str, default: u64) -> Result<u64, String> {
    match args.get(key) {
        Some(v) => v
            .parse::<u64>()
            .map_err(|e| format!("invalid --{key}: {e}")),
        None => Ok(default),
    }
}

pub fn parse_u16_hex_or_dec(s: &str, key: &str) -> Result<u16, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| format!("invalid --{key}: {e}"))
    } else {
        s.parse::<u16>()
            .map_err(|e| format!("invalid --{key}: {e}"))
    }
}

pub fn get_u16_hex_or_dec(
    args: &HashMap<String, String>,
    key: &str,
    default: u16,
) -> Result<u16, String> {
    match args.get(key) {
        Some(v) => parse_u16_hex_or_dec(v, key),
        None => Ok(default),
    }
}

pub fn get_opt_u16_hex_or_dec(
    args: &HashMap<String, String>,
    key: &str,
) -> Result<Option<u16>, String> {
    match args.get(key) {
        Some(v) => Ok(Some(parse_u16_hex_or_dec(v, key)?)),
        None => Ok(None),
    }
}

pub fn print_help() {
    println!(
        "motor_cli\n\
Usage:\n\
  cargo run -p motor_cli --release -- \\\n\
    --vendor damiao --channel can0 --model 4340P --motor-id 0x01 --feedback-id 0x11 \\\n\
    --mode mit --pos 0 --vel 0 --kp 30 --kd 1 --tau 0 --loop 200 --dt-ms 20\n\n\
Vendors:\n\
  --vendor damiao    default\n\
  --vendor robstride\n\
  --vendor all       scan both vendors\n\n\
Damiao modes:\n\
  --mode scan | enable | disable | mit | pos-vel | vel | force-pos\n\n\
RobStride modes:\n\
  --mode ping | scan | enable | disable | mit | vel | read-param | write-param\n\n\
Common args:\n\
  --channel      default can0\n\
  --model        default depends on vendor (damiao=4340, robstride=rs-00)\n\
  --motor-id     default 0x01\n\
  --feedback-id  default 0x11 for Damiao, 0xFF for RobStride host-id compatibility\n\
  --loop         send cycles, default 1\n\
  --dt-ms        period ms, default 20\n\
  --ensure-mode  1/0, default 1\n\n\
Damiao extras:\n\
  --verify-model 1/0, default 1\n\
  --verify-timeout-ms  default 500\n\
  --verify-tol   default 0.2\n\
  --set-motor-id <id> --set-feedback-id <id> --store 1/0 --verify-id 1/0\n\n\
RobStride extras:\n\
  --param-id <hex|dec>      for read-param / write-param\n\
  --param-value <number>    for write-param\n\
  --start-id <hex|dec>      for scan, default 1\n\
  --end-id <hex|dec>        for scan, default 255\n\
  (scan auto-fallbacks to blind pulse probing if no ping replies)\n\
\n\
All-vendor scan:\n\
  --vendor all --mode scan   run Damiao scan + RobStride scan in one command\n\
\n\
Examples:\n\
  cargo run -p motor_cli --release -- \\\n\
    --vendor robstride --channel can0 --model rs-00 --motor-id 127 --mode ping\n\
\n\
  cargo run -p motor_cli --release -- \\\n\
    --vendor robstride --channel can0 --model rs-00 --motor-id 127 \\\n\
    --mode mit --pos 0.0 --vel 0.0 --kp 8 --kd 0.2 --tau 0 --loop 200 --dt-ms 20\n"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u16_hex_or_dec_supports_both_formats() {
        assert_eq!(parse_u16_hex_or_dec("0x10", "x").expect("hex"), 16);
        assert_eq!(parse_u16_hex_or_dec("255", "x").expect("dec"), 255);
    }

    #[test]
    fn parse_u16_hex_or_dec_rejects_invalid_values() {
        assert!(parse_u16_hex_or_dec("0xZZ", "x").is_err());
        assert!(parse_u16_hex_or_dec("-1", "x").is_err());
    }

    #[test]
    fn get_u16_hex_or_dec_uses_default_when_missing() {
        let args = HashMap::new();
        assert_eq!(
            get_u16_hex_or_dec(&args, "motor-id", 0x01).expect("default"),
            0x01
        );
    }
}
