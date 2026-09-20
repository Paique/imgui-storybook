//! Port of `InputTextStory`, `SliderStory`, `DragNumberStory`, `ComboSelectStory`,
//! `CheckboxRadioStory` and `ColorPickerStory`.

use crate::api::{ArgSet, ArgValue, Story};
use crate::ctx::StoryCtx;
use crate::ctx::StoryUi;
use super::help;
use crate::demo::{f32_to_rgb, rgb_to_f32};
use easy_imgui::{self as imgui, lbl};

pub struct InputTextStory {
    buffer: String,
}

impl Default for InputTextStory {
    fn default() -> Self {
        InputTextStory {
            buffer: String::with_capacity(256),
        }
    }
}

impl Story for InputTextStory {
    fn title(&self) -> &str {
        "Inputs/Text Field"
    }

    fn description(&self) -> &str {
        "Single-line text input with hint and password modes."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.string("label", "Username")
            .string("hint", "e.g. paique")
            .bool("password", false)
            .bool("readOnly", false)
            .preset(
                "secret",
                [
                    ("password", ArgValue::from(true)),
                    ("label", "API token".into()),
                ],
            )
            .preset(
                "locked",
                [("readOnly", ArgValue::from(true)), ("hint", "".into())],
            );
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let mut flags = imgui::InputTextFlags::None;
        if ctx.bool("password") {
            flags |= imgui::InputTextFlags::Password;
        }
        if ctx.bool("readOnly") {
            flags |= imgui::InputTextFlags::ReadOnly;
        }
        let label = ctx.string("label");
        let hint = ctx.string("hint");
        let changed = if hint.is_empty() {
            ui.input_text_config(lbl(&format!("{label}##field")), &mut self.buffer)
                .flags(flags)
                .build()
        } else {
            ui.input_text_hint_config(lbl(&format!("{label}##field")), &hint, &mut self.buffer)
                .flags(flags)
                .build()
        };
        if changed {
            ctx.action("edit");
        }
        ui.text_disabled(format!("current value: {}", self.buffer));
    }
}

pub struct SliderStory;

impl Story for SliderStory {
    fn title(&self) -> &str {
        "Inputs/Slider"
    }

    fn description(&self) -> &str {
        "Float slider with configurable range and format."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.float32("value", 42.0)
            .float32("min", 0.0)
            .float32("max", 100.0)
            .string("format", "%.0f%%")
            .preset("high", [("value", 90.0f32)])
            .preset(
                "percent",
                [("value", ArgValue::from(65.0f32)), ("format", "%.1f%%".into())],
            );
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let mut value = ctx.float32("value");
        let changed = help::slider_float(
            "Value",
            &mut value,
            ctx.float32("min"),
            ctx.float32("max"),
            &ctx.string("format"),
        );
        if changed {
            ctx.set("value", ArgValue::Float(value));
            ctx.action("slide");
        }
        ui.text_disabled("drag it in live mode");
    }
}

pub struct DragNumberStory;

impl Story for DragNumberStory {
    fn title(&self) -> &str {
        "Inputs/Drag Number"
    }

    fn description(&self) -> &str {
        "Drag controls for float and int values."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.float32("amount", 12.5)
            .float32("speed", 0.25)
            .int32("count", 7)
            .preset("fast", [("speed", 1.5f32), ("amount", 99.5f32)]);
    }

    fn render(&mut self, _ui: &StoryUi, ctx: &mut StoryCtx) {
        let mut amount = ctx.float32("amount");
        if help::drag_float("Amount", &mut amount, ctx.float32("speed"), "%.3f") {
            ctx.set("amount", ArgValue::Float(amount));
            ctx.action("drag-float");
        }
        let mut count = ctx.int32("count");
        if help::drag_int("Count", &mut count, 1.0, 0, 100) {
            ctx.set("count", ArgValue::Int(count));
            ctx.action("drag-int");
        }
    }
}

const SIZES: [&str; 3] = ["Small", "Medium", "Large"];

pub struct ComboSelectStory;

impl Story for ComboSelectStory {
    fn title(&self) -> &str {
        "Inputs/Combo & Select"
    }

    fn description(&self) -> &str {
        "Dropdown combo and list box bound to an enum arg."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.enum_of("size", &SIZES, "Medium")
            .bool("asList", false)
            .preset("list", [("asList", true)])
            .preset("large", [("size", "Large")]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let current_name = ctx.enum_str("size");
        let mut current: &str = SIZES
            .iter()
            .copied()
            .find(|s| *s == current_name)
            .unwrap_or(SIZES[1]);
        let changed = if ctx.bool("asList") {
            ui.list_box(lbl("Size"), -1, SIZES, |s| s, &mut current)
        } else {
            ui.combo(lbl("Size"), SIZES, |s| s, &mut current)
        };
        if changed && current != current_name {
            ctx.set("size", ArgValue::Str(current.to_string()));
            ctx.action(&format!("select:{current}"));
        }
    }
}

const FREQUENCIES: [&str; 3] = ["Never", "Daily", "Weekly"];

pub struct CheckboxRadioStory;

impl Story for CheckboxRadioStory {
    fn title(&self) -> &str {
        "Inputs/Checkbox & Radio"
    }

    fn description(&self) -> &str {
        "Checkbox and radio group, optionally disabled."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.bool("enable", true)
            .enum_of("frequency", &FREQUENCIES, "Daily")
            .bool("disabled", false)
            .preset("disabled", [("disabled", true)])
            .preset("weekly", [("frequency", "Weekly")]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let disabled = ctx.bool("disabled");
        if disabled {
            help::begin_disabled(true);
        }
        let mut enable = ctx.bool("enable");
        if ui.checkbox(lbl("Enable notifications"), &mut enable) {
            ctx.set("enable", ArgValue::Bool(enable));
            ctx.action("toggle");
        }
        ui.separator();
        let current = ctx.enum_str("frequency");
        for option in FREQUENCIES {
            if ui.radio_button_config(lbl(option), option == current).build() {
                ctx.set("frequency", ArgValue::Str(option.to_string()));
                ctx.action(&format!("frequency:{option}"));
            }
        }
        if disabled {
            help::end_disabled();
        }
    }
}

pub struct ColorPickerStory;

impl Story for ColorPickerStory {
    fn title(&self) -> &str {
        "Inputs/Color Picker"
    }

    fn description(&self) -> &str {
        "Color edit swatch and full picker bound to a color arg."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.color("accent", 0x74502F)
            .bool("picker", false)
            .preset("picker", [("picker", true)])
            .preset("wolf", [("accent", 0xC28E5Fu32)]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let rgb = ctx.color("accent");
        let mut color = rgb_to_f32(rgb);
        let changed = if ctx.bool("picker") {
            help::color_picker3("Accent", &mut color)
        } else {
            help::color_edit3("Accent", &mut color)
        };
        if changed {
            ctx.set("accent", ArgValue::Color(f32_to_rgb(&color)));
            ctx.action("color");
        }
        ui.text_disabled(format!("0x{:06X}", ctx.color("accent")));
    }
}
