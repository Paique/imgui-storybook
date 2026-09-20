//! Coercion between JSON-ish values (`serde_json::Value`) coming from the web client and the
//! canonical [`ArgValue`] of each arg type. Rust mirror of `host/core/ArgValues.java`.

use std::collections::HashMap;

use serde_json::Value;

use crate::api::{ArgSet, ArgSpec, ArgType, ArgValue};

pub type ArgMap = HashMap<String, ArgValue>;

/// Typed read with fallback to the declared default when missing/invalid.
pub fn get(spec: &ArgSpec, values: &ArgMap) -> ArgValue {
    match values.get(&spec.name) {
        None => spec.default.clone(),
        Some(raw) => coerce(spec, &json_of(raw)).unwrap_or_else(|| spec.default.clone()),
    }
}

/// Serializes a canonical value back to JSON (for coercion round-trips and output).
pub fn json_of(value: &ArgValue) -> Value {
    match value {
        ArgValue::Str(s) => Value::String(s.clone()),
        ArgValue::Bool(b) => Value::Bool(*b),
        ArgValue::Int(i) => Value::from(*i),
        ArgValue::Float(f) => serde_json::Number::from_f64(*f as f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ArgValue::Color(rgb) => Value::from(*rgb as i64),
    }
}

/// Coerces one JSON value to the canonical form, or `None` when impossible.
pub fn coerce(spec: &ArgSpec, raw: &Value) -> Option<ArgValue> {
    if raw.is_null() {
        return None;
    }
    match spec.ty {
        ArgType::String => Some(ArgValue::Str(match raw {
            Value::String(s) => s.clone(),
            other => match other {
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                _ => return None,
            },
        })),
        ArgType::Boolean => Some(ArgValue::Bool(match raw {
            Value::Bool(b) => *b,
            Value::String(s) => parse_bool(s)?,
            _ => return None,
        })),
        ArgType::Int => Some(ArgValue::Int(match raw {
            Value::Number(n) => trunc_to_i32(n.as_f64()?)?,
            Value::String(s) => rint_to_i32(s.parse::<f64>().ok()?)?,
            _ => return None,
        })),
        ArgType::Float => Some(ArgValue::Float(match raw {
            Value::Number(n) => n.as_f64()? as f32,
            Value::String(s) => s.parse::<f64>().ok()? as f32,
            _ => return None,
        })),
        ArgType::Enum => {
            let name = raw.as_str()?;
            spec.enum_options
                .as_ref()?
                .iter()
                .find(|o| o.as_str() == name)
                .map(|o| ArgValue::Str(o.clone()))
        }
        ArgType::Color => Some(ArgValue::Color(match raw {
            Value::Number(n) => n.as_i64()? as u32 & 0xFFFFFF,
            Value::String(s) => parse_hex_color(s)?,
            _ => return None,
        })),
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Java `Number.intValue()` truncates toward zero; out-of-range wraps like the cast would.
fn trunc_to_i32(v: f64) -> Option<i32> {
    Some(v as i32)
}

/// Java `Math.rint` — round half to even.
fn rint_to_i32(v: f64) -> Option<i32> {
    Some(v.round_ties_even() as i32)
}

fn parse_hex_color(s: &str) -> Option<u32> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    u32::from_str_radix(hex, 16).ok().map(|v| v & 0xFFFFFF)
}

/// Keeps only known args (coerced); used to sanitize client input.
pub fn sanitize(set: &ArgSet, raw: &serde_json::Map<String, Value>) -> ArgMap {
    let mut out = ArgMap::new();
    for spec in set.specs() {
        if let Some(value) = raw.get(&spec.name) {
            if let Some(coerced) = coerce(spec, value) {
                out.insert(spec.name.clone(), coerced);
            }
        }
    }
    out
}

/// Full map coerced against the schema (missing → defaults).
pub fn materialize(set: &ArgSet, raw: &ArgMap) -> ArgMap {
    set.specs()
        .iter()
        .map(|spec| (spec.name.clone(), get(spec, raw)))
        .collect()
}

/// JSON-friendly form: colors as `#RRGGBB`, everything else natural.
/// Iterates the schema order, like the Java `LinkedHashMap` version.
pub fn for_json(values: &ArgMap, set: &ArgSet) -> serde_json::Map<String, Value> {
    let mut out = serde_json::Map::new();
    for spec in set.specs() {
        let Some(value) = values.get(&spec.name) else {
            continue;
        };
        let json = match spec.ty {
            ArgType::Color => {
                if let ArgValue::Color(rgb) = value {
                    Value::String(format!("#{:06X}", rgb & 0xFFFFFF))
                } else {
                    continue;
                }
            }
            _ => json_of(value),
        };
        out.insert(spec.name.clone(), json);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn specs() -> ArgSet {
        let mut s = ArgSet::new();
        s.string("label", "hi")
            .bool("on", false)
            .int32("n", 1)
            .float32("f", 0.5)
            .enum_of("size", &["S", "M"], "M")
            .color("c", 0x000000);
        s
    }

    #[test]
    fn coerce_basics() {
        let set = specs();
        let g = |name: &str, raw: Value| {
            let spec = set.get(name).unwrap();
            coerce(spec, &raw)
        };
        assert_eq!(g("label", json!("x")), Some(ArgValue::Str("x".into())));
        assert_eq!(g("label", json!(42)), Some(ArgValue::Str("42".into())));
        assert_eq!(g("on", json!(true)), Some(ArgValue::Bool(true)));
        assert_eq!(g("on", json!("true")), Some(ArgValue::Bool(true)));
        assert_eq!(g("n", json!(42.9)), Some(ArgValue::Int(42)));
        assert_eq!(g("n", json!("42.5")), Some(ArgValue::Int(42))); // rint: ties to even
        assert_eq!(g("f", json!("1.25")), Some(ArgValue::Float(1.25)));
        assert_eq!(g("size", json!("S")), Some(ArgValue::Str("S".into())));
        assert_eq!(g("size", json!("X")), None);
        assert_eq!(g("c", json!("#74502F")), Some(ArgValue::Color(0x74502F)));
        assert_eq!(g("c", json!("74502f")), Some(ArgValue::Color(0x74502F)));
        assert_eq!(g("c", json!(7622703)), Some(ArgValue::Color(0x74502F)));
        assert_eq!(g("c", json!("zzz")), None);
    }

    #[test]
    fn rint_ties_to_even_like_java() {
        assert_eq!(rint_to_i32(42.5), Some(42));
        assert_eq!(rint_to_i32(43.5), Some(44));
        assert_eq!(rint_to_i32(41.5), Some(42));
    }

    #[test]
    fn sanitize_drops_unknown_and_invalid() {
        let set = specs();
        let raw = serde_json::json!({
            "label": "ok",
            "n": 7,
            "size": "BAD",
            "ghost": 1,
            "c": "#FF0000"
        });
        let out = sanitize(&set, raw.as_object().unwrap());
        assert_eq!(out.len(), 3);
        assert_eq!(out.get("label").unwrap().to_string(), "ok");
        assert_eq!(out.get("n"), Some(&ArgValue::Int(7)));
        assert_eq!(out.get("c"), Some(&ArgValue::Color(0xFF0000)));
    }

    #[test]
    fn materialize_fills_defaults() {
        let set = specs();
        let mut raw = ArgMap::new();
        raw.insert("n".to_string(), ArgValue::Int(9));
        let out = materialize(&set, &raw);
        assert_eq!(out.get("n"), Some(&ArgValue::Int(9)));
        assert_eq!(out.get("label"), Some(&ArgValue::Str("hi".into())));
        assert_eq!(out.get("f"), Some(&ArgValue::Float(0.5)));
    }

    #[test]
    fn for_json_color_hex_others_natural() {
        let set = specs();
        let mut values = set.defaults();
        values.insert("c".to_string(), ArgValue::Color(0x74502F));
        let json = for_json(&values, &set);
        assert_eq!(json.get("c"), Some(&json!("#74502F")));
        assert_eq!(json.get("on"), Some(&json!(false)));
        assert_eq!(json.get("f"), Some(&json!(0.5)));
    }

    #[test]
    fn get_falls_back_to_default_on_invalid() {
        let set = specs();
        let mut raw = ArgMap::new();
        raw.insert("n".to_string(), ArgValue::Str("junk".into()));
        assert_eq!(get(set.get("n").unwrap(), &raw), ArgValue::Int(1));
    }
}
