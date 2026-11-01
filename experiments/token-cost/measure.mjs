#!/usr/bin/env node
// Counts the tokens an agent reads and writes when it works with Caveat.
//
// Usage (from the repository root):
//   npm install --no-save --prefix /tmp/tok js-tiktoken@1.0.21 @anthropic-ai/tokenizer@0.0.4 caveat-lang@1.4.1
//   node experiments/token-cost/measure.mjs --deps /tmp/tok --runtime /tmp/tok/node_modules/caveat-lang/runtime \
//     --output experiments/token-cost/results.json
//
// Three tokenizers are used because the tokenizer of current Claude models is
// not public: OpenAI o200k_base and cl100k_base (js-tiktoken), and the legacy
// Claude tokenizer (@anthropic-ai/tokenizer). Each is a stand-in; the spread
// between them is reported, not hidden.
//
// --runtime points at a built runtime directory. The MCP server, CLI and
// library are this checkout's kit/; only the WebAssembly runtime comes from
// there. --no-mcp skips everything that needs the runtime.

import { spawn, execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, existsSync, readdirSync, rmSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

const argv = process.argv.slice(2);
const option = name => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : undefined; };
const deps = option('--deps');
const runtime = option('--runtime');
const output = option('--output');
const extraDir = option('--extra'); // optional: a directory of further *.cav files to count (not in the repository)
const skipMcp = argv.includes('--no-mcp');
if (!deps || !runtime) {
  console.error('usage: measure.mjs --deps DIR --runtime DIR [--output FILE] [--extra DIR] [--no-mcp]');
  process.exit(2);
}

const root = resolve(new URL('../..', import.meta.url).pathname);
const req = createRequire(join(resolve(deps), 'package.json'));
const { getEncoding } = req('js-tiktoken');
const legacyTokenizer = req('@anthropic-ai/tokenizer').getTokenizer(); // countTokens() rebuilds it on every call
const legacyClaude = text => legacyTokenizer.encode(text.normalize('NFKC'), 'all').length;
const o200k = getEncoding('o200k_base');
const cl100k = getEncoding('cl100k_base');

const count = text => ({
  o200k: o200k.encode(text, 'all').length,
  cl100k: cl100k.encode(text, 'all').length,
  claude_legacy: legacyClaude(text),
  bytes: Buffer.byteLength(text, 'utf8'),
});
const sum = items => items.reduce((a, b) => ({ o200k: a.o200k + b.o200k, cl100k: a.cl100k + b.cl100k, claude_legacy: a.claude_legacy + b.claude_legacy, bytes: a.bytes + b.bytes }), { o200k: 0, cl100k: 0, claude_legacy: 0, bytes: 0 });
const read = path => readFileSync(join(root, path), 'utf8');

// ---------------------------------------------------------------- reading
// What an agent may read before writing a line. The skill and the MCP tool
// list are loaded by the client; the rest are read on demand.
const readingFiles = [
  'skills/caveatist/SKILL.md',
  'kit/docs/AGENT_START.md',
  'kit/docs/AUTHORING_CARD.md',
  'kit/docs/MCP.md',
  'kit/docs/REFERENCE.md',
  'kit/docs/WORKED_EXAMPLE.md',
  'docs/AI_AUTHORING.md',
  'llms.txt',
];
const reading = Object.fromEntries(readingFiles.map(f => [f, count(read(f))]));

// ---------------------------------------------------------------- programs
// Strips # and // comments outside string literals.
function stripComments(source) {
  let out = '';
  for (const line of source.split('\n')) {
    let inString = false, cut = line.length;
    for (let i = 0; i < line.length; i++) {
      const c = line[i];
      if (c === '"' && line[i - 1] !== '\\') inString = !inString;
      if (!inString && (c === '#' || (c === '/' && line[i + 1] === '/'))) { cut = i; break; }
    }
    out += line.slice(0, cut).replace(/\s+$/, '') + '\n';
  }
  return out.replace(/\n{2,}/g, '\n');
}

