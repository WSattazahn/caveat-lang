// break.html: runs a program and its events in the browser, checks the five
// rules after every event (break-check.js), and offers a prefilled GitHub
// issue. Nothing leaves the page unless the reader opens that issue.
import { loadRuntime } from './caveat-session.js';
import { PROBE, RULES, checkEvent, checkStart, probeSource, reportBody } from './break-check.js';

const ISSUES = 'https://github.com/WSattazahn/caveat-lang/issues/new';
// GitHub refuses very long new-issue links; past this the body says to paste.
const LINK_LIMIT = 7000;

const EXAMPLES = {
  apartment: {
    program: './break-apartment.cav',
    events: ['gather', 'decide', 'review_flagged', 'landlord_answers', 'decide', 'traffic_rechecked'],
  },
  guard: {
    program: './break-guard.cav',
    events: ['observe', 'decide', 'freeze_still_on', 'tests_misread'],
  },
};

const $ = id => document.getElementById(id);
const esc = text => String(text).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);
const eventsText = names => names.map(event => JSON.stringify({ event })).join('\n');

const RUNTIME = {
  module: new URL('./pkg-reactive/caveat_runtime.js', location.href).href,
  wasm: new URL('./pkg-reactive/caveat_runtime_bg.wasm', location.href),
};

let runtime = null;
let version = 'unknown';
let lastReport = null;

async function start() {
  try {
    runtime = await loadRuntime(RUNTIME);
    const info = await fetch('./build-info.json').then(response => response.json()).catch(() => null);
    if (info?.revision) version = `main at ${info.revision.slice(0, 12)}`;
    $('version').textContent = version;
    for (const button of document.querySelectorAll('[data-example]')) {
      button.addEventListener('click', () => load(button.dataset.example));
    }
    $('run').addEventListener('click', () => run());
    $('report').addEventListener('click', report);
    $('run').disabled = false;
    await load('apartment');
  } catch (error) {
    $('status').textContent = `The runtime did not load: ${error.message ?? error}`;
  }
}

async function load(name) {
  const example = EXAMPLES[name];
  $('program').value = await (await fetch(example.program)).text();
  $('events').value = eventsText(example.events);
  run();
}

function parseEvents(text) {
  const events = [];
  text.split(/\r?\n/).forEach((line, index) => {
    if (!line.trim()) return;
    let record;
    try { record = JSON.parse(line); } catch { throw new Error(`events line ${index + 1} is not JSON`); }
    if (!record || typeof record.event !== 'string') throw new Error(`events line ${index + 1} needs an "event" name`);
    events.push({ event: record.event, payload: record.payload ?? {} });
  });
  return events;
}

// Opens the program twice: once as written, to learn its decision series,
// then with the checker's display lines added.
function open(source) {
  const plain = runtime.open(source);
  const probed = probeSource(source, plain.snapshot());
  plain.close();
  try {
    return { session: runtime.open(probed), probed: true };
  } catch {
    return { session: runtime.open(source), probed: false };
  }
}

// WebKit 26.0 can trap while starting a session, an engine fault in its
// WebAssembly tier-up (bench/carmack/README.md), not the program's. A run is
// deterministic, so a run that trapped is repeated once on a fresh instance.
function retryAfterTrap(retried) {
  if (retried || !runtime.trapped) return false;
  $('status').textContent = 'The browser\'s WebAssembly engine faulted; running again on a fresh runtime.';
  loadRuntime(RUNTIME).then(fresh => { runtime = fresh; run(true); },
    error => { $('status').textContent = `The runtime did not reload: ${error.message ?? error}`; });
  return true;
}

