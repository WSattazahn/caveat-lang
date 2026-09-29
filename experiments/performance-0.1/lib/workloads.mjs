// The benchmark inputs: programs and event streams named in
// inputs/workloads.json, each checked against the sha256 recorded there.
// Inputs always come from this harness's own checkout, never from the tree
// being measured, so every target runs exactly the same bytes.
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const harnessDirectory = fileURLToPath(new URL('../', import.meta.url));
export const repositoryRoot = path.resolve(harnessDirectory, '..', '..');
export const manifestPath = path.join(harnessDirectory, 'inputs', 'workloads.json');

export const sha256 = (data) => createHash('sha256').update(data).digest('hex');

export async function loadManifest(file = manifestPath) {
  const manifest = JSON.parse(await readFile(file, 'utf8'));
  if (manifest.schema !== 'caveat-performance-inputs/0.1') throw new Error(`unexpected manifest schema ${manifest.schema}`);
  return manifest;
}

async function readChecked(relative, expected, what) {
  const bytes = await readFile(path.join(repositoryRoot, relative));
  const actual = sha256(bytes);
  if (expected && actual !== expected) {
    throw new Error(`${what} ${relative} has sha256 ${actual}; inputs/workloads.json records ${expected}`);
  }
  return { text: bytes.toString('utf8'), sha256: actual };
}

// A size-scaled Glowcap: the same program with more mushrooms declared after
// `entity grove`. Every per-mushroom `for` block, tick rule and binding grows
// with it; the first three mushrooms keep their positions, so the event
// stream is unchanged.
export function scaleGlowcap(source, extraMushrooms) {
  const anchor = 'entity grove kind mushroom at garden;\n';
  const at = source.indexOf(anchor);
  if (at < 0) throw new Error('glowcap.cav no longer declares `entity grove`; cannot scale it');
  const added = Array.from({ length: extraMushrooms }, (_, index) => `entity patch${index + 1} kind mushroom at garden;\n`).join('');
  return source.slice(0, at + anchor.length) + added + source.slice(at + anchor.length);
}

// A program without its `bind` statements: the same rules and states, and so
// the same transaction, with no bindings to evaluate. Against the program as
// written, it separates binding evaluation from the rest of `apply` without
// touching the runtime. Every binding in the Glowcap program is one line.
export function removeBindings(source) {
  const lines = source.split('\n');
  const kept = lines.filter((line) => !/^\s*bind .*;\s*$/.test(line));
  if (kept.length === lines.length) throw new Error('the program has no one-line bind statements to remove');
  if (kept.some((line) => /^\s*bind\b/.test(line))) throw new Error('a bind statement spans lines; cannot remove bindings safely');
  return kept.join('\n');
}

export async function loadProgram(spec) {
  const { text, sha256: fileSha256 } = await readChecked(spec.path, spec.sha256, 'program');
  let source = text;
  if (spec.addMushrooms) source = scaleGlowcap(source, spec.addMushrooms);
  if (spec.withoutBindings) source = removeBindings(source);
  const programSha256 = sha256(source);
  if (spec.generatedSha256 && programSha256 !== spec.generatedSha256) {
    throw new Error(`generated program for ${spec.path} has sha256 ${programSha256}; expected ${spec.generatedSha256}`);
  }
  return { path: spec.path, fileSha256, sha256: programSha256, source, addMushrooms: spec.addMushrooms ?? 0, withoutBindings: Boolean(spec.withoutBindings) };
}

// Steps are [event, payloadText]: the text a host passes to dispatch, with
// keys in the order written in the manifest or log.
const step = (event, payload) => [event, JSON.stringify(payload)];

async function expandEpisodes(spec) {
  if (spec.generate) {
    const { prefix, fill, length } = spec.generate;
    const episode = prefix.map(([event, payload]) => step(event, payload));
    while (episode.length < length) episode.push(step(fill[0], fill[1]));
    return [episode];
  }
  if (spec.jsonl) {
    const { text } = await readChecked(spec.jsonl.path, spec.jsonl.sha256, 'event log');
    const episode = text.split(/\r?\n/).filter((line) => line.trim()).map((line) => {
      const { event, payload } = JSON.parse(line);
      return step(event, payload ?? {});
    });
    return [episode];
  }
  if (spec.json) {
    const { text } = await readChecked(spec.json.path, spec.json.sha256, 'episode file');
    const file = JSON.parse(text);
    return file.episodes.map((episode) => episode.steps.map(([event, payload]) => [event, payload]));
  }
  throw new Error('episodes need generate, jsonl or json');
}

export const streamHash = (episodes) => sha256(JSON.stringify(episodes));

// Resolves one workload: program text, episodes (verified), and the options a
// suite may override. maxEvents truncates each episode after verification.
export async function loadWorkload(manifest, id, { maxEvents = null, repeat = null } = {}) {
  const spec = manifest.workloads.find((workload) => workload.id === id);
  if (!spec) throw new Error(`no workload ${id} in inputs/workloads.json`);
  const program = await loadProgram(spec.program);
  const episodes = await expandEpisodes(spec.episodes);
  const actual = streamHash(episodes);
  if (spec.streamSha256 && actual !== spec.streamSha256) {
    throw new Error(`workload ${id} stream has sha256 ${actual}; inputs/workloads.json records ${spec.streamSha256}`);
  }
  const truncated = maxEvents ? episodes.map((episode) => episode.slice(0, maxEvents)) : episodes;
  const repeats = repeat ?? spec.episodes.repeat ?? 1;
  let adapter = null;
  if (spec.adapter) {
    const { sha256: adapterSha256 } = await readChecked(spec.adapter.path, spec.adapter.sha256, 'adapter');
    adapter = { path: spec.adapter.path, sha256: adapterSha256, events: spec.adapter.events };
  }
  const events = truncated.reduce((sum, episode) => sum + episode.length, 0);
  return {
    id,
    description: spec.description,
    program,
    episodes: truncated,
    repeats,
    streamSha256: actual,
    truncatedTo: maxEvents,
    eventsPerPass: events * repeats,
    segments: maxEvents ? (spec.segments ?? []).filter((segment) => segment.from < maxEvents).map((segment) => ({ ...segment, to: Math.min(segment.to, maxEvents) })) : (spec.segments ?? []),
    adapter,
  };
}

// The glowcap adapter's own event shape, from a step of a glowcap stream.
export function glowcapAdapterEvent([event, payloadText]) {
  const payload = JSON.parse(payloadText);
  return event === 'tick' ? { type: 'tick', dt: payload.dt } : { type: event, id: payload.target, kind: payload.sort };
}
