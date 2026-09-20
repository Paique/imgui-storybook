package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class TooltipStory implements Story {

    @Override
    public String title() {
        return "Feedback/Tooltip";
    }

    @Override
    public String description() {
        return "Hover tooltip with help-marker pattern.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.string("text", "Extra explanation shown on hover.")
                .bool("helpMarker", true)
                .preset("button", "helpMarker", false);
    }

    @Override
    public void render(StoryContext ctx) {
        ImGui.text("Hover the marker");
        if (ctx.bool("helpMarker")) {
            ImGui.sameLine();
            ImGui.textDisabled("(?)");
        } else {
            ImGui.text("or the button:");
            ImGui.sameLine();
            ImGui.button("Hover me", 0f, 0f);
        }
        if (ImGui.isItemHovered()) {
            ImGui.setTooltip(ctx.string("text"));
            // hover state only shows in live mode
        }
    }
}
