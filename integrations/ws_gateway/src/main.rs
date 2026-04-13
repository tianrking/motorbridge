use tokio::net::TcpListener;

mod hightorque;
mod model;
mod ops;
mod router;
mod session;

use ops::parse_args;
use router::handle_socket;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = parse_args().map_err(|e| format!("arg parse error: {e}"))?;
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
                eprintln!("[ws_gateway] session error: {e}");
            }
        });
    }
}
