// Declared capabilities of the published kit. Supply-chain scanners such as
// Socket report filesystem, subprocess, environment and network use per file.
// Every packed source file using one must be listed here and in
// docs/PACKAGE_SECURITY.md, and every listed file must still use it, so a new
// alert is either expected or a regression.
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const CAPABILITIES = ['filesystem', 'subprocess', 'environment', 'network'];

// Paths are relative to the package root, as `npm pack` lists them.
export const DECLARED_CAPABILITIES = {
  filesystem: [
    'bin/caveat.mjs',
    'examples/agent-evidence/caller.py',
    'examples/agent-evidence/save-text.mjs',
    'examples/agent-evidence/test_branching.py',
    'examples/agent-evidence/test_caller.py',
    'examples/agent-evidence/test_grounds.py',
    'examples/agent-evidence/test_lifecycle.py',
    'examples/agent-evidence/test_qualification.py',
    'lib/demo.mjs',
    'lib/doctor.mjs',
    'lib/mcp.mjs',
    'lib/node.mjs',
  ],
  subprocess: [
    'examples/agent-evidence/caller.py',
    'examples/agent-evidence/test_caller.py',
    'lib/mcp.d.mts',
    'lib/mcp.mjs',
  ],
  environment: [
    'examples/agent-evidence/caller.py',
    'lib/doctor.mjs',
  ],
  // wasm-bindgen's loader fetches the .wasm when given a URL, as in a
  // browser. The kit's Node entry points pass the bytes instead.
  network: [
    'runtime/caveat_runtime.js',
  ],
};

// Coarse text patterns, like the scanners they anticipate: a match in a
// comment counts too. JavaScript patterns are the ones rc11-development.md 4b
// names; Python files ship in the agent-evidence example and get their own.
const JS = {
  filesystem: /['"](?:node:)?fs(?:\/promises)?['"]/,
  subprocess: /\bchild_process\b/,
  environment: /\bprocess\.env\b/,
  network: /['"](?:node:)?(?:https?|http2|net|tls|dgram)['"]|\bfetch\s*\(|\bWebSocket\b/,
};
const PYTHON = {
  filesystem: /\bopen\(|\bpathlib\b|\bPath\(|\btempfile\b|\bshutil\.(?:copy|move|rmtree)|\bos\.(?:remove|unlink|mkdir|makedirs|rename|replace|rmdir)\b/,
  subprocess: /\bsubprocess\b|\bos\.(?:system|popen|exec\w*|spawn\w*)\b/,
  environment: /\bos\.environ\b|\bos\.getenv\b/,
  network: /^\s*(?:import|from)\s+(?:socket|ssl|urllib|requests|http\.client|http\.server)\b|\basyncio\.(?:open_connection|start_server)\b/m,
};
const patternsFor = file => /\.(?:m?js|cjs|m?ts|d\.ts)$/.test(file) ? JS : file.endsWith('.py') ? PYTHON : null;

// files: Map of package-relative path to text. Returns what each capability
// is used by and every difference from the declared table.
export function scanCapabilities(files, declared = DECLARED_CAPABILITIES) {
  const found = Object.fromEntries(CAPABILITIES.map(name => [name, []]));
  for (const [file, text] of [...files].sort(([a], [b]) => a.localeCompare(b))) {
    const patterns = patternsFor(file);
    if (!patterns) continue;
    for (const name of CAPABILITIES) if (patterns[name].test(text)) found[name].push(file);
  }
  const failures = [];
  for (const name of CAPABILITIES) {
    const listed = new Set(declared[name] ?? []);
    for (const file of found[name]) if (!listed.has(file)) failures.push(`${file} uses ${name} but is not declared`);
    for (const file of listed) if (!found[name].includes(file)) failures.push(`${file} is declared for ${name} but does not use it`);
  }
  return { passed: failures.length === 0, found, failures };
}

// Reads every scannable file from a packed npm tarball.
export function tarballFiles(tarball) {
  const tar = (...args) => {
    const result = spawnSync('tar', args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
    if (result.status !== 0) throw new Error(`tar ${args[0]} failed: ${result.stderr}`);
    return result.stdout;
  };
  const files = new Map();
  for (const entry of tar('-tf', tarball).split(/\r?\n/)) {
    if (!entry.startsWith('package/') || entry.endsWith('/')) continue;
    const file = entry.slice('package/'.length);
    if (patternsFor(file)) files.set(file, tar('-xOf', tarball, entry));
  }
  return files;
}

// The table in docs/PACKAGE_SECURITY.md, as { capability: [files] }.
export function documentedCapabilities(markdown) {
  const section = markdown.split(/^## /m).find(part => part.startsWith('Declared capabilities'));
  if (!section) throw new Error('docs/PACKAGE_SECURITY.md has no "Declared capabilities" section');
  const table = {};
  for (const line of section.split(/\r?\n/)) {
    const cells = line.split('|').map(cell => cell.trim());
    const name = cells[1]?.toLowerCase();
    if (!CAPABILITIES.includes(name)) continue;
    table[name] = cells[2] === 'none' ? [] : [...cells[2].matchAll(/`([^`]+)`/g)].map(match => match[1]).sort();
  }
  return table;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length !== 2 || args[0] !== '--tarball') {
    console.error('Usage: node scripts/kit-capabilities.mjs --tarball FILE');
    process.exitCode = 2;
  } else {
    const result = scanCapabilities(tarballFiles(path.resolve(args[1])));
    for (const name of CAPABILITIES) console.log(`${name}: ${result.found[name].join(', ') || 'none'}`);
    for (const failure of result.failures) console.error(`  ${failure}`);
    console.log(`Declared capabilities ${result.passed ? 'match' : 'DIFFER from'} the packed files`);
    process.exitCode = result.passed ? 0 : 1;
  }
}
