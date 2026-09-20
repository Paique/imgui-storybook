//! `--serve` — WebSocket host streaming live frames. Rust mirror of `HostMain.serve`:
//! GL on the main thread at ~30 fps, WS in a tokio runtime on its own thread, shared state
//! via `Arc<ViewState>`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::broadcast;

use crate::cli::Cli;
use crate::gl::HeadlessGl;
use crate::grabber;
use crate::imgui_host::{FrameParams, ImGuiHost};
use crate::input::InputInjector;
use crate::net::{self, NetState};
use crate::registry::{StoryMeta, StoryRegistry};
use crate::state::{Inner, ViewState};

const FRAME_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / 30);

pub fn run(cli: &Cli, registry: &mut StoryRegistry) -> Result<()> {
    let mut gl = HeadlessGl::new(cli.width, cli.height)?;
    let mut host = ImGuiHost::new(gl.glow().clone()).context("ImGuiHost init")?;

    // Snapshot of story metadata, aligned with the registry's story list.
    let mut metas: Vec<StoryMeta> = Vec::new();
    let mut registry_index: Vec<usize> = Vec::new();
    let mut index_by_id: HashMap<String, usize> = HashMap::new();
    for entry in registry.entries() {
        index_by_id.insert(entry.meta.id.clone(), metas.len());
        registry_index.push(entry.index);
        metas.push(entry.meta.clone());
    }

    let state = Arc::new(ViewState::new());
    let input = Arc::new(InputInjector::new());

    let (tx, _rx) = broadcast::channel::<Arc<String>>(64);

    let net_state = Arc::new(NetState {
        metas,
        index_by_id,
        registry_index,
        state: state.clone(),
        input: input.clone(),
    });

    // WS runtime on its own thread (Java: WsServer on thread "ws-server").
    {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("tokio runtime")?,
        );
        let hello = crate::protocol::server::hello(
            crate::info::host_version(),
            crate::info::binding_version(),
            crate::info::DEAR_IMGUI,
            cli.width,
            cli.height,
            gl.framebuffer_width(),
            gl.framebuffer_height(),
        )
        .to_string();
        let tx = tx.clone();
        let net_thread = net_state.clone();

        // Bind on the calling thread so a port conflict fails loudly (and instantly) instead
        // of leaving a mute render loop; SO_REUSEADDR lets the dev watcher restart the host
        // without waiting for TIME_WAIT to drain.
        let addr: std::net::SocketAddr = ([0, 0, 0, 0], cli.port)
            .try_into()
            .expect("valid listen addr");
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        socket.listen(128)?;
        let listener: std::net::TcpListener = socket.into();

        std::thread::Builder::new()
            .name("ws-server".into())
            .spawn(move || {
                rt.block_on(async move {
                    if let Err(e) = net::run_ws(listener, hello, net_thread, tx).await {
                        eprintln!("ws server error: {e:#}");
                        std::process::exit(1);
                    }
                })
            })?;
    }

    println!(
        "imgui-storybook host (demo stories: {}, discovered: {})",
        if cli.no_demo { 0 } else { crate::demo::count() },
        registry.size()
    );
    println!("listening on ws://localhost:{} — ctrl+c to stop", cli.port);

    loop {
        let start = Instant::now();

        let snap: Inner = state.snapshot();
        if snap.scale != host.scale() {
            host.set_scale(snap.scale);
        }
        let has_clients = tx.receiver_count() > 0;
        if let (Some(metas_idx), true) = (snap.entry_index, has_clients) {
            let Some(registry_idx) = net_state.registry_index.get(metas_idx).copied() else {
                continue;
            };
            let Some(meta) = net_state.metas.get(metas_idx) else {
                continue;
            };
            gl.pump();
            gl.bind_framebuffer();
            let story = registry.story_mut(registry_idx);
            let result = host.render_frame(
                FrameParams {
                    width: gl.width() as f32,
                    height: gl.height() as f32,
                    theme: snap.theme,
                    scale: snap.scale,
                    backdrop: snap.backdrop,
                    canvas_mode: snap.canvas_mode,
                    meta,
                    args: &snap.args,
                    story,
                },
                &input,
            );
            if !result.sets.is_empty() {
                state.apply_sets(result.sets);
            }
            for action in &result.actions {
                let msg = crate::protocol::server::action(&action.name, action.at_ms).to_string();
                let _ = tx.send(Arc::new(msg));
            }
            let frame = grabber::grab(gl.glow(), gl.framebuffer_width(), gl.framebuffer_height());
            let jpeg = grabber::jpeg(&frame, 80)?;
            let data = base64_encode(&jpeg);
            let seq = state.next_frame_seq();
            let msg = crate::protocol::server::frame(seq, frame.width, frame.height, "image/jpeg", &data)
                .to_string();
            let _ = tx.send(Arc::new(msg));
        }

        let elapsed = start.elapsed();
        if elapsed < FRAME_INTERVAL {
            std::thread::sleep(FRAME_INTERVAL - elapsed);
        }
    }
}

pub(crate) fn base64_encode(data: &[u8]) -> String {
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    STANDARD.encode(data)
}
