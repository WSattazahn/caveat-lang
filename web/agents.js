// agents.html: three named agents (security, perf, comms) share one Caveat
// release ledger in the browser using Agents 0.1, Withdrawal 0.1, live
// `explain`, and live `dependents`. Nothing leaves the browser.
import { loadRuntime } from './caveat-session.js';
import { dependents, explain, formatDependents, formatExplanation } from './caveat-explain.js';

const RUNTIME = {
  module: new URL('./pkg-reactive/caveat_runtime.js', location.href).href,
  wasm: new URL('./pkg-reactive/caveat_runtime_bg.wasm', location.href),
};

const SERIES = ['sec_signoff', 'canary_deploy', 'release_notes'];
const DECIDE_STEPS = [
  { agent: 'security', event: 'sec_check', payload: { clean: 1, cached: 1 } },
  { agent: 'security', event: 'approve_security', payload: {} },
  { agent: 'perf', event: 'perf_check', payload: { p95_ms: 140 } },
  { agent: 'perf', event: 'approve_canary', payload: {} },
  { agent: 'comms', event: 'notes_check', payload: { entries: 12 } },
  { agent: 'comms', event: 'publish_notes', payload: {} },
];
const RECOMMIT_STEPS = [
  { agent: 'security', event: 'sec_check', payload: { clean: 1, cached: 0 } },
  { agent: 'security', event: 'approve_security', payload: {} },
  { agent: 'perf', event: 'approve_canary', payload: {} },
];

const $ = id => document.getElementById(id);
const esc = text => String(text).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);

let runtime = null;
let source = '';
let session = null;
let history = [];
let sentEvents = [];
let selectedSubject = 'cve_mirror';
let stopped = false;
let generation = 0;

function allButtons() {
  return [...document.querySelectorAll('main button')];
}

function halt(message) {
  stopped = true;
  if (session) {
    try { session.close(); } catch { /* runtime already trapped */ }
    session = null;
  }
  for (const button of allButtons()) button.disabled = true;
  const box = $('stopped');
  box.textContent = message;
  box.hidden = false;
  $('status').textContent = 'Stopped after runtime trap.';
}

function replayAll(targetRuntime, entries) {
  const fresh = targetRuntime.open(source);
  const recorded = [];
  try {
    for (const { agent, event, payload } of entries) {
      const raw = fresh.dispatch(event, payload, { agent });
      const { snapshot, ...outcome } = raw;
      recorded.push({
        event,
        payload,
        agent,
        outcome,
        ...(snapshot ? { sequence: snapshot.sequence, effects: snapshot.effects } : {}),
      });
    }
    return { fresh, recorded };
  } catch (error) {
    try { fresh.close(); } catch { /* abandon trapped session */ }
    throw error;
  }
}

async function recoverFromTrap(pendingEntries, attemptGeneration) {
  $('status').textContent = 'WebAssembly runtime trapped; retrying once on a fresh instance…';
  for (const button of allButtons()) button.disabled = true;
  let freshRuntime;
  try {
    freshRuntime = await loadRuntime(RUNTIME);
  } catch (error) {
    if (attemptGeneration === generation) halt(`Could not reload the WebAssembly runtime: ${error.message ?? error}`);
    return;
  }
  if (attemptGeneration !== generation) return;
  runtime = freshRuntime;
  try {
    const { fresh, recorded } = replayAll(runtime, pendingEntries);
    session = fresh;
    history = pendingEntries;
    sentEvents = recorded;
    render('Recovered on a fresh WebAssembly runtime instance.');
  } catch (error) {
    halt(`Runtime failed again after fresh-instance retry: ${error.message ?? error}`);
  }
}

function sendBatch(entries, summaryText) {
  if (stopped || !runtime || !session) return;
  const attemptGeneration = ++generation;
  const nextHistory = [...history, ...entries];
  let lastOutcome = null;
  try {
    for (const { agent, event, payload } of entries) {
      const raw = session.dispatch(event, payload, { agent });
      const { snapshot, ...outcome } = raw;
      lastOutcome = { agent, event, outcome };
      sentEvents.push({
        event,
        payload,
        agent,
        outcome,
        ...(snapshot ? { sequence: snapshot.sequence, effects: snapshot.effects } : {}),
      });
    }
    history = nextHistory;
    const message = summaryText ?? (lastOutcome.outcome.outcome === 'accepted'
      ? `Accepted ${lastOutcome.event} by ${lastOutcome.agent}.`
      : `Refused ${lastOutcome.event} by ${lastOutcome.agent} (${lastOutcome.outcome.origin}/${lastOutcome.outcome.code}): ${lastOutcome.outcome.message}`);
    render(message);
  } catch (error) {
    if (runtime.trapped) {
      try { session.close(); } catch { /* abandon trapped session */ }
      session = null;
      recoverFromTrap(nextHistory, attemptGeneration);
    } else {
      halt(`Unexpected runtime failure: ${error.message ?? error}`);
    }
  }
}

function resetSession() {
  if (stopped || !runtime) return;
  const attemptGeneration = ++generation;
  if (session) {
    try { session.close(); } catch { /* handled below */ }
    session = null;
  }
  if (runtime.trapped) {
    recoverFromTrap([], attemptGeneration);
    return;
  }
  try {
    session = runtime.open(source);
    history = [];
    sentEvents = [];
    render('Ledger reset. Click step 1 to let all three agents contribute claims and decide.');
  } catch (error) {
    if (runtime.trapped) recoverFromTrap([], attemptGeneration);
    else halt(`Could not reopen session: ${error.message ?? error}`);
  }
}

