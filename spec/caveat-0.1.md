# CAVEAT Language Specification — Draft 0.1

**Status:** experimental; semantics before syntax; deliberately incomplete.

## Core idea
CAVEAT operates on a persistent epistemic graph containing claims, evidence, caveats, contexts, commitments, actions, and observations. Contradictory positions may coexist. Computation may stop through provisional commitment while unresolved caveats remain, and later evidence may reopen that commitment.

## Six primitives
- **Claim** — proposition currently available to the program.
- **Evidence** — information bearing on claims, with provenance.
- **Caveat** — condition, exception, limitation, alternative, dependency, or uncertainty that can change applicability or consequence.
- **Context** — assumptions and scope under which positions are evaluated.
- **Commitment** — selection of a position/action while preserving unresolved caveats.
- **Reopening** — reactivation of a committed question when new information makes a retained caveat material.

## Draft semantic commitments
1. Contradiction is representable state, not automatically an exception.
2. Evidence has provenance and can itself be qualified.
3. Caveats may qualify claims, evidence, contexts, consequences, or other caveats.
4. Commitment terminates current deliberation without asserting certainty.
5. Reopening preserves the history of the earlier commitment.
6. Attention, verification, waiting, and computation have costs.
7. More caveats do not automatically mean a better result.

## Canonical test
A conforming early runtime must represent two incompatible claims, attach evidence to both, retain an unresolved caveat, commit to an action, receive new evidence, and reopen the earlier commitment without erasing history.

## Non-goals
No universal truth oracle; no requirement that uncertainty be numeric; no mandatory LLM; no final syntax; no claim to encode all human reasoning.

## Draft 0.2 questions
Formal contradiction-tolerant inference; claim identity across contexts; materiality; concurrent commitments; deterministic replay; context topology; I/O/effects; type semantics; reopening triggers.

**Rule:** Do not save CAVEAT by redefining it. If the model collapses into an existing paradigm, record that result.
