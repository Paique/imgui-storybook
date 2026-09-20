package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;
import imgui.type.ImBoolean;

public final class PopupModalStory implements Story {

    @Override
    public String title() {
        return "Feedback/Popup & Modal";
    }

    @Override
    public String description() {
        return "Confirmation modal opened from a button.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.string("title", "Confirm action")
                .string("message", "Proceed with this operation?")
                .bool("autoOpen", false)
                .preset("open", "autoOpen", true)
                .preset("delete", "autoOpen", true, "title", "Delete item",
                        "message", "This cannot be undone. Delete anyway?");
    }

    @Override
    public void render(StoryContext ctx) {
        String title = ctx.string("title");
        if (ctx.bool("autoOpen")) {
            ImGui.openPopup(title);
            ctx.set("autoOpen", false);
        }
        if (ImGui.button("Open...", 0f, 0f)) {
            ImGui.openPopup(title);
            ctx.action("open");
        }
        if (ImGui.beginPopupModal(title, new ImBoolean(true), 0)) {
            ImGui.text(ctx.string("message"));
            ImGui.spacing();
            if (ImGui.button("Confirm", 0f, 0f)) {
                ImGui.closeCurrentPopup();
                ctx.action("confirm");
            }
            ImGui.sameLine();
            if (ImGui.button("Cancel", 0f, 0f)) {
                ImGui.closeCurrentPopup();
                ctx.action("cancel");
            }
            ImGui.endPopup();
        }
    }
}
