package com.lattestudio.imguistorybook.host.net;

import com.lattestudio.imguistorybook.api.Backdrop;
import com.lattestudio.imguistorybook.api.CanvasMode;
import com.lattestudio.imguistorybook.api.StoryRegistry;
import com.lattestudio.imguistorybook.api.Theme;
import com.lattestudio.imguistorybook.host.HostInfo;
import com.lattestudio.imguistorybook.host.core.ArgValues;
import com.lattestudio.imguistorybook.host.core.ViewState;
import org.java_websocket.WebSocket;
import org.java_websocket.handshake.ClientHandshake;
import org.java_websocket.server.WebSocketServer;

import java.net.InetSocketAddress;
import java.util.Locale;
import java.util.Map;

/**
 * WebSocket server exposing the live host: story selection, args, view options and input events.
 * All callbacks run on WS threads and only touch volatile state / queues — GL work stays on the
 * render thread (see HostMain).
 */
public final class WsServer extends WebSocketServer {

    private final StoryRegistry registry;
    private final ViewState state;
    private final InputInjector input;
    private final int width;
    private final int height;
    private final int framebufferWidth;
    private final int framebufferHeight;

    public WsServer(int port, StoryRegistry registry, ViewState state, InputInjector input,
                    int width, int height, int framebufferWidth, int framebufferHeight) {
        super(new InetSocketAddress(port));
        this.registry = registry;
        this.state = state;
        this.input = input;
        this.width = width;
        this.height = height;
        this.framebufferWidth = framebufferWidth;
        this.framebufferHeight = framebufferHeight;
    }

    @Override
    public void onStart() {
        // nothing to do; HostMain prints the listening line
    }

    @Override
    public void onOpen(WebSocket conn, ClientHandshake handshake) {
        conn.send(Protocol.hello(HostInfo.hostVersion(), HostInfo.imguiJavaVersion(),
                width, height, framebufferWidth, framebufferHeight));
        if (state.entry() != null) {
            conn.send(Protocol.selected(state.entry().id(), state.args(), state.entry().args(), state));
            conn.send(Protocol.viewChanged(state));
        }
    }

    @Override
    public void onClose(WebSocket conn, int code, String reason, boolean remote) {
        // no per-connection state
    }

    @Override
    public void onError(WebSocket conn, Exception ex) {
        System.err.println("ws error: " + ex);
    }

    @Override
    public void onMessage(WebSocket conn, String message) {
        try {
            handle(Protocol.parse(message));
        } catch (RuntimeException e) {
            conn.send(Protocol.error("bad message: " + e.getMessage()));
        }
    }

    private void handle(com.google.gson.JsonObject obj) {
        String type = obj.get("type").getAsString();
        switch (type) {
            case "select" -> {
                String storyId = obj.get("storyId").getAsString();
                var entry = registry.byId(storyId);
                if (entry.isEmpty()) {
                    broadcast(Protocol.error("unknown story: " + storyId));
                    return;
                }
                state.select(entry.get());
                broadcast(Protocol.selected(storyId, state.args(), entry.get().args(), state));
            }
            case "setArgs" -> {
                if (state.entry() == null) {
                    return;
                }
                Map<String, Object> raw = Protocol.flatten(obj.getAsJsonObject("args"));
                state.setArgs(ArgValues.sanitize(state.entry().args(), raw));
            }
            case "setView" -> {
                applyView(obj.getAsJsonObject("view"));
                broadcast(Protocol.viewChanged(state));
            }
            case "input" -> applyInput(obj);
            case "ping" -> broadcast(Protocol.encode(typeOnly("pong")));
            default -> broadcast(Protocol.error("unknown message type: " + type));
        }
    }

    private void applyView(com.google.gson.JsonObject view) {
        if (view.has("theme")) {
            state.setTheme(Theme.valueOf(view.get("theme").getAsString().toUpperCase(Locale.ROOT)));
        }
        if (view.has("scale")) {
            state.setScale(view.get("scale").getAsFloat());
        }
        if (view.has("backdrop")) {
            state.setBackdrop(Backdrop.valueOf(
                    view.get("backdrop").getAsString().toUpperCase(Locale.ROOT).replace('-', '_')));
        }
        if (view.has("canvasMode")) {
            state.setCanvasMode(CanvasMode.valueOf(
                    view.get("canvasMode").getAsString().toUpperCase(Locale.ROOT)));
        }
    }

    private void applyInput(com.google.gson.JsonObject obj) {
        String kind = obj.get("kind").getAsString();
        switch (kind) {
            case "mouseMove" -> input.mouseMove(
                    obj.get("x").getAsFloat(), obj.get("y").getAsFloat());
            case "mouseDown" -> input.mouseButton(obj.get("button").getAsInt(), true);
            case "mouseUp" -> input.mouseButton(obj.get("button").getAsInt(), false);
            case "mouseLeave" -> input.mouseMove(-10000f, -10000f);
            case "wheel" -> input.wheel(
                    obj.get("dx").getAsFloat(), obj.get("dy").getAsFloat());
            case "keyDown" -> input.key(obj.get("key").getAsString(), true);
            case "keyUp" -> input.key(obj.get("key").getAsString(), false);
            case "text" -> input.text(obj.get("text").getAsString());
            default -> {
                // ignore unknown input kinds (forward compat)
            }
        }
    }

    private com.google.gson.JsonObject typeOnly(String type) {
        com.google.gson.JsonObject obj = new com.google.gson.JsonObject();
        obj.addProperty("type", type);
        return obj;
    }
}
