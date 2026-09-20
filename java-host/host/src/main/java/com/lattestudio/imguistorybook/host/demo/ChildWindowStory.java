package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class ChildWindowStory implements Story {

    @Override
    public String title() {
        return "Containers/Child Window";
    }

    @Override
    public String description() {
        return "Scrollable child region inside the story window.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.float32("height", 150f)
                .bool("border", true)
                .string("header", "Panel")
                .preset("tall", "height", 220f)
                .preset("borderless", "border", false, "height", 100f);
    }

    @Override
    public void render(StoryContext ctx) {
        ImGui.text(ctx.string("header"));
        ImGui.separator();
        if (ImGui.beginChild("##panel", 0f, ctx.float32("height"), ctx.bool("border"))) {
            for (int i = 1; i <= 12; i++) {
                ImGui.text("line " + i + " — scrollable content inside the child region");
            }
        }
        ImGui.endChild();
        ImGui.spacing();
        ImGui.textDisabled("child height = " + ctx.float32("height"));
    }
}
