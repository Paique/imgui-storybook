package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class TextStory implements Story {

    @Override
    public String title() {
        return "Basics/Text & Labels";
    }

    @Override
    public String description() {
        return "Plain, wrapped, bulleted and muted text.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.string("text", "The quick brown fox jumps over the lazy dog.")
                .bool("wrapped", false)
                .bool("bullet", false)
                .preset("wrapped", "wrapped", true)
                .preset("bullets", "bullet", true);
    }

    @Override
    public void render(StoryContext ctx) {
        String text = ctx.string("text");
        if (ctx.bool("bullet")) {
            ImGui.bulletText(text);
        } else if (ctx.bool("wrapped")) {
            ImGui.textWrapped(text);
        } else {
            ImGui.text(text);
        }
        ImGui.separator();
        ImGui.textDisabled("muted / secondary text");
        ImGui.spacing();
        ImGui.text("Value labels and sizes come from the active theme's fonts.");
    }
}
