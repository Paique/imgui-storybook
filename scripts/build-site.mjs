/**
 * Full static build: generate stories, capture PNGs, build the Storybook site, and emit
 * AI-friendly docs (STORIES.md + manifest.json) into the output.
 */
import { spawnSync } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import { paths, runHost } from './lib/host.mjs';

const distSite = path.join(paths.root, 'dist-site');
const capturesJsonPath = path.join(paths.root, 'app', 'public', 'captures', 'captures.json');

function run(command, args, opts = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', shell: process.platform === 'win32', ...opts });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed with exit ${result.status}`);
  }
}

function md(value) {
  return String(value ?? '').replaceAll('|', '\\|');
}

function buildStoriesMarkdown(catalog, captures) {
  const byStory = new Map();
  for (const entry of captures.entries) {
    if (!byStory.has(entry.storyId)) byStory.set(entry.storyId, []);
    byStory.get(entry.storyId).push(entry);
  }

  const lines = [];
  lines.push('# ImGui Storybook — Stories');
  lines.push('');
  lines.push(
    `> Rendered by the real Dear ImGui ${catalog.dearImgui} (binding ${catalog.imguiJavaVersion}, host ${catalog.hostVersion}). ` +
      `Canvas ${captures.width}×${captures.height}. Generated ${captures.generatedAt}.`,
  );
  lines.push('');
  lines.push('## Contents');
  lines.push('');
  for (const story of catalog.stories) {
    lines.push(`- [${story.title}](#${story.id}) — ${story.storyClass}`);
  }
  lines.push('');

  for (const story of catalog.stories) {
    const entries = byStory.get(story.id) ?? [];
    lines.push(`## ${story.title}`);
    lines.push('');
    lines.push(`\`${story.id}\` · source: \`${story.storyClass}\``);
    lines.push('');
    if (story.description) {
      lines.push(story.description);
      lines.push('');
    }
    if (story.args.length > 0) {
      lines.push('| Arg | Type | Default | Options |');
      lines.push('| --- | --- | --- | --- |');
      for (const arg of story.args) {
        lines.push(
          `| \`${arg.name}\` | ${arg.type.toLowerCase()} | ${md(arg.default)} | ${md((arg.options ?? []).join(', '))} |`,
        );
      }
      lines.push('');
    }
    const defaults = entries.filter((entry) => entry.preset === 'default' && entry.scale === 1);
    if (defaults.length > 0) {
      lines.push(...defaults.map(
        (entry) => `![${story.title} (${entry.theme})](captures/${entry.file})`),
      );
      lines.push('');
    }
    const presets = entries.filter((entry) => entry.preset !== 'default' && entry.scale === 1);
    if (presets.length > 0) {
      lines.push('### Presets');
      lines.push('');
      for (const entry of presets) {
        lines.push(`**${entry.preset}** (${entry.theme}):`);
        lines.push('');
        lines.push(`![${story.title} — ${entry.preset} (${entry.theme})](captures/${entry.file})`);
        lines.push('');
      }
    }
    lines.push('---');
    lines.push('');
  }
  return lines.join('\n');
}

function buildManifest(catalog, captures) {
  const byStory = new Map();
  for (const entry of captures.entries) {
    if (!byStory.has(entry.storyId)) byStory.set(entry.storyId, []);
    byStory.get(entry.storyId).push(entry);
  }
  return {
    generatedAt: captures.generatedAt,
    hostVersion: catalog.hostVersion,
    imguiJavaVersion: catalog.imguiJavaVersion,
    dearImgui: catalog.dearImgui,
    canvas: { width: captures.width, height: captures.height },
    stories: catalog.stories.map((story) => ({
      id: story.id,
      title: story.title,
      description: story.description,
      storyClass: story.storyClass,
      args: story.args,
      presets: story.presets,
      captures: (byStory.get(story.id) ?? []).map((entry) => ({
        preset: entry.preset,
        theme: entry.theme,
        scale: entry.scale,
        file: `captures/${entry.file}`,
        width: entry.width,
        height: entry.height,
        args: entry.args,
      })),
    })),
  };
}

async function main() {
  console.log('[build:site] 1/4 generating story files...');
  run('node', [path.join(paths.root, 'scripts', 'gen-stories.mjs')]);

  console.log('[build:site] 2/4 capturing PNGs...');
  run('node', [path.join(paths.root, 'scripts', 'capture.mjs')]);

  console.log('[build:site] 3/4 building storybook static site...');
  run('npm', ['run', 'build'], { cwd: path.join(paths.root, 'app') });

  console.log('[build:site] 4/4 writing AI docs (STORIES.md, manifest.json)...');
  const catalog = JSON.parse(await runHost(['--list', '--json']));
  const captures = JSON.parse(await readFile(capturesJsonPath, 'utf8'));
  await writeFile(path.join(distSite, 'STORIES.md'), buildStoriesMarkdown(catalog, captures), 'utf8');
  await writeFile(path.join(distSite, 'manifest.json'), JSON.stringify(buildManifest(catalog, captures), null, 2), 'utf8');

  console.log(`[build:site] done → ${path.relative(paths.root, distSite)} (index.html, STORIES.md, manifest.json, captures/)`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
