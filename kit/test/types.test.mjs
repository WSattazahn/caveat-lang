// The TypeScript declarations beside each library module name exactly what
// the code has. Without a TypeScript compiler in the repository, this reads
// the declarations as text and compares every name with the modules, with
// real sessions, reports and failures, with the runtime's Rust structs and
// with the specs and documents that list codes and kinds. A declared name the
// code does not have fails, and so does one the code has and the
// declarations leave out.
// Compiler assignability is checked separately in runtime.yml using
// kit/type-tests/dependents-format.mts and pinned TypeScript 5.9.3.
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { CaveatError, CaveatSession, createRuntime } from '../lib/session.mjs';
import { dependents, explain, parseEvents } from '../lib/explain.mjs';
import { createServer } from '../lib/serve.mjs';
import {
  ScenarioFileError, expectAt, firstDifference, groundsViolation, match as matchExpected, parseScenarioFile, report, runScenarioFile,
} from '../lib/scenarios.mjs';
import { Real, RealModule, faulty, real, repo, scenarioFile, sourceReader, thermostat, trail } from './helpers.mjs';
import { provenanceOverflow } from './fatal-fixture.mjs';

const kit = fileURLToPath(new URL('../', import.meta.url));
const manifest = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));

// ---------------------------------------------------------------- reading declarations

const stripComments = text => text.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/[^\n]*/g, '');

// Splits text at depth-0 separators, ignoring those inside (), [] and {}.
function splitTop(text, separator) {
  const parts = [];
  let depth = 0;
  let start = 0;
  for (let index = 0; index < text.length; index++) {
    const char = text[index];
    if ('([{'.includes(char)) depth++;
    else if (')]}'.includes(char)) depth--;
    else if (char === separator && depth === 0) { parts.push(text.slice(start, index)); start = index + 1; }
  }
  parts.push(text.slice(start));
  return parts.map(part => part.trim()).filter(Boolean);
}

function block(text, from) {
  const open = text.indexOf('{', from);
  let depth = 0;
  for (let index = open; index < text.length; index++) {
    if (text[index] === '{') depth++;
    else if (text[index] === '}' && --depth === 0) return text.slice(open + 1, index);
  }
  throw new Error('unbalanced braces');
}

