package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

import java.util.Locale;

public final class ColorPickerStory implements Story {

    @Override
    public String title() {
        return "Inputs/Color Picker";
    }

    @Override
    public String description() {
        return "Color edit swatch and full picker bound to a color arg.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.color("accent", 0x74502F)
                .bool("picker", false)
                .preset("picker", "picker", true)
                .preset("wolf", "accent", 0xC28E5F);
    }

    @Override
    public void render(StoryContext ctx) {
        int rgb = ctx.color("accent");
        float[] color = {
                ((rgb >> 16) & 0xFF) / 255f,
                ((rgb >> 8) & 0xFF) / 255f,
                (rgb & 0xFF) / 255f,
        };
        boolean changed = ctx.bool("picker")
                ? ImGui.colorPicker3("Accent", color)
                : ImGui.colorEdit3("Accent", color);
        if (changed) {
            ctx.set("accent", DemoColors.rgb(color));
            ctx.action("color");
        }
        ImGui.textDisabled(String.format(Locale.ROOT, "0x%06X", ctx.color("accent")));
    }
}
