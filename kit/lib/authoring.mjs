// Internal operation layer for the bounded, inline-source authoring bridge.
// No runtime is loaded until arguments and bridge limits have been checked.
import { createHash } from 'node:crypto';
import { loadRuntimeFromDirectory } from './node.mjs';
import { dependents, explain } from './explain.mjs';
import { validateScenarioFile, runScenarioFile, report as scenarioReport, ScenarioFileError } from './scenarios.mjs';

export const AUTHORING_SCHEMA = 'caveat-authoring/0.1';
export const AUTHORING_LIMITS = Object.freeze({
  messageBytes: 4 * 1024 * 1024, sourceBytes: 1024 * 1024, dataBytes: 1024 * 1024,
  depth: 64, events: 1000, scenarios: 32, steps: 2000, sends: 1000, outputBytes: 4 * 1024 * 1024,
});

export class AuthoringError extends Error {
  constructor(kind, message) { super(message); this.name = 'AuthoringError'; this.kind = kind; }
}

const fields = {
  caveat_validate: ['source'], caveat_check: ['source', 'strict'], caveat_test: ['source', 'scenarios'],
  caveat_explain: ['source', 'events'], caveat_dependents: ['source', 'subject', 'events'],
};
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value)
  && [Object.prototype, null].includes(Object.getPrototypeOf(value));
const fail = message => { throw new AuthoringError('input', message); };
function limit(condition, message) { if (!condition) throw new AuthoringError('limit', message); }
function knownFields(value, allowed, label) {
  if (!object(value)) fail(`${label} must be an object`);
  for (const key of Object.keys(value)) if (!allowed.includes(key)) fail(`${label} has no field ${key}`);
}
function inspectJson(value, depth = 0, ancestors = new Set()) {
  limit(depth <= AUTHORING_LIMITS.depth, `JSON nesting exceeds ${AUTHORING_LIMITS.depth}`);
  if (value === null || typeof value === 'string' || typeof value === 'boolean') return;
  if (typeof value === 'number' && Number.isFinite(value)) return;
  if (!Array.isArray(value) && !object(value)) fail('arguments must contain only JSON values and finite numbers');
  if (ancestors.has(value)) fail('arguments cannot contain cyclic data');
  ancestors.add(value);
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index++) {
      if (!Object.hasOwn(value, index)) fail('arguments cannot contain array holes');
      inspectJson(value[index], depth + 1, ancestors);
    }
  } else for (const item of Object.values(value)) inspectJson(item, depth + 1, ancestors);
  ancestors.delete(value);
}

export function validateAuthoringArguments(toolName, args) {
  if (typeof toolName !== 'string') fail('authoring tool name must be text');
  if (!Object.hasOwn(fields, toolName)) fail(`unknown authoring tool ${toolName}`);
  inspectJson(args);
  knownFields(args, fields[toolName], 'arguments');
  if (typeof args.source !== 'string') fail('source must be inline Caveat text');
  limit(Buffer.byteLength(args.source, 'utf8') <= AUTHORING_LIMITS.sourceBytes, 'source exceeds 1 MiB UTF-8');
  if ('strict' in args && typeof args.strict !== 'boolean') fail('strict must be a boolean');
  if (toolName === 'caveat_dependents' && (typeof args.subject !== 'string' || !args.subject.trim())) fail('subject must name evidence, a reading stream or a caveat');
  if ('events' in args) {
    if (!Array.isArray(args.events)) fail('events must be an array');
    limit(args.events.length <= AUTHORING_LIMITS.events, 'events exceeds 1000 entries');
    limit(Buffer.byteLength(JSON.stringify(args.events), 'utf8') <= AUTHORING_LIMITS.dataBytes, 'events exceeds 1 MiB serialized UTF-8');
    for (const event of args.events) {
      knownFields(event, ['event', 'payload'], 'event');
      if (typeof event.event !== 'string' || !event.event) fail('event must name a non-empty event');
      if ('payload' in event && !object(event.payload)) fail('event payload must be an object');
    }
  }
  if (toolName === 'caveat_test') {
    if (!object(args.scenarios)) fail('scenarios must be a scenario document object');
    limit(Buffer.byteLength(JSON.stringify(args.scenarios), 'utf8') <= AUTHORING_LIMITS.dataBytes, 'scenarios exceeds 1 MiB serialized UTF-8');
    if (args.scenarios.source !== 'inline.cav') fail('scenario source must be the virtual name inline.cav');
    if (Array.isArray(args.scenarios.scenarios)) {
      limit(args.scenarios.scenarios.length <= AUTHORING_LIMITS.scenarios, 'scenario count exceeds 32');
      let steps = 0;
      let sends = 0;
      for (const scenario of args.scenarios.scenarios) {
        if (!object(scenario)) continue; // Existing schema validator reports this.
        if ('source' in scenario && scenario.source !== 'inline.cav') fail('per-scenario source must be the virtual name inline.cav');
        if (!Array.isArray(scenario.steps)) continue;
        steps += scenario.steps.length;
        for (const step of scenario.steps) {
          if (!object(step) || !('send' in step)) continue;
          // Invalid repeat syntax is diagnosed by the existing scenario parser.
          sends += Number.isInteger(step.repeat) && step.repeat >= 1 ? step.repeat : 1;
        }
      }
      limit(steps <= AUTHORING_LIMITS.steps, 'total scenario steps exceeds 2000');
      limit(sends <= AUTHORING_LIMITS.sends, 'expanded scenario sends exceeds 1000');
    }
  }
  return args;
}

