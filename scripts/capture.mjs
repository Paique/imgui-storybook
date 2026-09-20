/** Runs the host offscreen capture into app/public/captures (PNGs + captures.json). */
import path from 'node:path';
import { runHost, paths, hostLabel } from './lib/host.mjs';

const outDir = path.join(paths.root, 'app', 'public', 'captures');

const themes = process.argv.includes('--themes')
  ? (process.argv[process.argv.indexOf('--themes') + 1] ?? 'light,dark')
  : 'light,dark';

console.log(`[capture] ${hostLabel()} rendering captures into ${path.relative(paths.root, outDir)} ...`);
await runHost(['--capture', '--out', outDir, '--themes', themes], { timeoutMs: 600_000 });
console.log('[capture] done');
