import { spawn, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const javaHostDir = path.join(root, 'java-host');
const hostBin = path.join(javaHostDir, 'host', 'build', 'install', 'host', 'bin');

function isWindows() {
  return process.platform === 'win32';
}

/** Gradle builds the host distribution if needed. Throws with output on failure. */
export function ensureHostBuilt() {
  const script = isWindows() ? 'host.bat' : 'host';
  if (existsSync(path.join(hostBin, script))) return;
  console.log('[host] building java host distribution (first run)...');
  const result = spawnSync(isWindows() ? 'gradlew.bat' : './gradlew', ['-q', ':host:installDist'], {
    cwd: javaHostDir,
    stdio: ['ignore', 'inherit', 'inherit'],
    shell: isWindows(),
  });
  if (result.status !== 0 || !existsSync(path.join(hostBin, script))) {
    throw new Error('failed to build the java host — run java-host/gradlew :host:installDist manually');
  }
}

/** Runs the host with the given args and resolves with its stdout. */
export function runHost(args, { timeoutMs = 120_000 } = {}) {
  ensureHostBuilt();
  const script = isWindows() ? 'host.bat' : 'host';
  return new Promise((resolve, reject) => {
    const child = spawn(script, args, {
      cwd: hostBin,
      stdio: ['ignore', 'pipe', 'inherit'],
      shell: isWindows(),
      windowsHide: true,
    });
    let stdout = '';
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error(`host ${args.join(' ')} timed out`));
    }, timeoutMs);
    child.stdout.on('data', (chunk) => (stdout += chunk));
    child.on('error', (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.on('close', (code) => {
      clearTimeout(timer);
      if (code === 0) resolve(stdout);
      else reject(new Error(`host ${args.join(' ')} exited with ${code}`));
    });
  });
}

/** Spawns the host without waiting; resolves the child process. */
export function spawnHost(args) {
  ensureHostBuilt();
  const script = isWindows() ? 'host.bat' : 'host';
  return spawn(script, args, {
    cwd: hostBin,
    stdio: 'inherit',
    shell: isWindows(),
    windowsHide: true,
  });
}

export const paths = { root, javaHostDir };
