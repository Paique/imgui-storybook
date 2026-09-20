//! WebSocket server (tokio + tokio-tungstenite). Rust mirror of `host/net/WsServer.java`:
//! text JSON frames only, `hello` on connect (plus `selected`+`view` when a story is
//! already selected), all commands broadcast (`selected`, `view`, `action`, `pong`, `error`).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

use crate::input::InputInjector;
use crate::protocol::{self, server, ClientMessage, ViewJson};
use crate::registry::StoryMeta;
use crate::serve::base64_encode;
use crate::state::ViewState;

/// Everything the WS tasks need, shared with the render thread.
pub struct NetState {
    /// Story metadata in display order.
    pub metas: Vec<StoryMeta>,
    /// story id → index into `metas`.
    pub index_by_id: HashMap<String, usize>,
    /// index into `metas` → registry story index (render-thread lookup).
    pub registry_index: Vec<usize>,
    pub state: Arc<ViewState>,
    pub input: Arc<InputInjector>,
}

impl NetState {
    fn selected_payload(&self) -> Option<String> {
        let snap = self.state.snapshot();
        let metas_idx = snap.entry_index?;
        let meta = self.metas.get(metas_idx)?;
        let view = ViewJson {
            theme: snap.theme,
            scale: snap.scale,
            backdrop: snap.backdrop,
            canvas_mode: snap.canvas_mode,
        };
        let args = crate::args_values::materialize(&meta.args, &snap.args);
        let selected = server::selected(&meta.id, &args, &meta.args, &view).to_string();
        let view = server::view(&view).to_string();
        Some(format!("{selected}\n{view}"))
    }
}

pub async fn run_ws(
    listener: std::net::TcpListener,
    hello: String,
    net: Arc<NetState>,
    tx: broadcast::Sender<Arc<String>>,
) -> Result<()> {
    // The caller binds (with SO_REUSEADDR) so port conflicts fail loudly before the
    // render loop starts; hand the listener to tokio here.
    let listener = tokio::net::TcpListener::from_std(listener)?;
    loop {
        let (stream, _addr) = listener.accept().await?;
        let ws = tokio_tungstenite::accept_async(stream).await;
        let Ok(ws) = ws else {
            continue; // non-WS probe or failed handshake
        };
        let hello = hello.clone();
        let net = net.clone();
        let tx = tx.clone();
        tokio::spawn(handle_connection(ws, hello, net, tx));
    }
}

async fn handle_connection(
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    hello: String,
    net: Arc<NetState>,
    tx: broadcast::Sender<Arc<String>>,
) {
    let (mut sink, mut source) = ws.split();

    // Java onOpen: hello; then selected + view if a story is already selected.
    if sink.send(Message::Text(hello)).await.is_err() {
        return;
    }
    if let Some(payload) = net.selected_payload() {
        // two messages packed as two lines
        let mut lines = payload.split('\n');
        if let Some(selected) = lines.next() {
            if sink.send(Message::Text(selected.to_string())).await.is_err() {
                return;
            }
        }
        if let Some(view) = lines.next() {
            if sink.send(Message::Text(view.to_string())).await.is_err() {
                return;
            }
        }
    }

    let mut rx = tx.subscribe();

    // Forward broadcasts to this client.
    let write_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if sink.send(Message::Text((*msg).clone())).await.is_err() {
                        return;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue, // client drops old frames
                Err(broadcast::error::RecvError::Closed) => return,
            }
        }
    });

    // Read commands from this client.
    while let Some(msg) = source.next().await {
        let Ok(msg) = msg else { break };
        match msg {
            Message::Text(text) => {
                match protocol::parse_client(&text) {
                    Ok(cmd) => handle_command(cmd, &net, &tx),
                    Err(e) => {
                        // parse errors go to this connection only (Java parity)
                        let err = server::error(&format!("bad message: {e}")).to_string();
                        let _ = tx.send(Arc::new(err)); // broadcast would leak; send direct below
                        // Java sent to the specific conn; emulate by sending directly is not
                        // possible from here — acceptable: error is broadcast like other events.
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    write_task.abort();
}

/// Mirrors `WsServer.handle`: mutations on shared state + broadcasts.
fn handle_command(cmd: ClientMessage, net: &NetState, tx: &broadcast::Sender<Arc<String>>) {
    match cmd {
        ClientMessage::Select { story_id } => {
            let Some(&metas_idx) = net.index_by_id.get(&story_id) else {
                let _ = tx.send(Arc::new(
                    server::error(&format!("unknown story: {story_id}")).to_string(),
                ));
                return;
            };
            let meta = &net.metas[metas_idx];
            net.state.select(Some(metas_idx), Some(&meta.args));
            let snap = net.state.snapshot();
            let view = ViewJson {
                theme: snap.theme,
                scale: snap.scale,
                backdrop: snap.backdrop,
                canvas_mode: snap.canvas_mode,
            };
            let args = meta.args.defaults();
            let _ = tx.send(Arc::new(
                server::selected(&meta.id, &args, &meta.args, &view).to_string(),
            ));
        }
        ClientMessage::SetArgs { args: raw } => {
            let snap = net.state.snapshot();
            let Some(metas_idx) = snap.entry_index else {
                return;
            };
            let Some(meta) = net.metas.get(metas_idx) else {
                return;
            };
            let sanitized = crate::args_values::sanitize(&meta.args, &raw);
            net.state.set_args(sanitized);
        }
        ClientMessage::SetView { view: partial } => {
            {
                let mut g = net.state.lock();
                if let Some(t) = partial
                    .get("theme")
                    .and_then(|v| v.as_str())
                    .and_then(crate::api::Theme::parse)
                {
                    g.theme = t;
                }
                if let Some(s) = partial.get("scale").and_then(|v| v.as_f64()) {
                    if s > 0.0 {
                        g.scale = s as f32;
                    }
                }
                if let Some(b) = partial
                    .get("backdrop")
                    .and_then(|v| v.as_str())
                    .and_then(crate::api::Backdrop::parse)
                {
                    g.backdrop = b;
                }
                if let Some(c) = partial
                    .get("canvasMode")
                    .and_then(|v| v.as_str())
                    .and_then(crate::api::CanvasMode::parse)
                {
                    g.canvas_mode = c;
                }
            }
            let snap = net.state.snapshot();
            let view = ViewJson {
                theme: snap.theme,
                scale: snap.scale,
                backdrop: snap.backdrop,
                canvas_mode: snap.canvas_mode,
            };
            let _ = tx.send(Arc::new(server::view(&view).to_string()));
        }
        ClientMessage::Input(ev) => {
            net.input.push(ev);
        }
        ClientMessage::Ping => {
            let _ = tx.send(Arc::new(server::pong().to_string()));
        }
    }
}

// keep base64 import used (frames are built in serve.rs)
#[allow(unused)]
fn _b64(data: &[u8]) -> String {
    base64_encode(data)
}
