//! Per-frame story context: typed arg readers, view info, action reporting and arg writes.
//! Rust mirror of `api/StoryContext.java` + `host/core/HostedStoryContext.java`.
//!
//! Actions and arg writes are collected in per-frame queues that the host drains right after
//! the story renders (same observable order as the Java implementation).

use std::collections::HashMap;

pub use crate::imgui_host::StoryUi;
pub use easy_imgui::{Ui, UiBuilder};

use crate::api::{ArgSpec, ArgValue, Theme};
use crate::args_values::{self, ArgMap};

pub struct StoryCtx<'a> {
    specs: &'a crate::api::ArgSet,
    args: &'a ArgMap,
    theme: Theme,
    scale: f32,
    /// Actions reported this frame (`ctx.action("clicked")`).
    pub actions: Vec<String>,
    /// Arg writes this frame, applied by the host after the frame (Java applies immediately;
    /// the observable behavior — next frame sees the new value — is the same).
    pub pending_sets: Vec<(String, ArgValue)>,
}

impl<'a> StoryCtx<'a> {
    pub fn new(specs: &'a crate::api::ArgSet, args: &'a ArgMap, theme: Theme, scale: f32) -> Self {
        Self {
            specs,
            args,
            theme,
            scale,
            actions: Vec::new(),
            pending_sets: Vec::new(),
        }
    }

    fn spec(&self, name: &str) -> Option<&ArgSpec> {
        self.specs.get(name)
    }

    pub fn string(&self, name: &str) -> String {
        match self.spec(name) {
            Some(spec) => args_values::get(spec, self.args).to_string(),
            None => String::new(),
        }
    }

    pub fn bool(&self, name: &str) -> bool {
        match self.spec(name) {
            Some(spec) => matches!(args_values::get(spec, self.args), ArgValue::Bool(b) if b),
            None => false,
        }
    }

    pub fn int32(&self, name: &str) -> i32 {
        match self.spec(name) {
            Some(spec) => match args_values::get(spec, self.args) {
                ArgValue::Int(i) => i,
                ArgValue::Float(f) => f as i32,
                ArgValue::Str(ref s) => s.parse::<f64>().map(|v| v as i32).unwrap_or(0),
                _ => 0,
            },
            None => 0,
        }
    }

    pub fn float32(&self, name: &str) -> f32 {
        match self.spec(name) {
            Some(spec) => match args_values::get(spec, self.args) {
                ArgValue::Int(i) => i as f32,
                ArgValue::Float(f) => f,
                ArgValue::Str(ref s) => s.parse::<f32>().unwrap_or(0.0),
                _ => 0.0,
            },
            None => 0.0,
        }
    }

    /// Enum arg as the option name. Unknown name falls back to the declared default
    /// (Java `Enum.valueOf` fallback), or the first option.
    pub fn enum_str(&self, name: &str) -> String {
        let Some(spec) = self.spec(name) else {
            return String::new();
        };
        let value = args_values::get(spec, self.args);
        let name = value.to_string();
        let options = spec.enum_options.as_deref().unwrap_or(&[]);
        if options.iter().any(|o| *o == name) {
            return name;
        }
        match spec.default {
            ArgValue::Str(ref d) if options.iter().any(|o| o == d) => d.clone(),
            _ => options.first().cloned().unwrap_or_default(),
        }
    }

    /// `0xRRGGBB`.
    pub fn color(&self, name: &str) -> u32 {
        match self.spec(name) {
            Some(spec) => match args_values::get(spec, self.args) {
                ArgValue::Color(rgb) => rgb,
                ArgValue::Str(ref s) => {
                    let hex = s.strip_prefix('#').unwrap_or(s);
                    u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF) & 0xFFFFFF
                }
                _ => 0xFFFFFF,
            },
            None => 0xFFFFFF,
        }
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Reports an interaction performed inside the story (toast on the canvas + forwarded to
    /// the web client).
    pub fn action(&mut self, name: &str) {
        self.actions.push(name.to_string());
    }

    /// Updates an arg from inside the story. Unknown names are ignored (coercion failure
    /// drops the write, like the Java version).
    pub fn set(&mut self, name: &str, value: ArgValue) {
        if let Some(spec) = self.spec(name) {
            // Validate against the schema before queueing.
            let json = args_values::json_of(&value);
            if let Some(coerced) = args_values::coerce(spec, &json) {
                self.pending_sets.push((name.to_string(), coerced));
            }
        }
    }

    /// Applies the pending sets to an args map (host side, after the frame).
    pub fn drain_sets(&mut self, target: &mut HashMap<String, ArgValue>) {
        for (name, value) in self.pending_sets.drain(..) {
            target.insert(name, value);
        }
    }
}