// Splits comment-free source into statements at top-level semicolons and
// braces, and classifies each by its first word.
function statements(source) {
  const parts = [];
  let current = '', inString = false;
  for (const c of source) {
    if (c === '"') inString = !inString;
    current += c;
    if (!inString && (c === ';' || c === '{' || c === '}')) { parts.push(current.trim()); current = ''; }
  }
  if (current.trim()) parts.push(current.trim());
  return parts.filter(Boolean).map(text => {
    let kind = text.split(/[\s(]/)[0];
    if (text.endsWith('{') || text === '}') kind = 'block';
    if (/^\w+$/.test(kind) && /^\w+ qualifies /.test(text)) kind = 'qualifies';
    if (kind === 'when') kind = 'when (in a block)';
    return { kind, text };
  });
}

// A rule's head is everything before its guard or action: "on gust when",
// "bind hud.decision =". A head that already appeared in the program is a
// repeated head: the author wrote it again because the language has no way to
// group alternatives under one head.
const headOf = s => {
  if (s.kind === 'on') return (s.text.match(/^on\s+\w+(\s+when)?/) || [s.text])[0];
  if (s.kind === 'bind') return (s.text.match(/^bind\s+[\w.]+\s*=/) || [s.text])[0];
  return null;
};

function program(path, text) {
  const total = count(text);
  const code = stripComments(text);
  const codeCount = count(code);
  const stmts = statements(code);
  const byKind = {};
  const seenHeads = new Set();
  const repeatedHeads = [];
  for (const s of stmts) {
    const c = count(s.text + '\n');
    byKind[s.kind] = sum([byKind[s.kind] ?? { o200k: 0, cl100k: 0, claude_legacy: 0, bytes: 0 }, c]);
    const head = headOf(s);
    if (head) {
      if (seenHeads.has(head)) repeatedHeads.push(count(head + ' '));
      seenHeads.add(head);
    }
  }
  const strings = [...code.matchAll(/"(?:[^"\\]|\\.)*"/g)].map(m => count(m[0] + ' '));
  return {
    path, lines: text.split('\n').length, total, comments: { o200k: total.o200k - codeCount.o200k, bytes: total.bytes - codeCount.bytes },
    code: codeCount, statements: stmts.length, byKind,
    stringLiterals: sum(strings), repeatedHeads: { count: repeatedHeads.length, ...sum(repeatedHeads) },
  };
}

const walk = (dir, test) => existsSync(dir) ? readdirSync(dir, { withFileTypes: true }).flatMap(d => d.isDirectory() ? walk(join(dir, d.name), test) : test(d.name) ? [join(dir, d.name)] : []) : [];
const programGroups = {
  examples: walk(join(root, 'examples'), n => n.endsWith('.cav')).filter(p => !p.includes('/modules/') && !p.includes('lean-bridge')),
  kit_examples: walk(join(root, 'kit/examples'), n => n.endsWith('.cav')),
  trial_v3: walk(join(root, 'experiments/agent-authoring/v3/runs'), n => n.endsWith('.cav') && !n.includes('.original.')).filter(p => !p.includes('/mutants/')),
  trial_v4: walk(join(root, 'experiments/agent-authoring/v4/runs'), n => n.endsWith('.cav')),
};
if (extraDir) programGroups.extra = walk(resolve(extraDir), n => n.endsWith('.cav'));
const programs = Object.fromEntries(Object.entries(programGroups).map(([group, paths]) => [group, paths.sort().map(p => program(extraDir && group === 'extra' ? relative(resolve(extraDir), p) : relative(root, p), readFileSync(p, 'utf8')))]));

// Scenario files the trial agents wrote alongside their programs.
const scenarioFiles = ['v3', 'v4'].flatMap(v => walk(join(root, `experiments/agent-authoring/${v}/runs`), n => n.endsWith('.scenarios.json')).filter(p => !p.includes('/mutants/')));
const scenarios = scenarioFiles.sort().map(p => ({ path: relative(root, p), ...count(readFileSync(p, 'utf8')) }));

// ---------------------------------------------------------------- MCP
function mcpSession() {
  const child = spawn(process.execPath, [join(root, 'kit/bin/caveat.mjs'), 'mcp', '--runtime', resolve(runtime)], { stdio: ['pipe', 'pipe', 'inherit'] });
  let buffer = '', id = 0;
  const waiting = new Map();
  child.stdout.on('data', chunk => {
    buffer += chunk;
    let i;
    while ((i = buffer.indexOf('\n')) >= 0) {
      const line = buffer.slice(0, i); buffer = buffer.slice(i + 1);
      if (!line.trim()) continue;
      const message = JSON.parse(line);
      waiting.get(message.id)?.(message); waiting.delete(message.id);
    }
  });
  const call = (method, params) => new Promise(done => { const k = ++id; waiting.set(k, done); child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id: k, method, params }) + '\n'); });
  return { call, notify: method => child.stdin.write(JSON.stringify({ jsonrpc: '2.0', method }) + '\n'), close: () => child.kill() };
}

