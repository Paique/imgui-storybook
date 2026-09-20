package com.lattestudio.imguistorybook.host.demo;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryContext;
import imgui.ImGui;

public final class TreeStory implements Story {

    @Override
    public String title() {
        return "Layout/Tree & Headers";
    }

    @Override
    public String description() {
        return "Collapsing header with nested tree nodes.";
    }

    @Override
    public void defineArgs(ArgSet args) {
        args.int32("depth", 2)
                .preset("deep", "depth", 3)
                .preset("flat", "depth", 1);
    }

    @Override
    public void render(StoryContext ctx) {
        int depth = Math.max(1, Math.min(3, ctx.int32("depth")));
        if (ImGui.collapsingHeader("Project", imgui.flag.ImGuiTreeNodeFlags.DefaultOpen)) {
            renderNodes("src", depth);
        }
        if (ImGui.collapsingHeader("Resources (collapsed)")) {
            ImGui.text("hidden by default");
        }
    }

    private void renderNodes(String label, int remainingDepth) {
        for (int i = 1; i <= 2; i++) {
            String node = label + "/" + i;
            if (remainingDepth <= 1) {
                ImGui.bulletText(node);
            } else if (ImGui.treeNode(node)) {
                renderNodes(node, remainingDepth - 1);
                ImGui.treePop();
            }
        }
    }
}
