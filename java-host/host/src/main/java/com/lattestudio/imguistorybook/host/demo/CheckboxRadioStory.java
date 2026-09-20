package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;
import imgui.type.ImBoolean;

public final class CheckboxRadioStory implements Story {

    enum Frequency { Never, Daily, Weekly }

    private static final String[] OPTIONS = {"Never", "Daily", "Weekly"};

    @Override
    public String title() {
        return "Inputs/Checkbox & Radio";
    }

    @Override
    public String description() {
        return "Checkbox and radio group, optionally disabled.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.bool("enable", true)
                .enumOf("frequency", Frequency.class, Frequency.Daily)
                .bool("disabled", false)
                .preset("disabled", "disabled", true)
                .preset("weekly", "frequency", Frequency.Weekly);
    }

    @Override
    public void render(StoryContext ctx) {
        boolean disabled = ctx.bool("disabled");
        if (disabled) {
            ImGui.beginDisabled(true);
        }
        ImBoolean enable = new ImBoolean(ctx.bool("enable"));
        if (ImGui.checkbox("Enable notifications", enable)) {
            ctx.set("enable", enable.get());
            ctx.action("toggle");
        }
        ImGui.separator();
        Frequency frequency = ctx.enumValue("frequency", Frequency.class);
        String current = frequency == null ? "Daily" : frequency.name();
        for (String option : OPTIONS) {
            if (ImGui.radioButton(option, option.equals(current))) {
                ctx.set("frequency", option);
                ctx.action("frequency:" + option);
            }
        }
        if (disabled) {
            ImGui.endDisabled();
        }
    }
}
