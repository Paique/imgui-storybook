//! Story API — the Rust mirror of `java-host/api` (`Story`, `ArgSet`, `ArgSpec`, `ArgType`,
//! `Theme`, `Backdrop`, `CanvasMode`, `StoryIds`).
//!
//! Values of args use the same canonical forms as the Java host: `STRING → Str`,
//! `BOOLEAN → Bool`, `INT → Int`, `FLOAT → Float`, `ENUM → Str(option name)`,
//! `COLOR → Color(0xRRGGBB)`.

use std::collections::HashMap;
use std::fmt;

use crate::ctx::StoryCtx;

/// Canonical typed value of one arg (Java: `Object` with per-type canonical classes).
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    Str(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    /// `0xRRGGBB`
    Color(u32),
}

impl ArgValue {
    /// Human-readable form (Java `ArgSpec.defaultAsString`).
    pub fn default_as_string(&self) -> String {
        match self {
            ArgValue::Str(s) => s.clone(),
            ArgValue::Bool(b) => b.to_string(),
            ArgValue::Int(i) => i.to_string(),
            ArgValue::Float(f) => trim_float(*f),
            ArgValue::Color(rgb) => format!("#{:06X}", rgb & 0xFFFFFF),
        }
    }
}

impl fmt::Display for ArgValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgValue::Str(s) => write!(f, "{s}"),
            other => write!(f, "{}", other.default_as_string()),
        }
    }
}

/// Conversion helper so story code can write `("label", "Ok")` in presets.
pub trait IntoArgValue {
    fn into_arg(self) -> ArgValue;
}

impl IntoArgValue for ArgValue {
    fn into_arg(self) -> ArgValue {
        self
    }
}

impl IntoArgValue for &str {
    fn into_arg(self) -> ArgValue {
        ArgValue::Str(self.to_string())
    }
}

impl IntoArgValue for String {
    fn into_arg(self) -> ArgValue {
        ArgValue::Str(self)
    }
}

impl IntoArgValue for bool {
    fn into_arg(self) -> ArgValue {
        ArgValue::Bool(self)
    }
}

impl IntoArgValue for i32 {
    fn into_arg(self) -> ArgValue {
        ArgValue::Int(self)
    }
}

impl IntoArgValue for f32 {
    fn into_arg(self) -> ArgValue {
        ArgValue::Float(self)
    }
}

/// `0xRRGGBB` — colors are declared as `u32` to match the Java `color(name, 0x...)` API.
impl IntoArgValue for u32 {
    fn into_arg(self) -> ArgValue {
        ArgValue::Color(self)
    }
}

impl From<&str> for ArgValue {
    fn from(v: &str) -> Self {
        ArgValue::Str(v.to_string())
    }
}

impl From<String> for ArgValue {
    fn from(v: String) -> Self {
        ArgValue::Str(v)
    }
}

impl From<bool> for ArgValue {
    fn from(v: bool) -> Self {
        ArgValue::Bool(v)
    }
}

impl From<i32> for ArgValue {
    fn from(v: i32) -> Self {
        ArgValue::Int(v)
    }
}

impl From<f32> for ArgValue {
    fn from(v: f32) -> Self {
        ArgValue::Float(v)
    }
}

/// `0xRRGGBB`.
impl From<u32> for ArgValue {
    fn from(v: u32) -> Self {
        ArgValue::Color(v)
    }
}

/// Java `trimFloat`: integral floats lose the decimal part, others keep up to 3 decimals.
pub fn trim_float(v: f32) -> String {
    if v == v.round() {
        return format!("{}", v as i64);
    }
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
}

/// Arg types supported by the story Controls panel and the export manifests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgType {
    String,
    Boolean,
    Int,
    Float,
    Enum,
    /// Stored as `#RRGGBB`.
    Color,
}

impl ArgType {
    pub fn as_str(self) -> &'static str {
        match self {
            ArgType::String => "STRING",
            ArgType::Boolean => "BOOLEAN",
            ArgType::Int => "INT",
            ArgType::Float => "FLOAT",
            ArgType::Enum => "ENUM",
            ArgType::Color => "COLOR",
        }
    }
}

/// Immutable declaration of one arg (Java `ArgSpec`).
#[derive(Clone, Debug)]
pub struct ArgSpec {
    pub name: String,
    pub ty: ArgType,
    pub default: ArgValue,
    /// Option names, `Some` only for `ArgType::Enum`.
    pub enum_options: Option<Vec<String>>,
}

impl ArgSpec {
    pub fn default_as_string(&self) -> String {
        self.default.default_as_string()
    }
}

