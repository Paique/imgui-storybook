package com.lattestudio.imguistorybook.api;

/**
 * A single ImGui story: one component, one scenario.
 *
 * <p>Stories are discovered via {@link ServiceLoader} ({@code META-INF/services}) or registered
 * programmatically on {@link StoryRegistry}. Implementations may be stateful, but per-frame
 * values should be read from the {@link StoryContext} (and written back with
 * {@link StoryContext#set}) so that the host and the web Controls panel stay in sync.
 */
public interface Story {

    /** Story path using '/' as group separator, e.g. {@code "Inputs/Text Input"}. */
    String title();

    /** Short human/AI-readable description shown in docs exports. */
    default String description() {
        return "";
    }

    /** Declares the editable args of this story (drives the Controls panel and captures). */
    default void defineArgs(ArgSet args) {
    }

    /**
     * Screen-like stories render bare on the full canvas (like the in-game
     * frame) instead of inside a titled auto-resized window. Captures honor
     * this too, so full-frame compositions are captured full-bleed.
     */
    default boolean fullscreen() {
        return false;
    }

    /** Renders the story for the current frame. Called once per frame by the host. */
    void render(StoryContext ctx);
}
