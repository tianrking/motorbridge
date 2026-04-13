use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

use crate::model::ServerConfig;
use crate::session::SessionCtx;

mod dispatch;
mod handlers;

async fn send_json<S>(tx: &mut S, obj: Value) -> Result<(), String>
where
    S: futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    tx.send(Message::Text(obj.to_string().into()))
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn handle_socket(stream: TcpStream, cfg: ServerConfig) -> Result<(), String> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let ws = accept_async(stream).await.map_err(|e| e.to_string())?;
    let (mut tx, mut rx) = ws.split();

    let mut ctx = SessionCtx::new(cfg.target.clone());
    let _ = send_json(
        &mut tx,
        json!({
            "type":"event",
            "event":"connected",
            "data": {
                "peer": peer,
                "router_mode": "standby",
                "connected_bus": false,
                "default_target": {
                    "vendor": ctx.target.vendor.as_str(),
                    "transport": ctx.target.transport.as_str(),
                    "channel": ctx.target.channel,
                    "model": ctx.target.model
                }
            }
        }),
    )
    .await;

    let mut ticker = time::interval(Duration::from_millis(cfg.dt_ms));
    let mut state_stream_enabled: bool = false;
    let mut state_tick_counter: u64 = 0;
    let state_tick_div: u64 = 5;
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
                        let req_id = v.get("req_id").cloned();

                        let result = dispatch::dispatch_op(&op, &v, &mut ctx, &mut state_stream_enabled);
                        match result {
                            Ok(data) => {
                                let mut resp = json!({"ok": true, "op": op, "data": data});
                                if let Some(id) = req_id.clone() {
                                    if let Some(obj) = resp.as_object_mut() {
                                        obj.insert("req_id".to_string(), id);
                                    }
                                }
                                send_json(&mut tx, resp).await?
                            }
                            Err(err) => {
                                let mut resp = json!({"ok": false, "op": op, "error": err});
                                if let Some(id) = req_id.clone() {
                                    if let Some(obj) = resp.as_object_mut() {
                                        obj.insert("req_id".to_string(), id);
                                    }
                                }
                                send_json(&mut tx, resp).await?
                            }
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
                if state_stream_enabled && ctx.motor.is_some() {
                    state_tick_counter = state_tick_counter.wrapping_add(1);
                    if state_tick_counter % state_tick_div == 0 {
                        match ctx.build_state_snapshot() {
                            Ok(st) => send_json(&mut tx, json!({"type":"state", "data": st})).await?,
                            Err(err) => send_json(&mut tx, json!({"ok": false, "op":"state_tick","error": err})).await?,
                        }
                    }
                }
            }
        }
    }

    ctx.disconnect(false);
    Ok(())
}
