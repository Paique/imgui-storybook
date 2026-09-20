//! Built-in demo stories — Rust port of `host/demo/*` (16 stories covering the standard
//! ImGui widget vocabulary). Same titles, args, presets and behaviors as the Java host.

mod basics;
mod containers;
mod feedback;
pub(crate) mod help;
mod inputs;
mod layout;

use crate::registry::StoryRegistry;

/// Registers all demo stories (Java: `DemoStories.all()`).
pub fn register_all(registry: &mut StoryRegistry) {
    registry
        .add(basics::ButtonStory)
        .add(basics::TextStory)
        .add(inputs::InputTextStory::default())
        .add(inputs::SliderStory)
        .add(inputs::DragNumberStory)
        .add(inputs::ComboSelectStory)
        .add(inputs::CheckboxRadioStory)
        .add(inputs::ColorPickerStory)
        .add(layout::TableStory)
        .add(layout::TabsStory::default())
        .add(layout::TreeStory)
        .add(feedback::ProgressBarStory)
        .add(feedback::TooltipStory)
        .add(feedback::PopupModalStory)
        .add(containers::SelectableListStory)
        .add(containers::ChildWindowStory);
}

/// Number of built-in demo stories (for the serve banner).
pub fn count() -> usize {
    16
}

/// `0xRRGGBB` → `[r, g, b]` floats (Java `DemoColors.vec4` / `DemoColors.rgb`).
pub(crate) fn rgb_to_f32(rgb: u32) -> [f32; 3] {
    [
        ((rgb >> 16) & 0xFF) as f32 / 255.0,
        ((rgb >> 8) & 0xFF) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
    ]
}

pub(crate) fn f32_to_rgb(color: &[f32; 3]) -> u32 {
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    (ch(color[0]) << 16) | (ch(color[1]) << 8) | ch(color[2])
}
