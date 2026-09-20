//! Port of `ButtonStory` and `TextStory`.

use crate::api::{ArgSet, ArgValue, Story};
use crate::ctx::StoryCtx;
use crate::ctx::StoryUi;
use super::help;
use easy_imgui::{self as imgui, lbl};

pub struct ButtonStory;

impl Story for ButtonStory {
    fn title(&self) -> &str {
        "Basics/Button"
    }

    fn description(&self) -> &str {
        "Standard button with tone and size variants."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.string("label", "Confirm")
            .bool("danger", false)
            .bool("small", false)
            .preset(
                "danger",
                [("label", ArgValue::from("Delete")), ("danger", true.into())],
            )
            .preset("small", [("label", ArgValue::from("OK")), ("small", true.into())]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let label = ctx.string("label");
        let danger = ctx.bool("danger");
        if danger {
            help::push_style_color(imgui::ColorId::Button.bits(), [0.62, 0.16, 0.16, 1.0]);
            help::push_style_color(imgui::ColorId::ButtonHovered.bits(), [0.78, 0.20, 0.20, 1.0]);
        }
        let clicked = if ctx.bool("small") {
            ui.small_button(lbl(&label))
        } else {
            ui.button(lbl(&label))
        };
        if danger {
            help::pop_style_color(2);
        }
        if clicked {
            ctx.action("clicked");
        }
        ui.text_disabled("click it in live mode");
    }
}

pub struct TextStory;

impl Story for TextStory {
    fn title(&self) -> &str {
        "Basics/Text & Labels"
    }

    fn description(&self) -> &str {
        "Plain, wrapped, bulleted and muted text."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.string("text", "The quick brown fox jumps over the lazy dog.")
            .bool("wrapped", false)
            .bool("bullet", false)
            .preset("wrapped", [("wrapped", true)])
            .preset("bullets", [("bullet", true)]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let text = ctx.string("text");
        if ctx.bool("bullet") {
            ui.bullet_text(&text);
        } else if ctx.bool("wrapped") {
            ui.text_wrapped(&text);
        } else {
            ui.text(&text);
        }
        ui.separator();
        ui.text_disabled("muted / secondary text");
        ui.spacing();
        ui.text("Value labels and sizes come from the active theme's fonts.");
    }
}
