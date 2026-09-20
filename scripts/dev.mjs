/** Dev mode: java host (WebSocket) + storybook dev server, killed together. */
import { spawn } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { ensureHostBuilt, spawnHost, paths } from './lib/host.mjs';

ensureHostBuilt();

const port = process.argv.includes('--port')
  ? process.argv[process.argv.indexOf('--port') + 1]
  : '8765';

const host = spawnHost(['--serve', '--port', port]);
console.log(`[dev] java host starting on ws://localhost:${port}`);

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
  console.error('[dev] java host exited unexpectedly');
  shutdown();
});