function readDeclarations(source) {
  const text = stripComments(source);
  const values = new Set();
  const shapes = new Map();
  const aliases = new Map();
  // Every export is a type, an interface or a declared function, constant or
  // class, so the values below are all the module's declared values.
  for (const line of text.match(/^export\b.*/gm) ?? []) {
    assert.match(line, /^export (?:type [\w{]|(?:interface|declare (?:function|const|class)) \w)/, `an export this test cannot read: ${line}`);
  }
  for (const [, name] of text.matchAll(/^export declare (?:function|const|class) (\w+)/gm)) values.add(name);
  for (const match of text.matchAll(/^export (?:declare )?(interface|class) (\w+)[^{]*/gm)) {
    const required = new Set();
    const optional = new Set();
    const statics = new Set();
    for (const member of splitTop(block(text, match.index), ';')) {
      if (member === '#private' || member.startsWith('[') || member.startsWith('constructor(') || member.startsWith('new (')) continue;
      const found = /^(readonly\s+)?(static\s+)?(get\s+|set\s+)?(\w+)(\?)?\s*[(:]/.exec(member);
      assert.ok(found, `cannot read the member "${member}" of ${match[2]}`);
      (found[2] ? statics : found[5] ? optional : required).add(found[4]);
    }
    shapes.set(match[2], { required, optional, statics });
  }
  for (const match of text.matchAll(/^export type (\w+) =/gm)) {
    aliases.set(match[1], splitTop(text.slice(match.index + match[0].length), ';')[0]);
  }
  return { values, shapes, aliases };
}

const declarations = {};
for (const file of (await readdir(path.join(kit, 'lib'))).filter(name => name.endsWith('.d.mts'))) {
  declarations[file.replace(/\.d\.mts$/, '')] = readDeclarations(await readFile(path.join(kit, 'lib', file), 'utf8'));
}
const allShapes = new Map(Object.values(declarations).flatMap(declared => [...declared.shapes]));
const allAliases = new Map(Object.values(declarations).flatMap(declared => [...declared.aliases]));

function shape(name) {
  const found = allShapes.get(name);
  assert.ok(found, `no interface or class ${name} is declared`);
  return found;
}
const members = name => new Set([...shape(name).required, ...shape(name).optional]);
const literals = name => {
  assert.ok(allAliases.has(name), `no type ${name} is declared`);
  return [...allAliases.get(name).matchAll(/'([^']*)'/g)].map(match => match[1]).sort();
};
const sorted = values => [...new Set(values)].sort();

// Each object has every required member, and nothing undeclared. With
// complete, the objects between them also have every optional member.
function assertShape(name, objects, { complete = false, ignore = [] } = {}) {
  const { required, optional } = shape(name);
  const seen = new Set();
  assert.ok(objects.length > 0, `no ${name} to compare`);
  for (const object of objects) {
    const keys = Object.keys(object).filter(key => !ignore.includes(key));
    for (const key of keys) {
      assert.ok(required.has(key) || optional.has(key), `${name} has ${key}, which is not declared`);
      seen.add(key);
    }
    for (const key of required) assert.ok(keys.includes(key), `${name} declares ${key}, which ${JSON.stringify(object).slice(0, 120)} lacks`);
  }
  if (complete) for (const key of optional) assert.ok(seen.has(key), `${name} declares ${key}?, which no sample has`);
}

// ---------------------------------------------------------------- programs

const PERMISSION = `identifiers limit 8; claim ready; claim may_merge;
  evidence ci from "checks"; evidence go from "a go-ahead";
  readings checks from ci limit 4; readings approvals from go limit 4; decisions merge limit 2;
  state head = 0;
  event pushed commit id; event check; event approved commit id; event merge;
  on pushed set head = commit;
  on check sample checks = 1 supports ready;
  on approved sample approvals = commit supports may_merge;
  on merge commit merge because enough using latest(checks) permitted by latest(approvals) for head;`;
const WITHDRAWAL = `claim safe; claim misreading;
  evidence ci from "checks"; evidence recheck from "a re-read";
  readings checks from ci limit 4; decisions merge limit 2;
  event check; event decide; event misread;
  on check sample checks = 1 supports safe;
  on decide commit merge because enough using latest(checks);
  on misread reveal recheck supports misreading;
  on misread withdraw latest(checks) because recheck;`;
// Lines 12-13 repeat lines 10-11: a check warning with a related line.
const FROST = `claim frost_risk;
evidence probe from "a soil probe";
evidence drone from "a survey drone";
readings soil from probe limit 12;
readings aerial from drone limit 12;
event probe_read celsius min -40 max 60;
event drone_read celsius min -40 max 60;
decisions uncover limit 4;
event decide;
on probe_read when celsius <= 2 sample soil = celsius supports frost_risk;
on probe_read when celsius > 2 sample soil = celsius opposes frost_risk;
on drone_read when celsius <= 2 sample aerial = celsius supports frost_risk;
on drone_read when celsius > 2 sample aerial = celsius opposes frost_risk;
on decide when not committed(uncover) commit uncover because enough using latest(soil);
`;
const RENEWAL = `claim safe; evidence bite from "a bite"; caveat faded consequence low; renewable bite limit 4;
  event tick dt min 0 max 0.1; event eat; event regrow;
  on eat reveal bite supports safe; on eat qualify bite with faded after 5; on regrow renew bite;`;
// Windows retire a reading, a revision and a journal entry
// (spec/caveat-windows-0.1.md).
const WINDOWED = `claim seen; evidence glimpse from "a glimpse";
  readings sighting from glimpse window 2; decisions trust limit 8; journal window 1;
  event look; event decide;
  on look sample sighting = 1 supports seen;
  on decide when committed(trust) reopen trust because latest(sighting);
  on decide commit trust because enough using history_count(sighting);`;
const WINDOWED_EVENTS = [['look'], ['decide'], ['look'], ['look'], ['decide']];

// Each program with events, and the snapshot after every event.
function sessionsOf() {
  const runs = [
    [thermostat, [['read', { value: 17 }], ['read', { value: 25 }], ['read', { value: 99 }]]],
    [trail, []],
    [PERMISSION, [['pushed', { commit: 'abc' }], ['check'], ['approved', { commit: 'abc' }], ['merge']]],
    [WITHDRAWAL, [['check'], ['decide'], ['misread']]],
    ['evidence memory from "lookup"; event consult; on consult reveal memory;', [['consult']]],
    [WINDOWED, WINDOWED_EVENTS],
    // Last: the checks below read its scheduled qualification and renewal.
    [RENEWAL, [['eat'], ['regrow'], ['tick', { dt: 0.1 }]]],
  ];
  const snapshots = [];
  const views = [];
  for (const [source, events] of runs) {
    const session = real.open(source);
    snapshots.push(session.snapshot());
    views.push(session.view());
    for (const [event, payload] of events) {
      const outcome = session.dispatch(event, payload ?? {});
      if (outcome.outcome === 'accepted') snapshots.push(outcome.snapshot);
      views.push(session.view());
    }
    session.close();
  }
  return { snapshots, views };
}
const { snapshots, views } = sessionsOf();

// ---------------------------------------------------------------- entry points

test('every entry point names its declarations, which declare exactly its exports', async () => {
  const modules = (await readdir(path.join(kit, 'lib'))).filter(name => name.endsWith('.mjs')).map(name => name.slice(0, -4));
  assert.deepEqual(sorted(Object.keys(declarations)), sorted(modules), 'every library module has declarations');
  for (const [entry, target] of Object.entries(manifest.exports)) {
    if (entry === './runtime/*') continue;
    assert.deepEqual(Object.keys(target), ['types', 'default'], `${entry} lists types first`);
    assert.equal(target.types, target.default.replace(/\.mjs$/, '.d.mts'), `${entry} declares its own module`);
    assert.ok(existsSync(path.join(kit, target.types)), `${target.types} exists`);
    const name = path.basename(target.default, '.mjs');
    const exported = await import(pathToFileURL(path.join(kit, target.default)).href);
    assert.deepEqual(sorted([...declarations[name].values]), sorted(Object.keys(exported)), `${entry}: declared values against exports`);
  }
});

test('generated runtime declarations name the library their disposal members require', async () => {
  for (const directory of ['pkg', 'pkg-reactive']) {
    const file = path.join(repo, 'dist', directory, 'caveat_runtime.d.ts');
    const text = await readFile(file, 'utf8');
    assert.match(text, /\[Symbol\.dispose\]\(\): void;/, `${directory} declares a disposal member`);
    assert.match(text, /^\/\/\/ <reference lib="esnext\.disposable" \/>\r?\n/, `${directory} loads the disposal types before its declarations`);
  }
});

test('a declared constant is the value the module exports', async () => {
  for (const name of Object.keys(declarations)) {
    const text = stripComments(await readFile(path.join(kit, 'lib', `${name}.d.mts`), 'utf8'));
    const exported = await import(pathToFileURL(path.join(kit, 'lib', `${name}.mjs`)).href);
    for (const [, constant, type] of text.matchAll(/^export declare const (\w+): ([^;]*);/gm)) {
      if (/^'[^']*'$/.test(type)) assert.equal(exported[constant], type.slice(1, -1), `${name}.${constant}`);
      else if (/^readonly \[/.test(type)) {
        assert.ok(Object.isFrozen(exported[constant]), `${constant} is frozen`);
        assert.deepEqual(exported[constant], [...type.matchAll(/'([^']*)'/g)].map(match => match[1]), `${name}.${constant}`);
      }
    }
  }
});

// ---------------------------------------------------------------- session library

test('a session, a runtime and a CaveatError have exactly the declared members', () => {
  const methods = Object.getOwnPropertyNames(CaveatSession.prototype).filter(name => name !== 'constructor');
  assert.deepEqual(sorted(methods), sorted(members('CaveatSession')));
  assert.equal(shape('CaveatSession').statics.size, 0);
  assert.deepEqual(sorted(Object.keys(real)), sorted(members('CaveatRuntime')));
  assert.deepEqual(sorted(Object.keys(createRuntime(Real))), sorted(members('CaveatRuntime')));

  const errors = [new CaveatError('load', 'made here')];
  const capture = action => { try { action(); } catch (error) { errors.push(error); return; } assert.fail('expected a CaveatError'); };
  capture(() => real.open('this is not caveat'));
  capture(() => real.restore(thermostat, '{"not":"a save"}'));
  const session = real.open(thermostat);
  capture(() => session.dispatch('read', { value: NaN }));
  session.close();
  capture(() => session.dispatch('read', { value: 17 }));
  const fatal = real.open('state output = 0; event run; on run set output = 1;' + provenanceOverflow('on run'));
  capture(() => fatal.dispatch('run'));
  const { runtime } = faulty({ dispatch: () => '{"schema":"caveat-dispatch/0.1","outcome":"maybe"}' });
  capture(() => runtime.open(thermostat).dispatch('read', { value: 17 }));
  assert.ok(errors.every(error => error instanceof CaveatError));
  assert.deepEqual(sorted(errors.map(error => error.kind)), literals('CaveatErrorKind'));
  assertShape('CaveatError', errors, { complete: true, ignore: ['name', 'message', 'stack'] });
  assertShape('DispatchFatal', errors.filter(error => error.report).map(error => error.report), { complete: true });

  const where = new ScenarioFileError('file', 'made here');
  assertShape('ScenarioFileError', [where], { complete: true, ignore: ['name', 'message', 'stack'] });
});

test('outcomes have the declared shapes, and the declared codes and kinds are the documented ones', async () => {
  const session = real.open(thermostat);
  assertShape('DispatchAccepted', [session.dispatch('read', { value: 17 })], { complete: true });
  assertShape('DispatchViewAccepted', [session.dispatchView('read', { value: 20 })], { complete: true });
  assertShape('DispatchRejected', [session.dispatch('read', { value: 41 }), session.dispatchView('warm')], { complete: true });
  session.close();

  const { ORIGINS } = await import('../lib/session.mjs');
  assert.deepEqual(literals('Origin'), sorted(ORIGINS));
  const spec = await readFile(path.join(repo, 'spec', 'caveat-dispatch-0.1.md'), 'utf8');
  const catalog = [...spec.matchAll(/^\| `(\w+)` \| `(\w+)` \|/gm)];
  assert.deepEqual(sorted(catalog.map(([, origin]) => origin)), literals('Origin'), 'the catalog\'s origins');
  assert.deepEqual(literals('KnownRejectionCode'), sorted(catalog.map(([, , code]) => code)), 'the catalog\'s codes');
  const readme = await readFile(path.join(kit, 'README.md'), 'utf8');
  assert.deepEqual(literals('CaveatErrorKind'), sorted([...readme.matchAll(/^\| `(\w+)` \|/gm)].map(match => match[1]).filter(name => name !== 'kind')), 'the README\'s kinds');
  const library = await readFile(path.join(kit, 'lib', 'session.mjs'), 'utf8');
  assert.deepEqual(literals('SessionState'), sorted([...library.matchAll(/#state = '(\w+)'/g)].map(match => match[1])), 'the states a session takes');
});

// The structs the runtime serializes, field by field: a field it skips is
// absent, and one it skips when empty is optional.
async function rustShapes() {
  const found = new Map();
  for (const file of ['reactive.rs', 'reactive_expr.rs', 'map.rs', 'game_session.rs']) {
    const text = await readFile(path.join(repo, 'runtime', 'src', file), 'utf8');
    for (const match of text.matchAll(/^pub (struct|enum) (\w+)(?:<[^>]*>)? \{\n([\s\S]*?)^\}/gm)) {
      const [, kind, name, body] = match;
      assert.ok(!found.has(name), `${name} is defined twice`);
      if (kind === 'struct') {
        const required = new Set();
        const optional = new Set();
        let attributes = '';
        for (const line of body.split('\n')) {
          if (/^\s*#\[serde\(/.test(line)) { attributes += line; continue; }
          const field = /^ {4}(?:pub(?:\([^)]*\))? )?(\w+): /.exec(line);
          if (!field) continue;
          const rename = /rename = "([^"]+)"/.exec(attributes)?.[1];
          if (!/\bskip\b(?!_)/.test(attributes)) ((/skip_serializing_if/.test(attributes)) ? optional : required).add(rename ?? field[1]);
          attributes = '';
        }
        found.set(name, { required, optional });
      } else {
        const variants = new Map();
        let current = null;
        for (const line of body.split('\n')) {
          const variant = /^ {4}(\w+)(?: \{|,)/.exec(line);
          if (variant) { current = new Set(); variants.set(variant[1], current); continue; }
          const field = /^ {8}(\w+): /.exec(line);
          if (field && current) current.add(field[1]);
        }
        found.set(name, { variants });
      }
    }
  }
  return found;
}

const RUST_STRUCTS = {
  Snapshot: 'ReactiveSnapshot', View: 'ReactiveView', Provenance: 'Provenance', QualifiedValue: 'Tracked',
  CommitmentBasis: 'CommitmentBasis', ReadingOccurrence: 'ReadingOccurrence', ReadingStream: 'ReadingStream',
  DecisionRevision: 'DecisionRevision', DecisionSeries: 'DecisionSeries', DecisionJournalEntry: 'JournalEntry',
  Renewal: 'Renewal', ScheduledQualification: 'ScheduledQualification', Withdrawal: 'Withdrawal',
  PermissionRecord: 'PermissionRecord', PermissionScope: 'PermissionScope', EventSignature: 'EventSignature',
  EventParameter: 'Parameter', Control: 'Control', Clock: 'Clock', World: 'MapWorld', Place: 'MapPlace',
  Entity: 'MapEntity', Connection: 'MapConnection', SnapshotSymbol: 'GameSymbol', Relation: 'MapRelation',
  Commitment: 'MapCommitment', RetainedCaveat: 'RetainedCaveat', Budget: 'MapBudget',
};
const RUST_ENUMS = { Cue: 'Cue', Effect: 'EffectReport' };
const snake = name => name.replace(/[A-Z]/g, (letter, index) => `${index ? '_' : ''}${letter.toLowerCase()}`);

test('snapshot and view types have the fields the runtime serializes', async () => {
  const rust = await rustShapes();
  for (const [declared, struct] of Object.entries(RUST_STRUCTS)) {
    assert.ok(rust.has(struct), `runtime/src has no struct ${struct}`);
    const { required, optional } = shape(declared);
    assert.deepEqual(sorted([...required]), sorted([...rust.get(struct).required]), `${declared}: required fields of ${struct}`);
    assert.deepEqual(sorted([...optional]), sorted([...rust.get(struct).optional]), `${declared}: optional fields of ${struct}`);
  }
  for (const [declared, enumeration] of Object.entries(RUST_ENUMS)) {
    const { variants } = rust.get(enumeration);
    const union = allAliases.get(declared).split('|').map(item => item.trim()).filter(Boolean);
    assert.deepEqual(sorted(union), sorted([...variants.keys()].map(variant => `${declared}${variant}`)), `${declared}: variants of ${enumeration}`);
    for (const [variant, fields] of variants) {
      assert.deepEqual(sorted([...members(`${declared}${variant}`)]), sorted(['kind', ...fields]), `${declared}${variant}`);
      assert.match(stripComments(await readFile(path.join(kit, 'lib', 'session.d.mts'), 'utf8')),
        new RegExp(`interface ${declared}${variant} \\{\\s*kind: '${snake(variant)}';`), `${declared}${variant}'s kind`);
    }
  }
  // And the real thing, on programs that use permissions, withdrawals, renewals and identifiers.
  assertShape('Snapshot', snapshots, { complete: true });
  assertShape('View', views, { complete: true });
  assert.ok(snapshots.every(snapshot => snapshot.schema === 'caveat-reactive/0.1'));
  assert.ok(views.every(view => view.schema === 'caveat-reactive-view/0.1'));
  const last = snapshots.at(-1);
  assertShape('ScheduledQualification', last.scheduled_qualifications, { complete: true });
  assertShape('Renewal', Object.values(last.renewals), { complete: true });
  assertShape('DecisionJournalEntry', snapshots.flatMap(snapshot => snapshot.decision_journal), { complete: true });
  assertShape('ReadingStream', snapshots.flatMap(snapshot => Object.values(snapshot.reading_streams)), { complete: true });
  assertShape('ReadingOccurrence', snapshots.flatMap(snapshot => Object.values(snapshot.reading_streams).flatMap(stream => stream.occurrences)), { complete: true });
  assertShape('EffectSample', snapshots.flatMap(snapshot => snapshot.effects.filter(effect => effect.kind === 'sample')), { complete: true });
  assertShape('EffectReveal', snapshots.flatMap(snapshot => snapshot.effects.filter(effect => effect.kind === 'reveal')), { complete: true });
  assertShape('EffectRetire', snapshots.flatMap(snapshot => snapshot.effects.filter(effect => effect.kind === 'retire')), { complete: true });
});

// The runtime contract is what the library calls on a runtime's module, class
// and sessions: an optional member is one it checks for first, and the
// repository's runtime has every member.
test('the runtime contract and the options name what the library reads', async () => {
  const read = async file => stripComments(await readFile(path.join(kit, 'lib', file), 'utf8'));
  const library = await read('session.mjs');
  const named = (text, pattern) => sorted([...text.matchAll(pattern)].map(found => found[1]));
  for (const [declared, called, checked, real] of [
    ['RuntimeSessionHandle', /this\.#inner\.(\w+)\(/g, /typeof this\.#inner\.(\w+) !== 'function'/g, Real.prototype],
    ['RuntimeSessionClass', /SessionClass\.(\w+)\(/g, /typeof SessionClass\.(\w+) !== 'function'/g, Real],
    ['RuntimeModule', /namespace\.(\w+)\b/g, /$^/g, RealModule],
  ]) {
    assert.deepEqual(named(library, called), sorted(members(declared)), `${declared}: what the library calls`);
    assert.deepEqual(named(library, checked), sorted(shape(declared).optional), `${declared}: what it checks for first`);
    for (const name of members(declared)) assert.ok(name in real, `the runtime has ${declared}.${name}`);
  }

  const destructured = (text, entry) => {
    const found = new RegExp(`export (?:async )?function ${entry}\\((?:\\w+, )?\\{ ([^}]*) \\}`).exec(text);
    assert.ok(found, `${entry} takes an options object`);
    return sorted(found[1].split(',').map(item => item.trim().split(/\s*=/)[0]));
  };
  assert.deepEqual(destructured(library, 'loadRuntime'), sorted(members('LoadRuntimeOptions')));
  assert.deepEqual(destructured(await read('serve.mjs'), 'createServer'), sorted(members('ServeOptions')));
  assert.deepEqual(destructured(await read('scenarios.mjs'), 'runScenarioFile'), sorted(members('RunScenarioOptions')));
  const node = await read('node.mjs');
  const identity = [/const identity = \{([^}]*)\}/, /Object\.assign\(identity, \{([^}]*)\}/].flatMap(pattern => named(pattern.exec(node)[1], /(\w+):/g));
  assert.deepEqual(sorted(identity), sorted(members('RuntimeIdentity')), 'the identity loadRuntimeFromDirectory gives');

  assertShape('Difference', [
    firstDifference({ a: 1 }, { a: 2 }), firstDifference([1], [1, 2]), matchExpected({ a: 1, b: 2 }, { a: 1, b: 3 }),
    expectAt({ a: 1 }, '/a', 2), expectAt({}, '/b', 1),
  ], { complete: true });
  assert.equal(firstDifference({ a: 1 }, { a: 1 }), null);
  assertShape('GroundsViolation', [
    groundsViolation({ value_grounds: { x: { evidence: ['e'] } }, qualified_values: { x: { provenance: { evidence: [] } } } }),
    groundsViolation({ commitment_grounds: { d: { caveats: ['c'] } }, commitment_bases: {} }),
  ], { complete: true });
  assert.equal(groundsViolation(snapshots.at(-1)), null);
  assertShape('EventLine', parseEvents('{"event":"read","payload":{"value":17}}\n{"event":"warm"}\n'), { complete: true });
});

test('a program interface has the declared shapes and kinds', () => {
  const gauge = `identifiers limit 8; claim frost_risk; evidence probe from "a probe"; caveat old consequence low;
    readings soil from probe limit 4; decisions cover limit 2; place yard kind yard; entity north kind bed at yard;
    state level = 0 min -40 max 60; event read celsius min -40 max 60; event pick bed kind bed, size in small large, who id;
    cue ping sound 440 0.2 0.5;
    on read sample soil = celsius supports frost_risk; on read set level = celsius;
    bind gauge.value = level; bind gauge.label = "cold" when level < 0; bind gauge.low = level < 0;`;
  const interfaces = [real.interface(gauge), real.interface(thermostat), real.interface(trail)];
  assertShape('ProgramInterface', interfaces, { complete: true });
  const parameters = interfaces.flatMap(found => found.events.flatMap(event => event.parameters));
  assertShape('InterfaceParameter', parameters, { complete: true });
  assert.deepEqual(sorted(parameters.map(parameter => parameter.type)), literals('InterfaceParameterType'));
  const bindings = interfaces.flatMap(found => found.bindings);
  assertShape('InterfaceBinding', bindings, { complete: true });
  assert.deepEqual(sorted(bindings.map(binding => binding.type)), literals('InterfaceBindingType'));
  assertShape('InterfaceEvent', interfaces.flatMap(found => found.events), { complete: true });
  assertShape('InterfaceState', interfaces.flatMap(found => found.states), { complete: true });
  assertShape('InterfaceCue', interfaces.flatMap(found => found.cues), { complete: true });
  assertShape('InterfaceDecisions', interfaces.flatMap(found => found.decisions), { complete: true });
  assertShape('InterfaceReadings', interfaces.flatMap(found => found.readings), { complete: true });
});

// ---------------------------------------------------------------- reports

test('explain, dependents and check reports have the declared fields', () => {
  const explained = [];
  const rests = [];
  for (const [source, events, subject] of [
    [thermostat, [['read', { value: 17 }], ['read', { value: 25 }]], 'temperature@1'],
    [PERMISSION, [['pushed', { commit: 'abc' }], ['check'], ['approved', { commit: 'abc' }], ['merge']], 'approvals'],
    [WITHDRAWAL, [['check'], ['decide'], ['misread']], 'checks'],
    // Before the last decide sighting@1 is retired; after it, departed
    // (spec/caveat-departure-0.1.md).
    [WINDOWED, WINDOWED_EVENTS.slice(0, 4), 'sighting'],
    [WINDOWED, WINDOWED_EVENTS, 'sighting@1'],
  ]) {
    const session = real.open(source);
    const sent = events.map(([event, payload]) => {
      const { snapshot: _snapshot, ...outcome } = session.dispatch(event, payload ?? {});
      return { event, payload: payload ?? {}, outcome };
    });
    explained.push(explain(session.snapshot(), sent));
    rests.push(dependents(session.snapshot(), subject));
    session.close();
  }
  assertShape('ExplainReport', explained, { complete: true });
  const series = explained.flatMap(report => report.decisions);
  assertShape('ExplainedSeries', series, { complete: true });
  const revisions = series.flatMap(item => item.revisions);
  assertShape('ExplainedRevision', revisions, { complete: true });
  assertShape('RevisionChange', revisions.flatMap(revision => revision.history), { complete: true });
  assertShape('Withdrawal', revisions.flatMap(revision => revision.withdrawn), { complete: true });
  assertShape('PermissionRecord', revisions.flatMap(revision => revision.permission ?? []));
  const evidence = explained.flatMap(report => report.evidence);
  assertShape('ExplainedEvidence', evidence, { complete: true });
  assertShape('WithdrawalNote', evidence.flatMap(item => item.withdrawn ?? []), { complete: true });
  assertShape('ExplainedDisplay', explained.flatMap(report => report.displayed), { complete: true });
  assertShape('ExplainEvent', explained.flatMap(report => report.events), { complete: true });

  assertShape('DependentsReport', rests, { complete: true });
  assertShape('RetiredRecord', rests.flatMap(item => item.retired ?? []), { complete: true });
  assertShape('DependentDecision', rests.flatMap(item => item.decisions), { complete: true });
  assertShape('DependentChange', rests.flatMap(item => item.changes), { complete: true });
  assertShape('DependentValue', rests.flatMap(item => item.values), { complete: true });
  assertShape('DependentDisplay', rests.flatMap(item => item.displayed), { complete: true });

  const allowed = FROST.replace('on drone_read when celsius <= 2', '# caveat check: allow C001\non drone_read when celsius <= 2');
  const checked = [real.check(thermostat), real.check(trail), real.check(FROST), real.check(allowed)];
  assertShape('CheckReport', checked, { complete: true });
  const warnings = checked.flatMap(report => report.diagnostics);
  const suppressed = checked.flatMap(report => report.suppressed);
  assert.ok(suppressed.length > 0, 'an allow comment silenced a warning');
  assertShape('CheckDiagnostic', [...warnings, ...suppressed], { complete: true });
  assert.ok(warnings.every(warning => warning.severity === 'warning'));
  assertShape('CheckRelated', warnings.flatMap(warning => warning.related), { complete: true });
});

test('a server, its lines and its responses have the declared fields', () => {
  const server = createServer({ runtime: real, source: thermostat, program: 'thermostat_history.cav' });
  assertShape('CaveatServer', [server], { complete: true });
  assertShape('ServeReady', [server.ready], { complete: true });
  const lines = [
    { id: 1, op: 'dispatch', event: 'read', payload: { value: 17 }, snapshot: true },
    { id: 2, op: 'dispatch', event: 'read', payload: { value: 41 } },
    { id: 3, op: 'snapshot' }, { id: 4, op: 'explain' }, { id: 5, op: 'dependents', of: 'temperature@1' },
    { id: 6, op: 'save' }, { id: 7, op: 'dispatch', event: 'read' }, { id: 8, op: 'unknown' },
  ];
  const results = lines.map(line => server.handle(JSON.stringify(line)));
  results.push(server.handle(JSON.stringify({ id: 9, op: 'restore', save: results[5].response.save })));
  results.push(server.handle(JSON.stringify({ id: 10, op: 'close' })));
  const fatal = createServer({ runtime: real, source: 'state output = 0; event run; on run set output = 1;' + provenanceOverflow('on run') });
  results.push(fatal.handle('{"op":"dispatch","event":"run"}'));
  assertShape('ServeResult', results, { complete: true });
  assert.deepEqual(sorted(results.map(result => String(result.exit))), ['0', '1', 'null']);
  assertShape('ServeResponse', results.map(result => result.response), { complete: true });
  assertShape('ServeError', results.flatMap(result => result.response.error ?? []), { complete: true });
});

test('scenario files, reports and failures have the declared fields', async () => {
  // Each list of fields the validator allows is one declared step or part.
  const library = await readFile(path.join(kit, 'lib', 'scenarios.mjs'), 'utf8');
  const parts = { origin: ['ExpectedRejection'], send: ['SendStep'], expect: ['ExpectStep'], same_as: ['SameAsStep'],
    checkpoint: ['CheckpointStep'], resume: ['ResumeStep'], size: ['SizeStep'], save: ['SizeBounds'],
    max: ['SizeMax', 'SizeGrowth'], schema: ['ScenarioFile'], id: ['Scenario'] };
  const lists = [...library.matchAll(/fields\([\w.]+, \[([^\]]*)\]/g)].map(match => [...match[1].matchAll(/'(\w+)'/g)].map(item => item[1]));
  assert.equal(lists.length, Object.keys(parts).length);
  for (const list of lists) {
    assert.ok(parts[list[0]], `no declaration for the fields ${list.join(', ')}`);
    assert.deepEqual(sorted(parts[list[0]].flatMap(name => [...members(name)])), sorted(list), parts[list[0]].join(' and '));
  }
  assert.deepEqual(literals('ScenarioStep'), [], 'steps are interfaces');
  assert.deepEqual(sorted(allAliases.get('ScenarioStep').split('|').map(item => item.trim())),
    sorted(['send', 'expect', 'same_as', 'checkpoint', 'resume', 'size'].map(first => parts[first][0])));

  const doc = parseScenarioFile(await readFile(path.join(repo, 'examples', 'thermostat_history.scenarios.json'), 'utf8'));
  const failing = {
    ...scenarioFile('thermostat.cav', [{ send: 'read', payload: { value: 17 }, repeat: 2 }, { send: 'read', payload: { value: 41 } }]).scenarios[0], id: 'F1',
  };
  const extra = [
    failing,
    { id: 'F2', title: 'expect', steps: [{ expect: { '/bindings/heating/text': '50%' } }] },
    { id: 'F3', title: 'same as', steps: [{ send: 'read', payload: { value: 17 } }, { same_as: 'before', paths: ['/sequence'] }] },
    { id: 'F4', title: 'size', steps: [{ size: { save: { max: 1 } } }] },
    { id: 'F5', title: 'missing source', source: 'nowhere.cav', steps: [{ checkpoint: 'here' }] },
  ];
  const files = [
    await runScenarioFile(doc, { runtime: real, readSource: sourceReader({ 'thermostat_history.cav': thermostat }), file: 'a.json' }),
    await runScenarioFile({ ...doc, source: 'thermostat.cav', scenarios: extra }, { runtime: real, readSource: sourceReader({ 'thermostat.cav': thermostat }) }),
  ];
  const { runtime } = faulty({ dispatch: (fake, event, payload) => (fake.inner.dispatch_outcome(event, payload), '{"schema":"caveat-dispatch/0.1","outcome":"rejected","origin":"policy","code":"reject","message":"no"}') });
  files.push(await runScenarioFile(scenarioFile('t.cav', [{ send: 'read', payload: { value: 17 }, rejected: true }]), { runtime, readSource: sourceReader({ 't.cav': thermostat }) }));
  assertShape('ScenarioFileReport', files, { complete: true });
  const results = files.flatMap(file => file.scenarios);
  assertShape('ScenarioResult', results, { complete: true });
  const failures = results.flatMap(result => result.failure ?? []);
  assertShape('ScenarioFailure', failures, { complete: true });
  for (const failure of failures) assert.ok(literals('FailureKind').includes(failure.kind) && literals('FailureCategory').includes(failure.category), failure.kind);
  assertShape('ScenarioRunReport', [report(files, real.identity)], { complete: true });
});
