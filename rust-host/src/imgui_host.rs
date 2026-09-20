//! Owns the ImGui context + renderer and renders one story per frame into the offscreen FBO.
//! Rust mirror of `host/core/ImGuiHost.java`.
//!
//! Frame order (Java parity): bind FBO → (clear happens in the renderer's `pre_render`)
//! → apply synthetic input + theme in the current context → `ImGui_NewFrame` (inside
//! `do_frame`) → checker/story/toasts → render → draw data into the FBO. No swap: callers
//! read the framebuffer back.
//!
//! There is no platform backend (the Java host only had GLFW for the GL context and its
//! cursor-polling fallback), so synthetic input can be applied before `NewFrame` — the
//! ordering trap from imgui-java does not exist here. Font scale uses Dear ImGui 1.92's
//! dynamic font system (`style.FontScaleMain`), replacing the Java atlas-rebuild dance.

use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use easy_imgui::{
    self as imgui, lbl, BackendFlags, Color, Cond, Ui, UiBuilder, Vector2, WindowFlags,
};
use easy_imgui_renderer::glow;
use easy_imgui_renderer::Renderer;
use easy_imgui_sys as sys;

use crate::api::{ArgValue, Backdrop, CanvasMode, Story, Theme};
use crate::args_values::ArgMap;
use crate::ctx::StoryCtx;
use crate::input::InputInjector;
use crate::keymap::map_key;
use crate::protocol::InputEvent;
use crate::registry::StoryMeta;

pub const CANVAS_PADDING: f32 = 24.0;
pub const ACTION_TOAST_MS: u64 = 3000;

/// Interaction reported by a story via `ctx.action(...)`.
#[derive(Clone, Debug)]
pub struct ActionEvent {
    pub name: String,
    pub at_ms: u64,
}

#[derive(Clone, Debug)]
struct Toast {
    name: String,
    at_ms: u64,
}

/// Everything the host needs to render one frame.
pub struct FrameParams<'a> {
    pub width: f32,
    pub height: f32,
    pub theme: Theme,
    pub scale: f32,
    pub backdrop: Backdrop,
    pub canvas_mode: CanvasMode,
    pub meta: &'a StoryMeta,
    pub args: &'a ArgMap,
    pub story: &'a mut dyn Story,
}

/// What came out of a frame: actions to broadcast and arg writes to persist.
#[derive(Default)]
pub struct FrameResult {
    pub actions: Vec<ActionEvent>,
    pub sets: Vec<(String, ArgValue)>,
}

pub struct ImGuiHost {
    renderer: Renderer,
    toasts: VecDeque<Toast>,
    last_frame: Instant,
    scale: f32,
}

