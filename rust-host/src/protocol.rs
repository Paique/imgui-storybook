//! Wire protocol — byte-compatible with `packages/protocol/src/index.ts` and the Java host's
//! `host/net/Protocol.java`. All messages are JSON text frames.
//!
//! Server → client: `hello`, `selected`, `view`, `frame`, `action`, `error`, `pong`.
//! Client → server: `select`, `setArgs`, `setView`, `input`, `ping`.

use serde_json::{json, Map, Value};

use crate::api::{Backdrop, CanvasMode, Theme};
use crate::args_values;
use crate::args_values::ArgMap;
use crate::api::ArgSet;

/// `{ theme, scale, backdrop, canvasMode }` (wire forms: `"light"`, `1.5`, `"neutral-dark"`, `"inline"`).
#[derive(Clone, Debug, PartialEq)]
pub struct ViewJson {
    pub theme: Theme,
    pub scale: f32,
    pub backdrop: Backdrop,
    pub canvas_mode: CanvasMode,
}

impl Default for ViewJson {
    fn default() -> Self {
        ViewJson {
            theme: Theme::Dark,
            scale: 1.0,
            backdrop: Backdrop::NeutralDark,
            canvas_mode: CanvasMode::Windowed,
        }
    }
}

impl ViewJson {
    pub fn to_json(&self) -> Value {
        json!({
            "theme": self.theme.as_str(),
            "scale": self.scale,
            "backdrop": self.backdrop.as_str(),
            "canvasMode": self.canvas_mode.as_str(),
        })
    }

    pub fn from_json(v: &Value) -> Option<ViewJson> {
        let mut out = ViewJson::default();
        let theme = v.get("theme").and_then(Value::as_str);
        let scale = v.get("scale").and_then(Value::as_f64);
        let backdrop = v.get("backdrop").and_then(Value::as_str);
        let canvas_mode = v.get("canvasMode").and_then(Value::as_str);
        if theme.is_none() && scale.is_none() && backdrop.is_none() && canvas_mode.is_none() {
            return None;
        }
        if let Some(t) = theme {
            out.theme = Theme::parse(t)?;
        }
        if let Some(s) = scale {
            out.scale = s as f32;
        }
        if let Some(b) = backdrop {
            out.backdrop = Backdrop::parse(b)?;
        }
        if let Some(c) = canvas_mode {
            out.canvas_mode = CanvasMode::parse(c)?;
        }
        Some(out)
    }
}

/// Server → client builders (JSON text ready to send).
pub mod server {
    use super::*;

    pub fn hello(
        host_version: &str,
        binding_version: &str,
        dear_imgui: &str,
        width: i32,
        height: i32,
        fb_width: i32,
        fb_height: i32,
    ) -> Value {
        json!({
            "type": "hello",
            "host": "rust",
            "hostVersion": host_version,
            "imguiJavaVersion": binding_version,
            "dearImgui": dear_imgui,
            "width": width,
            "height": height,
            "fbWidth": fb_width,
            "fbHeight": fb_height,
        })
    }

    pub fn selected(story_id: &str, args: &ArgMap, arg_set: &ArgSet, view: &ViewJson) -> Value {
        json!({
            "type": "selected",
            "storyId": story_id,
            "args": Value::Object(args_values::for_json(args, arg_set)),
            "view": view.to_json(),
        })
    }

    pub fn view(view: &ViewJson) -> Value {
        json!({ "type": "view", "view": view.to_json() })
    }

    pub fn frame(seq: u64, width: i32, height: i32, mime: &str, base64_data: &str) -> Value {
        json!({
            "type": "frame",
            "seq": seq,
            "width": width,
            "height": height,
            "mime": mime,
            "data": base64_data,
        })
    }

    pub fn action(name: &str, at_ms: u64) -> Value {
        json!({ "type": "action", "name": name, "t": at_ms })
    }

    pub fn error(message: &str) -> Value {
        json!({ "type": "error", "message": message })
    }

    pub fn pong() -> Value {
        json!({ "type": "pong" })
    }
}

/// Synthetic input event (Java: `InputInjector` records).
#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    /// `mouseLeave` is normalized to this before enqueueing (Java: `-10000, -10000`).
    MouseMove(f32, f32),
    MouseButton(i32, bool),
    Wheel(f32, f32),
    Key(String, bool),
    Text(String),
}

/// Client → server.
#[derive(Clone, Debug, PartialEq)]
pub enum ClientMessage {
    Select { story_id: String },
    SetArgs { args: Map<String, Value> },
    /// Partial view update (only present fields apply).
    SetView { view: Value },
    Input(InputEvent),
    Ping,
}

