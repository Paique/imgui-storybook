package com.lattestudio.imguistorybook.api;

/**
 * Per-frame view of the current story state: typed arg readers, view info and action reporting.
 */
public interface StoryContext {

    String string(String name);

    boolean bool(String name);

    int int32(String name);

    float float32(String name);

    <T extends Enum<T>> T enumValue(String name, Class<T> type);

    /** @return the color arg as {@code 0xRRGGBB} */
    int color(String name);

    /** Current theme of the canvas (light/dark), so stories can adapt if needed. */
    Theme theme();

    /** Current UI scale (font/metric scale, 1 = native). */
    float scale();

    /**
     * Reports an interaction performed inside the story (shown as a toast on the canvas overlay
     * and forwarded to the web client). Example: {@code ctx.action("clicked")}.
     */
    void action(String name);

    /**
     * Updates an arg from inside the story (e.g. a slider dragged in live mode). Keeps host,
     * canvas and Controls panel coherent; unknown names are ignored.
     */
    void set(String name, Object value);
}
