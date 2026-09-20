package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

import java.util.Locale;

public final class SelectableListStory implements Story {

    @Override
    public String title() {
        return "Containers/Selectable List";
    }

    @Override
    public String description() {
        return "List of selectable rows emitting selection actions.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.int32("count", 5)
                .string("pattern", "Item %d")
                .preset("many", "count", 10)
                .preset("tasks", "count", 3, "pattern", "Task %d");
    }

    @Override
    public void render(StoryContext ctx) {
        int count = Math.max(1, Math.min(20, ctx.int32("count")));
        for (int i = 1; i <= count; i++) {
            String label = String.format(Locale.ROOT, ctx.string("pattern"), i);
            if (ImGui.selectable(label + "##" + i, false, 0, 0f, 0f)) {
                ctx.action("select:" + i);
            }
        }
    }
}