const events = path => readFileSync(join(root, path), 'utf8').trim().split('\n').filter(Boolean).map(l => JSON.parse(l));
const ferryEvents = [
  { event: 'gust', payload: { kt: 20 } }, { event: 'gust', payload: { kt: 25 } },
  { event: 'wave', payload: { m: 1.0 } }, { event: 'wave', payload: { m: 1.2 } },
  { event: 'decide' }, { event: 'warning' },
];
const mcpCases = [
  { name: 'ferry (trial v4, run A)', source: 'experiments/agent-authoring/v4/runs/A/ferry.cav', events: ferryEvents, subject: 'anemometer' },
  { name: 'retractions (kit example)', source: 'kit/examples/retractions/retraction.cav', events: events('kit/examples/retractions/retraction.events.jsonl'), subject: null },
];

// The parts of a dispatch report an agent rarely needs to read, measured by
// removing each and counting what is left.
function dispatchParts(text) {
  const whole = JSON.parse(text);
  const parts = {};
  const without = (label, mutate) => { const copy = JSON.parse(text); mutate(copy); parts[label] = count(text).o200k - count(JSON.stringify(copy)).o200k; };
  without('save (passed back verbatim next call)', c => { delete c.report.save; });
  without('hashes and runtime identity', c => {
    delete c.sourceSha256; delete c.runtime; if (c.report.source) delete c.report.source.sha256;
    for (const d of c.report.decisions ?? []) for (const r of d.revisions ?? []) for (const h of r.history ?? []) delete h.source_sha256;
  });
  without('per-event outcome schema tags', c => { for (const e of c.report.events ?? []) if (e.outcome) delete e.outcome.schema; });
  without('observed_by (repeats the evidence list)', c => { for (const d of c.report.decisions ?? []) for (const r of d.revisions ?? []) delete r.observed_by; });
  without('lineage (repeats grounds when equal)', c => { for (const d of c.report.decisions ?? []) for (const r of d.revisions ?? []) if (JSON.stringify(r.lineage) === JSON.stringify(r.grounds)) delete r.lineage; });
  return { total: count(text).o200k, parts, saveBytes: Buffer.byteLength(whole.report.save ?? '', 'utf8') };
}

async function measureMcp() {
  const s = mcpSession();
  await s.call('initialize', { protocolVersion: '2025-11-25', capabilities: {}, clientInfo: { name: 'token-cost', version: '0' } });
  s.notify('notifications/initialized');
  const list = await s.call('tools/list', {});
  const tools = list.result.tools;
  const toolList = { tools: tools.length, all: count(JSON.stringify(tools)), perTool: Object.fromEntries(tools.map(t => [t.name, count(JSON.stringify(t)).o200k])) };
  const tool = async (name, args) => {
    const r = await s.call('tools/call', { name, arguments: args });
    const text = r.result.content.map(c => c.text ?? '').join('');
    return { text, structuredDuplicatesText: JSON.stringify(r.result.structuredContent) === text };
  };
  const cases = [];
  for (const c of mcpCases) {
    const source = read(c.source);
    const outputs = {};
    outputs.caveat_validate = await tool('caveat_validate', { source });
    outputs.caveat_check = await tool('caveat_check', { source });
    outputs.caveat_explain = await tool('caveat_explain', { source, events: c.events });
    if (c.subject) outputs.caveat_dependents = await tool('caveat_dependents', { source, events: c.events, subject: c.subject });
    outputs.caveat_dispatch = await tool('caveat_dispatch', { source, events: c.events });
    // A ledger kept one event per call: the save round trip at each step.
    const saveGrowth = [];
    let save;
    for (let i = 0; i < c.events.length; i++) {
      const step = await tool('caveat_dispatch', save ? { source, save, events: [c.events[i]] } : { source, events: [c.events[i]] });
      const report = JSON.parse(step.text).report;
      save = report.save;
      saveGrowth.push({ events: i + 1, reportTokens: count(step.text).o200k, saveTokens: count(JSON.stringify(save)).o200k });
    }
    cases.push({
      name: c.name, source: c.source, sourceTokens: count(source),
      eventsTokens: count(JSON.stringify(c.events)).o200k, events: c.events.length,
      outputs: Object.fromEntries(Object.entries(outputs).map(([k, v]) => [k, { ...count(v.text), structuredDuplicatesText: v.structuredDuplicatesText }])),
      dispatchParts: dispatchParts(outputs.caveat_dispatch.text),
      saveGrowth,
    });
  }
  s.close();
  return { toolList, cases };
}

