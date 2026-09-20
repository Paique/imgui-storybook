package com.lattestudio.imguistorybook.host.core;

import com.lattestudio.imguistorybook.api.Backdrop;
import com.lattestudio.imguistorybook.api.CanvasMode;
import com.lattestudio.imguistorybook.api.Story;
import com.lattestudio.imguistorybook.api.StoryRegistry;
import com.lattestudio.imguistorybook.api.StoryTheme;
import com.lattestudio.imguistorybook.host.gl.HeadlessGl;
import com.lattestudio.imguistorybook.host.net.InputInjector;
import imgui.ImGui;
import imgui.ImGuiIO;
import imgui.ImGuiViewport;
import imgui.gl3.ImGuiImplGl3;
import imgui.glfw.ImGuiImplGlfw;
import imgui.ImVec2;
import imgui.type.ImBoolean;

import java.util.ArrayDeque;
import java.util.Iterator;
import java.util.Map;
import java.util.function.Consumer;

import org.lwjgl.glfw.GLFWNativeWin32;
import org.lwjgl.system.Platform;

import static org.lwjgl.opengl.GL11.GL_COLOR_BUFFER_BIT;
import static org.lwjgl.opengl.GL11.glClear;
import static org.lwjgl.opengl.GL11.glClearColor;

/**
 * Owns the ImGui context + GLFW/GL3 backends and renders one story per frame into the hidden
 * window's framebuffer. Frame order follows the official imgui-java loop:
 * clear → gl3.newFrame → glfw.newFrame → (synthetic input) → style → ImGui.newFrame → story →
 * render → renderDrawData. No swap: callers read the framebuffer back.
 */
public final class ImGuiHost implements AutoCloseable {

    private static final String GLSL_VERSION = "#version 330";
    private static final int CANVAS_PADDING = 24;
    private static final long ACTION_TOAST_MS = 3000;

    /** Interaction reported by a story via {@code ctx.action(...)}. */
    public record ActionEvent(String name, long atMs) {
    }

    private final HeadlessGl gl;
    private final StoryTheme theme;
    private final ImGuiImplGlfw imGuiGlfw = new ImGuiImplGlfw();
    private final ImGuiImplGl3 imGuiGl3 = new ImGuiImplGl3();
    private final ArrayDeque<ActionEvent> actions = new ArrayDeque<>();
    private volatile Consumer<ActionEvent> actionListener;
    private float scale = 1f;

    public ImGuiHost(HeadlessGl gl, StoryTheme theme) {
        this.gl = gl;
        this.theme = theme;
    }

    public void init() {
        ImGui.createContext();
        ImGuiIO io = ImGui.getIO();
        io.setIniFilename(null);
        io.setConfigFlags(imgui.flag.ImGuiConfigFlags.NavEnableKeyboard);
        imGuiGlfw.initForOpenGL(gl.window(), false);
        imGuiGl3.init(GLSL_VERSION);
        theme.loadFonts(io, scale);
    }

    public float scale() {
        return scale;
    }

    /** Applies a new scale: reloads the font atlas and recreates the GL3 backend texture. */
    public void setScale(float newScale) {
        if (newScale <= 0f || newScale == scale) {
            return;
        }
        scale = newScale;
        theme.loadFonts(ImGui.getIO(), scale);
        imGuiGl3.shutdown();
        imGuiGl3.init(GLSL_VERSION);
        restorePlatformHandles();
    }

    /**
     * Re-points the main viewport at our GLFW window. The GL3 backend's shutdown tears down the
     * platform interfaces and nulls the main viewport's platform handle; the next
     * {@code imGuiGlfw.newFrame()} then reads it in {@code updateMouseData} and crashes on
     * {@code glfwGetWindowAttrib(NULL)} if nobody restores it.
     */
    private void restorePlatformHandles() {
        ImGuiViewport mainViewport = ImGui.getMainViewport();
        mainViewport.setPlatformHandle(gl.window());
        if (Platform.get() == Platform.WINDOWS) {
            mainViewport.setPlatformHandleRaw(GLFWNativeWin32.glfwGetWin32Window(gl.window()));
        }
    }

    public void setActionListener(Consumer<ActionEvent> listener) {
        this.actionListener = listener;
    }

    public void recordAction(String name) {
        ActionEvent event = new ActionEvent(name, System.currentTimeMillis());
        synchronized (actions) {
            actions.addLast(event);
            while (actions.size() > 20) {
                actions.removeFirst();
            }
        }
        Consumer<ActionEvent> listener = actionListener;
        if (listener != null) {
            listener.accept(event);
        }
    }

    /** Drops queued action toasts, so one story's actions don't bleed into the next capture. */
    public void clearActions() {
        synchronized (actions) {
            actions.clear();
        }
    }

