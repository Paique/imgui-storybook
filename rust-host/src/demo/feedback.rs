//! Port of `ProgressBarStory`, `TooltipStory` and `PopupModalStory`.

use crate::api::{ArgSet, ArgValue, Story};
use crate::ctx::StoryCtx;
use crate::ctx::StoryUi;
use super::help;
use easy_imgui::{id, lbl};

pub struct ProgressBarStory;

impl Story for ProgressBarStory {
    fn title(&self) -> &str {
        "Feedback/Progress Bar"
    }

    fn description(&self) -> &str {
        "Progress bar with overlay label and an advance action."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.float32("value", 0.65)
            .string("overlay", "")
            .preset("done", [("value", 1.0f32)])
            .preset(
                "labeled",
                [("value", ArgValue::from(0.3f32)), ("overlay", "3 of 10".into())],
            );
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let value = ctx.float32("value").clamp(0.0, 1.0);
        let mut overlay = ctx.string("overlay");
        if overlay.is_empty() {
            overlay = format!("{:.0}%", value * 100.0);
        }
        help::progress_bar(value, &overlay);
        ui.spacing();
        if ui.button(lbl("Advance")) {
            ctx.set("value", ArgValue::Float((value + 0.1).min(1.0)));
            ctx.action("advance");
        }
        ui.same_line();
        ui.text_disabled(format!("value = {value:.2}"));
    }
}

pub struct TooltipStory;

impl Story for TooltipStory {
    fn title(&self) -> &str {
        "Feedback/Tooltip"
    }

    fn description(&self) -> &str {
        "Hover tooltip with help-marker pattern."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.string("text", "Extra explanation shown on hover.")
            .bool("helpMarker", true)
            .preset("button", [("helpMarker", false)]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        ui.text("Hover the marker");
        if ctx.bool("helpMarker") {
            ui.same_line();
            ui.text_disabled("(?)");
        } else {
            ui.text("or the button:");
            ui.same_line();
            ui.button(lbl("Hover me"));
        }
        if ui.is_item_hovered() {
            help::set_tooltip(&ctx.string("text"));
            // hover state only shows in live mode
        }
    }
}

pub struct PopupModalStory;

impl Story for PopupModalStory {
    fn title(&self) -> &str {
        "Feedback/Popup & Modal"
    }

    fn description(&self) -> &str {
        "Confirmation modal opened from a button."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.string("title", "Confirm action")
            .string("message", "Proceed with this operation?")
            .bool("autoOpen", false)
            .preset("open", [("autoOpen", true)])
            .preset(
                "delete",
                [
                    ("autoOpen", ArgValue::from(true)),
                    ("title", ArgValue::from("Delete item")),
                    ("message", "This cannot be undone. Delete anyway?".into()),
                ],
            );
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let title = ctx.string("title");
        if ctx.bool("autoOpen") {
            ui.open_popup(id(&title));
            ctx.set("autoOpen", ArgValue::Bool(false));
        }
        if ui.button(lbl("Open...")) {
            ui.open_popup(id(&title));
            ctx.action("open");
        }
        if help::begin_popup_modal(&title) {
            ui.text(&ctx.string("message"));
            ui.spacing();
            if ui.button(lbl("Confirm")) {
                ui.close_current_popup();
                ctx.action("confirm");
            }
            ui.same_line();
            if ui.button(lbl("Cancel")) {
                ui.close_current_popup();
                ctx.action("cancel");
            }
            help::end_popup();
        }
    }
}
