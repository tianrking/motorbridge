use serde_json::Value;

const DAMIAO_SCAN_MODEL_HINTS: &[&str] = &[
    "4340P", "4340", "4310", "4310P", "3507", "6006", "8006", "8009", "10010L", "10010",
    "H3510", "G6215", "H6220", "JH11", "6248P",
];

pub(crate) fn build_scan_model_hints(preferred_model: &str) -> Vec<String> {
    let preferred = preferred_model.trim();
    if !preferred.is_empty()
        && preferred.to_lowercase() != "auto"
        && preferred.to_lowercase() != "all"
        && preferred != "*"
    {
        return vec![preferred.to_string()];
    }
    let mut out: Vec<String> = Vec::new();
    for m in DAMIAO_SCAN_MODEL_HINTS {
        if !out.iter().any(|x| x.eq_ignore_ascii_case(m)) {
            out.push((*m).to_string());
        }
    }
    out
}

pub(crate) fn build_scan_feedback_hints(base_feedback_id: u16, motor_id: u16) -> Vec<u16> {
    let mut out = Vec::new();
    let inferred = motor_id.saturating_add(0x10);
    for fid in [inferred, base_feedback_id, 0x0011, 0x0017] {
        if !out.contains(&fid) {
            out.push(fid);
        }
    }
    out
}

pub(crate) fn parse_hex_or_dec(s: &str) -> Result<u16, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s}: {e}"))
    } else {
        s.parse::<u16>()
            .map_err(|e| format!("invalid integer {s}: {e}"))
    }
}

pub(crate) fn parse_u32_hex_or_dec(s: &str) -> Result<u32, String> {
    if let Some(hex) = s.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).map_err(|e| format!("invalid integer {s}: {e}"))
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid integer {s}: {e}"))
    }
}

pub(crate) fn parse_id_list_csv(s: &str) -> Vec<u16> {
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

pub(crate) fn as_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(default)
}

pub(crate) fn as_u64(v: &Value, key: &str, default: u64) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(default)
}

pub(crate) fn as_f32(v: &Value, key: &str, default: f32) -> f32 {
    v.get(key)
        .and_then(Value::as_f64)
        .map(|x| x as f32)
        .unwrap_or(default)
}

pub(crate) fn as_u16(v: &Value, key: &str, default: u16) -> u16 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_u64().map(|x| x as u16).unwrap_or(default),
        Some(Value::String(s)) => parse_hex_or_dec(s).unwrap_or(default),
        _ => default,
    }
}
