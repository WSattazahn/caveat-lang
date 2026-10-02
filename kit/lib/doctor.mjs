// CLI diagnostics inspect package files and PATH shims. They never run an
// executable found on PATH. Only the selected Caveat runtime is loaded.
import { constants } from 'node:fs';
import { access, readFile, realpath, stat } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isDeepStrictEqual } from 'node:util';
import { defaultRuntimeDirectory, loadRuntimeFromDirectory } from './node.mjs';

const kitRoot = fileURLToPath(new URL('../', import.meta.url));
const FILES = ['caveat_runtime.js', 'caveat_runtime_bg.wasm'];
const SOURCE = `evidence probe from "doctor's synthetic observation";
state value = 0; decisions choice limit 2;
event observe; event refuse;
on observe reveal probe;
on observe set value = qualified(7, probe);
on observe commit choice because enough using value;
on refuse set value = 9;
on refuse reject "doctor's expected refusal";
bind report.value = value;`;
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const messageOf = error => String(error?.message ?? error);
const samePath = (a, b, platform) => platform === 'win32' ? a.toLowerCase() === b.toLowerCase() : a === b;

async function exists(file) {
  try { return (await stat(file)).isFile(); } catch { return false; }
}

// Recognize only npm's invocation line; a mention in a comment is insufficient.
// Unknown wrappers remain unknown rather than being executed to identify them.
function npmTarget(text, extension) {
  const lines = text.split(/\r?\n/).map(line => line.trim());
  for (const line of lines) {
    if (extension === '.cmd') {
      const match = line.match(/^(?:endLocal & goto #_undefined_# 2>NUL \|\| title %COMSPEC% & )?"%_prog%"\s+"%dp0%([^"]+)"\s+%\*\s*$/i);
      if (match) return match[1];
    } else if (extension === '.ps1') {
      const match = line.match(/^&\s+[^\r\n]+\s+"\$basedir\/([^"]+)"\s+\$args\s*$/);
      if (match) return match[1];
    } else {
      const match = line.match(/^exec\s+[^\r\n]+\s+"\$basedir\/([^"]+)"\s+"\$@"\s*$/);
      if (match) return match[1];
    }
  }
  return null;
}

async function ownerOf(target, name, platform) {
  let directory = path.dirname(target);
  for (;;) {
    try {
      const manifest = JSON.parse(await readFile(path.join(directory, 'package.json'), 'utf8'));
      const bins = typeof manifest.bin === 'string'
        ? { [String(manifest.name).split('/').at(-1)]: manifest.bin } : manifest.bin;
      if (typeof manifest.name === 'string' && typeof manifest.version === 'string' && typeof bins?.[name] === 'string') {
        const expected = await realpath(path.resolve(directory, bins[name]));
        if (samePath(expected, target, platform)) return { name: manifest.name, version: manifest.version, directory };
      }
      // A package boundary is authoritative even when this isn't its named bin.
      return null;
    } catch (error) {
      if (error.code !== 'ENOENT' && error.code !== 'ENOTDIR') return null;
    }
    const parent = path.dirname(directory);
    if (parent === directory) return null;
    directory = parent;
  }
}

