package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class TabsStory implements Story {

    enum Tab { Overview, Settings, About }

    private static final String[] TABS = {"Overview", "Settings", "About"};

    @Override
    public String title() {
        return "Layout/Tabs";
    }

    @Override
    public String description() {
        return "Tab bar with per-tab content.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.enumOf("tab", Tab.class, Tab.Overview)
                .preset("settings", "tab", Tab.Settings)
                .preset("about", "tab", Tab.About);
    }

    @Override
    public void render(StoryContext ctx) {
        Tab current = ctx.enumValue("tab", Tab.class);
        String active = current == null ? "Overview" : current.name();
        if (ImGui.beginTabBar("##tabs", 0)) {
            for (String tab : TABS) {
                if (ImGui.beginTabItem(tab)) {
                    if (!tab.equals(active)) {
                        ctx.set("tab", tab);
                        ctx.action("tab:" + tab);
                        active = tab;
                    }
                    renderTabContent(tab);
                    ImGui.endTabItem();
                }
            }
            ImGui.endTabBar();
        }
    }

    private void renderTabContent(String tab) {
        switch (tab) {
            case "Overview" -> {
                ImGui.text("Summary of the current selection.");
                ImGui.bulletText("items: 12");
                ImGui.bulletText("pending: 3");
            }
            case "Settings" -> {
                ImGui.checkbox("Compact mode", new imgui.type.ImBoolean(false));
                ImGui.textDisabled("persisted by the product");
            }
            default -> ImGui.textWrapped("Tabbed navigation demo for imgui-storybook.");
        }
    }
}
