/** Runs the host offscreen capture into app/public/captures (PNGs + captures.json). */
import path from 'node:path';
import { runHost, paths, hostLabel } from './lib/host.mjs';

const outDir = path.join(paths.root, 'app', 'public', 'captures');

const themes = process.argv.includes('--themes')
  ? (process.argv[process.argv.indexOf('--themes') + 1] ?? 'light,dark')
  : 'light,dark';

console.log(`[capture] ${hostLabel()} rendering captures into ${path.relative(paths.root, outDir)} ...`);
const captureArgs = ['--capture', '--out', outDir, '--themes', themes];
if (process.argv.includes('--no-demo')) captureArgs.push('--no-demo');
for (const flag of ['--width', '--height', '--scales']) {
  if (process.argv.includes(flag)) {
    captureArgs.push(flag, process.argv[process.argv.indexOf(flag) + 1]);
  }
}
await runHost(captureArgs, { timeoutMs: 600_000 });
console.log('[capture] done');