pub fn parse_client(text: &str) -> Result<ClientMessage, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let ty = v
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing type".to_string())?;
    match ty {
        "select" => {
            let story_id = v
                .get("storyId")
                .and_then(Value::as_str)
                .ok_or_else(|| "select: missing storyId".to_string())?;
            Ok(ClientMessage::Select {
                story_id: story_id.to_string(),
            })
        }
        "setArgs" => {
            let args = v
                .get("args")
                .and_then(Value::as_object)
                .cloned()
                .ok_or_else(|| "setArgs: missing args object".to_string())?;
            Ok(ClientMessage::SetArgs { args })
        }
        "setView" => {
            let view = v
                .get("view")
                .cloned()
                .ok_or_else(|| "setView: missing view".to_string())?;
            Ok(ClientMessage::SetView { view })
        }
        "input" => {
            let kind = v
                .get("kind")
                .and_then(Value::as_str)
                .ok_or_else(|| "input: missing kind".to_string())?;
            let ev = match kind {
                "mouseMove" => InputEvent::MouseMove(
                    num(&v, "x").unwrap_or(0.0),
                    num(&v, "y").unwrap_or(0.0),
                ),
                "mouseLeave" => InputEvent::MouseMove(-10000.0, -10000.0),
                "mouseDown" | "mouseUp" => InputEvent::MouseButton(
                    num(&v, "button").unwrap_or(0.0) as i32,
                    kind == "mouseDown",
                ),
                "wheel" => InputEvent::Wheel(
                    num(&v, "dx").unwrap_or(0.0),
                    num(&v, "dy").unwrap_or(0.0),
                ),
                "keyDown" | "keyUp" => InputEvent::Key(
                    v.get("key")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "input: missing key".to_string())?
                        .to_string(),
                    kind == "keyDown",
                ),
                "text" => InputEvent::Text(
                    v.get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| "input: missing text".to_string())?
                        .to_string(),
                ),
                other => return Err(format!("input: unknown kind '{other}'")),
            };
            Ok(ClientMessage::Input(ev))
        }
        "ping" => Ok(ClientMessage::Ping),
        other => Err(format!("unknown message type '{other}'")),
    }
}

fn num(v: &Value, field: &str) -> Option<f32> {
    v.get(field).and_then(Value::as_f64).map(|f| f as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ArgValue;

    #[test]
    fn hello_shape() {
        let h = server::hello("0.1.0", "1.92.9.0", "1.92.9", 900, 600, 900, 600);
        assert_eq!(h["type"], "hello");
        assert_eq!(h["host"], "rust");
        assert_eq!(h["imguiJavaVersion"], "1.92.9.0");
        assert_eq!(h["dearImgui"], "1.92.9");
        assert_eq!(h["width"], 900);
        assert_eq!(h["fbHeight"], 600);
    }

    #[test]
    fn selected_args_colors_hex() {
        let mut set = ArgSet::new();
        set.string("label", "x").color("c", 0x74502F).int32("n", 3);
        let mut args = set.defaults();
        args.insert("c".into(), ArgValue::Color(0x74502F));
        let view = ViewJson::default();
        let s = server::selected("my-story", &args, &set, &view);
        assert_eq!(s["type"], "selected");
        assert_eq!(s["storyId"], "my-story");
        assert_eq!(s["args"]["c"], "#74502F");
        assert_eq!(s["args"]["n"], 3);
        assert_eq!(s["view"]["backdrop"], "neutral-dark");
        assert_eq!(s["view"]["canvasMode"], "windowed");
    }

    #[test]
    fn frame_shape() {
        let f = server::frame(7, 900, 600, "image/jpeg", "aGVsbG8=");
        assert_eq!(f["seq"], 7);
        assert_eq!(f["mime"], "image/jpeg");
        assert_eq!(f["data"], "aGVsbG8=");
    }

    #[test]
    fn parse_roundtrip() {
        assert_eq!(
            parse_client(r#"{"type":"select","storyId":"a-b"}"#).unwrap(),
            ClientMessage::Select {
                story_id: "a-b".into()
            }
        );
        assert_eq!(
            parse_client(r#"{"type":"input","kind":"mouseLeave"}"#).unwrap(),
            ClientMessage::Input(InputEvent::MouseMove(-10000.0, -10000.0))
        );
        assert_eq!(
            parse_client(r#"{"type":"input","kind":"mouseDown","button":0}"#).unwrap(),
            ClientMessage::Input(InputEvent::MouseButton(0, true))
        );
        assert_eq!(
            parse_client(r#"{"type":"input","kind":"keyDown","key":"KeyA"}"#).unwrap(),
            ClientMessage::Input(InputEvent::Key("KeyA".into(), true))
        );
        assert_eq!(
            parse_client(r#"{"type":"input","kind":"wheel","dx":0,"dy":-1.5}"#).unwrap(),
            ClientMessage::Input(InputEvent::Wheel(0.0, -1.5))
        );
        assert!(matches!(parse_client(r#"{"type":"ping"}"#), Ok(ClientMessage::Ping)));
        assert!(parse_client(r#"{"type":"nope"}"#).is_err());
        assert!(parse_client("not json").is_err());
    }

    #[test]
    fn view_partial_and_invalid() {
        let v = ViewJson::from_json(&serde_json::json!({"theme": "light"})).unwrap();
        assert_eq!(v.theme, Theme::Light);
        assert_eq!(v.scale, 1.0); // untouched
        assert!(ViewJson::from_json(&serde_json::json!({})).is_none());
        assert!(ViewJson::from_json(&serde_json::json!({"theme": "sepia"})).is_none());
    }
}
