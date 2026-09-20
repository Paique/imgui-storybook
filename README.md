# imgui-storybook

A [Storybook](https://storybook.js.org) for [Dear ImGui](https://github.com/ocornut/imgui) — document, browse and interact with ImGui components in a real browser, rendered by the **real ImGui**.

Storybook's story workflow (sidebar, Controls, docs, static export) applied to immediate-mode GUIs. Stories run as real code on a language-specific **host** that renders Dear ImGui offscreen and streams frames to the Storybook preview; a static export of PNG captures plus structured docs makes every component readable by humans **and AI models** without running anything.

```
┌─────────────────────────┐   WebSocket (JSON + JPEG frames)   ┌──────────────────────────────┐
│  Storybook web app      │ ◄──────────────────────────────►   │  host: java (default) | rust │
│  sidebar · Controls ·   │   select / setArgs / setView       │  hidden window + exact FBO   │
│  docs · canvas          │   input (mouse/keys) ──────────►   │  java: imgui-java 1.92.7     │
│                         │   ◄────────── frames / actions     │  rust: easy-imgui 1.92.9b    │
│  offline: PNG captures  │                                    │  real fonts, style, backends │
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
| `rust-host` | The Rust host — same CLI, same wire protocol, same 16 demo stories, on `easy-imgui` (Dear ImGui 1.92.9b). |
| `packages/protocol` | TypeScript types for the host wire protocol. |
| `packages/client` | Preview runtime: live canvas + input forwarding, with automatic static-PNG fallback. |
| `app` | The Storybook app (`@storybook/html-vite`) + toolbar globals (theme, scale, backdrop, canvas mode). |
| `scripts` | Orchestration: `gen-stories.mjs`, `capture.mjs`, `dev.mjs`, `build-site.mjs`. |

## Quick start (Windows, JDK 25+, Node 20+)

```bash
npm install

# interactive: java host (ws://localhost:8765) + storybook dev server
npm run dev            # open http://localhost:6006 — pill shows LIVE

# same thing on the rust host
npm run dev -- --host rust

# static site with captures + AI docs (dist-site/)
npm run build:site
```

`npm run dev` builds the chosen host on first run (Gradle wrapper for java, `cargo build --release` for rust). Every script accepts `--host java|rust`; `IMGUI_HOST=rust` works too.

On the rust host, the dev watcher watches `rust-host/src` (and `Cargo.toml`): saving a file rebuilds and restarts the host automatically — the storybook preview reconnects on its own. `IMGUI_NO_STORYBOOK=1` runs the host alone (handy with `cargo test` in a second terminal).

## Rust host

`rust-host/` is a second implementation of the same host contract in Rust: identical CLI (`--list/--serve/--capture`), identical WebSocket protocol (`hello.host: "rust"`), identical story catalog shape, captures and `captures.json`. Stories implement a `Story` trait (`rust-host/src/api.rs`) mirroring the Java API; the 16 demo stories are ported 1:1.

Differences from the Java host, by design:

- Binding: [easy-imgui](https://crates.io/crates/easy-imgui) 0.24, tracking upstream Dear ImGui **1.92.9b** (vs 1.92.7 on imgui-java) — the protocol field stays `imguiJavaVersion` so the TS layer never branches per host.
- Fonts: Dear ImGui 1.92's dynamic font system (`style.FontScaleMain`) replaces the Java atlas-rebuild-on-scale-change.
- Window/context: hidden winit window + glutin (WGL) + glow — no GLFW, no CMake; the Dear ImGui C++ is compiled by the `cc` crate and bound with bindgen, which needs **libclang** (`winget install LLVM.LLVM`; `rust-host/.cargo/config.toml` sets `LIBCLANG_PATH` to the default install path).

```bash
cd rust-host
cargo test                 # unit tests (catalog shape, protocol, coercion, keymap...)
cargo run -- --list --json
cargo run --example spike  # renders a story offscreen to target/spike.png
```

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

## Commands (both hosts — same CLI)

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

- Version pinning is part of the contract: keep `imguiJavaVersion` in `java-host/gradle.properties` equal to the imgui-java version your product ships with (the `+imgui.x.y.z` suffix in `fabric-gui-imgui` versions tells you which). On the rust host, keep `BINDING_VERSION` in `rust-host/src/info.rs` in sync with the `easy-imgui` dependency.
- The offscreen renderer uses an exact-size FBO (hidden windows report a 0×0 framebuffer on Windows) with `DisplayFramebufferScale` forced to 1 — output is deterministic across machines and DPI settings.
- MC 26.2+ note: `fabric-gui-imgui` versions for Minecraft 26.2/26.3 switch to a custom Blaze3D renderer; captures then reflect the GLFW/OpenGL path, which may differ slightly from the in-game one.

## License

MIT © Latte Studios. Built on [Storybook](https://github.com/storybookjs/storybook) (MIT), [imgui-java](https://github.com/SpaiR/imgui-java) (MIT), [easy-imgui](https://github.com/rodrigorc/easy-imgui-rs) (MIT), [Dear ImGui](https://github.com/ocornut/imgui) (MIT), [LWJGL](https://www.lwjgl.org) (BSD).
