//! Port of `SelectableListStory` and `ChildWindowStory`.

use crate::api::{ArgSet, ArgValue, Story};
use crate::ctx::StoryCtx;
use crate::ctx::StoryUi;
use super::help;
use easy_imgui::lbl;

pub struct SelectableListStory;

impl Story for SelectableListStory {
    fn title(&self) -> &str {
        "Containers/Selectable List"
    }

    fn description(&self) -> &str {
        "List of selectable rows emitting selection actions."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.int32("count", 5)
            .string("pattern", "Item %d")
            .preset("many", [("count", 10)])
            .preset(
                "tasks",
                [("count", ArgValue::from(3)), ("pattern", "Task %d".into())],
            );
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let count = ctx.int32("count").clamp(1, 20);
        for i in 1..=count {
            let label = format_pattern(&ctx.string("pattern"), i);
            // "##i" keeps the row ids distinct while showing the same text (Java parity).
            if ui.selectable(lbl(&format!("{label}##{i}"))) {
                ctx.action(&format!("select:{i}"));
            }
        }
    }
}

/// Java `String.format(Locale.ROOT, pattern, i)` for the `%d` patterns used by stories.
fn format_pattern(pattern: &str, i: i32) -> String {
    match pattern.find("%d") {
        Some(pos) => format!("{}{}{}", &pattern[..pos], i, &pattern[pos + 2..]),
        None => pattern.to_string(),
    }
}

pub struct ChildWindowStory;

impl Story for ChildWindowStory {
    fn title(&self) -> &str {
        "Containers/Child Window"
    }

    fn description(&self) -> &str {
        "Scrollable child region inside the story window."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.float32("height", 150.0)
            .bool("border", true)
            .string("header", "Panel")
            .preset("tall", [("height", 220.0f32)])
            .preset(
                "borderless",
                [("border", ArgValue::from(false)), ("height", 100.0f32.into())],
            );
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        ui.text(&ctx.string("header"));
        ui.separator();
        let height = ctx.float32("height");
        let border = ctx.bool("border");
        if help::begin_child("##panel", height, border) {
            for i in 1..=12 {
                ui.text(&format!("line {i} — scrollable content inside the child region"));
            }
        }
        help::end_child();
        ui.spacing();
        ui.text_disabled(format!("child height = {height}"));
    }
}

#[cfg(test)]
mod tests {
    use super::format_pattern;

    #[test]
    fn pattern_substitution() {
        assert_eq!(format_pattern("Item %d", 3), "Item 3");
        assert_eq!(format_pattern("Task %d done", 7), "Task 7 done");
        assert_eq!(format_pattern("no pattern", 1), "no pattern");
    }
}
