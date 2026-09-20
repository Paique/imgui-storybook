package com.lattestudio.imguistorybook.api;

import imgui.ImFont;
import imgui.ImGui;
import imgui.ImFontAtlas;
import imgui.ImGuiIO;
import imgui.ImGuiStyle;

/**
 * Fonts + style of the product being documented. The host calls these around story rendering so
 * captures and live frames are pixel-identical to the real application.
 *
 * <p>Consumer projects implement this to replicate their in-product setup (fonts loaded from the
 * same TTFs at the same pixel sizes, same style/colors). The default keeps raw ImGui defaults,
 * which is what the {@code fabric-gui-imgui} integration uses before any customization.
 */
public interface StoryTheme {

    default String id() {
        return "default";
    }

    /**
     * (Re)loads the font atlas for the given scale. Called on startup and whenever the scale
     * changes; the host recreates the GL3 backend right after, so the new atlas texture is
     * picked up automatically.
     */
    default void loadFonts(ImGuiIO io, float scale) {
        ImFontAtlas fonts = io.getFonts();
        fonts.clear();
        fonts.addFontDefault();
        fonts.build();
        io.setFontGlobalScale(scale);
    }

    /**
     * Applies colors/metrics to the global style, once per frame, before the story renders.
     * At {@code scale == 1} the default implementation must not touch metrics, to keep raw
     * ImGui fidelity.
     */
    default void apply(ImGuiStyle style, Theme theme, float scale) {
        if (theme == Theme.LIGHT) {
            ImGui.styleColorsLight(style);
        } else {
            ImGui.styleColorsDark(style);
        }
        if (scale != 1f) {
            style.setWindowPadding(8 * scale, 8 * scale);
            style.setFramePadding(4 * scale, 3 * scale);
            style.setItemSpacing(8 * scale, 4 * scale);
            style.setItemInnerSpacing(4 * scale, 4 * scale);
            style.setIndentSpacing(21 * scale);
            style.setScrollbarSize(14 * scale);
        }
    }

    /** Default theme: raw ImGui (what the Fabric integration uses before customization). */
    StoryTheme DEFAULT = new StoryTheme() {
    };

    /** Helper for subclasses that manage named fonts. */
    default ImFont font(String name) {
        return null;
    }
}
