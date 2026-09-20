//! `--capture` — offscreen PNG captures + captures.json. Rust mirror of
//! `host/capture/CaptureRunner.java`: story × preset(default+named) × theme × scale,
//! 3 warm-up frames, PNG at `imgui/<id>/<preset>-<theme>[@Nx].png`, manifest `captures.json`.

use std::path::PathBuf;

use anyhow::{bail, Result};
use serde_json::{json, Map, Value};

use crate::api::{Backdrop, CanvasMode, Theme};
use crate::args_values;
use crate::args_values::ArgMap;
use crate::cli::Cli;
use crate::gl::HeadlessGl;
use crate::grabber;
use crate::imgui_host::{FrameParams, ImGuiHost};
use crate::input::InputInjector;
use crate::registry::StoryRegistry;

pub fn run(cli: &Cli, registry: &mut StoryRegistry) -> Result<()> {
    let mut themes = Vec::new();
    for t in &cli.themes {
        themes.push(Theme::parse(t).ok_or_else(|| anyhow::anyhow!("unknown theme '{t}'"))?);
    }
    if themes.is_empty() {
        bail!("no themes given");
    }
    let mut scales = Vec::new();
    for s in &cli.scales {
        let v: f32 = s
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid scale '{s}'"))?;
        if v <= 0.0 {
            bail!("invalid scale '{s}'");
        }
        scales.push(v);
    }
    if scales.is_empty() {
        bail!("no scales given");
    }

    let out = PathBuf::from(&cli.out);
    std::fs::create_dir_all(&out)?;

    let mut gl = HeadlessGl::new(cli.width, cli.height)?;
    let mut host = ImGuiHost::new(gl.glow().clone())?;
    let input = InputInjector::new();

    let mut entries: Vec<Value> = Vec::new();
    let metas: Vec<(usize, crate::registry::StoryMeta)> = registry
        .entries()
        .into_iter()
        .map(|e| (e.index, e.meta.clone()))
        .collect();

    for (story_index, meta) in &metas {
        let story_index = *story_index;
        let meta = meta;
        let mut preset_names: Vec<&str> = vec!["default"];
        preset_names.extend(meta.args.presets().iter().map(|(n, _)| n.as_str()));

        for preset in preset_names {
            let overrides: ArgMap = if preset == "default" {
                meta.args.defaults()
            } else {
                args_values::materialize(&meta.args, &meta.args.preset_values(preset))
            };

            for &theme in &themes {
                for &scale in &scales {
                    host.set_scale(scale);
                    let backdrop = match theme {
                        Theme::Light => Backdrop::NeutralLight,
                        Theme::Dark => Backdrop::NeutralDark,
                    };
                    let args = overrides.clone();
                    let story = registry.story_mut(story_index);

                    let mut render = |host: &mut ImGuiHost, gl: &mut HeadlessGl| {
                        gl.bind_framebuffer();
                        host.render_frame(
                            FrameParams {
                                width: gl.width() as f32,
                                height: gl.height() as f32,
                                theme,
                                scale,
                                backdrop,
                                canvas_mode: CanvasMode::Windowed,
                                meta,
                                args: &args,
                                story,
                            },
                            &input,
                        )
                    };

                    // 3 warm-up frames so auto-resized windows settle (Java parity).
                    for _ in 0..3 {
                        render(&mut host, &mut gl);
                    }

                    let frame = grabber::grab(gl.glow(), gl.framebuffer_width(), gl.framebuffer_height());

                    let scale_suffix = if scale == 1.0 {
                        String::new()
                    } else {
                        format!("@{}x", crate::api::trim_float(scale))
                    };
                    let rel = format!("imgui/{}/{preset}-{}{}.png", meta.id, theme.as_str(), scale_suffix);
                    let path = out.join(&rel);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    grabber::png(&frame, &path)?;

                    entries.push(json!({
                        "storyId": meta.id,
                        "title": meta.title,
                        "description": meta.description,
                        "storyClass": meta.story_class,
                        "preset": preset,
                        "theme": theme.as_str(),
                        "scale": scale,
                        "file": rel,
                        "width": frame.width,
                        "height": frame.height,
                        "args": Value::Object(args_json(&args, &meta.args)),
                    }));
                }
            }
        }
    }

    let manifest = json!({
        "generatedAt": iso8601_now(),
        "hostVersion": crate::info::host_version(),
        "imguiJavaVersion": crate::info::binding_version(),
        "dearImgui": crate::info::DEAR_IMGUI,
        "width": cli.width,
        "height": cli.height,
        "entries": entries,
    });
    std::fs::write(out.join("captures.json"), serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

fn args_json(args: &ArgMap, set: &crate::api::ArgSet) -> Map<String, Value> {
    args_values::for_json(args, set)
}

/// UTC timestamp like `2026-09-20T12:34:56Z` (no external date dependency).
fn iso8601_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Howard Hinnant's `civil_from_days` (proleptic Gregorian).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2026-09-20 (leap-adjusted): days since epoch = 20716
        assert_eq!(civil_from_days(20_716), (2026, 9, 20));
    }
}
