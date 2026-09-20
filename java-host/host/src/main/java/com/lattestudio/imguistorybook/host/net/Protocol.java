package com.lattestudio.imguistorybook.host.net;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.lattestudio.imguistorybook.api.ArgSet;
import com.lattestudio.imguistorybook.host.core.ArgValues;
import com.lattestudio.imguistorybook.host.core.StoryCatalog;
import com.lattestudio.imguistorybook.host.core.ViewState;

import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;

/** JSON messages of the host↔client protocol. Kept in sync with packages/protocol (TS). */
public final class Protocol {

    public static final Gson GSON = new Gson();

    private Protocol() {
    }

    public static JsonObject parse(String message) {
        return JsonParser.parseString(message).getAsJsonObject();
    }

    public static String encode(JsonObject obj) {
        return GSON.toJson(obj);
    }

    public static String hello(String hostVersion, String imguiJavaVersion,
                               int width, int height, int framebufferWidth, int framebufferHeight) {
        JsonObject obj = base("hello");
        obj.addProperty("host", "java");
        obj.addProperty("hostVersion", hostVersion);
        obj.addProperty("imguiJavaVersion", imguiJavaVersion);
        obj.addProperty("dearImgui", StoryCatalog.dearImguiVersion(imguiJavaVersion));
        obj.addProperty("width", width);
        obj.addProperty("height", height);
        obj.addProperty("fbWidth", framebufferWidth);
        obj.addProperty("fbHeight", framebufferHeight);
        return encode(obj);
    }

    public static String selected(String storyId, Map<String, Object> args, ArgSet set, ViewState view) {
        JsonObject obj = base("selected");
        obj.addProperty("storyId", storyId);
        obj.add("args", gsonFromMap(ArgValues.forJson(ArgValues.materialize(set, args), set)));
        obj.add("view", viewJson(view));
        return encode(obj);
    }

    public static String viewChanged(ViewState view) {
        JsonObject obj = base("view");
        obj.add("view", viewJson(view));
        return encode(obj);
    }

    public static JsonObject viewJson(ViewState view) {
        JsonObject obj = new JsonObject();
        obj.addProperty("theme", view.theme().name().toLowerCase(Locale.ROOT));
        obj.addProperty("scale", view.scale());
        obj.addProperty("backdrop", view.backdrop().name().toLowerCase(Locale.ROOT)
                .replace('_', '-'));
        obj.addProperty("canvasMode", view.canvasMode().name().toLowerCase(Locale.ROOT));
        return obj;
    }

    public static String frame(int seq, int width, int height, byte[] jpeg) {
        JsonObject obj = base("frame");
        obj.addProperty("seq", seq);
        obj.addProperty("width", width);
        obj.addProperty("height", height);
        obj.addProperty("mime", "image/jpeg");
        obj.addProperty("data", Base64.getEncoder().encodeToString(jpeg));
        return encode(obj);
    }

    public static String action(String name, long atMs) {
        JsonObject obj = base("action");
        obj.addProperty("name", name);
        obj.addProperty("t", atMs);
        return encode(obj);
    }

    public static String error(String message) {
        JsonObject obj = base("error");
        obj.addProperty("message", message);
        return encode(obj);
    }

    /** Flattens one level of a JSON object into JSON-ish Java values (Boolean/Number/String). */
    public static Map<String, Object> flatten(JsonObject args) {
        Map<String, Object> out = new LinkedHashMap<>();
        for (Map.Entry<String, JsonElement> entry : args.entrySet()) {
            JsonElement element = entry.getValue();
            if (element == null || element.isJsonNull()) {
                continue;
            }
            if (element.isJsonPrimitive()) {
                var primitive = element.getAsJsonPrimitive();
                if (primitive.isBoolean()) {
                    out.put(entry.getKey(), primitive.getAsBoolean());
                } else if (primitive.isNumber()) {
                    out.put(entry.getKey(), primitive.getAsNumber());
                } else {
                    out.put(entry.getKey(), primitive.getAsString());
                }
            } else {
                out.put(entry.getKey(), element.toString());
            }
        }
        return out;
    }

    private static JsonObject gsonFromMap(Map<String, Object> values) {
        JsonObject out = new JsonObject();
        values.forEach((key, value) -> {
            if (value instanceof Boolean b) {
                out.addProperty(key, b);
            } else if (value instanceof Number n) {
                out.addProperty(key, n);
            } else {
                out.addProperty(key, String.valueOf(value));
            }
        });
        return out;
    }

    private static JsonObject base(String type) {
        JsonObject obj = new JsonObject();
        obj.addProperty("type", type);
        return obj;
    }
}