impl ImGuiHost {
    pub fn new(glow: Rc<glow::Context>) -> Result<Self> {
        let mut renderer = Renderer::new(glow)?;
        {
            let ctx = renderer.imgui();
            let io = ctx.io_mut();
            io.nav_enable_keyboard(true);
            // Java ran ImGuiImplGlfw which sets these; mirror them for identical behavior.
            unsafe {
                io.inner().add_backend_flags(
                    BackendFlags::HasMouseCursors | BackendFlags::HasSetMousePos,
                );
            }
        }
        // Clear color is set per frame from the backdrop.
        renderer.set_background_color(None);
        Ok(ImGuiHost {
            renderer,
            toasts: VecDeque::new(),
            last_frame: Instant::now(),
            scale: 1.0,
        })
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Applies a new scale. Dear ImGui 1.92's dynamic fonts make this just a style write
    /// per frame (the Java host had to rebuild the font atlas and recreate the GL3 backend).
    pub fn set_scale(&mut self, new_scale: f32) {
        if new_scale <= 0.0 || new_scale == self.scale {
            return;
        }
        self.scale = new_scale;
    }

    /// Records an interaction: toast on the canvas + broadcast to the web client.
    /// Records an interaction: toast on the canvas + broadcast to the web client.
    /// (Handled inside `render_frame` via the story's `ctx.action` queue.)

    pub fn render_frame(&mut self, p: FrameParams, input: &InputInjector) -> FrameResult {
        let FrameParams {
            width,
            height,
            theme,
            scale,
            backdrop,
            canvas_mode,
            meta,
            args,
            story,
        } = p;

        // --- pre-frame: delta time, synthetic input, theme (Java order, minus the
        // platform backend).
        unsafe {
            let mut cur = self.renderer.imgui().set_current();
            let now = Instant::now();
            let dt = now - self.last_frame;
            self.last_frame = now;
            cur.io_mut().inner().set_delta_time(dt);

            let io = cur.io_mut();
            for ev in input.drain() {
                apply_io_event(io, ev);
            }

            let style = current_style_mut();
            match theme {
                Theme::Light => sys::ImGui_StyleColorsLight(style),
                Theme::Dark => sys::ImGui_StyleColorsDark(style),
            }
            (*style).FontScaleMain = scale;
            if scale != 1.0 {
                let s = scale;
                (*style).WindowPadding = sys::ImVec2 { x: 8.0 * s, y: 8.0 * s };
                (*style).FramePadding = sys::ImVec2 { x: 4.0 * s, y: 3.0 * s };
                (*style).ItemSpacing = sys::ImVec2 { x: 8.0 * s, y: 4.0 * s };
                (*style).ItemInnerSpacing = sys::ImVec2 { x: 4.0 * s, y: 4.0 * s };
                (*style).IndentSpacing = 21.0 * s;
                (*style).ScrollbarSize = 14.0 * s;
            }
        }

        self.renderer
            .set_size(Vector2::new(width, height), 1.0);
        self.renderer.set_background_color(Some(clear_color(backdrop)));

        // Toasts live on the host (they persist across frames); move them out to satisfy
        // borrows during the frame.
        let mut toasts = std::mem::take(&mut self.toasts);
        let mut result = FrameResult::default();
        {
            let mut app = StoryHost {
                meta,
                args,
                theme,
                scale,
                backdrop,
                canvas_mode,
                story,
                width,
                height,
                toasts: &mut toasts,
                now_ms: epoch_ms(),
                out_actions: &mut result.actions,
                out_sets: &mut result.sets,
            };
            self.renderer.do_frame(&mut app);
        }
        self.toasts = toasts;
        result
    }
}

/// `pre_render` of the renderer clears with the backdrop color before the draw lists render.
fn clear_color(backdrop: Backdrop) -> Color {
    match backdrop {
        Backdrop::NeutralDark => Color::new(0.12, 0.12, 0.14, 1.0),
        Backdrop::NeutralLight | Backdrop::Checker => Color::new(0.92, 0.92, 0.94, 1.0),
    }
}

/// The host's own `UiBuilder` implementation. Public so story traits can name a concrete
/// `Ui` type (trait objects require it): stories take `&StoryUi` in `render`.
pub struct StoryHost<'a> {
    meta: &'a StoryMeta,
    args: &'a ArgMap,
    #[allow(dead_code)]
    theme: Theme,
    #[allow(dead_code)]
    scale: f32,
    backdrop: Backdrop,
    canvas_mode: CanvasMode,
    story: &'a mut dyn Story,
    width: f32,
    height: f32,
    toasts: &'a mut VecDeque<Toast>,
    now_ms: u64,
    out_actions: &'a mut Vec<ActionEvent>,
    out_sets: &'a mut Vec<(String, ArgValue)>,
}

/// The `Ui` type stories receive. Alias so story signatures stay short.
pub type StoryUi<'a> = Ui<StoryHost<'a>>;

impl StoryHost<'_> {
    fn push_action(&mut self, name: String) {
        let at_ms = self.now_ms;
        self.toasts.push_back(Toast {
            name: name.clone(),
            at_ms,
        });
        while self.toasts.len() > 20 {
            self.toasts.pop_front();
        }
        self.out_actions.push(ActionEvent { name, at_ms });
    }
}

