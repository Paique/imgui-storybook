package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

import java.util.Locale;

public final class ProgressBarStory implements Story {

    @Override
    public String title() {
        return "Feedback/Progress Bar";
    }

    @Override
    public String description() {
        return "Progress bar with overlay label and an advance action.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.float32("value", 0.65f)
                .string("overlay", "")
                .preset("done", "value", 1f)
                .preset("labeled", "value", 0.3f, "overlay", "3 of 10");
    }

    @Override
    public void render(StoryContext ctx) {
        float value = Math.max(0f, Math.min(1f, ctx.float32("value")));
        String overlay = ctx.string("overlay");
        if (overlay.isEmpty()) {
            overlay = String.format(Locale.ROOT, "%.0f%%", value * 100f);
        }
        ImGui.progressBar(value, -1f, 0f, overlay);
        ImGui.spacing();
        if (ImGui.button("Advance", 0f, 0f)) {
            ctx.set("value", Math.min(1f, value + 0.1f));
            ctx.action("advance");
        }
        ImGui.sameLine();
        ImGui.textDisabled("value = " + String.format(Locale.ROOT, "%.2f", value));
    }
}
