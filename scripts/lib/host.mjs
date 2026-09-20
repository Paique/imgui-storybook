import { spawn, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const javaHostDir = path.join(root, 'java-host');
const javaHostBin = path.join(javaHostDir, 'host', 'build', 'install', 'host', 'bin');
const rustHostDir = path.join(root, 'rust-host');
const rustHostBin = path.join(rustHostDir, 'target', 'release');

function isWindows() {
  return process.platform === 'win32';
}

/**
 * Which host to use: `--host rust|java` (anywhere in argv), `IMGUI_HOST` env, or java.
 * Both hosts speak the same WebSocket protocol and expose the same CLI.
 */
export function resolveHostKind() {
  const i = process.argv.indexOf('--host');
  const fromArgv = i !== -1 ? process.argv[i + 1] : undefined;
  const kind = (fromArgv || process.env.IMGUI_HOST || 'java').toLowerCase();
  if (kind !== 'java' && kind !== 'rust') {
    throw new Error(`unknown host "${kind}" — use --host java|rust or IMGUI_HOST=java|rust`);
  }
  return kind;
}

function javaHostScript() {
  return isWindows() ? 'host.bat' : 'host';
}

function rustHostScript() {
  return isWindows() ? 'host.exe' : 'host';
}

/** Gradle/cargo build the host distribution if needed. Throws with output on failure. */
export function ensureHostBuilt(kind = resolveHostKind()) {
  if (kind === 'rust') {
    const bin = path.join(rustHostBin, rustHostScript());
    if (existsSync(bin)) return;
    console.log('[host] building rust host (release, first run)...');
    const result = spawnSync('cargo', ['build', '--release'], {
      cwd: rustHostDir,
      stdio: ['ignore', 'inherit', 'inherit'],
      shell: isWindows(),
    });
    if (result.status !== 0 || !existsSync(bin)) {
      throw new Error('failed to build the rust host — run cargo build --release in rust-host/ manually');
    }
    return;
  }
  const script = javaHostScript();
  if (existsSync(path.join(javaHostBin, script))) return;
  console.log('[host] building java host distribution (first run)...');
  const result = spawnSync(isWindows() ? 'gradlew.bat' : './gradlew', ['-q', ':host:installDist'], {
    cwd: javaHostDir,
    stdio: ['ignore', 'inherit', 'inherit'],
    shell: isWindows(),
  });
  if (result.status !== 0 || !existsSync(path.join(javaHostBin, script))) {
    throw new Error('failed to build the java host — run java-host/gradlew :host:installDist manually');
  }
}

function hostCommand(kind) {
  if (kind === 'rust') {
    // Direct exe spawn (no shell) so child.kill() reaches the host process itself.
    return { script: path.join(rustHostBin, rustHostScript()), cwd: rustHostBin, shell: false };
  }
  return { script: javaHostScript(), cwd: javaHostBin, shell: isWindows() };
}

/** Runs the host with the given args and resolves with its stdout. */
export function runHost(args, { timeoutMs = 120_000 } = {}, kind = resolveHostKind()) {
  ensureHostBuilt(kind);
  const { script, cwd, shell } = hostCommand(kind);
  return new Promise((resolve, reject) => {
    const child = spawn(script, args, {
      cwd,
      stdio: ['ignore', 'pipe', 'inherit'],
      shell,
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
export function spawnHost(args, kind = resolveHostKind()) {
  ensureHostBuilt(kind);
  const { script, cwd, shell } = hostCommand(kind);
  return spawn(script, args, {
    cwd,
    stdio: 'inherit',
    shell,
    windowsHide: true,
  });
}

/** Kills the host and any child processes (needed for the shell-wrapped java host). */
export function killHostTree(child) {
  if (!child || child.pid === undefined) return;
  if (isWindows()) {
    spawn('taskkill', ['/pid', String(child.pid), '/T', '/F'], { stdio: 'ignore' });
  } else {
    child.kill('SIGTERM');
  }
}

export function hostLabel(kind = resolveHostKind()) {
  return kind === 'rust' ? 'rust host' : 'java host';
}

export const paths = { root, javaHostDir, rustHostDir };