/// Declaration of the editable args of a story, plus named presets (arg overrides).
#[derive(Clone, Debug, Default)]
pub struct ArgSet {
    specs: Vec<ArgSpec>,
    /// (preset name, [(arg name, override value)]) in declaration order.
    presets: Vec<(String, Vec<(String, ArgValue)>)>,
}

impl ArgSet {
    pub fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, spec: ArgSpec) -> &mut Self {
        assert!(
            !self.specs.iter().any(|s| s.name == spec.name),
            "duplicate arg '{}'",
            spec.name
        );
        self.specs.push(spec);
        self
    }

    pub fn string(&mut self, name: &str, default: &str) -> &mut Self {
        self.add(ArgSpec {
            name: name.to_string(),
            ty: ArgType::String,
            default: ArgValue::Str(default.to_string()),
            enum_options: None,
        })
    }

    pub fn bool(&mut self, name: &str, default: bool) -> &mut Self {
        self.add(ArgSpec {
            name: name.to_string(),
            ty: ArgType::Boolean,
            default: ArgValue::Bool(default),
            enum_options: None,
        })
    }

    pub fn int32(&mut self, name: &str, default: i32) -> &mut Self {
        self.add(ArgSpec {
            name: name.to_string(),
            ty: ArgType::Int,
            default: ArgValue::Int(default),
            enum_options: None,
        })
    }

    pub fn float32(&mut self, name: &str, default: f32) -> &mut Self {
        self.add(ArgSpec {
            name: name.to_string(),
            ty: ArgType::Float,
            default: ArgValue::Float(default),
            enum_options: None,
        })
    }

    /// Declares an enum arg. `default` must be one of `options` (stored as the option name).
    pub fn enum_of(&mut self, name: &str, options: &[&str], default: &str) -> &mut Self {
        assert!(
            options.contains(&default),
            "enum arg '{name}': default '{default}' is not an option"
        );
        self.add(ArgSpec {
            name: name.to_string(),
            ty: ArgType::Enum,
            default: ArgValue::Str(default.to_string()),
            enum_options: Some(options.iter().map(|s| s.to_string()).collect()),
        })
    }

    /// `rgb` as `0xRRGGBB`.
    pub fn color(&mut self, name: &str, rgb: u32) -> &mut Self {
        self.add(ArgSpec {
            name: name.to_string(),
            ty: ArgType::Color,
            default: ArgValue::Color(rgb & 0xFFFFFF),
            enum_options: None,
        })
    }

    /// Declares a named preset. Unknown arg names are rejected at declaration time.
    pub fn preset<I, K, V>(&mut self, name: &str, overrides: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: IntoArgValue,
    {
        let values: Vec<(String, ArgValue)> = overrides
            .into_iter()
            .map(|(k, v)| {
                let key: String = k.into();
                assert!(
                    self.get(&key).is_some(),
                    "preset '{name}': unknown arg '{key}'"
                );
                (key, v.into_arg())
            })
            .collect();
        self.presets.push((name.to_string(), values));
        self
    }

    pub fn get(&self, name: &str) -> Option<&ArgSpec> {
        self.specs.iter().find(|s| s.name == name)
    }

    /// Spec list in declaration order.
    pub fn specs(&self) -> &[ArgSpec] {
        &self.specs
    }

    /// (name, overrides) in declaration order.
    pub fn presets(&self) -> &[(String, Vec<(String, ArgValue)>)] {
        &self.presets
    }

    /// Default values for all args, keyed by name.
    pub fn defaults(&self) -> HashMap<String, ArgValue> {
        self.specs
            .iter()
            .map(|s| (s.name.clone(), s.default.clone()))
            .collect()
    }

    /// Defaults overridden by the given preset. Unknown preset returns plain defaults.
    pub fn preset_values(&self, preset_name: &str) -> HashMap<String, ArgValue> {
        let mut out = self.defaults();
        if let Some((_, overrides)) = self.presets.iter().find(|(n, _)| n == preset_name) {
            for (k, v) in overrides {
                out.insert(k.clone(), v.clone());
            }
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

/// Canvas theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    /// Wire form used by the protocol and CLI (`"light"` / `"dark"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn parse(s: &str) -> Option<Theme> {
        match s {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

/// What is drawn behind the story window on the canvas (and in captures).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backdrop {
    NeutralDark,
    NeutralLight,
    Checker,
}

impl Backdrop {
    /// Wire form (`_` → `-`, like the Java serialization).
    pub fn as_str(self) -> &'static str {
        match self {
            Backdrop::NeutralDark => "neutral-dark",
            Backdrop::NeutralLight => "neutral-light",
            Backdrop::Checker => "checker",
        }
    }

    pub fn parse(s: &str) -> Option<Backdrop> {
        match s {
            "neutral-dark" => Some(Backdrop::NeutralDark),
            "neutral-light" => Some(Backdrop::NeutralLight),
            "checker" => Some(Backdrop::Checker),
            _ => None,
        }
    }
}

/// How the story is laid out on the canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasMode {
    /// Story renders inside a titled, auto-resized ImGui window (as it appears in-game).
    Windowed,
    /// Story renders bare on a borderless full-canvas host window.
    Inline,
}

impl CanvasMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CanvasMode::Windowed => "windowed",
            CanvasMode::Inline => "inline",
        }
    }

    pub fn parse(s: &str) -> Option<CanvasMode> {
        match s {
            "windowed" => Some(CanvasMode::Windowed),
            "inline" => Some(CanvasMode::Inline),
            _ => None,
        }
    }
}

