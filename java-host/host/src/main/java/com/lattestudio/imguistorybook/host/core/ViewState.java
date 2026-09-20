package com.lattestudio.imguistorybook.host.core;

import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Backdrop;
import com.lattestudio.imguistorybook.api.CanvasMode;
import com.lattestudio.imguistorybook.api.StoryRegistry;
import com.lattestudio.imguistorybook.api.Theme;

import java.util.Map;

/**
 * Current selection + view. Mutated from WS threads (volatile field swaps only),
 * consumed by the render thread.
 */
public final class ViewState {

    private volatile StoryRegistry.Entry entry;
    private volatile Map<String, Object> args = Map.of();
    private volatile Theme theme = Theme.DARK;
    private volatile Backdrop backdrop = Backdrop.NEUTRAL_DARK;
    private volatile CanvasMode canvasMode = CanvasMode.WINDOWED;
    private volatile float scale = 1f;

    public StoryRegistry.Entry entry() {
        return entry;
    }

    public Map<String, Object> args() {
        return args;
    }

    public Theme theme() {
        return theme;
    }

    public Backdrop backdrop() {
        return backdrop;
    }

    public CanvasMode canvasMode() {
        return canvasMode;
    }

    public float scale() {
        return scale;
    }

    public void select(StoryRegistry.Entry newEntry) {
        this.entry = newEntry;
        this.args = newEntry == null ? Map.of() : newEntry.args().defaults();
    }

    public void setArgs(Map<String, Object> sanitized) {
        this.args = Map.copyOf(sanitized);
    }

    /** Copy-on-write single arg update (used by {@code StoryContext.set}). */
    public void putArg(ArgSet set, String name, Object value) {
        Map<String, Object> copy = new java.util.HashMap<>(args);
        copy.put(name, value);
        this.args = Map.copyOf(copy);
    }

    public void setTheme(Theme theme) {
        this.theme = theme;
    }

    public void setBackdrop(Backdrop backdrop) {
        this.backdrop = backdrop;
    }

    public void setCanvasMode(CanvasMode canvasMode) {
        this.canvasMode = canvasMode;
    }

    public void setScale(float scale) {
        this.scale = scale;
    }
}
