use tokio::net::TcpListener;

mod model;
mod commands;
mod router;
mod session;
mod vendors;

use commands::parse_args;
use router::handle_socket;

fn is_benign_ws_disconnect(err: &str) -> bool {
    let s = err.to_ascii_lowercase();
    s.contains("broken pipe")
        || s.contains("connection reset")
        || s.contains("ws recv error")
        || s.contains("connection closed")
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = parse_args().map_err(|e| format!("arg parse error: {e}"))?;
    let bind_is_local = cfg.bind.starts_with("127.0.0.1:")
        || cfg.bind.starts_with("[::1]:")
        || cfg.bind.starts_with("localhost:");
    if !bind_is_local && std::env::var("MOTORBRIDGE_WS_TOKEN").is_err() {
        return Err("MOTORBRIDGE_WS_TOKEN is required when binding ws_gateway to non-loopback addresses".into());
    }
    let listener = TcpListener::bind(&cfg.bind).await?;

    println!(
        "ws_gateway listening on ws://{} (router_mode=standby, dynamic_target=true, dt_ms={})",
        cfg.bind, cfg.dt_ms
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let cfg_cloned = cfg.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_socket(stream, cfg_cloned).await {
                if is_benign_ws_disconnect(&e) {
                    println!("[ws_gateway] session closed: {e}");
                } else {
                    eprintln!("[ws_gateway] session error: {e}");
                }
            }
        });
    }
}
