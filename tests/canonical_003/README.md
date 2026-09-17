# Canonical 003 — The Eternal Caveat

Purpose: test the distinctively Caveatist stopping operation.

A claim may have an unresolved qualification chain whose logical extension is not declared complete. The runtime must permit an explicit commitment while preserving that chain unchanged.

Scenario:

- Claim: `release_ready`.
- `edge_case_1` qualifies the claim.
- `edge_case_2` qualifies `edge_case_1`.
- `edge_case_3` qualifies `edge_case_2`.
- Qualification depth is three before commitment.
- `ship_v1` commits with stop reason `Enough` and retains all three caveats.
- Qualification depth remains three after commitment.
- No caveat is marked solved merely because execution stopped.

This is the first test of CAVEAT's proposed distinction between **stopping computation** and **completing inference**.
