/** Dev mode: imgui host (WebSocket) + storybook dev server, killed together.
 *  Pick the host with `npm run dev -- --host rust` (default: java).
 *  With `--host rust`, saving any file under rust-host/src (or Cargo.toml) rebuilds and
 *  restarts the host — the storybook client reconnects on its own (4s retry).
 *  Set IMGUI_NO_STORYBOOK=1 to run the host alone. */
import { spawn, spawnSync } from 'node:child_process';
import { watch } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import {
  ensureHostBuilt,
  spawnHost,
  killHostTree,
  paths,
  resolveHostKind,
  hostLabel,
} from './lib/host.mjs';

const kind = resolveHostKind();
ensureHostBuilt(kind);

const port = process.argv.includes('--port')
  ? process.argv[process.argv.indexOf('--port') + 1]
  : '8765';

let host = spawnHost(['--serve', '--port', port], kind);
let shuttingDown = false;
let restarting = false;
let restartQueued = false;

console.log(`[dev] ${hostLabel(kind)} starting on ws://localhost:${port}`);

const storybook = process.env.IMGUI_NO_STORYBOOK
  ? null
  : spawn('npm', ['run', 'storybook'], {
      cwd: path.join(paths.root, 'app'),
      stdio: 'inherit',
      shell: true,
    });

const shutdown = () => {
  shuttingDown = true;
  killHostTree(host);
  storybook?.kill();
  process.exit(0);
};
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

storybook?.on('exit', () => {
  if (!shuttingDown && !restarting) shutdown();
});

const attachHostHandlers = () =>
  host.on('exit', () => {
    if (shuttingDown || restarting) return;
    console.error(`[dev] ${hostLabel(kind)} exited unexpectedly`);
    shutdown();
  });
attachHostHandlers();

// --- rust host: rebuild + restart on source changes ---
const scheduleRestart = () => {
  clearTimeout(scheduleRestart.timer);
  scheduleRestart.timer = setTimeout(() => {
    restartHost().catch((error) => console.error('[dev] restart failed:', error.message));
  }, 400);
};

const restartHost = async () => {
  if (restarting) {
    restartQueued = true;
    return;
  }
  restarting = true;
  console.log('[dev] rust host source changed — rebuilding...');
  killHostTree(host);
  const build = spawn('cargo', ['build', '--release'], {
    cwd: paths.rustHostDir,
    stdio: 'inherit',
  });
  const code = await new Promise((resolve) => build.on('close', resolve));
  if (code === 0) {
    host = spawnHost(['--serve', '--port', port], kind);
    attachHostHandlers();
    console.log(`[dev] rust host restarted on ws://localhost:${port}`);
  } else {
    console.error('[dev] build failed — host is down; fix the errors and save to try again');
  }
  restarting = false;
  if (restartQueued) {
    restartQueued = false;
    scheduleRestart();
  }
};

if (kind === 'rust') {
  const watchTargets = [
    path.join(paths.rustHostDir, 'src'),
    path.join(paths.rustHostDir, 'Cargo.toml'),
  ];
  for (const target of watchTargets) {
    watch(target, { recursive: true }, scheduleRestart);
  }
  console.log('[dev] watching rust host sources for changes (save to rebuild + restart)');
}
