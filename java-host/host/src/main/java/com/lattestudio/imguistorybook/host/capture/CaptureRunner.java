package com.lattestudio.imguistorybook.host.capture;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.Backdrop;
import com.lattestudio.imguistorybook.api.CanvasMode;
import com.lattestudio.imguistorybook.api.StoryRegistry;
import com.lattestudio.imguistorybook.api.StoryTheme;
import com.lattestudio.imguistorybook.api.Theme;
import com.lattestudio.imguistorybook.host.Cli;
import com.lattestudio.imguistorybook.host.HostInfo;
import com.lattestudio.imguistorybook.host.core.ArgValues;
import com.lattestudio.imguistorybook.host.core.HostedStoryContext;
import com.lattestudio.imguistorybook.host.core.ImGuiHost;
import com.lattestudio.imguistorybook.host.core.StoryCatalog;
import com.lattestudio.imguistorybook.host.core.ViewState;
import com.lattestudio.imguistorybook.host.gl.FrameGrabber;
import com.lattestudio.imguistorybook.host.gl.HeadlessGl;
import com.lattestudio.imguistorybook.host.net.InputInjector;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * Renders every story × preset × theme × scale offscreen and writes PNG captures plus a
 * captures.json manifest. These PNGs back the static (host-less) Storybook build and the
 * STORIES.md / manifest.json docs for AI consumption.
 */
public final class CaptureRunner {

    private CaptureRunner() {
    }

    public static void run(Cli cli, StoryRegistry registry, StoryTheme theme) throws Exception {
        List<Theme> themes = parseThemes(cli.themes());
        List<Float> scales = parseScales(cli.scales());
        Path outRoot = Path.of(cli.out());

        try (HeadlessGl gl = new HeadlessGl(cli.width(), cli.height())) {
            ImGuiHost host = new ImGuiHost(gl, theme);
            host.init();
            ViewState view = new ViewState();
            HostedStoryContext ctx = new HostedStoryContext(host, view);
            InputInjector noInput = new InputInjector();
            FrameGrabber grabber = new FrameGrabber();

            JsonArray entriesJson = new JsonArray();
            int count = 0;
            for (StoryRegistry.Entry entry : registry.entries()) {
                List<String> presetNames = new ArrayList<>();
                presetNames.add("default");
                presetNames.addAll(entry.args().presets().keySet());

                for (String preset : presetNames) {
                    Map<String, Object> args = ArgValues.materialize(
                            entry.args(), entry.args().presetValues(preset));
                    for (Theme themeMode : themes) {
                        for (float scale : scales) {
                            view.select(entry);
                            view.setArgs(args);
                            view.setTheme(themeMode);
                            view.setBackdrop(themeMode == Theme.LIGHT
                                    ? Backdrop.NEUTRAL_LIGHT : Backdrop.NEUTRAL_DARK);
                            view.setCanvasMode(CanvasMode.WINDOWED);
                            host.setScale(scale);

                            // warm-up frames so auto-resized windows settle
                            for (int i = 0; i < 3; i++) {
                                host.renderFrame(entry, view.args(), view, ctx, noInput);
                            }
                            java.nio.ByteBuffer rgba = grabber.grab(
                                    gl.framebufferWidth(), gl.framebufferHeight());

                            String file = "imgui/" + entry.id() + "/"
                                    + preset + "-" + themeDir(themeMode) + scaleSuffix(scale) + ".png";
                            grabber.png(rgba, gl.framebufferWidth(), gl.framebufferHeight(),
                                    outRoot.resolve(file));
                            count++;

                            JsonObject entryJson = new JsonObject();
                            entryJson.addProperty("storyId", entry.id());
                            entryJson.addProperty("title", entry.story().title());
                            entryJson.addProperty("description", entry.story().description());
                            entryJson.addProperty("storyClass", entry.story().getClass().getName());
                            entryJson.addProperty("preset", preset);
                            entryJson.addProperty("theme", themeDir(themeMode));
                            entryJson.addProperty("scale", scale);
                            entryJson.addProperty("file", file);
                            entryJson.addProperty("width", gl.framebufferWidth());
                            entryJson.addProperty("height", gl.framebufferHeight());
                            JsonObject argsJson = new JsonObject();
                            ArgValues.forJson(args, entry.args()).forEach((name, value) -> {
                                if (value instanceof Number n) {
                                    argsJson.addProperty(name, n);
                                } else if (value instanceof Boolean b) {
                                    argsJson.addProperty(name, b);
                                } else {
                                    argsJson.addProperty(name, String.valueOf(value));
                                }
                            });
                            entryJson.add("args", argsJson);
                            entriesJson.add(entryJson);
                        }
                    }
                }
            }

            JsonObject manifest = new JsonObject();
            manifest.addProperty("generatedAt", Instant.now().toString());
            manifest.addProperty("hostVersion", HostInfo.hostVersion());
            manifest.addProperty("imguiJavaVersion", HostInfo.imguiJavaVersion());
            manifest.addProperty("dearImgui", StoryCatalog.dearImguiVersion(HostInfo.imguiJavaVersion()));
            manifest.addProperty("width", gl.framebufferWidth());
            manifest.addProperty("height", gl.framebufferHeight());
            manifest.add("entries", entriesJson);
            Files.createDirectories(outRoot);
            Files.writeString(outRoot.resolve("captures.json"), manifest.toString());
            System.out.println("captured " + count + " images into " + outRoot.toAbsolutePath());
        }
    }

    private static List<Theme> parseThemes(List<String> themes) {
        List<Theme> out = new ArrayList<>();
        for (String theme : themes) {
            out.add(Theme.valueOf(theme.toUpperCase(Locale.ROOT)));
        }
        return out;
    }

    private static List<Float> parseScales(List<String> scales) {
        List<Float> out = new ArrayList<>();
        for (String scale : scales) {
            out.add(Float.parseFloat(scale));
        }
        return out;
    }

    private static String themeDir(Theme theme) {
        return theme == Theme.LIGHT ? "light" : "dark";
    }

    private static String scaleSuffix(float scale) {
        if (scale == 1f) {
            return "";
        }
        String trimmed = String.format(Locale.ROOT, "%.2f", scale)
                .replaceAll("0+$", "").replaceAll("\\.$", "");
        return "@" + trimmed + "x";
    }
}
