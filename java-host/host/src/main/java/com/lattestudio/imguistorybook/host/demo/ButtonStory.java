package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;
import imgui.ImVec4;
import imgui.flag.ImGuiCol;

public final class ButtonStory implements Story {

    @Override
    public String title() {
        return "Basics/Button";
    }

    @Override
    public String description() {
        return "Standard button with tone and size variants.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.string("label", "Confirm")
                .bool("danger", false)
                .bool("small", false)
                .preset("danger", "label", "Delete", "danger", true)
                .preset("small", "label", "OK", "small", true);
    }

    @Override
    public void render(StoryContext ctx) {
        String label = ctx.string("label");
        boolean danger = ctx.bool("danger");
        if (danger) {
            ImGui.pushStyleColor(ImGuiCol.Button, new ImVec4(0.62f, 0.16f, 0.16f, 1f));
            ImGui.pushStyleColor(ImGuiCol.ButtonHovered, new ImVec4(0.78f, 0.20f, 0.20f, 1f));
        }
        boolean clicked = ctx.bool("small")
                ? ImGui.smallButton(label)
                : ImGui.button(label, 0f, 0f);
        if (danger) {
            ImGui.popStyleColor(2);
        }
        if (clicked) {
            ctx.action("clicked");
        }
        ImGui.textDisabled("click it in live mode");
    }
}
