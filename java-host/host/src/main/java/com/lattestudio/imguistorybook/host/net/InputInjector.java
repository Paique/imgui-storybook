package com.lattestudio.imguistorybook.host.net;

import imgui.ImGuiIO;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.HashMap;
import java.util.Map;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * Queue of synthetic input events coming from the web client. Applied to the ImGuiIO event
 * queue on the render thread, right before {@code ImGui.newFrame()} — after the GLFW backend
 * ran, so browser events take precedence over the backend's cursor-polling fallback.
 */
public final class InputInjector {

    public record MouseMove(float x, float y) {
    }

    public record MouseButton(int button, boolean down) {
    }

    public record Wheel(float dx, float dy) {
    }

    public record Key(String key, boolean down) {
    }

    public record Text(String text) {
    }

    /**
     * Named ImGuiKey constants (512..666) from the binding, indexed by name.
     * Built via reflection so it stays correct across binding updates.
     */
    private static final Map<String, Integer> KEY_MAP = buildKeyMap();

    private final Queue<Object> queue = new ConcurrentLinkedQueue<>();

    private static Map<String, Integer> buildKeyMap() {
        Map<String, Integer> map = new HashMap<>();
        for (Field field : imgui.flag.ImGuiKey.class.getDeclaredFields()) {
            if (!Modifier.isStatic(field.getModifiers()) || field.getType() != int.class) {
                continue;
            }
            String name = field.getName();
            if (name.equals("None") || name.equals("COUNT") || name.startsWith("Mod")) {
                continue;
            }
            try {
                int value = field.getInt(null);
                if (value >= 512 && value < 667) {
                    map.put(name, value);
                }
            } catch (IllegalAccessException ignored) {
                // skip unreadable field
            }
        }
        return map;
    }

    /** Maps a browser {@code KeyboardEvent.code} to an ImGuiKey constant (0 = unsupported). */
    public static int mapKey(String browserCode) {
        if (browserCode == null || browserCode.isEmpty()) {
            return 0;
        }
        // "KeyA".."KeyZ"
        if (browserCode.length() == 4 && browserCode.startsWith("Key")) {
            return KEY_MAP.getOrDefault(browserCode.substring(3), 0);
        }
        // "Digit0".."Digit9" → binding names them "_0".."_9"
        if (browserCode.length() == 6 && browserCode.startsWith("Digit")) {
            return KEY_MAP.getOrDefault("_" + browserCode.charAt(5), 0);
        }
        // "ArrowLeft" → "LeftArrow" etc.
        if (browserCode.startsWith("Arrow") && browserCode.length() > 5) {
            return KEY_MAP.getOrDefault(browserCode.substring(5) + "Arrow", 0);
        }
        String mapped = switch (browserCode) {
            case "ShiftLeft" -> "LeftShift";
            case "ShiftRight" -> "RightShift";
            case "ControlLeft" -> "LeftCtrl";
            case "ControlRight" -> "RightCtrl";
            case "AltLeft" -> "LeftAlt";
            case "AltRight" -> "RightAlt";
            case "MetaLeft" -> "LeftSuper";
            case "MetaRight" -> "RightSuper";
            case "Escape" -> "Escape";
            default -> browserCode; // Enter, Space, Tab, Backspace, Delete, Home, End, PageUp, PageDown, Insert, F1..F12
        };
        return KEY_MAP.getOrDefault(mapped, 0);
    }

    public void mouseMove(float x, float y) {
        queue.offer(new MouseMove(x, y));
    }

    public void mouseButton(int button, boolean down) {
        queue.offer(new MouseButton(button, down));
    }

    public void wheel(float dx, float dy) {
        queue.offer(new Wheel(dx, dy));
    }

    public void key(String key, boolean down) {
        queue.offer(new Key(key, down));
    }

    public void text(String text) {
        if (!text.isEmpty()) {
            queue.offer(new Text(text));
        }
    }

    public void apply(ImGuiIO io) {
        Object event;
        while ((event = queue.poll()) != null) {
            if (event instanceof MouseMove m) {
                io.addMousePosEvent(m.x(), m.y());
            } else if (event instanceof MouseButton b) {
                io.addMouseButtonEvent(b.button(), b.down());
            } else if (event instanceof Wheel w) {
                io.addMouseWheelEvent(w.dx(), w.dy());
            } else if (event instanceof Key k) {
                int key = mapKey(k.key());
                if (key != 0) {
                    io.addKeyEvent(key, k.down());
                }
            } else if (event instanceof Text t) {
                io.addInputCharactersUTF8(t.text());
            }
        }
    }
}
