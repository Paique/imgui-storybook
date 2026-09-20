//! Spike: renders a fake story offscreen and writes `target/spike.png`.
//! Validates the whole GL pipeline (winit+glutin+glow → FBO → easy-imgui → readback) before
//! the demo stories and WS wiring are exercised. Run: `cargo run --example spike`.

use imgui_storybook_rust_host::api::{ArgSet, Story, Theme};
use imgui_storybook_rust_host::ctx::StoryCtx;
use imgui_storybook_rust_host::ctx::StoryUi;
use imgui_storybook_rust_host::easy_imgui::lbl;
use imgui_storybook_rust_host::gl::HeadlessGl;
use imgui_storybook_rust_host::grabber;
use imgui_storybook_rust_host::imgui_host::{FrameParams, ImGuiHost};
use imgui_storybook_rust_host::input::InputInjector;
use imgui_storybook_rust_host::registry::StoryRegistry;

struct SpikeStory;

impl Story for SpikeStory {
    fn title(&self) -> &str {
        "Spike/Offscreen"
    }

    fn description(&self) -> &str {
        "Validates the offscreen render pipeline"
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.string("label", "Olá do host Rust!")
            .bool("show_button", true)
            .color("accent", 0x74502F)
            .float32("value", 42.0);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        ui.text(&ctx.string("label"));
        ui.text(&format!("accent = {}", ctx.color("accent")));
        ui.separator();
        if ctx.bool("show_button") && ui.button(lbl("Clique-me")) {
            ctx.action("clicked");
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut gl = HeadlessGl::new(900, 600)?;
    let mut host = ImGuiHost::new(gl.glow().clone())?;

    let mut registry = StoryRegistry::new();
    registry.add(SpikeStory);
    let entry = &registry.entries()[0];
    let meta = entry.meta.clone();
    let story_index = entry.index;

    let input = InputInjector::new();
    let args = meta.args.defaults();
    let meta_ref = &meta;

    let mut render_one = |host: &mut ImGuiHost, gl: &mut HeadlessGl| {
        gl.bind_framebuffer();
        let story = registry.story_mut(story_index);
        host.render_frame(
            FrameParams {
                width: gl.width() as f32,
                height: gl.height() as f32,
                theme: Theme::Dark,
                scale: 1.0,
                backdrop: imgui_storybook_rust_host::api::Backdrop::NeutralDark,
                canvas_mode: imgui_storybook_rust_host::api::CanvasMode::Windowed,
                meta: meta_ref,
                args: &args,
                story,
            },
            &input,
        )
    };

    for _ in 0..3 {
        render_one(&mut host, &mut gl);
    }

    let frame = grabber::grab(gl.glow(), gl.framebuffer_width(), gl.framebuffer_height());
    let out = std::path::Path::new("target/spike.png");
    grabber::png(&frame, out)?;
    println!("wrote {} ({}x{})", out.display(), frame.width, frame.height);

    // Non-trivial pixels: count distinct colors in the center band.
    let mut distinct = std::collections::HashSet::new();
    for px in frame.data.chunks_exact(4).step_by(61) {
        distinct.insert([px[0], px[1], px[2]]);
    }
    println!("distinct sampled colors: {}", distinct.len());
    if distinct.len() < 3 {
        anyhow::bail!("frame looks empty — pipeline broken?");
    }
    println!("SPIKE OK");
    Ok(())
}
