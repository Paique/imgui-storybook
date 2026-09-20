package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class SliderStory implements Story {

    @Override
    public String title() {
        return "Inputs/Slider";
    }

    @Override
    public String description() {
        return "Float slider with configurable range and format.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.float32("value", 42f)
                .float32("min", 0f)
                .float32("max", 100f)
                .string("format", "%.0f%%")
                .preset("high", "value", 90f)
                .preset("percent", "value", 65f, "format", "%.1f%%");
    }

    @Override
    public void render(StoryContext ctx) {
        float[] value = {ctx.float32("value")};
        boolean changed = ImGui.sliderFloat("Value", value,
                ctx.float32("min"), ctx.float32("max"), ctx.string("format"));
        if (changed) {
            ctx.set("value", value[0]);
            ctx.action("slide");
        }
        ImGui.textDisabled("drag it in live mode");
    }
}