    /**
     * Renders one frame. Synthetic input is applied after {@code glfw.newFrame()} so queued
     * browser events win over the backend's cursor-polling fallback.
     */
    public void renderFrame(StoryRegistry.Entry entry,
                            Map<String, Object> args,
                            ViewState view,
                            HostedStoryContext ctx,
                            InputInjector input) {
        gl.bindFramebuffer();
        clearBackdrop(view.backdrop());
        imGuiGl3.newFrame();
        imGuiGlfw.newFrame();
        // hidden window reports 0×0: force the logical size to our exact offscreen framebuffer
        ImGuiIO io = ImGui.getIO();
        io.setDisplaySize(gl.width(), gl.height());
        io.setDisplayFramebufferScale(1f, 1f);
        input.apply(io);
        theme.apply(ImGui.getStyle(), view.theme(), scale);
        ImGui.newFrame();
        if (view.backdrop() == Backdrop.CHECKER) {
            drawChecker(ImGui.getIO());
        }
        renderStory(entry, args, view, ctx);
        drawActionToasts(ImGui.getIO());
        ImGui.render();
        imGuiGl3.renderDrawData(ImGui.getDrawData());
    }

    private void clearBackdrop(Backdrop backdrop) {
        float r;
        float g;
        float b;
        switch (backdrop) {
            case NEUTRAL_LIGHT -> {
                r = 0.92f;
                g = 0.92f;
                b = 0.94f;
            }
            case CHECKER -> {
                r = 0.92f;
                g = 0.92f;
                b = 0.94f;
            }
            default -> {
                r = 0.12f;
                g = 0.12f;
                b = 0.14f;
            }
        }
        glClearColor(r, g, b, 1f);
        glClear(GL_COLOR_BUFFER_BIT);
    }

    private void drawChecker(ImGuiIO io) {
        var background = ImGui.getBackgroundDrawList();
        ImVec2 display = io.getDisplaySize();
        float width = display.x;
        float height = display.y;
        int square = packAbgr(255, 220, 220, 224);
        float cell = 24f;
        ImVec2 pMin = new ImVec2();
        ImVec2 pMax = new ImVec2();
        for (int cellY = 0; cellY * cell < height; cellY++) {
            for (int cellX = 0; cellX * cell < width; cellX++) {
                if (((cellX + cellY) & 1) == 0) {
                    continue;
                }
                pMin.x = cellX * cell;
                pMin.y = cellY * cell;
                pMax.x = Math.min((cellX + 1) * cell, width);
                pMax.y = Math.min((cellY + 1) * cell, height);
                background.addRectFilled(pMin, pMax, square, 0f);
            }
        }
    }

    private void drawActionToasts(ImGuiIO io) {
        long now = System.currentTimeMillis();
        float y = io.getDisplaySize().y - 36f;
        int color = packAbgr(235, 255, 255, 255);
        synchronized (actions) {
            if (actions.isEmpty()) {
                return;
            }
            var foreground = ImGui.getForegroundDrawList();
            Iterator<ActionEvent> it = actions.descendingIterator();
            int drawn = 0;
            while (it.hasNext() && drawn < 5) {
                ActionEvent event = it.next();
                if (now - event.atMs() > ACTION_TOAST_MS) {
                    break;
                }
                foreground.addText(new ImVec2(14f, y), color, "[action] " + event.name());
                y -= 22f;
                drawn++;
            }
        }
    }

    private void renderStory(StoryRegistry.Entry entry,
                             Map<String, Object> args,
                             ViewState view,
                             HostedStoryContext ctx) {
        ctx.update(entry, args, view);
        Story story = entry.story();
        if (view.canvasMode() == CanvasMode.WINDOWED && !entry.story().fullscreen()) {
            ImGui.setNextWindowPos(CANVAS_PADDING, CANVAS_PADDING, imgui.flag.ImGuiCond.Appearing);
            int flags = imgui.flag.ImGuiWindowFlags.NoCollapse
                    | imgui.flag.ImGuiWindowFlags.NoSavedSettings;
            if (ImGui.begin(story.title(), new ImBoolean(true), flags)) {
                story.render(ctx);
            }
            ImGui.end();
        } else {
            ImVec2 display = ImGui.getIO().getDisplaySize();
            ImGui.setNextWindowPos(0f, 0f, imgui.flag.ImGuiCond.Always);
            ImGui.setNextWindowSize(display.x, display.y, imgui.flag.ImGuiCond.Always);
            int flags = imgui.flag.ImGuiWindowFlags.NoDecoration
                    | imgui.flag.ImGuiWindowFlags.NoMove
                    | imgui.flag.ImGuiWindowFlags.NoSavedSettings
                    | imgui.flag.ImGuiWindowFlags.NoBringToFrontOnFocus;
            if (ImGui.begin("##imgui-storybook-canvas", new ImBoolean(true), flags)) {
                story.render(ctx);
            }
            ImGui.end();
        }
    }

    /** Packs RGBA into the ABGR int ImGui expects for draw-list colors. */
    public static int packAbgr(int r, int g, int b, int a) {
        return (a << 24) | (b << 16) | (g << 8) | r;
    }

    @Override
    public void close() {
        imGuiGl3.shutdown();
        imGuiGlfw.shutdown();
        ImGui.destroyContext();
    }
}
