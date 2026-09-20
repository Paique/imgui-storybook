package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class DragNumberStory implements Story {

    @Override
    public String title() {
        return "Inputs/Drag Number";
    }

    @Override
    public String description() {
        return "Drag controls for float and int values.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.float32("amount", 12.5f)
                .float32("speed", 0.25f)
                .int32("count", 7)
                .preset("fast", "speed", 1.5f, "amount", 99.5f);
    }

    @Override
    public void render(StoryContext ctx) {
        float[] amount = {ctx.float32("amount")};
        if (ImGui.dragFloat("Amount", amount, ctx.float32("speed"), 0f, 0f, "%.3f")) {
            ctx.set("amount", amount[0]);
            ctx.action("drag-float");
        }
        int[] count = {ctx.int32("count")};
        if (ImGui.dragInt("Count", count, 1f, 0, 100)) {
            ctx.set("count", count[0]);
            ctx.action("drag-int");
        }
    }
}