impl UiBuilder for StoryHost<'_> {
    fn do_ui(&mut self, ui: &Ui<Self>) {
        if self.backdrop == Backdrop::Checker {
            draw_checker(ui, self.width, self.height);
        }

        let mut ctx = StoryCtx::new(&self.meta.args, self.args, self.theme, self.scale);
        match self.canvas_mode {
            CanvasMode::Windowed => {
                ui.set_next_window_pos(
                    Vector2::new(CANVAS_PADDING, CANVAS_PADDING),
                    Cond::Appearing,
                    Vector2::new(0.0, 0.0),
                );
                ui.window_config(lbl(self.meta.title.as_str()))
                    .flags(WindowFlags::NoCollapse | WindowFlags::NoSavedSettings)
                    .with(|| self.story.render(ui, &mut ctx));
            }
            CanvasMode::Inline => {
                ui.set_next_window_pos(
                    Vector2::new(0.0, 0.0),
                    Cond::Always,
                    Vector2::new(0.0, 0.0),
                );
                ui.set_next_window_size(
                    Vector2::new(self.width, self.height),
                    Cond::Always,
                );
                ui.window_config(lbl("##imgui-storybook-canvas"))
                    .flags(
                        WindowFlags::NoDecoration
                            | WindowFlags::NoMove
                            | WindowFlags::NoSavedSettings
                            | WindowFlags::NoBringToFrontOnFocus,
                    )
                    .with(|| self.story.render(ui, &mut ctx));
            }
        }

        draw_action_toasts(ui, self.toasts, self.now_ms, self.height);

        for name in ctx.actions.drain(..) {
            self.push_action(name);
        }
        self.out_sets.append(&mut ctx.pending_sets);
    }
}

fn draw_checker(ui: &Ui<impl UiBuilder>, width: f32, height: f32) {
    let background = ui.background_draw_list();
    // packAbgr(255, 220, 220, 224) == rgba(255, 220, 220, 224)
    let square = Color::new(1.0, 220.0 / 255.0, 220.0 / 255.0, 224.0 / 255.0);
    let cell = 24.0f32;
    let mut cell_y = 0.0;
    let mut iy = 0u32;
    while cell_y < height {
        let mut cell_x = 0.0;
        let mut ix = 0u32;
        while cell_x < width {
            if (ix + iy) & 1 == 1 {
                let p_min = Vector2::new(cell_x, cell_y);
                let p_max = Vector2::new((cell_x + cell).min(width), (cell_y + cell).min(height));
                background.add_rect_filled(p_min, p_max, square, 0.0, easy_imgui::DrawFlags::None);
            }
            cell_x += cell;
            ix += 1;
        }
        cell_y += cell;
        iy += 1;
    }
}

fn draw_action_toasts(
    ui: &Ui<impl UiBuilder>,
    toasts: &VecDeque<Toast>,
    now_ms: u64,
    height: f32,
) {
    if toasts.is_empty() {
        return;
    }
    let foreground = ui.foreground_draw_list();
    // packAbgr(235, 255, 255, 255) == rgba(235, 255, 255, 255)
    let color = Color::new(235.0 / 255.0, 1.0, 1.0, 1.0);
    let mut y = height - 36.0;
    let mut drawn = 0;
    for event in toasts.iter().rev() {
        if drawn >= 5 {
            break;
        }
        if now_ms - event.at_ms > ACTION_TOAST_MS {
            break;
        }
        foreground.add_text(
            Vector2::new(14.0, y),
            color,
            &format!("[action] {}", event.name),
        );
        y -= 22.0;
        drawn += 1;
    }
}

fn apply_io_event(io: &mut imgui::IoMut, ev: InputEvent) {
    // IoMut is a transparent wrapper of ImGuiIO; the event setters are C wrappers
    // generated by easy-imgui-sys.
    unsafe {
        let ptr: *mut sys::ImGuiIO = std::ptr::from_mut(io).cast();
        match ev {
            InputEvent::MouseMove(x, y) => sys::ImGuiIO_AddMousePosEvent(ptr, x, y),
            InputEvent::MouseButton(button, down) => {
                sys::ImGuiIO_AddMouseButtonEvent(ptr, button, down)
            }
            InputEvent::Wheel(dx, dy) => sys::ImGuiIO_AddMouseWheelEvent(ptr, dx, dy),
            InputEvent::Key(code, down) => {
                if let Some(key) = map_key(&code) {
                    sys::ImGuiIO_AddKeyEvent(ptr, key, down);
                }
            }
            InputEvent::Text(text) => {
                if let Ok(c) = std::ffi::CString::new(text) {
                    sys::ImGuiIO_AddInputCharactersUTF8(ptr, c.as_ptr());
                }
            }
        }
    }
}

/// Current context style (call inside `set_current`).
unsafe fn current_style_mut() -> *mut sys::ImGuiStyle {
    let ctx = sys::ImGui_GetCurrentContext();
    &mut (*ctx).Style
}

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
