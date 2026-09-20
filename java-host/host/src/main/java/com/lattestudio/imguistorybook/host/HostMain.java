package com.lattestudio.imguistorybook.host;

import com.google.gson.GsonBuilder;
import com.lattestudio.imguistorybook.api.StoryRegistry;
import com.lattestudio.imguistorybook.api.StoryTheme;
import com.lattestudio.imguistorybook.host.capture.CaptureRunner;
import com.lattestudio.imguistorybook.host.core.HostedStoryContext;
import com.lattestudio.imguistorybook.host.core.ImGuiHost;
import com.lattestudio.imguistorybook.host.core.StoryCatalog;
import com.lattestudio.imguistorybook.host.core.ViewState;
import com.lattestudio.imguistorybook.host.demo.DemoStories;
import com.lattestudio.imguistorybook.host.gl.FrameGrabber;
import com.lattestudio.imguistorybook.host.gl.HeadlessGl;
import com.lattestudio.imguistorybook.host.net.InputInjector;
import com.lattestudio.imguistorybook.host.net.Protocol;
import com.lattestudio.imguistorybook.host.net.WsServer;

import java.util.ServiceLoader;

/**
 * Entry point of the Java host.
 *
 * <ul>
 *   <li>{@code --list --json} — prints the story catalog (no GL needed)</li>
 *   <li>{@code --serve} — WebSocket host streaming live frames</li>
 *   <li>{@code --capture} — offscreen PNG captures + captures.json</li>
 * </ul>
 */
public final class HostMain {

    private HostMain() {
    }

    public static void main(String[] args) {
        try {
            Cli cli = Cli.parse(args);
            StoryRegistry registry = StoryRegistry.load(HostMain.class.getClassLoader());
            if (!cli.noDemo()) {
                registry.addAll(DemoStories.all());
            }
            switch (cli.command()) {
                case LIST -> System.out.println(new GsonBuilder().setPrettyPrinting().create()
                        .toJson(StoryCatalog.toJson(registry, HostInfo.hostVersion(),
                                HostInfo.imguiJavaVersion())));
                // loadStoryTheme prints to stdout; keep it out of --list, whose stdout is the
                // catalog JSON consumed by the TS generator.
                case CAPTURE -> CaptureRunner.run(cli, registry, loadStoryTheme());
                case SERVE -> serve(cli, registry, loadStoryTheme());
            }
        } catch (Exception e) {
            System.err.println(Cli.usage());
            System.err.println("error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    /**
     * Consumer-provided fonts+style (META-INF/services/com.lattestudio.imguistorybook.api.StoryTheme);
     * the first provider wins, raw ImGui defaults otherwise.
     */
    private static StoryTheme loadStoryTheme() {
        for (StoryTheme theme : ServiceLoader.load(StoryTheme.class, HostMain.class.getClassLoader())) {
            System.out.println("story theme: " + theme.id());
            return theme;
        }
        return StoryTheme.DEFAULT;
    }

    private static void serve(Cli cli, StoryRegistry registry, StoryTheme theme) throws Exception {
        try (HeadlessGl gl = new HeadlessGl(cli.width(), cli.height())) {
            ImGuiHost host = new ImGuiHost(gl, theme);
            host.init();
            ViewState state = new ViewState();
            InputInjector input = new InputInjector();

            WsServer server = new WsServer(cli.port(), registry, state, input,
                    cli.width(), cli.height(), gl.framebufferWidth(), gl.framebufferHeight());
            host.setActionListener(event -> server.broadcast(Protocol.action(event.name(), event.atMs())));
            Thread serverThread = new Thread(server, "ws-server");
            serverThread.start();
            Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                try {
                    server.stop();
                } catch (Throwable ignored) {
                    // shutting down
                }
            }, "ws-shutdown"));

            System.out.println("imgui-storybook host (demo stories: " + (cli.noDemo() ? 0 : DemoStories.all().size())
                    + ", discovered: " + registry.size() + ")");
            System.out.println("listening on ws://localhost:" + cli.port()
                    + " — ctrl+c to stop");

            HostedStoryContext ctx = new HostedStoryContext(host, state);
            FrameGrabber grabber = new FrameGrabber();
            long frameIntervalNs = 1_000_000_000L / 30;
            int seq = 0;
            while (true) {
                long start = System.nanoTime();
                if (state.scale() != host.scale()) {
                    host.setScale(state.scale());
                }
                if (state.entry() != null && !server.getConnections().isEmpty()) {
                    host.renderFrame(state.entry(), state.args(), state, ctx, input);
                    java.nio.ByteBuffer rgba = grabber.grab(
                            gl.framebufferWidth(), gl.framebufferHeight());
                    byte[] jpeg = grabber.jpeg(rgba, gl.framebufferWidth(),
                            gl.framebufferHeight(), 80);
                    server.broadcast(Protocol.frame(seq++, gl.framebufferWidth(),
                            gl.framebufferHeight(), jpeg));
                }
                long elapsed = System.nanoTime() - start;
                long sleepNs = frameIntervalNs - elapsed;
                if (sleepNs > 0) {
                    Thread.sleep(sleepNs / 1_000_000, (int) (sleepNs % 1_000_000));
                }
            }
        }
    }
}
