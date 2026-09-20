# imgui-storybook

A [Storybook](https://storybook.js.org) for [Dear ImGui](https://github.com/ocornut/imgui) — document, browse and interact with ImGui components in a real browser, rendered by the **real ImGui**.

Storybook's story workflow (sidebar, Controls, docs, static export) applied to immediate-mode GUIs. Stories run as real code on a language-specific **host** that renders Dear ImGui offscreen and streams frames to the Storybook preview; a static export of PNG captures plus structured docs makes every component readable by humans **and AI models** without running anything.

```
┌─────────────────────────┐   WebSocket (JSON + JPEG frames)   ┌──────────────────────────────┐
│  Storybook web app      │ ◄──────────────────────────────►   │  java host (this repo)       │
│  sidebar · Controls ·   │   select / setArgs / setView       │  hidden GLFW window + FBO    │
│  docs · canvas          │   input (mouse/keys) ──────────►   │  imgui-java = Dear ImGui     │
│                         │   ◄────────── frames / actions     │  1.92.7 (docking), real      │
│  offline: PNG captures  │                                    │  fonts, style and backends   │
└─────────────────────────┘                                    └──────────────────────────────┘
```

## Why it looks exactly like your app

The Java host uses `io.github.spair:imgui-java:1.92.7.1` — the same binding, native binaries and GLFW/OpenGL3 backend classes that [`cn.enaium:fabric-gui-imgui`](https://github.com/Enaium/fabric-mod-ImGui)-based Minecraft mods run. Rendering happens in the same Dear ImGui `1.92.7` (docking) native code, so captures and live frames are pixel-faithful by construction, not by imitation.

To document **your** product's look, implement [`StoryTheme`](java-host/api/src/main/java/com/lattestudio/imguistorybook/api/StoryTheme.java) (load the same TTFs, apply the same style/colors as your app) and register your stories — see below.

## Repository layout

| Path | What it is |
| --- | --- |
| `java-host/api` | Story API for authors: `Story`, `ArgSet`, `StoryContext`, `StoryTheme`, `StoryRegistry`. Zero dependencies beyond the imgui binding. |
| `java-host/host` | The Java host: offscreen renderer, `--list` catalog, `--capture` PNG export, `--serve` WebSocket live mode, 16 built-in demo stories. |
| `packages/protocol` | TypeScript types for the host wire protocol. |
| `packages/client` | Preview runtime: live canvas + input forwarding, with automatic static-PNG fallback. |
| `app` | The Storybook app (`@storybook/html-vite`) + toolbar globals (theme, scale, backdrop, canvas mode). |
| `scripts` | Orchestration: `gen-stories.mjs`, `capture.mjs`, `dev.mjs`, `build-site.mjs`. |

## Quick start (Windows, JDK 25+, Node 20+)

```bash
npm install

# interactive: java host (ws://localhost:8765) + storybook dev server
npm run dev            # open http://localhost:6006 — pill shows LIVE

# static site with captures + AI docs (dist-site/)
npm run build:site
```

`npm run dev` builds the Java host on first run (Gradle wrapper, no global Gradle needed).

## Writing stories (Java)

Implement `Story`, declare typed args (they become Storybook Controls automatically), and register via `META-INF/services/com.lattestudio.imguistorybook.api.Story` or programmatically:

```java
public final class StatusChipStory implements Story {
    public String title() { return "Widgets/Status Chip"; }   // '/' = sidebar groups
    public String description() { return "Chip with tone per status enum."; }

    public void defineArgs(ArgSet args) {
        args.string("label", "Em andamento")
            .enumOf("status", Status.class, Status.PROGRESS)
            .preset("compacto", "label", "Ok");
    }

    public void render(StoryContext ctx) {
        // draw with ImGui.* exactly as in your product
        chip(ctx.enumValue("status", Status.class), ctx.string("label"));
        if (button("confirm")) ctx.action("clicked");   // shows as a toast + WS event
    }
}
```

Run the host with your stories on the classpath:

```bash
java -cp "your-mod-classpath;imgui-storybook-host/*" com.lattestudio.imguistorybook.host.HostMain --serve
npm run dev   # storybook picks it up on ws://localhost:8765
```

Regenerate the CSF bridge after adding stories: `npm run gen`.

## Commands (java host)

```
host --list --json                                 # story catalog (drives the TS generator)
host --serve [--port 8765] [--width 900] [--height 600] [--no-demo]
host --capture [--out DIR] [--themes light,dark] [--scales 1,2]
```

## Static export (for humans and AI models)

`npm run build:site` produces `dist-site/`:

- `index.html` — the full Storybook UI; every preview falls back to the real captured PNG when no host is running (pill shows `OFFLINE`).
- `STORIES.md` — one consolidated document: every story with description, arg table, presets and inline screenshots (relative links). Feed this to an AI model — no browser required.
- `manifest.json` — the same content as structured data: arg schemas with types/defaults, preset definitions, capture file lists, host/imgui versions.
- `captures/` — PNGs rendered by the real Dear ImGui (per story × preset × theme × scale).

## Fidelity notes

- Version pinning is part of the contract: keep `imguiJavaVersion` in `java-host/gradle.properties` equal to the imgui-java version your product ships with (the `+imgui.x.y.z` suffix in `fabric-gui-imgui` versions tells you which).
- The offscreen renderer uses an exact-size FBO (hidden windows report a 0×0 framebuffer on Windows) with `DisplayFramebufferScale` forced to 1 — output is deterministic across machines and DPI settings.
- MC 26.2+ note: `fabric-gui-imgui` versions for Minecraft 26.2/26.3 switch to a custom Blaze3D renderer; captures then reflect the GLFW/OpenGL path, which may differ slightly from the in-game one.

## License

MIT © Latte Studios. Built on [Storybook](https://github.com/storybookjs/storybook) (MIT), [imgui-java](https://github.com/SpaiR/imgui-java) (MIT), [Dear ImGui](https://github.com/ocornut/imgui) (MIT), [LWJGL](https://www.lwjgl.org) (BSD).
