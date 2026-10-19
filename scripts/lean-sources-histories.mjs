// Random event histories for the Lean/runtime Caveat Sources comparison.
// One fixed program; each host event is the model events its rules perform.
// This module generates and translates histories; it evaluates no semantics.
export const SOURCES_SCHEMA = 'caveat-sources/0.1';
export const SOURCES_STATES = Object.freeze(['v', 'g', 'w', 'r']);

export const SOURCES_PROGRAM = `claim c;
caveat flaky consequence material;
evidence e from "tool";
renewable e limit 64;
evidence f from "operator";
renewable f limit 64;
decisions trust limit 64;
state v = 0;
state g = 0;
state w = 0;
state r = 0;
event reveal_e;
event renew_e;
event renew_ef;
event reveal_f;
event renew_f;
event flag;
event flag_f;
event unflag;
event unflag_f;
event read;
event read_f;
event copy;
event decide;
event recall;
on reveal_e reveal e;
on renew_e renew e;
on renew_ef when observed(f) renew e;
on reveal_f reveal f;
on renew_f renew f;
on flag qualify e with flaky;
on flag_f qualify f with flaky;
on unflag unqualify e with flaky;
on unflag_f unqualify f with flaky;
on read set v = qualified(1, e);
on read_f set g = qualified(1, f);
on copy set w = v + g;
on decide commit trust because enough using v;
on recall when committed(trust) set r = qualified(1, e);
`;

const set = (state, body, guard = []) => ({ set: { state, body, guard } });

/** Each host event: when its rule's guard holds, and the model events it performs. */
export const HOST_EVENTS = Object.freeze({
  reveal_e: { guard: () => true, model: [{ reveal: { evidence: 'e', declared: [] } }] },
  renew_e: { guard: () => true, model: [{ renew: { evidence: 'e', declared: [], guard: [] } }] },
  renew_ef: { guard: seen => seen.f, model: [{ renew: { evidence: 'e', declared: [], guard: [{ evidence: 'f' }] } }] },
  reveal_f: { guard: () => true, model: [{ reveal: { evidence: 'f', declared: [] } }] },
  renew_f: { guard: () => true, model: [{ renew: { evidence: 'f', declared: [], guard: [] } }] },
  flag: { guard: () => true, model: [{ qualify: { evidence: 'e', caveat: 'flaky', guard: [] } }] },
  flag_f: { guard: () => true, model: [{ qualify: { evidence: 'f', caveat: 'flaky', guard: [] } }] },
  unflag: { guard: () => true, model: [{ unqualify: { evidence: 'e', caveat: 'flaky' } }] },
  unflag_f: { guard: () => true, model: [{ unqualify: { evidence: 'f', caveat: 'flaky' } }] },
  read: { guard: () => true, model: [set('v', [{ evidence: 'e' }])] },
  read_f: { guard: () => true, model: [set('g', [{ evidence: 'f' }])] },
  copy: { guard: () => true, model: [set('w', [{ state: 'v' }, { state: 'g' }])] },
  // A second commit of a decision in force is refused (`decision_in_force`),
  // which the model does not cover, so a history commits once.
  decide: { guard: seen => !seen.committed, model: [{ commit: { decision: 'trust', basis: [{ state: 'v' }] } }] },
  recall: { guard: seen => seen.committed, model: [set('r', [{ evidence: 'e' }], [{ basis: 'trust' }])] },
});

/** Track only what the guards read: whether `f`'s current occurrence is observed,
 * and whether `trust` has been committed. A refused event changes neither. */
function advance(seen, event, accepted) {
  if (!accepted) return seen;
  if (event === 'reveal_f') return { ...seen, f: true };
  if (event === 'renew_f') return { ...seen, f: false };
  if (event === 'decide') return { ...seen, committed: true };
  return seen;
}

/** A deterministic history of host events whose guards hold when they run.
 * `accepts(event, index)` reports the runtime's outcome, since a refused event
 * changes nothing a guard reads. Without it, every event counts as accepted. */
export function* history(seed, length, accepts = () => true) {
  const names = Object.keys(HOST_EVENTS);
  let state = BigInt.asUintN(64, BigInt(seed));
  let seen = { f: false, committed: false };
  for (let index = 0; index < length; index += 1) {
    let event;
    do {
      state = BigInt.asUintN(64, state * 6364136223846793005n + 1442695040888963407n);
      event = names[Number(state >> 33n) % names.length];
    } while (!HOST_EVENTS[event].guard(seen));
    const accepted = yield event;
    seen = advance(seen, event, accepted ?? accepts(event, index));
  }
}

export function modelRequest(id, events, hostEvents = HOST_EVENTS) {
  return { schema: SOURCES_SCHEMA, id, states: [...SOURCES_STATES], events: events.map(event => hostEvents[event].model) };
}

/** Translations that drop what a record contributes, in the shapes of findings.
 * The comparison must find each: a check that cannot see them proves nothing. */
export const TRANSLATION_CONTROLS = Object.freeze([
  { id: 'guard-record-dropped', finding: 'D014', event: 'renew_ef',
    model: [{ renew: { evidence: 'e', declared: [], guard: [] } }] },
  { id: 'basis-read-dropped', finding: 'D015', event: 'recall',
    model: [set('r', [{ evidence: 'e' }])] },
]);

export function controlEvents(control) {
  return { ...HOST_EVENTS, [control.event]: { ...HOST_EVENTS[control.event], model: control.model } };
}

/** The compared projection of a runtime snapshot: each state's caveats. */
export function runtimeStates(snapshot) {
  return Object.fromEntries(SOURCES_STATES.map(name => [name, {
    lineage: [...new Set(snapshot.qualified_values?.[name]?.provenance?.caveats ?? [])].sort(),
    grounds: [...new Set(snapshot.value_grounds?.[name]?.caveats ?? [])].sort(),
  }]));
}
