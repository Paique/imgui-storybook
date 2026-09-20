/**
 * imgui-storybook preview runtime.
 *
 * `mountImGuiStory()` is used as the CSF `render` function of generated stories. It shows a live
 * canvas when the java host is reachable (WebSocket on localhost) and falls back to the static
 * PNG captures otherwise — so the same story file works in dev (interactive) and in the built
 * static site (documented, AI-readable).
 */
import type {
  Backdrop,
  CanvasMode,
  HelloMsg,
  ServerMessage,
  Theme,
  ViewOptions,
} from '@imgui-storybook/protocol';

export interface ImGuiStoryMountOptions {
  storyId: string;
  args?: Record<string, unknown>;
  globals?: Record<string, unknown>;
  description?: string;
  /** preset name → arg values, used to resolve the static fallback image */
  presets?: Record<string, Record<string, unknown>>;
  /** captures root relative to the storybook static dir (default: 'captures') */
  capturesBase?: string;
  /** host ws url (default: ws://localhost:8765, override with ?imguiHost=...) */
  hostUrl?: string;
}

type Status = 'connecting' | 'live' | 'offline';

interface Frame {
  bytes: Uint8Array;
  mime: string;
  width: number;
  height: number;
}

interface Surface {
  paint(frame: Frame): void;
  onAction(name: string): void;
  onStatus(status: Status): void;
  /** Host rejected something (e.g. `unknown story` after a select). */
  onHostError?(message: string): void;
}

const DEFAULT_HOST_URL = 'ws://localhost:8765';

function resolveHostUrl(override?: string): string {
  if (override) return override;
  try {
    const fromQuery = new URLSearchParams(window.location.search).get('imguiHost');
    if (fromQuery) return fromQuery;
    const stored = window.localStorage.getItem('imguiHost');
    if (stored) return stored;
  } catch {
    /* non-browser context */
  }
  return DEFAULT_HOST_URL;
}

function viewFromGlobals(globals?: Record<string, unknown>): Partial<ViewOptions> {
  const view: Partial<ViewOptions> = {};
  if (globals?.imguiTheme === 'light' || globals?.imguiTheme === 'dark') {
    view.theme = globals.imguiTheme as Theme;
  }
  const scale = Number(globals?.imguiScale ?? '1');
  if ([1, 1.5, 2].includes(scale)) view.scale = scale;
  if (
    globals?.imguiBackdrop === 'neutral-dark'
    || globals?.imguiBackdrop === 'neutral-light'
    || globals?.imguiBackdrop === 'checker'
  ) {
    view.backdrop = globals.imguiBackdrop as Backdrop;
  }
  if (globals?.imguiCanvasMode === 'windowed' || globals?.imguiCanvasMode === 'inline') {
    view.canvasMode = globals.imguiCanvasMode as CanvasMode;
  }
  return view;
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

/** Single shared connection to the java host. The latest mounted surface receives frames. */
class HostConnection {
  status: Status = 'connecting';
  hello?: HelloMsg;
  private ws?: WebSocket;
  private surface?: Surface;
  private storyId?: string;
  private args?: Record<string, unknown>;
  private view: Partial<ViewOptions> = {};
  private retryTimer?: number;

  setSurface(surface: Surface | undefined) {
    this.surface = surface;
    surface?.onStatus(this.status);
  }

  select(storyId: string, args: Record<string, unknown>, view: Partial<ViewOptions>) {
    this.storyId = storyId;
    this.args = args;
    this.view = view;
    if (this.isOpen()) {
      this.send({ type: 'select', storyId });
      this.send({ type: 'setView', view });
      this.send({ type: 'setArgs', args });
    }
  }

  sendInput(message: Parameters<HostConnection['send']>[0]) {
    if (this.status === 'live') this.send(message);
  }

  connect(url: string) {
    if (this.isOpen()) return;
    this.setStatus('connecting');
    try {
      this.ws = new WebSocket(url);
    } catch {
      this.setStatus('offline');
      this.scheduleRetry(url);
      return;
    }
    this.ws.onopen = () => {
      this.setStatus('live');
      if (this.storyId) {
        this.send({ type: 'select', storyId: this.storyId });
        this.send({ type: 'setView', view: this.view });
        if (this.args) this.send({ type: 'setArgs', args: this.args });
      }
    };
    this.ws.onmessage = (event) => {
      let message: ServerMessage;
      try {
        message = JSON.parse(event.data as string);
      } catch {
        return;
      }
      switch (message.type) {
        case 'hello':
          this.hello = message;
          break;
        case 'frame':
          this.surface?.paint({
            bytes: base64ToBytes(message.data),
            mime: message.mime,
            width: message.width,
            height: message.height,
          });
          break;
        case 'action':
          this.surface?.onAction(message.name);
          break;
        case 'error':
          console.warn('[imgui-storybook] host:', message.message);
          this.surface?.onHostError?.(message.message);
          break;
        default:
          break;
      }
    };
    const gone = () => {
      this.ws = undefined;
      this.hello = undefined;
      this.setStatus('offline');
      this.scheduleRetry(url);
    };
    this.ws.onclose = gone;
    this.ws.onerror = gone;
  }

  private send(message: unknown) {
    if (this.isOpen()) this.ws!.send(JSON.stringify(message));
  }

  private isOpen(): boolean {
    return this.ws !== undefined && this.ws.readyState === WebSocket.OPEN;
  }

  private setStatus(status: Status) {
    if (this.status === status) return;
    this.status = status;
    this.surface?.onStatus(status);
  }

  private scheduleRetry(url: string) {
    if (this.retryTimer !== undefined) return;
    this.retryTimer = window.setTimeout(() => {
      this.retryTimer = undefined;
      if (!this.isOpen()) this.connect(url);
    }, 4000);
  }
}

const sharedConnection = typeof window !== 'undefined' ? new HostConnection() : undefined;

let styleInjected = false;

function injectStyles() {
  if (styleInjected || typeof document === 'undefined') return;
  styleInjected = true;
  const style = document.createElement('style');
  style.textContent = `
.isb-root { font-family: ui-sans-serif, system-ui, sans-serif; color: #333; display: flex; flex-direction: column; gap: 8px; }
.isb-topbar { display: flex; align-items: center; gap: 10px; font-size: 12px; color: #666; }
.isb-pill { border: none; border-radius: 999px; padding: 2px 10px; font-size: 11px; font-weight: 600; cursor: pointer; }
.isb-pill.isb-live { background: #16a34a; color: #fff; }
.isb-pill.isb-connecting { background: #d97706; color: #fff; }
.isb-pill.isb-offline { background: #9ca3af; color: #fff; }
.isb-stage { position: relative; width: 100%; max-width: 900px; border: 1px solid rgba(0,0,0,.12); border-radius: 8px; overflow: hidden; background: #141417; line-height: 0; }
.isb-stage:focus-visible { outline: 2px solid #2563eb; }
.isb-stage canvas, .isb-stage img { width: 100%; height: auto; display: block; }
.isb-caption { font-size: 13px; color: #555; }
.isb-actions { display: flex; flex-wrap: wrap; gap: 6px; min-height: 22px; }
.isb-action-chip { font-size: 11px; font-family: ui-monospace, monospace; background: #eef2ff; color: #3730a3; border-radius: 999px; padding: 2px 8px; }
.isb-notice { font-size: 12px; color: #92400e; background: #fef3c7; border-radius: 6px; padding: 8px 10px; }
@media (prefers-color-scheme: dark) {
  .isb-root { color: #ddd; }
  .isb-topbar { color: #999; }
  .isb-stage { border-color: rgba(255,255,255,.14); }
  .isb-caption { color: #aaa; }
  .isb-action-chip { background: #312e81; color: #c7d2fe; }
  .isb-notice { background: #451a03; color: #fbbf24; }
}
`;
  document.head.appendChild(style);
}

function argsEqual(a: Record<string, unknown>, b: Record<string, unknown>): boolean {
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const key of keys) {
    if (String(a[key]) !== String(b[key])) return false;
  }
  return true;
}