function run(retried = false) {
  const source = $('program').value;
  const out = $('results');
  out.innerHTML = '';
  $('report').hidden = true;
  lastReport = null;
  let events;
  try { events = parseEvents($('events').value); } catch (error) { $('status').textContent = error.message; return; }
  let opened;
  try { opened = open(source); } catch (error) {
    if (!retryAfterTrap(retried)) $('status').textContent = `The program does not load: ${error.message}`;
    return;
  }
  const { session } = opened;
  const rows = [];
  const findings = [];
  const notes = new Set();
  let before = session.snapshot();
  const first = checkStart(before);
  first.findings.forEach(item => findings.push({ ...item, at: 'start' }));
  first.notes.forEach(note => notes.add(note));
  try {
    events.forEach(({ event, payload }, index) => {
      const outcome = session.dispatch(event, payload);
      const after = session.snapshot();
      const result = checkEvent(before, after, outcome.outcome === 'accepted');
      result.findings.forEach(item => findings.push({ ...item, at: `#${index + 1} ${event}` }));
      result.notes.forEach(note => notes.add(note));
      rows.push({ index: index + 1, event, outcome, failed: new Set(result.findings.map(item => item.rule)) });
      before = after;
    });
  } catch (error) {
    rows.push({ index: rows.length + 1, event: events[rows.length]?.event ?? '', outcome: { outcome: 'fatal', message: error.message }, failed: new Set() });
  }
  if (runtime.trapped) {
    session.close();
    if (retryAfterTrap(retried)) return;
  }
  out.innerHTML = table(rows) + decisions(before) + findingsList(findings) + [...notes].map(note => `<p class="note">${esc(note)}</p>`).join('');
  if (!opened.probed) out.innerHTML += '<p class="note">R3 could not be checked: the checker could not add its display lines to this program.</p>';
  session.close();
  $('status').textContent = findings.length
    ? `${findings.length} rule ${findings.length === 1 ? 'failure' : 'failures'} found.`
    : `Every rule held on ${rows.length} ${rows.length === 1 ? 'event' : 'events'}.`;
  if (findings.length) {
    lastReport = { version, source, events: $('events').value, findings };
    $('report').hidden = false;
  }
}

function table(rows) {
  const head = `<tr><th>#</th><th>Event</th><th>Outcome</th>${RULES.map(rule => `<th title="${esc(rule.title)}">${rule.id}</th>`).join('')}</tr>`;
  const body = rows.map(row => {
    const accepted = row.outcome.outcome === 'accepted';
    const cells = RULES.map(rule => {
      const relevant = accepted ? rule.id !== 'R5' : rule.id === 'R5';
      if (row.outcome.outcome === 'fatal' || !relevant) return '<td class="na">·</td>';
      return row.failed.has(rule.id) ? '<td class="bad">✗</td>' : '<td class="ok">✓</td>';
    }).join('');
    const said = accepted ? 'accepted' : `${row.outcome.outcome}${row.outcome.message ? `: ${esc(row.outcome.message)}` : ''}`;
    return `<tr><td>${row.index}</td><td><code>${esc(row.event)}</code></td><td>${said}</td>${cells}</tr>`;
  }).join('');
  return `<div class="scroll"><table>${head}${body}</table></div>`;
}

function decisions(snapshot) {
  const withdrawn = new Set((snapshot.withdrawals ?? []).map(record => record.evidence));
  const reopened = new Map((snapshot.commitments ?? []).map(item => [item.action, item.open]));
  const lines = [];
  for (const [series, record] of Object.entries(snapshot.decision_series ?? {})) {
    for (const revision of record.revisions) {
      const grounds = snapshot.commitment_grounds?.[revision.id]?.evidence ?? [];
      const status = revision.id !== record.current ? 'superseded' : reopened.get(revision.id) ? 'reopened' : 'in force';
      const marked = grounds.map(name => (withdrawn.has(name) ? `<s>${esc(name)}</s> (withdrawn)` : esc(name)));
      const rests = snapshot.bindings?.[PROBE]?.[series];
      const restsText = revision.id === record.current && rests !== undefined ? `; rests_on_withdrawn(${esc(series)}) = ${rests === 1}` : '';
      lines.push(`<li><code>${esc(revision.id)}</code> ${status}, grounds: ${marked.join(', ') || 'none'}${restsText}</li>`);
    }
  }
  return lines.length ? `<h3>Decisions at the end</h3><ul>${lines.join('')}</ul>` : '';
}

function findingsList(findings) {
  if (!findings.length) return '';
  return `<h3>What failed</h3><ul class="findings">${findings.map(item => `<li><b>${item.rule}</b> (${esc(item.kind)}) at ${esc(item.at)}: ${esc(item.detail)}</li>`).join('')}</ul>`;
}

function report() {
  if (!lastReport) return;
  const title = `[Break report] ${[...new Set(lastReport.findings.map(item => item.rule))].join(', ')}`;
  let body = reportBody(lastReport);
  let url = `${ISSUES}?labels=break-report&title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}`;
  if (url.length > LINK_LIMIT) {
    body = 'The program is too long for a link. Paste the report the page copied to your clipboard here.';
    navigator.clipboard?.writeText(reportBody(lastReport)).catch(() => {});
    url = `${ISSUES}?labels=break-report&title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}`;
  }
  window.open(url, '_blank', 'noopener');
}

start();
