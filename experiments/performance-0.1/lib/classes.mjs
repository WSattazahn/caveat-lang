// Event classes, fixed before any timing run and derived from what each
// event did, never from how long it took. classify.mjs replays every
// workload once, untimed, and records for each event what the runtime
// reported (its dispatch outcome and effects) and which parts of the session's
// save it changed; the rules below turn that record into one class.
//
// The workloads themselves are fixed by inputs/workloads.json. A class that
// is rare in a workload is reported with its count and marked unreliable; no
// workload is added or rebalanced to fill it.
//
// Every effect kind the runtime reports (reactive.rs EffectReport: sample,
// reveal, examine, commit, reopen, qualify, renew, withdraw) lands in exactly
// one rule. The rules are tried in order and the first that holds names the
// class, so an event with a commit and a reveal is a commit.

export const CLASS_RULES = [
  ['refused', 'the dispatch outcome is "rejected" (a reject rule, a bad payload, a bound or a limit): the transaction rolled back and the session is unchanged'],
  ['commit+reopen', 'accepted; the effects include both a commit and a reopen'],
  ['commit', 'accepted; the effects include a commit (a decision made or re-made)'],
  ['reopen', 'accepted; the effects include a reopen (a decision withdrawn for reconsideration)'],
  ['evidence', 'accepted; the effects include a reveal, sample, withdraw or examine (an observation entering or leaving the evidence graph), and no commit or reopen'],
  ['qualify', 'accepted; none of the above, and the effects include a qualify (a caveat learned late, applied to values already derived from the evidence)'],
  ['renew', 'accepted; none of the above, and the effects include a renew (renewable evidence given a new, unobserved occurrence)'],
  ['state-changing', 'accepted; no effects, but the save differs from the save before the event outside sequence, last_event, elapsed, effects and the workload\'s clock states (a decay, a counter, a scheduled change)'],
  ['idle', 'accepted; no effects, and the save differs only in sequence, last_event, elapsed, effects and the workload\'s clock states'],
];

export const CLASS_ORDER = CLASS_RULES.map(([name]) => name);

// A clock state is a program state that only counts time: every tick adds
// dt to it. Changing it alone does not make a tick state-changing.
export const CLOCK_STATES = {
  'glowcap-replay': ['now'],
  'glowcap-resume': ['now'],
  'glowcap-unbound': ['now'],
  'glowcap-scaled-16': ['now'],
  'glowcap-scaled-64': ['now'],
  'ledger-session': [],
  'trail-rescue-scenarios': ['now'],
};

// Save fields that change on every accepted event whatever it did.
const VOLATILE = ['sequence', 'last_event', 'elapsed', 'effects'];

const EVIDENCE_EFFECTS = new Set(['reveal', 'sample', 'withdraw', 'examine']);
export const KNOWN_EFFECTS = new Set([...EVIDENCE_EFFECTS, 'commit', 'reopen', 'qualify', 'renew']);

// The parts of a save that differ between two saves, ignoring the volatile
// fields and the clock states. Saves are compared as parsed JSON, key by key.
export function changedParts(before, after, clockStates) {
  const changed = [];
  const keys = new Set([...Object.keys(before), ...Object.keys(after)]);
  for (const key of keys) {
    if (VOLATILE.includes(key)) continue;
    if (key === 'states') {
      const names = new Set([...Object.keys(before.states ?? {}), ...Object.keys(after.states ?? {})]);
      for (const name of names) {
        if (clockStates.includes(name)) continue;
        if (JSON.stringify(before.states?.[name]) !== JSON.stringify(after.states?.[name])) changed.push(`states.${name}`);
      }
      continue;
    }
    if (JSON.stringify(before[key]) !== JSON.stringify(after[key])) changed.push(key);
  }
  return changed;
}

// record: { outcome: 'accepted'|'rejected', effects: [kind...], changed: [part...] }
export function classify(record) {
  if (record.outcome !== 'accepted') return 'refused';
  const kinds = new Set(record.effects);
  const unknown = [...kinds].filter((kind) => !KNOWN_EFFECTS.has(kind));
  if (unknown.length) throw new Error(`unknown effect kind(s) ${unknown.join(', ')}: add a rule to lib/classes.mjs`);
  if (kinds.has('commit') && kinds.has('reopen')) return 'commit+reopen';
  if (kinds.has('commit')) return 'commit';
  if (kinds.has('reopen')) return 'reopen';
  if ([...kinds].some((kind) => EVIDENCE_EFFECTS.has(kind))) return 'evidence';
  if (kinds.has('qualify')) return 'qualify';
  if (kinds.has('renew')) return 'renew';
  return record.changed.length ? 'state-changing' : 'idle';
}

// A finer, descriptive key for tables: the event, its class and the sorted
// set of effect kinds (and a refusal's code), e.g. "advance reopen [qualify,reopen]".
export function signature(event, eventClass, effects, code) {
  const kinds = [...new Set(effects)].sort();
  return `${event} ${eventClass} [${kinds.join(',')}]${code ? ` ${code}` : ''}`;
}

// The owner's reporting categories for a Glowcap stream, from the class and
// the event name: ticks split into idle and state-changing, and every other
// event is an observation, labelled with its class.
export function glowcapCategory(eventName, eventClass) {
  if (eventName === 'tick') return eventClass === 'idle' ? 'idle tick' : 'state-changing tick';
  return `observation (${eventClass})`;
}
