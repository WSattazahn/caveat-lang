# Tracker evidence history

Installed package: caveat-lang 0.1.0-rc.7. The exact five accepted events are in events.jsonl. Unedited explain, dependents, JSON explain, and replay outputs are in receipts (both JSON receipts and raw .stdout.txt/.stderr.txt files).

1. Sequence 1: observe 85 neutrally observes tool and records checks@1 supporting ready. No answer is committed.
2. Sequence 2: assess commits answer@1, value 85. Its exact grounds are { evidence: ["checks@1"], caveats: ["uncertain"] }. Its frozen basis is value 85 with that same provenance.
3. Sequence 3: correct neutrally observes correction and withdraws checks@1 because correction. The exact withdrawal record is { evidence: "checks@1", because: "correction", sequence: 3, event: "correct" }. The explicit authored rule reopens answer@1 because correction.
4. Sequence 4: observe 92 records checks@2 supporting ready. It does not commit.
5. Sequence 5: assess commits answer@2, value 92, previous answer@1. Its exact grounds are { evidence: ["checks@2"], caveats: ["uncertain"] }.

Grounds and lineage differ. answer@2 is based only on checks@2; its frozen basis lineage also contains checks@1 and correction because the decision/reopening conditions read earlier history. Its lineage evidence is ["checks@1", "checks@2", "correction"], with caveat uncertain. Graph relies_on edges expose lineage; they do not make every listed identity a content ground.

The checks@1 archive keeps value 85, its original supporting relation and recorded uncertain caveat. answer@1 keeps its original basis and grounds; withdrawn is not retroactively inserted into either frozen record. Later reads of checks@1 carry withdrawn. The archive is not erased, and the decision gains reopening history rather than altered original grounds. T03 compares the archive and original basis/grounds by checkpoint/same_as, before and after correction/revision; resume preserves the result.

Neither tool nor correction has a supports/opposes relation. Explain explicitly reports both as observed with no stance. Only checks@1 and checks@2 support ready.

T05/T06 verify that learning stale leaves earlier archives, committed bases/grounds, and journal unchanged, does not reopen a decision, permits an earlier clean reading, and attaches stale to subsequent readings. Every rejected event is also checked for identical save, snapshot, and view by the scenario runner.
