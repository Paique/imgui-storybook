package com.lattestudio.imguistorybook.host.demo;

import imgui.ImVec4;

/** Small color helpers for the demo stories. */
final class DemoColors {

    private DemoColors() {
    }

    /** @param rgb color as {@code 0xRRGGBB} */
    static ImVec4 vec4(int rgb, float alpha) {
        float r = ((rgb >> 16) & 0xFF) / 255f;
        float g = ((rgb >> 8) & 0xFF) / 255f;
        float b = (rgb & 0xFF) / 255f;
        return new ImVec4(r, g, b, alpha);
    }

    static int rgb(float[] color) {
        int r = (int) Math.round(Math.clamp(color[0], 0f, 1f) * 255f);
        int g = (int) Math.round(Math.clamp(color[1], 0f, 1f) * 255f);
        int b = (int) Math.round(Math.clamp(color[2], 0f, 1f) * 255f);
        return (r << 16) | (g << 8) | b;
    }
}
