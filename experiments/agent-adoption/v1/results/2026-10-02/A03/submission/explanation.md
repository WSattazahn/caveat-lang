# Decision history

The installed CLI's unedited explain and dependents outputs are in receipts/ (indexed in commands.jsonl). events.jsonl sends observe 85, assess, correct, observe 92, assess.

At sequence 2, answer@1 commits value 85 with grounds exactly {evidence: ["checks@1"], caveats: ["uncertain"]}. At sequence 3, correct neutrally observes correction and withdraws checks@1 because correction; the same correction reopens answer@1. The withdrawal reason is the evidence identity correction, describing a correction of a reading. It makes no opposite claim.

At sequence 5, answer@2 commits value 92 with grounds exactly {evidence: ["checks@2"], caveats: ["uncertain"]}; it is in force and supersedes answer@1. Old/new reading occurrence IDs are checks@1 and checks@2. The template is tool, not either reading occurrence.

answer@1's numeric basis, original provenance and grounds remain unchanged, and checks@1 remains in the archive with its original 85 and uncertain provenance. The withdrawal record and current withdrawn caveat mark it as no longer stood behind without rewriting its archive. S3 checks the old grounds, basis and archive occurrence with checkpoint/same_as across correction, resume and revision.

Grounds describe what the using expression reads: checks@2 alone for answer@2. Its conservative lineage also includes checks@1 and correction from decision selection/control dependencies. Those additional influences are not new grounds. The display's explanation likewise includes older control dependencies. Neither tool nor correction has a fabricated supports/opposes relation; explain reports both as observed (no stance).

S5 verifies that late qualification leaves the old archive and committed grounds/basis frozen and does not reopen the answer. S6 verifies that an earlier clean reading remains assessable after learn_stale and restore. New samples inherit stale and uncertain and cannot be assessed.
