package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;
import imgui.type.ImInt;

public final class ComboSelectStory implements Story {

    enum Size { Small, Medium, Large }

    private static final String[] OPTIONS = {"Small", "Medium", "Large"};

    @Override
    public String title() {
        return "Inputs/Combo & Select";
    }

    @Override
    public String description() {
        return "Dropdown combo and list box bound to an enum arg.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.enumOf("size", Size.class, Size.Medium)
                .bool("asList", false)
                .preset("list", "asList", true)
                .preset("large", "size", Size.Large);
    }

    @Override
    public void render(StoryContext ctx) {
        Size current = ctx.enumValue("size", Size.class);
        int index = current == null ? 1 : current.ordinal();
        ImInt selected = new ImInt(index);
        if (ctx.bool("asList")) {
            // binding quirk: listBox returns void; detect change via ImInt
            ImGui.listBox("Size", selected, OPTIONS);
        } else {
            ImGui.combo("Size", selected, OPTIONS);
        }
        int chosen = selected.get();
        if (chosen != index && chosen >= 0 && chosen < OPTIONS.length) {
            ctx.set("size", OPTIONS[chosen]);
            ctx.action("select:" + OPTIONS[chosen]);
        }
    }
}
