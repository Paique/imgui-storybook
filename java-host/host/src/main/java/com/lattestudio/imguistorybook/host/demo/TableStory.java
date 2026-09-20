package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;
import imgui.flag.ImGuiTableFlags;

public final class TableStory implements Story {

    @Override
    public String title() {
        return "Layout/Table";
    }

    @Override
    public String description() {
        return "Data table with borders, row striping and header.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.int32("rows", 4)
                .bool("striped", true)
                .bool("withHeader", true)
                .preset("dense", "rows", 8)
                .preset("plain", "striped", false, "withHeader", false);
    }

    @Override
    public void render(StoryContext ctx) {
        int flags = ImGuiTableFlags.Borders;
        if (ctx.bool("striped")) {
            flags |= ImGuiTableFlags.RowBg;
        }
        int rows = Math.max(1, Math.min(20, ctx.int32("rows")));
        if (ImGui.beginTable("demo", 3, flags)) {
            ImGui.tableSetupColumn("ID");
            ImGui.tableSetupColumn("Name");
            ImGui.tableSetupColumn("Status");
            if (ctx.bool("withHeader")) {
                ImGui.tableHeadersRow();
            }
            for (int i = 0; i < rows; i++) {
                ImGui.tableNextRow();
                ImGui.tableNextColumn();
                ImGui.text("#" + (1000 + i));
                ImGui.tableNextColumn();
                ImGui.text("Task " + (i + 1));
                ImGui.tableNextColumn();
                ImGui.text(i % 3 == 0 ? "open" : "progress");
            }
            ImGui.endTable();
        }
    }
}
