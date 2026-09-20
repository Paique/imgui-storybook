package com.lattestudio.imguistorybook.host.core;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.api.ArgSpec;
import com.lattestudio.imguistorybook.api.ArgType;
import com.lattestudio.imguistorybook.api.StoryRegistry;

import java.util.Map;

/**
 * Catalog JSON consumed by the TS story generator and by the static docs build.
 * Shape (versioned): { hostVersion, imguiJavaVersion, dearImgui, stories: [...] }
 */
public final class StoryCatalog {

    private StoryCatalog() {
    }

    public static JsonObject toJson(StoryRegistry registry, String hostVersion, String imguiJavaVersion) {
        JsonObject root = new JsonObject();
        root.addProperty("hostVersion", hostVersion);
        root.addProperty("imguiJavaVersion", imguiJavaVersion);
        root.addProperty("dearImgui", dearImguiVersion(imguiJavaVersion));

        JsonArray stories = new JsonArray();
        for (StoryRegistry.Entry entry : registry.entries()) {
            stories.add(storyJson(entry));
        }
        root.add("stories", stories);
        return root;
    }

    private static JsonObject storyJson(StoryRegistry.Entry entry) {
        JsonObject story = new JsonObject();
        story.addProperty("id", entry.id());
        story.addProperty("title", entry.story().title());
        story.addProperty("description", entry.story().description());
        story.addProperty("storyClass", entry.story().getClass().getName());

        ArgSet args = entry.args();
        JsonArray argArray = new JsonArray();
        for (ArgSpec spec : args.specs()) {
            JsonObject arg = new JsonObject();
            arg.addProperty("name", spec.name());
            arg.addProperty("type", spec.type().name());
            arg.add("default", typedDefault(spec));
            if (spec.type() == ArgType.ENUM) {
                JsonArray options = new JsonArray();
                for (String option : spec.enumOptions()) {
                    options.add(option);
                }
                arg.add("options", options);
            }
            argArray.add(arg);
        }
        story.add("args", argArray);

        JsonObject presets = new JsonObject();
        args.presets().forEach((name, values) ->
                presets.add(name, gsonObject(ArgValues.forJson(
                        ArgValues.materialize(args, values), args))));
        story.add("presets", presets);
        return story;
    }

    /** JSON-typed default: numbers stay numbers, colors become {@code #RRGGBB}. */
    private static JsonElement typedDefault(ArgSpec spec) {
        return switch (spec.type()) {
            case COLOR -> new JsonPrimitive(String.format("#%06X", (Integer) spec.defaultValue() & 0xFFFFFF));
            case INT -> new JsonPrimitive((Integer) spec.defaultValue());
            case FLOAT -> new JsonPrimitive((Float) spec.defaultValue());
            case BOOLEAN -> new JsonPrimitive((Boolean) spec.defaultValue());
            default -> new JsonPrimitive(String.valueOf(spec.defaultValue()));
        };
    }

    private static JsonObject gsonObject(Map<String, Object> values) {
        JsonObject out = new JsonObject();
        values.forEach((key, value) -> {
            if (value instanceof Float f) {
                out.addProperty(key, f);
            } else if (value instanceof Number n) {
                out.addProperty(key, n);
            } else if (value instanceof Boolean b) {
                out.addProperty(key, b);
            } else {
                out.addProperty(key, String.valueOf(value));
            }
        });
        return out;
    }

    /** imgui-java convention: {@code v<dear-imgui>.<build>} → Dear ImGui version without build suffix. */
    public static String dearImguiVersion(String imguiJavaVersion) {
        String[] parts = imguiJavaVersion.split("\\.");
        if (parts.length >= 3) {
            return parts[0] + "." + parts[1] + "." + parts[2];
        }
        return imguiJavaVersion;
    }
}
