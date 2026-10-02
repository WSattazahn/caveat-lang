# Decision, correction, and revision

[events.jsonl](events.jsonl) supplies observe 85, assess, correct, observe 92, assess. All five events are accepted. Unedited outputs are [explain](receipts/009.stdout.txt), [dependents checks](receipts/010.stdout.txt), and [replay](receipts/011.stdout.txt).

The old reading is checks@1 (85, sequence 1). The old decision answer@1 commits at sequence 2. Its exact grounds are evidence [checks@1], caveats [uncertain], and its frozen basis value is 85.

At sequence 3, correct neutrally observes correction and withdraws checks@1 because correction. The withdrawal record is {evidence: "checks@1", because: "correction", sequence: 3, event: "correct"}. answer@1 rests on that reading, so it reopens because correction. Neither tool nor correction has a supports/opposes relation; explain reports null stance/claim for both, and integration-test.mjs checks the graph directly.

The new reading is checks@2 (92, sequence 4). answer@2 commits at sequence 5 with exact grounds evidence [checks@2], caveats [uncertain], basis value 92. It is in force; answer@1 is superseded.

Grounds are the content read by using latest(checks). Lineage includes control history: answer@2's basis provenance is evidence [checks@1, checks@2, correction], caveats [uncertain]. That larger lineage is not a claim that all three are its grounds. The dependents report correctly identifies answer@1 via checks@1 and answer@2 via checks@2.

checks@1's archived value, recorded provenance and stance remain unchanged. answer@1's original commitment_bases and commitment_grounds remain unchanged, including its uncertain caveat; they do not acquire withdrawn. Current reads of checks@1 carry withdrawn. S4 uses checkpoint/same_as to compare the archive, old basis and old grounds before and after correction and revision, across resume. S6 verifies the same frozen records after late qualification, while S7 proves that an earlier clean reading can still be assessed after learning stale.

The selected state is a temporary policy inspection value set only during assess. has_caveat(selected, stale) tests the selected occurrence, not today's tool template. A refused assess rolls back that assignment along with the complete event. Learning stale can qualify live values, but does not rewrite reading archives or committed grounds. The every-reading reopening trigger is authored on answer, not inferred from score changes.