/// A single ImGui story: one component, one scenario.
///
/// Stories are registered programmatically on [`crate::registry::StoryRegistry`].
/// Implementations may be stateful, but per-frame values should be read from the
/// `StoryCtx` (and written back with `StoryCtx::set`) so the host and the web Controls
/// panel stay in sync.
pub trait Story {
    /// Story path using '/' as group separator, e.g. `"Inputs/Text Input"`.
    fn title(&self) -> &str;

    /// Short human/AI-readable description shown in docs exports.
    fn description(&self) -> &str {
        ""
    }

    /// Declares the editable args of this story (drives the Controls panel and captures).
    fn define_args(&self, _args: &mut ArgSet) {}

    /// Renders the story for the current frame. Called once per frame by the host, inside
    /// the host window (windowed or inline). The `Ui` is Dear ImGui's per-frame UI handle
    /// (the Java port used `ImGui.*` statics; easy-imgui passes it explicitly).
    fn render(&mut self, ui: &crate::imgui_host::StoryUi, ctx: &mut StoryCtx);
}

/// Stable id for a story title: lowercase, non-alphanumeric runs collapsed to '-'.
/// `"Inputs/Text Input"` → `"inputs-text-input"`.
pub fn story_slug(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut slug = String::with_capacity(lower.len());
    let mut in_run = false;
    for ch in lower.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
            in_run = false;
        } else if !in_run && !slug.is_empty() {
            slug.push('-');
            in_run = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    assert!(!slug.is_empty(), "story title produces empty id: {title}");
    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_matches_java() {
        assert_eq!(story_slug("Inputs/Text Input"), "inputs-text-input");
        assert_eq!(story_slug("Basics/Button"), "basics-button");
        assert_eq!(story_slug("  --Foo__Bar!!  "), "foo-bar");
        assert_eq!(story_slug("A1 b2"), "a1-b2");
    }

    #[test]
    #[should_panic(expected = "empty id")]
    fn slug_empty_panics() {
        story_slug("///");
    }

    #[test]
    fn argset_builder_and_presets() {
        let mut args = ArgSet::new();
        args.string("label", "Confirm")
            .bool("danger", false)
            .enum_of("size", &["Small", "Medium", "Large"], "Medium")
            .color("accent", 0x74502F)
            .float32("speed", 0.25)
            .int32("count", 7)
            .preset("small", [("label", "Ok"), ("size", "Small")]);

        assert_eq!(args.specs().len(), 6);
        assert_eq!(args.get("accent").unwrap().ty, ArgType::Color);
        assert_eq!(
            args.get("size").unwrap().enum_options.as_deref(),
            Some(&["Small".to_string(), "Medium".to_string(), "Large".to_string()][..])
        );
        let pv = args.preset_values("small");
        assert_eq!(pv.get("label"), Some(&ArgValue::Str("Ok".into())));
        assert_eq!(pv.get("size"), Some(&ArgValue::Str("Small".into())));
        assert_eq!(pv.get("danger"), Some(&ArgValue::Bool(false)));
        // unknown preset = plain defaults
        assert_eq!(args.preset_values("nope").get("label").unwrap().to_string(), "Confirm");
    }

    #[test]
    fn default_as_string_matches_java() {
        assert_eq!(ArgValue::Float(42.0).default_as_string(), "42");
        assert_eq!(ArgValue::Float(0.25).default_as_string(), "0.25");
        assert_eq!(ArgValue::Float(12.5).default_as_string(), "12.5");
        assert_eq!(ArgValue::Color(0x74502F).default_as_string(), "#74502F");
        assert_eq!(ArgValue::Int(-3).default_as_string(), "-3");
        assert_eq!(ArgValue::Bool(true).default_as_string(), "true");
    }

    #[test]
    fn enums_wire_forms() {
        assert_eq!(Theme::parse("dark"), Some(Theme::Dark));
        assert_eq!(Backdrop::Checker.as_str(), "checker");
        assert_eq!(CanvasMode::parse("windowed"), Some(CanvasMode::Windowed));
        assert_eq!(Theme::parse("nope"), None);
    }
}