function render(statusMessage) {
  if (stopped || !session) return;
  let snapshot;
  try {
    snapshot = session.snapshot();
  } catch (error) {
    if (runtime.trapped) {
      try { session.close(); } catch { /* ignore */ }
      session = null;
      recoverFromTrap([...history], ++generation);
    } else {
      halt(`Snapshot failed: ${error.message ?? error}`);
    }
    return;
  }

  const explainReport = explain(snapshot, sentEvents);
  const depReport = dependents(snapshot, selectedSubject);
  const withdrawnSet = new Set((snapshot.withdrawals ?? []).map(item => item.evidence));
  const seriesByName = new Map(explainReport.decisions.map(series => [series.name, series]));

  for (const name of SERIES) {
    const card = document.querySelector(`[data-series="${name}"]`);
    const statusText = snapshot.bindings?.status?.[name] ?? 'not decided';
    const badge = card.querySelector('[data-role="status"]');
    badge.textContent = statusText;
    badge.className = 'badge' + (statusText === 'in force' ? ' in-force' : statusText === 'reopened' ? ' reopened' : '');

    const series = seriesByName.get(name);
    const current = series?.revisions?.at(-1) ?? null;
    card.querySelector('[data-role="revision"]').textContent = current
      ? `${current.id}${current.value !== null ? ` = ${current.value}` : ''} (${series.revisions.length} of ${series.limit})`
      : 'none';

    const groundsEl = card.querySelector('[data-role="grounds"]');
    if (!current || !current.grounds.evidence.length) {
      groundsEl.textContent = 'none';
    } else {
      const items = current.grounds.evidence.map(ev =>
        withdrawnSet.has(ev)
          ? `<span class="withdrawn"><code>${esc(ev)}</code></span> (withdrawn)`
          : `<code>${esc(ev)}</code>`
      );
      const caveats = current.grounds.caveats.map(c => `<span class="caveat-pill">${esc(c)}</span>`).join('');
      groundsEl.innerHTML = items.join(', ') + caveats;
    }

    const last = current?.history?.at(-1) ?? null;
    card.querySelector('[data-role="history"]').textContent = last
      ? `#${last.sequence} ${last.event}${last.agent ? ` by ${last.agent}` : ''}: ${last.change} because ${last.because.join(', ')}`
      : 'no decision yet';
  }

  $('dependents-out').textContent = formatDependents(depReport, 'agents.cav', sentEvents.length);
  $('explain-out').textContent = formatExplanation(explainReport, 'agents.cav');
  $('status').textContent = statusMessage;

  for (const button of allButtons()) button.disabled = false;
  for (const tab of document.querySelectorAll('[data-subject]')) {
    tab.classList.toggle('active', tab.dataset.subject === selectedSubject);
  }
}

async function start() {
  try {
    const [loadedRuntime, programText, info] = await Promise.all([
      loadRuntime(RUNTIME),
      fetch('./agents.cav').then(response => {
        if (!response.ok) throw new Error(`agents.cav HTTP ${response.status}`);
        return response.text();
      }),
      fetch('./build-info.json').then(response => response.json()).catch(() => null),
    ]);
    runtime = loadedRuntime;
    source = programText;
    $('source-out').textContent = source;
    $('version').textContent = info?.revision ? `main at ${info.revision.slice(0, 12)}` : 'local build';

    session = runtime.open(source);

    $('step-decide').addEventListener('click', () => {
      sendBatch(DECIDE_STEPS, 'All three agents checked their sources and committed their decisions.');
    });
    $('step-wrong-agent').addEventListener('click', () => {
      sendBatch(
        [{ agent: 'comms', event: 'approve_security', payload: {} }],
        'Refused approve_security by comms (input/wrong_agent): comms cannot send security events. Nothing changed.'
      );
    });
    $('step-withdraw').addEventListener('click', () => {
      sendBatch(
        [{ agent: 'security', event: 'retract_advisory', payload: {} }],
        'Security withdrew advisory@1. sec_signoff and canary_deploy reopened; release_notes stayed in force.'
      );
    });
    $('step-recommit').addEventListener('click', () => {
      sendBatch(RECOMMIT_STEPS, 'Security rechecked the live mirror (advisory@2); sec_signoff@2 and canary_deploy@2 are now in force.');
    });
    $('step-reset').addEventListener('click', resetSession);

    for (const button of document.querySelectorAll('#manual-controls button')) {
      button.addEventListener('click', () => {
        sendBatch([{
          agent: button.dataset.agent,
          event: button.dataset.event,
          payload: JSON.parse(button.dataset.payload),
        }]);
      });
    }
    for (const tab of document.querySelectorAll('[data-subject]')) {
      tab.addEventListener('click', () => {
        selectedSubject = tab.dataset.subject;
        render($('status').textContent);
      });
    }

    render('Ready. Click "1. All three agents decide" to run the three-agent scenario.');
    window.__agents = {
      ready: true,
      stopped: () => stopped,
      sequence: () => session?.snapshot().sequence ?? null,
    };
  } catch (error) {
    halt(`Could not start the multi-agent demo: ${error.message ?? error}`);
  }
}

start();