// Same declaration projection as the validate CLI; no language interpretation.
export function describeValidation(snapshot, program = 'inline.cav') {
  // A windowed history's limit is its window (spec/caveat-windows-0.1.md).
  const windowed = name => (snapshot.windows?.includes(name) ? { window: true } : {});
  return {
    schema: 'caveat-validate/0.1', program, loads: true,
    events: snapshot.events ?? [],
    reading_streams: Object.fromEntries(Object.entries(snapshot.reading_streams ?? {})
      .map(([name, stream]) => [name, { from: stream.template, limit: stream.limit, ...windowed(name) }])),
    decision_series: Object.fromEntries(Object.entries(snapshot.decision_series ?? {})
      .map(([name, series]) => [name, { limit: series.limit }])),
    displayed: Object.entries(snapshot.bindings ?? {}).flatMap(([target, properties]) =>
      Object.keys(properties).map(property => `${target}.${property}`)),
  };
}

// Preserve accepted/rejected outcomes and stop at the first fatal event. The
// report then describes the last good snapshot, as explain/dependents CLI do.
function replay(runtime, source, events) {
  const session = runtime.open(source);
  let snapshot;
  const sent = [];
  let failed = false;
  try {
    snapshot = session.snapshot();
    for (const { event, payload = {} } of events) {
      try {
        const { snapshot: after, ...outcome } = session.dispatch(event, payload);
        sent.push({ event, payload, outcome });
        if (outcome.outcome === 'accepted') snapshot = after;
      } catch (error) {
        sent.push({ event, payload, outcome: { outcome: 'fatal', kind: error.kind ?? null, message: error.message } });
        failed = true;
        break;
      }
    }
    return { snapshot, sent, failed };
  } finally { session.close(); }
}

export async function runAuthoringOperation(toolName, args, { runtimeDirectory } = {}) {
  validateAuthoringArguments(toolName, args);
  const operation = toolName.slice('caveat_'.length);
  let runtime;
  try { runtime = await loadRuntimeFromDirectory(runtimeDirectory); }
  catch (error) { throw new AuthoringError('runtime', error.message); }
  const wrap = (exitCode, report) => ({
    schema: AUTHORING_SCHEMA, operation,
    sourceSha256: createHash('sha256').update(args.source).digest('hex'),
    runtime: { ...runtime.identity }, exitCode, report,
  });
  const program = 'inline.cav';
  if (operation === 'validate') {
    let session;
    try {
      session = runtime.open(args.source);
      return wrap(0, describeValidation(session.snapshot(), program));
    } catch (error) {
      if (error.kind !== 'load') throw new AuthoringError('runtime', error.message);
      return wrap(2, { schema: 'caveat-validate/0.1', program, loads: false, error: error.message });
    } finally { session?.close(); }
  }
  if (operation === 'check') {
    if (args.source.startsWith('#caveat-bundle')) {
      return wrap(2, { schema: 'caveat-check/0.1', program, loads: false, error: 'is a bundle; check reads a single-file program' });
    }
    try {
      const { schema, diagnostics, suppressed } = runtime.check(args.source);
      const strict = args.strict ?? false;
      return wrap(strict && diagnostics.length ? 1 : 0, { schema, program, loads: true, strict, diagnostics, suppressed });
    } catch (error) {
      if (error.kind !== 'load') throw new AuthoringError('runtime', error.message);
      return wrap(2, { schema: 'caveat-check/0.1', program, loads: false, error: `does not load: ${error.message}` });
    }
  }
  if (operation === 'test') {
    try { validateScenarioFile(args.scenarios); }
    catch (error) {
      if (!(error instanceof ScenarioFileError)) throw error;
      return wrap(2, { file: 'inline.scenarios.json', valid: false, error: error.message });
    }
    const sourceSha256 = createHash('sha256').update(args.source).digest('hex');
    // As the CLI does: check the program before the first scenario; a program
    // that does not load is left to its scenarios to report.
    const check = {};
    if (!args.source.startsWith('#caveat-bundle')) {
      try {
        const { diagnostics, suppressed } = runtime.check(args.source);
        check[program] = { diagnostics, suppressed };
      } catch { /* reported by the scenarios */ }
    }
    if (check[program]?.diagnostics.some(diagnostic => diagnostic.severity === 'error')) {
      const unrun = { file: 'inline.scenarios.json', sources: {}, check, scenarios: [], passed: 0, failed: 0 };
      return wrap(2, scenarioReport([unrun], runtime.identity));
    }
    const result = await runScenarioFile(args.scenarios, {
      runtime, file: 'inline.scenarios.json',
      readSource: name => {
        if (name !== program) throw new AuthoringError('input', 'only inline.cav is available');
        return args.source;
      },
      reload: () => loadRuntimeFromDirectory(runtimeDirectory),
    });
    result.sources = { [program]: sourceSha256 };
    result.check = check;
    return wrap(result.failed ? 1 : 0, scenarioReport([result], runtime.identity));
  }
  let replayed;
  try { replayed = replay(runtime, args.source, args.events ?? []); }
  catch (error) {
    if (error.kind !== 'load') throw new AuthoringError('runtime', error.message);
    return wrap(2, { program, loads: false, error: error.message });
  }
  const { snapshot, sent, failed } = replayed;
  if (operation === 'explain') return wrap(failed ? 1 : 0, { program, ...explain(snapshot, sent) });
  try {
    return wrap(failed ? 1 : 0, { program, events: sent, ...dependents(snapshot, args.subject) });
  } catch (error) {
    return wrap(2, { program, subject: args.subject, error: error.message });
  }
}
