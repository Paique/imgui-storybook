package com.lattestudio.imguistorybook.api;

/** How the story is laid out on the canvas. */
public enum CanvasMode {
    /** Story renders inside a titled, auto-resized ImGui window (as it appears in-game). */
    WINDOWED,
    /** Story renders bare on a borderless full-canvas host window. */
    INLINE
}
