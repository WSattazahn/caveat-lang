// Shows that each converted file carries the original expectations rather than
// restating the reference run. Every expected value in the originals, and
// every expected acceptance or rejection, is altered one at a time; the altered
// scenario is converted again and run. Each alteration must be refused by the
// conversion or fail in the runner. A pass would mean that expectation was not
// carried into the converted file.
//
//   node experiments/scenario-conversion/mutation-check.mjs   (writes results-mutations.json)
import { readFile, writeFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from '../../kit/lib/node.mjs';
import {
  ConversionError, convertGlowcapScenario, convertTrailScenario, glowcapScenarios, runConverted, trailScenarios,
} from './convert.mjs';

const runtime = await loadRuntimeFromDirectory();
const clone = value => JSON.parse(JSON.stringify(value));
const isObject = value => value !== null && typeof value === 'object' && !Array.isArray(value);

// Replacements are valid names where possible, so an alteration reaches the
// runner instead of being refused as an unknown name.
const TRAIL_LISTS = {
  names: ['clearBy', 'blockedBy', 'because', 'basis', 'reopenedBy'], name: ['report_stone', 'report_reed'], caveat: ['stale', 'secondhand'],
  swaps: { id: ['report_stone', 'scout_stone_1'], tunnel: ['stone', 'reed'], method: ['report', 'scout'], condition: ['clear', 'blocked'], change: ['committed', 'reopened'], status: ['clear', 'blocked'] },
};
const GLOWCAP_LISTS = {
  names: ['because', 'supportedBy', 'contradictedBy', 'basis', 'reopenedBy'], name: ['absorb_cave', 'taste_ruin'], caveat: ['tasted_in_dark', 'taste_faded'],
  swaps: { change: ['committed', 'reopened'] },
};

function alterations(value, key, lists) {
  const swap = lists.swaps[key];
  if (typeof value === 'string' && swap) {
    const other = swap.find(item => item !== value);
    return [[`${JSON.stringify(value)} -> ${JSON.stringify(other)}`, other]];
  }
  if (typeof value === 'string') return [[`${JSON.stringify(value)} -> altered`, `${value}·`]];
  if (typeof value === 'number') return [[`${value} -> ${value + 1}`, value + 1]];
  if (typeof value === 'boolean') return [[`${value} -> ${!value}`, !value]];
  if (value === null) return [['null -> "altered"', 'altered']];
  if (Array.isArray(value) && value.every(item => typeof item !== 'object' || item === null)) {
    const pool = lists.names.includes(key) ? lists.name : key === 'caveats' ? lists.caveat : null;
    const out = [];
    if (value.length) out.push(['drop last', value.slice(0, -1)]);
    const extra = pool?.find(item => !value.includes(item));
    if (extra) out.push([`add ${extra}`, [...value, extra]]);
    else out.push(['add "altered"', [...value, 'altered']]);
    return out;
  }
  return [];
}

// Leaves of an expected value: [path within the value, key, value].
function leaves(value, path = [], key = null) {
  if (isObject(value)) return Object.entries(value).flatMap(([name, item]) => leaves(item, [...path, name], name));
  if (Array.isArray(value) && value.some(item => isObject(item))) {
    return value.flatMap((item, index) => leaves(item, [...path, index], key));
  }
  return [[path, key, value]];
}

function setAt(target, path, value) {
  if (!path.length) return value;
  let node = target;
  for (const key of path.slice(0, -1)) node = node[key];
  node[path.at(-1)] = value;
  return target;
}

async function attempt(convert, altered) {
  let converted;
  try {
    converted = convert(altered);
  } catch (error) {
    if (error instanceof ConversionError) return 'refused';
    return `error: ${error.message}`;
  }
  const doc = { schema: 'caveat-scenarios/0.1', source: converted.source, scenarios: [converted.scenario] };
  const result = await runConverted(doc, runtime);
  return result.failed ? 'failed' : 'UNDETECTED';
}

const results = { trail: [], glowcap: [] };

// Trail Rescue: step expectations, expected outcomes and the shared initial view.
const trailSource = '../../game/trail_rescue.cav';
const convertTrail = (scenario, options) => ({ source: trailSource, ...convertTrailScenario(scenario, { ...options, mutate: true }) });
for (const scenario of trailScenarios()) {
  for (const [index, step] of scenario.steps.entries()) {
    if (step.op === 'dispatch') {
      const altered = clone(scenario);
      if (altered.steps[index].reject) delete altered.steps[index].reject; else altered.steps[index].reject = true;
      results.trail.push({ scenario: scenario.id, step: index + 1, what: step.reject ? 'expect acceptance' : 'expect rejection', result: await attempt(item => convertTrail(item), altered) });
    }
    for (const [pointer, expected] of Object.entries(step.expect ?? {})) {
      for (const [path, key, value] of leaves(expected, [], pointer.split('/').at(-1))) {
        for (const [label, replacement] of alterations(value, key, TRAIL_LISTS)) {
          const altered = clone(scenario);
          altered.steps[index].expect[pointer] = setAt(clone(expected), path, replacement);
          results.trail.push({ scenario: scenario.id, step: index + 1, what: `${pointer}${path.length ? `/${path.join('/')}` : ''}: ${label}`, result: await attempt(item => convertTrail(item), altered) });
        }
      }
    }
  }
}
{
  const [first] = trailScenarios();
  const base = JSON.parse(await readFile(new URL('../trail-rescue/scenarios.json', import.meta.url), 'utf8')).initialView;
  for (const [path, key, value] of leaves(base)) {
    for (const [label, replacement] of alterations(value, key, TRAIL_LISTS)) {
      const altered = setAt(clone(base), path, replacement);
      results.trail.push({ scenario: `${first.id} (initial view)`, step: 0, what: `/${path.join('/')}: ${label}`, result: await attempt(item => convertTrail(item, { initialView: altered }), first) });
    }
  }
}

// Glowcap: expectation leaves and expected outcomes.
const glowcapSource = '../../experiments/glowcap/caveat5/glowcap.cav';
const convertGlowcap = scenario => ({ source: glowcapSource, ...convertGlowcapScenario(scenario, { mutate: true }) });
for (const scenario of glowcapScenarios()) {
  for (const [index, step] of scenario.steps.entries()) {
    if (step[0] === 'expect') {
      for (const [path, key, value] of leaves(step[1])) {
        for (const [label, replacement] of alterations(value, key, GLOWCAP_LISTS)) {
          const altered = clone(scenario);
          altered.steps[index][1] = setAt(clone(step[1]), path, replacement);
          results.glowcap.push({ scenario: scenario.id, step: index + 1, what: `${path.join('.')}: ${label}`, result: await attempt(convertGlowcap, altered) });
        }
      }
    } else if (step[0] === 'reject') {
      const altered = clone(scenario);
      const event = step[1];
      altered.steps[index] = event.type === 'tick' ? ['tick', event.dt, 1] : [event.type, event.id, event.kind];
      results.glowcap.push({ scenario: scenario.id, step: index + 1, what: 'expect acceptance', result: await attempt(convertGlowcap, altered) });
    } else if (step[0] !== 'resume') {
      const altered = clone(scenario);
      altered.steps[index] = ['reject', step[0] === 'tick' ? { type: 'tick', dt: step[1] } : { type: step[0], id: step[1], kind: step[2] }];
      results.glowcap.push({ scenario: scenario.id, step: index + 1, what: 'expect rejection', result: await attempt(convertGlowcap, altered) });
    }
  }
}

const summarize = list => list.reduce((counts, item) => {
  const key = item.result.startsWith('error') ? 'error' : item.result;
  counts[key] = (counts[key] ?? 0) + 1;
  return counts;
}, {});
const report = {
  schema: 1,
  runtime: runtime.identity,
  summary: { trail: summarize(results.trail), glowcap: summarize(results.glowcap) },
  notDetected: [...results.trail, ...results.glowcap].filter(item => item.result === 'UNDETECTED' || item.result.startsWith('error')),
  results,
};
await writeFile(new URL('./results-mutations.json', import.meta.url), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report.summary));
for (const item of report.notDetected) console.log(`${item.result} ${item.scenario} step ${item.step}: ${item.what}`);
if (report.notDetected.length) process.exitCode = 1;
