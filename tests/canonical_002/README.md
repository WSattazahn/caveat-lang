# Canonical 002 — Material caveats

Purpose: prove that CAVEAT can preserve multiple caveats while directing finite examination toward the caveat whose consequence can change the present commitment.

Scenario:

- Claim: `bridge_safe`
- Evidence: `static_load_test` supports the claim.
- Caveat: `paint_color_variation`, consequence negligible.
- Caveat: `untested_crosswind`, consequence catastrophic.
- The runtime must preserve both caveats.
- The test must be able to mark/examine the consequential caveat without deleting the other one.
- Commitment may retain unresolved non-blocking caveats.

This test intentionally does **not** define a universal decision-theory formula. Draft 0.1 is testing representation and lifecycle semantics first.
