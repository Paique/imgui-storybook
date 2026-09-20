//! Headless OpenGL: hidden 1×1 winit window + exact-size RGBA8 FBO.
//!
//! Same reasoning as the Java `HeadlessGl`: a hidden window reports a 0×0 framebuffer on
//! Windows and a visible one would suffer OS DPI scaling; rendering into our own FBO
//! guarantees deterministic `width × height` output on any machine/DPI. No swap: callers
//! read the framebuffer back.
//!
//! Stack: winit (window) + glutin (GL context, WGL on Windows) + glow (loader, re-exported
//! by easy-imgui-renderer). No CMake/C toolchain required — unlike GLFW.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use easy_imgui_renderer::glow::{self, HasContext as _};
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::Display as GlutinDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SurfaceAttributesBuilder, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoop;
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowAttributes};

pub struct HeadlessGl {
    event_loop: EventLoop<()>,
    _window: Window,
    /// Must stay alive: dropping it would release the current GL context.
    #[allow(dead_code)]
    gl_context: PossiblyCurrentContext,
    _surface: Surface<WindowSurface>,
    glow: Rc<glow::Context>,
    fbo: glow::Framebuffer,
    color_tex: glow::Texture,
    width: i32,
    height: i32,
}

struct PumpHandler;

impl ApplicationHandler<()> for PumpHandler {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }
}

impl HeadlessGl {
    pub fn new(width: i32, height: i32) -> Result<Self> {
        let event_loop: EventLoop<()> = EventLoop::new().context("winit EventLoop::new")?;
        let attrs: WindowAttributes = Window::default_attributes()
            .with_visible(false)
            .with_resizable(false)
            .with_title("imgui-storybook host")
            .with_inner_size(winit::dpi::LogicalSize::new(1.0, 1.0));

        let (window, config) = DisplayBuilder::new()
            .with_window_attributes(Some(attrs))
            .build(
                &event_loop,
                ConfigTemplateBuilder::new().with_alpha_size(8),
                |mut configs| configs.next().expect("no available GL pixel format/config"),
            )
            .map_err(|e| anyhow!("glutin-winit display/window build: {e}"))?;
        let window = window.ok_or_else(|| anyhow!("winit window was not created"))?;

        let raw_display = window.display_handle()?.as_raw();
        let raw_window = window.window_handle()?.as_raw();

        // SAFETY: handles are valid and owned by `window`, kept alive in `self`.
        // WGL preference (Windows); EGL/GLX paths use the same call per-platform preference.
        #[cfg(windows)]
        let preference = glutin::display::DisplayApiPreference::Wgl(Some(raw_window));
        #[cfg(not(windows))]
        let preference = glutin::display::DisplayApiPreference::Egl;
        let gl_display =
            unsafe { GlutinDisplay::new(raw_display, preference) }.context("glutin Display")?;
        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window));
        // SAFETY: same as above; context and surface are used only on this thread.
        let not_current =
            unsafe { gl_display.create_context(&config, &context_attributes) }.context("create_context")?;
        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window,
            NonZeroU32::new(1).unwrap(),
            NonZeroU32::new(1).unwrap(),
        );
        let surface = unsafe { gl_display.create_window_surface(&config, &surface_attributes) }
            .context("create_window_surface")?;
        let gl_context = not_current
            .make_current(&surface)
            .context("make_current")?;

        // SAFETY: the proc-address function is valid while `gl_display` is alive.
        let glow = Rc::new(unsafe {
            glow::Context::from_loader_function(|name| {
                gl_display.get_proc_address(std::ffi::CString::new(name).unwrap().as_c_str())
            })
        });

        let (color_tex, fbo) = create_fbo(&glow, width, height)?;

        Ok(HeadlessGl {
            event_loop,
            _window: window,
            gl_context,
            _surface: surface,
            glow,
            fbo,
            color_tex,
            width,
            height,
        })
    }

    /// Drains window events (the window is hidden, so this is effectively a no-op kept for
    /// WGL hygiene).
    pub fn pump(&mut self) {
        let mut handler = PumpHandler;
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut handler);
    }

    pub fn glow(&self) -> &Rc<glow::Context> {
        &self.glow
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn framebuffer_width(&self) -> i32 {
        self.width
    }

    pub fn framebuffer_height(&self) -> i32 {
        self.height
    }

    /// Binds the offscreen FBO and sets the viewport, once per frame.
    pub fn bind_framebuffer(&self) {
        unsafe {
            let gl = &*self.glow;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.width, self.height);
        }
    }

    /// Recreates the FBO at a new size (serve `--width/--height` are fixed, so this is only
    /// used at construction today).
    pub fn resize(&mut self, width: i32, height: i32) -> Result<()> {
        if width == self.width && height == self.height {
            return Ok(());
        }
        unsafe {
            let gl = &*self.glow;
            gl.delete_framebuffer(self.fbo);
            gl.delete_texture(self.color_tex);
        }
        let (tex, fbo) = create_fbo(&self.glow, width, height)?;
        self.color_tex = tex;
        self.fbo = fbo;
        self.width = width;
        self.height = height;
        Ok(())
    }
}

fn create_fbo(glow: &glow::Context, width: i32, height: i32) -> Result<(glow::Texture, glow::Framebuffer)> {
    unsafe {
        let gl = glow;
        let tex = gl
            .create_texture()
            .map_err(|e| anyhow!("create_texture: {e}"))?;
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            width,
            height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(None),
        );
        let fbo = gl
            .create_framebuffer()
            .map_err(|e| anyhow!("create_framebuffer: {e}"))?;
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(tex),
            0,
        );
        if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
            return Err(anyhow!("framebuffer incomplete ({width}x{height})"));
        }
        gl.bind_texture(glow::TEXTURE_2D, None);
        Ok((tex, fbo))
    }
}