async function inspectExecutable(file, name, platform) {
  try {
    let target = await realpath(file);
    const extension = path.extname(file).toLowerCase();
    if (samePath(target, path.resolve(file), platform) && ['.cmd', '.ps1', ''].includes(extension)) {
      const size = (await stat(file)).size;
      if (size <= 65536) {
        const relative = npmTarget(await readFile(file, 'utf8'), extension);
        if (relative) target = await realpath(path.resolve(path.dirname(file), relative.replaceAll('\\', '/').replace(/^\//, '')));
      }
    }
    return { target, owner: await ownerOf(target, name, platform) };
  } catch (error) { return { target: null, owner: null, error: messageOf(error) }; }
}

// This is PATH/PATHEXT file resolution, not an assertion about shell aliases,
// functions, shell profiles or a shell's cached command table.
export async function inspectExecutables({ env = process.env, cwd = process.cwd(), platform = process.platform } = {}) {
  const envValue = name => Object.entries(env).find(([key]) => key.toUpperCase() === name)?.[1];
  const delimiter = platform === 'win32' ? ';' : ':';
  const pathValue = envValue('PATH');
  // An explicit empty PATH component names cwd; an absent PATH names nothing.
  const directories = [...new Set((pathValue === undefined ? [] : String(pathValue).split(delimiter)).map(dir => path.resolve(cwd, dir)))];
  const suffixes = platform === 'win32'
    ? [...new Set(String(envValue('PATHEXT') || '.COM;.EXE;.BAT;.CMD').split(';').filter(Boolean).map(ext => ext.toLowerCase()))]
    : [''];
  const reports = [];
  for (const name of ['caveat-lang', 'caveat']) {
    const candidates = [];
    for (const directory of directories) {
      for (const suffix of suffixes) {
        const file = path.join(directory, `${name}${suffix}`);
        if (!(await exists(file))) continue;
        if (platform !== 'win32') {
          try { await access(file, constants.X_OK); } catch { continue; }
        }
        candidates.push(file);
      }
    }
    const file = candidates[0] ?? null;
    reports.push({ name, path: file, target: null, owner: null, candidates,
      ...(file ? await inspectExecutable(file, name, platform) : {}) });
  }
  return reports;
}

function smokeSession(runtime) {
  let session;
  let resumed;
  const require = (condition, message) => { if (!condition) throw new Error(message); };
  try {
    require(runtime.check(SOURCE)?.schema === 'caveat-check/0.1', 'check returned an incompatible report');
    session = runtime.open(SOURCE);
    require(session.view().bindings.report.value === 0, 'initial view differs from the synthetic program');
    require(session.dispatch('observe').outcome === 'accepted', 'synthetic observation was not accepted');
    const before = session.snapshot();
    require(session.view().bindings.report.value === 7, 'accepted observation did not update the view');
    require(isDeepStrictEqual(before.commitment_grounds['choice@1'], { evidence: ['probe'], caveats: [] }), 'decision grounds differ from the synthetic observation');
    const saved = session.save();
    const rejected = session.dispatch('refuse');
    require(rejected.outcome === 'rejected' && rejected.origin === 'policy' && rejected.code === 'reject', 'synthetic policy refusal did not return policy/reject');
    require(session.save() === saved && isDeepStrictEqual(session.snapshot(), before), 'rejected event changed the session');
    resumed = runtime.restore(SOURCE, saved);
    require(isDeepStrictEqual(resumed.snapshot(), before), 'restored snapshot differs from the saved session');
    return { accepted: true, rejectedWithoutChange: true, exactGrounds: true, restored: true };
  } finally { resumed?.close(); session?.close(); }
}

export async function doctor({ runtimeDirectory = defaultRuntimeDirectory(), packageDirectory = kitRoot,
  nodeVersion = process.versions.node, env = process.env, cwd = process.cwd() } = {}) {
  const result = {
    schema: 'caveat-doctor/0.1', ok: false,
    node: { version: nodeVersion, required: '>=20', executable: process.execPath },
    package: { directory: path.resolve(packageDirectory), name: null, version: null },
    runtime: { directory: path.resolve(runtimeDirectory), files: [], buildInfo: null, identity: null, session: null },
    resolution: 'PATH files (PATHEXT on Windows); shell aliases, functions and caches are not inspected',
    executables: [], checks: [],
  };
  const check = (id, status, message) => result.checks.push({ id, status, message });
  const supportedNode = /^\d+\.\d+\.\d+(?:[-+].*)?$/.test(nodeVersion) && Number(nodeVersion.split('.')[0]) >= 20;
  check('node', supportedNode ? 'pass' : 'fail', `Node ${nodeVersion}; this CLI requires >=20`);
  try {
    const manifest = JSON.parse(await readFile(path.join(result.package.directory, 'package.json'), 'utf8'));
    const valid = manifest.name === 'caveat-lang' && typeof manifest.version === 'string'
      && /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(manifest.version)
      && manifest.engines?.node === '>=20' && manifest.bin?.['caveat-lang'] === 'bin/caveat.mjs'
      && manifest.bin?.caveat === 'bin/caveat.mjs';
    if (!valid) throw new Error('expected caveat-lang, a package version, Node >=20 and both CLI aliases');
    result.package.name = manifest.name;
    result.package.version = manifest.version;
    check('package', 'pass', `${manifest.name}@${manifest.version}`);
  } catch (error) { check('package', 'fail', `Cannot establish package identity: ${messageOf(error)}`); }

  let runtimeFilesValid = true;
  for (const name of FILES) {
    try {
      const bytes = await readFile(path.join(result.runtime.directory, name));
      result.runtime.files.push({ name, bytes: bytes.length, sha256: sha256(bytes) });
      if (name.endsWith('.wasm') && !WebAssembly.validate(bytes)) throw new Error('invalid WebAssembly binary');
      check(`runtime.${name}`, 'pass', `${name}: ${bytes.length} bytes, sha256 ${sha256(bytes)}`);
    } catch (error) {
      runtimeFilesValid = false;
      check(`runtime.${name}`, 'fail', `Cannot use ${name}: ${messageOf(error)}`);
    }
  }
  const metadataPaths = [path.join(result.runtime.directory, 'build-info.json'), path.join(result.runtime.directory, '..', 'build-info.json')];
  let metadataPath;
  for (const candidate of metadataPaths) if (await exists(candidate)) { metadataPath = candidate; break; }
  if (!metadataPath) check('runtime.build', 'warn', 'No build-info.json; runtime file hashes are available, build revision is unknown');
  else {
    try {
      const metadata = JSON.parse(await readFile(metadataPath, 'utf8'));
      if (!metadata || !/^[0-9a-f]{40}$/i.test(metadata.revision ?? '')
        || typeof metadata.clean !== 'boolean' || typeof metadata.compiled !== 'boolean'
        || typeof metadata.host !== 'string' || !metadata.host) throw new Error('expected revision, clean, compiled and host fields');
      result.runtime.buildInfo = metadata;
      check('runtime.build', metadata.clean && metadata.compiled ? 'pass' : 'warn',
        `Build ${metadata.revision}; clean=${metadata.clean}, compiled=${metadata.compiled}, host=${metadata.host} (local metadata, not authenticated)`);
    } catch (error) { check('runtime.build', 'fail', `Malformed build-info.json: ${messageOf(error)}`); }
  }
  if (runtimeFilesValid && supportedNode) {
    try {
      const runtime = await loadRuntimeFromDirectory(result.runtime.directory);
      result.runtime.identity = runtime.identity;
      check('runtime.load', 'pass', 'Selected JavaScript adapter and WASM load together');
      try {
        result.runtime.session = smokeSession(runtime);
        check('runtime.session', 'pass', 'Synthetic observation, exact decision grounds, atomic refusal and save/restore succeeded');
      } catch (error) { check('runtime.session', 'fail', `Test session failed: ${messageOf(error)}`); }
    } catch (error) { check('runtime.load', 'fail', `Runtime is incompatible or could not load: ${messageOf(error)}`); }
  } else check('runtime.session', 'warn', 'Test session skipped because Node or runtime files failed checks');

  result.executables = await inspectExecutables({ env, cwd });
  for (const executable of result.executables) {
    const id = `executable.${executable.name}`;
    if (!executable.path) check(id, 'warn', `${executable.name}: not found on PATH; a direct file invocation can still work`);
    else if (!executable.owner) check(id, 'warn', `${executable.name}: ${executable.path}; package owner could not be established without executing it`);
    else if (executable.owner.name !== 'caveat-lang') check(id, executable.name === 'caveat-lang' ? 'fail' : 'warn',
      `${executable.name}: ${executable.path} belongs to ${executable.owner.name}@${executable.owner.version}; prefer the caveat-lang alias`);
    else check(id, executable.owner.version === result.package.version ? 'pass' : 'warn',
      `${executable.name}: ${executable.path} belongs to ${executable.owner.name}@${executable.owner.version}`);
  }
  result.ok = !result.checks.some(item => item.status === 'fail');
  return result;
}

export function formatDoctor(result) {
  const lines = [`CAVEAT Language doctor: ${result.ok ? 'checks passed' : 'checks failed'}`,
    `Package directory: ${result.package.directory}`, `Runtime directory: ${result.runtime.directory}`,
    `Executable resolution: ${result.resolution}`];
  for (const check of result.checks) lines.push(`[${check.status.toUpperCase()}] ${check.id}: ${check.message}`);
  for (const executable of result.executables) {
    if (executable.target) lines.push(`  ${executable.name} target: ${executable.target}`);
    if (executable.candidates.length > 1) lines.push(`  ${executable.name} other PATH candidates: ${executable.candidates.slice(1).join(', ')}`);
  }
  return lines.join('\n');
}