// ---------------------------------------------------------------- saves
// The save caveat_dispatch returns today (session.save()) against the compact
// saves the library already offers (session.save({ compact: true }) and, with
// positional: true, the replay save 0.2), each as the JSON string an agent
// passes back.
async function measureSaves() {
  const { loadRuntimeFromDirectory } = await import(join(root, 'kit/lib/node.mjs'));
  const rt = await loadRuntimeFromDirectory(resolve(runtime));
  return mcpCases.map(c => {
    const session = rt.open(read(c.source));
    for (const e of c.events) session.dispatch(e.event, e.payload ?? {});
    const tokens = save => count(JSON.stringify(save)).o200k;
    return {
      name: c.name, events: c.events.length, eventsTokens: count(JSON.stringify(c.events)).o200k,
      fullSave: tokens(session.save()), compactSave: tokens(session.save({ compact: true })),
      positionalSave: tokens(session.save({ compact: true, positional: true })),
    };
  });
}

// ---------------------------------------------------------------- explain
// The CLI's explain text, which the agent tests' readers ran. Each revision
// lists every grounding observation again, so the text grows with grounds
// times revisions. growth.cav is a fixed probe: G readings, then K decisions
// with a reopening between each.
const explainText = (program, eventsList) => {
  const file = join(root, 'experiments/token-cost/.events.tmp.jsonl');
  writeFileSync(file, eventsList.map(e => JSON.stringify(e)).join('\n') + '\n');
  try {
    return execFileSync(process.execPath, [join(root, 'kit/bin/caveat.mjs'), 'explain', program, file, '--runtime', resolve(runtime)], { encoding: 'utf8', maxBuffer: 1 << 28 });
  } finally { rmSync(file, { force: true }); }
};
const withoutRepeatedObservations = text => text.split('\n').filter(l => !/^\s+\S+: observed at #/.test(l)).join('\n');
function measureExplain() {
  const growth = [];
  for (const grounds of [4, 16, 60]) for (const revisions of [1, 4, 16]) {
    const list = [];
    for (let i = 0; i < grounds; i++) list.push({ event: 'read', payload: { v: (i * 7) % 100 } });
    for (let k = 0; k < revisions; k++) { list.push({ event: 'decide' }); if (k < revisions - 1) list.push({ event: 'recheck' }); }
    const text = explainText(join(root, 'experiments/token-cost/growth.cav'), list);
    growth.push({ grounds, revisions, events: list.length, explain: count(text).o200k, withoutRepeatedObservations: count(withoutRepeatedObservations(text)).o200k });
  }
  // The ledger grows with every merge; its first 242 events keep this number fixed.
  const ledgerLines = read('ledger/events.jsonl').split('\n').slice(0, 242);
  const ledgerEvents = ledgerLines.filter(l => l.trim()).map(l => JSON.parse(l));
  const ledgerText = explainText(join(root, 'ledger/project.cav'), ledgerEvents);
  const ferryText = explainText(join(root, mcpCases[0].source), mcpCases[0].events);
  return {
    growth,
    ledger: { events: ledgerEvents.length, program: count(read('ledger/project.cav')).o200k, eventsFile: count(ledgerLines.join('\n') + '\n').o200k, explain: count(ledgerText).o200k, withoutRepeatedObservations: count(withoutRepeatedObservations(ledgerText)).o200k },
    ferryText: count(ferryText).o200k,
  };
}

const mcp = skipMcp ? null : await measureMcp();
const saves = skipMcp ? null : await measureSaves();
const explain = skipMcp ? null : measureExplain();
const result = { schema: 'caveat-token-cost/0.1', tokenizers: { o200k: 'js-tiktoken o200k_base', cl100k: 'js-tiktoken cl100k_base', claude_legacy: '@anthropic-ai/tokenizer (pre-Claude-3)' }, reading, mcp, saves, explain, programs, scenarios };
const text = JSON.stringify(result, null, 2) + '\n';
if (output) writeFileSync(output, text); else process.stdout.write(text);
