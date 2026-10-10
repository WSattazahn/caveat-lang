// Release closeout check: do npm `latest`, npm `next` and the MCP Registry's
// latest entry all name the released version, and does the registry entry
// still match server.json? Comparison stays here so it is testable without the
// network; main() only fetches and prints. It changes nothing on npm or the
// registry: moving `next` and publishing the registry entry are owner steps
// (docs/CONSOLIDATION_PLAN.md, "Distribution closeout").
//
//   node scripts/check-distribution.mjs --version 0.1.0-rc.16
//   node scripts/check-distribution.mjs --version 0.1.0-rc.16 --lag next="owner keeps next on rc.15 until ..."
//
// Exit 0: every channel names the version or lags with a stated reason.
// Exit 1: an unexplained mismatch or a usage error. Exit 2: a source could not be read, which
// means the check is unavailable, not that a channel is wrong.
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const PACKAGE = 'caveat-lang';
export const SERVER_NAME = 'io.github.WSattazahn/caveat-lang';
export const NPM_URL = `https://registry.npmjs.org/${PACKAGE}`;
export const REGISTRY_URL = `https://registry.modelcontextprotocol.io/v0/servers/${encodeURIComponent(SERVER_NAME)}/versions/latest`;
export const CHANNELS = ['npm-latest', 'npm-next', 'registry'];
const OFFICIAL = 'io.modelcontextprotocol.registry/official';

export function parseArguments(argv) {
  const options = { version: null, lag: {} };
  for (let i = 0; i < argv.length; i++) {
    const argument = argv[i];
    if (argument === '--version') {
      options.version = argv[++i] ?? null;
    } else if (argument === '--lag') {
      const value = argv[++i] ?? '';
      const at = value.indexOf('=');
      const channel = at < 0 ? value : value.slice(0, at);
      const reason = at < 0 ? '' : value.slice(at + 1).trim();
      if (!CHANNELS.includes(channel)) throw new Error(`--lag names one of ${CHANNELS.join(', ')}, not ${JSON.stringify(channel)}.`);
      if (!reason) throw new Error(`--lag ${channel} needs a reason: --lag ${channel}="why it lags".`);
      options.lag[channel] = reason;
    } else {
      throw new Error(`Unknown argument ${JSON.stringify(argument)}.`);
    }
  }
  if (!/^\d+\.\d+\.\d+(?:-rc\.\d+)?$/.test(options.version ?? '')) {
    throw new Error('Name the released version: --version 0.1.0-rc.16. A checkout version is not a publication target.');
  }
  return options;
}

// One row per channel and one per metadata comparison. A row is `ok`, `lag`
// (differs, with the owner's stated reason) or `mismatch`.
export function compareDistribution({ version, serverJson, distTags, registry, lag = {} }) {
  const rows = [];
  const channel = (name, found) => {
    if (found === version) rows.push({ check: name, status: 'ok', found });
    else if (lag[name]) rows.push({ check: name, status: 'lag', found, expected: version, reason: lag[name] });
    else rows.push({ check: name, status: 'mismatch', found: found ?? null, expected: version });
  };
  const metadata = (name, found, expected) => {
    rows.push(found === expected
      ? { check: name, status: 'ok', found }
      : { check: name, status: 'mismatch', found: found ?? null, expected });
  };

  channel('npm-latest', distTags?.latest);
  channel('npm-next', distTags?.next);

  const server = registry?.server ?? {};
  const official = registry?._meta?.[OFFICIAL] ?? {};
  const npmPackage = (server.packages ?? []).find(entry => entry.registryType === 'npm');
  const registryVersion = server.version === npmPackage?.version ? server.version : `${server.version} (package ${npmPackage?.version})`;
  channel('registry', registryVersion);
  metadata('registry-name', server.name, SERVER_NAME);
  metadata('registry-package', npmPackage?.identifier, PACKAGE);
  metadata('registry-status', official.status, 'active');
  metadata('registry-is-latest', official.isLatest, true);
  // The live entry is compared with server.json on the checked-out revision:
  // a description or launch change on main is not live until republished.
  metadata('registry-description', server.description, serverJson.description);
  metadata('registry-launch', canonical(launch(npmPackage)), canonical(launch(serverJson.packages?.find(entry => entry.registryType === 'npm'))));

  return { version, rows, ok: rows.every(row => row.status !== 'mismatch') };
}

// The registry may return keys in another order than server.json.
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value ?? null);
}

function launch(entry) {
  return entry ? { runtimeHint: entry.runtimeHint, transport: entry.transport, packageArguments: entry.packageArguments } : null;
}

async function fetchJson(url) {
  const response = await fetch(url, { headers: { accept: 'application/json' } });
  if (!response.ok) throw new Error(`${url} answered HTTP ${response.status}.`);
  return { status: response.status, body: await response.json() };
}

async function main() {
  const options = parseArguments(process.argv.slice(2));
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const serverJson = JSON.parse(await readFile(path.join(root, 'server.json'), 'utf8'));
  const fetchedAt = new Date().toISOString();
  let npm;
  let registry;
  try {
    [npm, registry] = await Promise.all([fetchJson(NPM_URL), fetchJson(REGISTRY_URL)]);
  } catch (error) {
    console.error(`Check unavailable at ${fetchedAt}: ${error.message}`);
    process.exitCode = 2;
    return;
  }
  const distTags = npm.body['dist-tags'] ?? {};
  const result = compareDistribution({ version: options.version, serverJson, distTags, registry: registry.body, lag: options.lag });
  // Keep the responses that matter for the record: the whole registry entry,
  // and npm's channels with their publish times rather than the whole packument.
  const times = Object.fromEntries(Object.values(distTags).map(v => [v, npm.body.time?.[v] ?? null]));
  console.log(JSON.stringify({ fetchedAt, ...result,
    responses: { npm: { url: NPM_URL, status: npm.status, distTags, time: times }, registry: { url: REGISTRY_URL, status: registry.status, body: registry.body } } }, null, 2));
  if (!result.ok) process.exitCode = 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
