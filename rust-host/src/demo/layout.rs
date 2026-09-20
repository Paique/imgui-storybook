//! Port of `TableStory`, `TabsStory` and `TreeStory`.

use crate::api::{ArgSet, ArgValue, Story};
use crate::ctx::StoryCtx;
use crate::ctx::StoryUi;
use super::help;
use easy_imgui::{lbl, TreeNodeFlags};

pub struct TableStory;

impl Story for TableStory {
    fn title(&self) -> &str {
        "Layout/Table"
    }

    fn description(&self) -> &str {
        "Data table with borders, row striping and header."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.int32("rows", 4)
            .bool("striped", true)
            .bool("withHeader", true)
            .preset("dense", [("rows", 8)])
            .preset("plain", [("striped", false), ("withHeader", false)]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let mut flags = help::table_flags_borders();
        if ctx.bool("striped") {
            flags |= help::table_flags_row_bg();
        }
        let rows = ctx.int32("rows").clamp(1, 20);
        if help::begin_table("demo", 3, flags) {
            help::table_setup_column("ID");
            help::table_setup_column("Name");
            help::table_setup_column("Status");
            if ctx.bool("withHeader") {
                help::table_headers_row();
            }
            for i in 0..rows {
                help::table_next_row();
                help::table_next_column();
                ui.text(&format!("#{}", 1000 + i));
                help::table_next_column();
                ui.text(&format!("Task {}", i + 1));
                help::table_next_column();
                ui.text(if i % 3 == 0 { "open" } else { "progress" });
            }
            help::end_table();
        }
    }
}

const TABS: [&str; 3] = ["Overview", "Settings", "About"];

pub struct TabsStory {
    compact: bool,
}

impl Default for TabsStory {
    fn default() -> Self {
        TabsStory { compact: false }
    }
}

impl Story for TabsStory {
    fn title(&self) -> &str {
        "Layout/Tabs"
    }

    fn description(&self) -> &str {
        "Tab bar with per-tab content."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.enum_of("tab", &TABS, "Overview")
            .preset("settings", [("tab", "Settings")])
            .preset("about", [("tab", "About")]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let active = ctx.enum_str("tab");
        if help::begin_tab_bar("##tabs") {
            for tab in TABS {
                if help::begin_tab_item(tab) {
                    if tab != active {
                        ctx.set("tab", ArgValue::Str(tab.to_string()));
                        ctx.action(&format!("tab:{tab}"));
                    }
                    self.render_tab_content(ui, tab);
                    help::end_tab_item();
                }
            }
            help::end_tab_bar();
        }
    }
}

impl TabsStory {
    fn render_tab_content(&mut self, ui: &StoryUi, tab: &str) {
        match tab {
            "Overview" => {
                ui.text("Summary of the current selection.");
                ui.bullet_text("items: 12");
                ui.bullet_text("pending: 3");
            }
            "Settings" => {
                ui.checkbox(lbl("Compact mode"), &mut self.compact);
                ui.text_disabled("persisted by the product");
            }
            _ => ui.text_wrapped("Tabbed navigation demo for imgui-storybook."),
        }
    }
}

pub struct TreeStory;

impl Story for TreeStory {
    fn title(&self) -> &str {
        "Layout/Tree & Headers"
    }

    fn description(&self) -> &str {
        "Collapsing header with nested tree nodes."
    }

    fn define_args(&self, args: &mut ArgSet) {
        args.int32("depth", 2)
            .preset("deep", [("depth", 3)])
            .preset("flat", [("depth", 1)]);
    }

    fn render(&mut self, ui: &StoryUi, ctx: &mut StoryCtx) {
        let depth = ctx.int32("depth").clamp(1, 3);
        ui.collapsing_header_config(lbl("Project"))
            .flags(TreeNodeFlags::DefaultOpen)
            .with(|| render_nodes(ui, "src", depth));
        ui.collapsing_header_config(lbl("Resources (collapsed)"))
            .with(|| ui.text("hidden by default"));
    }
}

fn render_nodes(ui: &StoryUi, label: &str, remaining_depth: i32) {
    for i in 1..=2 {
        let node = format!("{label}/{i}");
        if remaining_depth <= 1 {
            ui.bullet_text(&node);
        } else {
            ui.tree_node_config(lbl(&node)).with(|| {
                render_nodes(ui, &node, remaining_depth - 1);
            });
        }
    }
}
