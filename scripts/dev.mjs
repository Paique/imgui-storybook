/** Dev mode: imgui host (WebSocket) + storybook dev server, killed together.
 *  Pick the host with `npm run dev -- --host rust` (default: java). */
import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { ensureHostBuilt, spawnHost, paths, resolveHostKind, hostLabel } from './lib/host.mjs';

const kind = resolveHostKind();
ensureHostBuilt(kind);

const port = process.argv.includes('--port')
  ? process.argv[process.argv.indexOf('--port') + 1]
  : '8765';

const host = spawnHost(['--serve', '--port', port], kind);
console.log(`[dev] ${hostLabel(kind)} starting on ws://localhost:${port}`);

const storybook = spawn('npm', ['run', 'storybook'], {
  cwd: path.join(paths.root, 'app'),
  stdio: 'inherit',
  shell: true,
});

const shutdown = () => {
  host.kill();
  storybook.kill();
  process.exit(0);
};
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);

storybook.on('exit', shutdown);
host.on('exit', () => {
  console.error(`[dev] ${hostLabel(kind)} exited unexpectedly`);
  shutdown();
});
