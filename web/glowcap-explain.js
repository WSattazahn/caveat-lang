// Turns a Caveat view of game/glowcap.cav into sentences a player can read.
// No DOM: web/glowcap.html renders these, and scripts/test-glowcap.mjs checks
// them against the real runtime.
//
// Nothing here decides *why* anything is true. Every "because" comes from the
// runtime's grounded explanations and grounds, and every caveat from the
// evidence it qualifies. This module only looks up the words.

/** Evidence sources and caveat display text, by name, from a full snapshot. */
export function glossary(snapshot) {
  const evidence = {};
  const caveats = {};
  for (const symbol of snapshot.symbols) {
    if (symbol.kind === 'evidence') evidence[symbol.name] = symbol.source ?? symbol.name;
    if (symbol.kind === 'caveat') caveats[symbol.name] = symbol.display ?? symbol.name;
  }
  return { evidence, caveats };
}

/** A provenance ({evidence, caveats}) in words, evidence in the given order. */
export function cite(provenance, words, order = provenance?.evidence ?? []) {
  return {
    because: order.map((id) => words.evidence[id] ?? id),
    caveats: (provenance?.caveats ?? []).map((id) => words.caveats[id] ?? id),
  };
}

/** One card per mushroom: what the slime believes about it, and why. */
export function mushroomCards(view, words, ids) {
  return ids.map((id) => {
    const shown = view.bindings[id];
    return {
      id,
      present: shown.present,
      label: shown.label,
      canAbsorb: shown.canAbsorb,
      canTaste: shown.canTaste,
      ...cite(view.binding_explanations[id]?.label, words),
    };
  });
}

const BELIEF_TITLES = {
  none: 'No belief yet',
  probably_safe: 'Probably safe',
  probably_unsafe: 'Probably unsafe',
  uncertain: 'Uncertain',
};

/** The slime's belief about glowing mushrooms, and what it rests on. */
export function beliefCard(view, words) {
  const belief = view.bindings.belief;
  return {
    state: belief.state,
    title: BELIEF_TITLES[belief.state] ?? belief.state,
    text: belief.text,
    note: belief.note,
    ...cite(view.binding_explanations.belief?.state, words),
  };
}

/** Every change to the trust decision, oldest first, in words. */
export function journalLines(view, words) {
  let trusted = false;
  return view.decision_journal
    .filter((entry) => entry.decision === 'trust')
    .map((entry) => {
      const text = entry.change === 'committed'
        ? (trusted ? 'Trusted glowing mushrooms again' : 'Started trusting glowing mushrooms')
        : 'Stopped trusting glowing mushrooms';
      trusted = true;
      return {
        text,
        change: entry.change,
        commitment: entry.commitment,
        sequence: entry.sequence,
        because: entry.because.map((id) => words.evidence[id] ?? id),
        caveats: entry.caveats.map((id) => words.caveats[id] ?? id),
      };
    });
}

/** Caveats that arrived late in the last event, in words. */
export function lateCaveats(view, words) {
  return view.effects
    .filter((effect) => effect.kind === 'qualify')
    .map((effect) => ({
      evidence: words.evidence[effect.evidence] ?? effect.evidence,
      caveat: words.caveats[effect.caveat] ?? effect.caveat,
    }));
}

/** A runtime rejection ("rejected: already absorbed") as a short reason. */
export function rejection(error) {
  const text = String(error?.message ?? error);
  const match = text.match(/rejected: (.*)$/);
  return match ? match[1] : text;
}