export function mountImGuiStory(options: ImGuiStoryMountOptions): HTMLElement {
  injectStyles();

  const root = document.createElement('div');
  root.className = 'isb-root';

  const topbar = document.createElement('div');
  topbar.className = 'isb-topbar';

  const pill = document.createElement('button');
  pill.className = 'isb-pill isb-connecting';
  pill.textContent = 'CONNECTING';

  const meta = document.createElement('span');
  meta.textContent = 'imgui-storybook';

  topbar.append(pill, meta);

  const stage = document.createElement('div');
  stage.className = 'isb-stage';
  stage.tabIndex = 0;
  stage.title = 'Click to focus, then interact — input is forwarded to the real ImGui host';

  const canvas = document.createElement('canvas');
  canvas.style.display = 'none';
  const staticImg = document.createElement('img');
  staticImg.alt = `${options.storyId} static capture`;
  staticImg.style.display = 'none';
  const notice = document.createElement('div');
  notice.className = 'isb-notice';
  notice.style.display = 'none';
  notice.textContent =
    'Host offline and no static capture found for this story/theme. Start the java host (npm run dev) or run npm run capture.';
  stage.append(canvas, staticImg, notice);

  const caption = document.createElement('div');
  caption.className = 'isb-caption';
  if (options.description) caption.textContent = options.description;

  const actions = document.createElement('div');
  actions.className = 'isb-actions';

  root.append(topbar, stage, caption, actions);

  // --- static fallback ------------------------------------------------------
  const view = viewFromGlobals(options.globals);
  const theme = view.theme ?? 'dark';
  const capturesBase = options.capturesBase ?? 'captures';
  const args = options.args ?? {};
  let preset = 'default';
  if (options.presets) {
    for (const [name, presetArgs] of Object.entries(options.presets)) {
      if (argsEqual(args, presetArgs)) {
        preset = name;
        break;
      }
    }
  }
  let imgTried = false;
  let imgFailed = false;
  const showStatic = () => {
    canvas.style.display = 'none';
    if (!imgTried) {
      imgTried = true;
      staticImg.src = `${capturesBase}/imgui/${options.storyId}/${preset}-${theme}.png`;
      staticImg.onerror = () => {
        staticImg.style.display = 'none';
        imgFailed = true;
      };
    }
    if (imgFailed) {
      notice.style.display = 'block';
    } else {
      staticImg.style.display = 'block';
    }
  };

  // --- live painting ---------------------------------------------------------
  let paintPending = false;
  const ctx2d = canvas.getContext('2d');

  const paint = (frame: Frame) => {
    if (paintPending || !ctx2d) return;
    paintPending = true;
    const blob = new Blob([frame.bytes as unknown as BlobPart], { type: frame.mime });
    createImageBitmap(blob)
      .then((bitmap) => {
        if (canvas.width !== frame.width || canvas.height !== frame.height) {
          canvas.width = frame.width;
          canvas.height = frame.height;
        }
        ctx2d.drawImage(bitmap, 0, 0);
        bitmap.close();
        canvas.style.display = 'block';
        staticImg.style.display = 'none';
        notice.style.display = 'none';
      })
      .catch(() => {
        /* skip broken frame */
      })
      .finally(() => {
        paintPending = false;
      });
  };

  const surface: Surface = {
    paint,
    onAction(name) {
      const chip = document.createElement('span');
      chip.className = 'isb-action-chip';
      chip.textContent = `${name}`;
      actions.prepend(chip);
      while (actions.childElementCount > 5) actions.lastChild?.remove();
      window.setTimeout(() => chip.remove(), 4000);
    },
    onStatus(status) {
      pill.className = `isb-pill isb-${status}`;
      pill.textContent = status.toUpperCase();
      const hello = sharedConnection?.hello;
      meta.textContent = hello
        ? `${hello.host} host ${hello.hostVersion} · Dear ImGui ${hello.dearImgui} · ${hello.width}×${hello.height}`
        : 'imgui-storybook';
      if (status === 'live') {
        canvas.style.display = 'block';
      } else {
        showStatic();
      }
    },
    onHostError(message) {
      // The host rejected this story (e.g. started without the consumer classpath) — show the
      // static capture instead of an empty canvas under a LIVE badge.
      if (/unknown story/i.test(message)) showStatic();
    },
  };

  pill.onclick = () => {
    if (sharedConnection && sharedConnection.status !== 'live') {
      sharedConnection.connect(resolveHostUrl(options.hostUrl));
    }
  };

  // --- input forwarding -------------------------------------------------------
  const hostW = () => sharedConnection?.hello?.width ?? 900;
  const hostH = () => sharedConnection?.hello?.height ?? 600;
  const toHost = (event: MouseEvent) => {
    const rect = stage.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / Math.max(rect.width, 1)) * hostW(),
      y: ((event.clientY - rect.top) / Math.max(rect.height, 1)) * hostH(),
    };
  };
  const forward = (message: unknown) => sharedConnection?.sendInput(message);
  stage.addEventListener('mousemove', (event) => {
    const { x, y } = toHost(event as MouseEvent);
    forward({ type: 'input', kind: 'mouseMove', x, y });
  });
  stage.addEventListener('mousedown', (event) => {
    event.preventDefault();
    const { x, y } = toHost(event as MouseEvent);
    forward({ type: 'input', kind: 'mouseMove', x, y });
    forward({ type: 'input', kind: 'mouseDown', button: (event as MouseEvent).button });
    stage.focus();
  });
  stage.addEventListener('mouseup', (event) => {
    forward({ type: 'input', kind: 'mouseUp', button: (event as MouseEvent).button });
  });
  stage.addEventListener('mouseleave', () => {
    forward({ type: 'input', kind: 'mouseLeave' });
  });
  stage.addEventListener(
    'wheel',
    (event) => {
      if (sharedConnection?.status !== 'live') return;
      event.preventDefault();
      forward({
        type: 'input',
        kind: 'wheel',
        dx: (event as WheelEvent).deltaX,
        dy: (event as WheelEvent).deltaY,
      });
    },
    { passive: false },
  );
  stage.addEventListener('keydown', (event) => {
    const keyEvent = event as KeyboardEvent;
    if (sharedConnection?.status !== 'live') return;
    if (keyEvent.ctrlKey || keyEvent.metaKey || keyEvent.altKey) return;
    if (['F5', 'F12', 'Tab'].includes(keyEvent.code)) return;
    keyEvent.preventDefault();
    forward({ type: 'input', kind: 'keyDown', key: keyEvent.code });
    if (keyEvent.key.length === 1) {
      forward({ type: 'input', kind: 'text', text: keyEvent.key });
    }
  });
  stage.addEventListener('keyup', (event) => {
    forward({ type: 'input', kind: 'keyUp', key: (event as KeyboardEvent).code });
  });

  // --- activation --------------------------------------------------------------
  if (sharedConnection) {
    sharedConnection.setSurface(surface);
    sharedConnection.select(options.storyId, args, view);
    sharedConnection.connect(resolveHostUrl(options.hostUrl));
    if (sharedConnection.status !== 'live') showStatic();
  } else {
    showStatic();
  }

  return root;
}
