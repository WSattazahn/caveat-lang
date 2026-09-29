// The recorded streams the dispatch-view differential replays: the Glowcap
// replay, Glowcap with 16 mushrooms, the agent ledger's session log and Trail
// Rescue's scenarios. Their episodes are Performance Baseline 0.1's inputs
// (experiments/performance-0.1/inputs), and each program and log is read as it
// is now, with no hash check. The differential compares two paths on the same
// program, so an ordinary edit to a game, the ledger or its log must not fail
// it. The baseline's own loader (lib/workloads.mjs) refuses any input whose
// sha256 changed; it is for the performance harness, which must time the
// bytes it recorded.
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { repo } from './helpers.mjs';

export const STREAMS = ['glowcap-replay', 'glowcap-scaled-16', 'ledger-session', 'trail-rescue-scenarios'];
const MANIFEST = 'experiments/performance-0.1/inputs/workloads.json';

// Glowcap with more mushrooms, declared after `entity grove`, as the baseline
// scales it. Returns null when the program no longer declares that entity.
function scaled(source, extra) {
  const anchor = 'entity grove kind mushroom at garden;\n';
  const at = source.indexOf(anchor);
  if (at < 0) return null;
  const added = Array.from({ length: extra }, (_, index) => `entity patch${index + 1} kind mushroom at garden;\n`).join('');
  return source.slice(0, at + anchor.length) + added + source.slice(at + anchor.length);
}

async function episodesOf(root, spec) {
  if (spec.generate) {
    const { prefix, fill, length } = spec.generate;
    const episode = prefix.map(([event, payload]) => [event, payload]);
    while (episode.length < length) episode.push([fill[0], fill[1]]);
    return [episode];
  }
  if (spec.jsonl) {
    const text = await readFile(path.join(root, spec.jsonl.path), 'utf8');
    return [text.split(/\r?\n/).filter(line => line.trim()).map(line => {
      const { event, payload } = JSON.parse(line);
      return [event, payload ?? {}];
    })];
  }
  if (spec.json) {
    const file = JSON.parse(await readFile(path.join(root, spec.json.path), 'utf8'));
    return file.episodes.map(episode => episode.steps.map(([event, payload]) => [event, JSON.parse(payload)]));
  }
  throw new Error('episodes need generate, jsonl or json');
}

// Each stream as { id, program, source, episodes, skipped }, where an episode
// is a list of [event, payload object]. A stream whose program cannot be built
// any more (Glowcap without `entity grove`) has source null and says why.
export async function loadStreams(root = repo, ids = STREAMS) {
  const manifest = JSON.parse(await readFile(path.join(root, MANIFEST), 'utf8'));
  const streams = [];
  for (const id of ids) {
    const spec = manifest.workloads.find(workload => workload.id === id);
    if (!spec) throw new Error(`no workload ${id} in ${MANIFEST}`);
    let source = await readFile(path.join(root, spec.program.path), 'utf8');
    let skipped = null;
    if (spec.program.addMushrooms) {
      source = scaled(source, spec.program.addMushrooms);
      if (source === null) skipped = `${spec.program.path} no longer declares entity grove, so it cannot be scaled`;
    }
    streams.push({ id, program: spec.program.path, source, episodes: await episodesOf(root, spec.episodes), skipped });
  }
  return streams;
}
