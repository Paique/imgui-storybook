//! Catalog JSON consumed by the TS story generator and the static docs build.
//! Shape (versioned): `{ hostVersion, bindingVersion, dearImgui, stories: [...] }`.
//! Rust mirror of `host/core/StoryCatalog.java` — field names stay byte-compatible with the
//! Java host (`imguiJavaVersion`) so the Node tooling does not branch per host.

use serde_json::{json, Map, Value};

use crate::api::{ArgSpec, ArgType};
use crate::args_values;
use crate::registry::StoryRegistry;

/// `imguiJavaVersion`-equivalent field for this host: the Dear ImGui version the Rust binding
/// tracks.
pub fn to_json(registry: &StoryRegistry, host_version: &str, binding_version: &str) -> Value {
    let mut stories = Vec::new();
    for entry in registry.entries() {
        stories.push(story_json(entry));
    }
    json!({
        "hostVersion": host_version,
        "imguiJavaVersion": binding_version,
        "dearImgui": dear_imgui_version(binding_version),
        "stories": stories,
    })
}

pub fn to_json_pretty(registry: &StoryRegistry, host_version: &str, binding_version: &str) -> String {
    serde_json::to_string_pretty(&to_json(registry, host_version, binding_version))
        .expect("catalog json")
}

fn story_json(entry: &crate::registry::Entry) -> Value {
    let meta = &entry.meta;
    let args: Vec<Value> = meta
        .args
        .specs()
        .iter()
        .map(|spec| {
            let mut obj = Map::new();
            obj.insert("name".into(), json!(spec.name));
            obj.insert("type".into(), json!(spec.ty.as_str()));
            obj.insert("default".into(), typed_default(spec));
            if spec.ty == ArgType::Enum {
                let options: Vec<&str> = spec
                    .enum_options
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                obj.insert("options".into(), json!(options));
            }
            Value::Object(obj)
        })
        .collect();

    let mut presets = Map::new();
    for (name, overrides) in meta.args.presets() {
        let materialized = args_values::materialize(&meta.args, &overrides_map(overrides));
        presets.insert(
            name.clone(),
            Value::Object(args_values::for_json(&materialized, &meta.args)),
        );
    }

    json!({
        "id": meta.id,
        "title": meta.title,
        "description": meta.description,
        "storyClass": meta.story_class,
        "args": args,
        "presets": Value::Object(presets),
    })
}

fn overrides_map(overrides: &[(String, crate::api::ArgValue)]) -> crate::args_values::ArgMap {
    overrides.iter().cloned().collect()
}

/// JSON-typed default: numbers stay numbers, colors become `#RRGGBB`.
fn typed_default(spec: &ArgSpec) -> Value {
    match &spec.default {
        crate::api::ArgValue::Color(rgb) => json!(format!("#{:06X}", rgb & 0xFFFFFF)),
        crate::api::ArgValue::Int(i) => json!(i),
        crate::api::ArgValue::Float(f) => json!(f),
        crate::api::ArgValue::Bool(b) => json!(b),
        crate::api::ArgValue::Str(s) => json!(s),
    }
}

/// imgui-java convention: `v<dear-imgui>.<build>` → Dear ImGui version without build suffix.
pub fn dear_imgui_version(binding_version: &str) -> String {
    let parts: Vec<&str> = binding_version.split('.').collect();
    if parts.len() >= 3 {
        return parts[..3].join(".");
    }
    binding_version.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ArgSet, Story};
    use crate::ctx::StoryCtx;
    use crate::registry::StoryRegistry;

    struct S;

    impl Story for S {
        fn title(&self) -> &str {
            "Inputs/Demo"
        }

        fn description(&self) -> &str {
            "desc"
        }

        fn define_args(&self, args: &mut crate::api::ArgSet) {
            args.string("label", "Confirm")
                .enum_of("size", &["S", "M"], "M")
                .color("accent", 0x74502F)
                .preset("alt", [("label", "Ok")]);
        }

        fn render(&mut self, _ui: &crate::imgui_host::StoryUi, _ctx: &mut StoryCtx) {}
    }

    #[test]
    fn catalog_shape_matches_java() {
        let mut reg = StoryRegistry::new();
        reg.add(S);
        let v = to_json(&reg, "0.1.0", "1.92.9.0");
        assert_eq!(v["hostVersion"], "0.1.0");
        assert_eq!(v["imguiJavaVersion"], "1.92.9.0");
        assert_eq!(v["dearImgui"], "1.92.9");
        let story = &v["stories"][0];
        assert_eq!(story["id"], "inputs-demo");
        assert_eq!(story["title"], "Inputs/Demo");
        assert_eq!(story["description"], "desc");
        assert_eq!(story["args"][0]["type"], "STRING");
        assert_eq!(story["args"][0]["default"], "Confirm");
        assert_eq!(story["args"][1]["type"], "ENUM");
        assert_eq!(story["args"][1]["options"][0], "S");
        assert_eq!(story["args"][2]["default"], "#74502F");
        assert_eq!(story["presets"]["alt"]["label"], "Ok");
        assert_eq!(story["presets"]["alt"]["size"], "M");
        assert_eq!(story["presets"]["alt"]["accent"], "#74502F");
    }

    #[test]
    fn dear_imgui_truncation() {
        assert_eq!(dear_imgui_version("1.92.7.1"), "1.92.7");
        assert_eq!(dear_imgui_version("1.92"), "1.92");
    }
}
