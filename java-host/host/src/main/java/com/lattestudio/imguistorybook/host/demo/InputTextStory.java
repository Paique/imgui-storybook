package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;
import imgui.flag.ImGuiInputTextFlags;
import imgui.type.ImString;

public final class InputTextStory implements Story {

    private final ImString buffer = new ImString(256);

    @Override
    public String title() {
        return "Inputs/Text Field";
    }

    @Override
    public String description() {
        return "Single-line text input with hint and password modes.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.string("label", "Username")
                .string("hint", "e.g. paique")
                .bool("password", false)
                .bool("readOnly", false)
                .preset("secret", "password", true, "label", "API token")
                .preset("locked", "readOnly", true, "hint", "");
    }

    @Override
    public void render(StoryContext ctx) {
        int flags = 0;
        if (ctx.bool("password")) {
            flags |= ImGuiInputTextFlags.Password;
        }
        if (ctx.bool("readOnly")) {
            flags |= ImGuiInputTextFlags.ReadOnly;
        }
        boolean changed;
        String hint = ctx.string("hint");
        if (hint.isEmpty()) {
            changed = ImGui.inputText(ctx.string("label") + "##field", buffer, flags);
        } else {
            changed = ImGui.inputTextWithHint(ctx.string("label") + "##field", hint, buffer, flags);
        }
        if (changed) {
            ctx.action("edit");
        }
        ImGui.textDisabled("current value: " + buffer.get());
    }
}
