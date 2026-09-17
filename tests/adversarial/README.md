# Adversarial probes — Draft 0.1

These are intended to break CAVEAT rather than demonstrate it.

1. **Qualification cycle** — `a qualifies b; b qualifies a;` must terminate graph inspection without pretending the cycle is a finite chain.
2. **Duplicate symbol** — defining the same claim twice must fail explicitly.
3. **Unknown provenance target** — relations to an undefined symbol must fail explicitly.
4. **Repeated reopening** — reopening an already-open commitment must preserve history rather than silently manufacture a new semantic event.
5. **Contradictory evidence** — support and opposition may coexist without explosion.
6. **Attention exhaustion** — Draft 0.1 currently names budget exhaustion but has no resource ledger. This is an acknowledged semantic hole and a Draft 0.2 target.

A failure is useful evidence. Do not patch a probe by redefining what it was testing.
