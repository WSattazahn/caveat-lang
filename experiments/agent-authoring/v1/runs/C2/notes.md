# C2 author notes

## Scope and access

Implemented the assigned repair-queue task in candidate.cav, entirely as Caveat source. Only the frozen C2 prompt, assigned task, public packet README and runner, packet/reference files, and this C2 run directory were accessed. No runtime implementation, existing games/examples, other candidates, private verifier, web, or study results were read. No agents were contacted or spawned. No commits were made. No access-rule deviations are known.

## References read

- packet/README.md and packet/runner.mjs
- tasks/repair-queue.md and prompts/C2.md
- packet/reference/docs/AI_AUTHORING.md
- packet/reference/docs/CAVEAT_ESSENCE.md
- packet/reference/spec/caveat-reactive-0.1.md through caveat-reactive-0.7.md (the first combined tool output was truncated; 0.3 through 0.6 were subsequently read individually/in smaller batches; the visible 0.7 procedure details were not needed)
- packet/reference/spec/caveat-explanations-0.2.md
- packet/reference/spec/caveat-renewal-0.1.md
- packet/reference/spec/caveat-reject-0.1.md
- packet/reference/spec/caveat-decision-journal-0.1.md
- packet/reference/spec/caveat-text-0.1.md
- packet/reference/spec/caveat-typed-parameters-0.1.md
- packet/reference/spec/caveat-save-0.1.md
- packet/reference/spec/caveat-observation-order-0.1.md

Also listed filenames under packet/reference to locate these documents. No external links in the references were followed.

## Source design

Numeric event signatures enforce exact payload names, finite values and bounds; explicit floor comparisons reject fractional values. Admission uses reject before any mutation. Phase and investigation state gate all domain actions. Three diagnostic tokens are decremented only on accepted inspect events and never replenished.

Bearing and motor diagnoses use independent renewable evidence with capacity three. Repeated inspections renew only the selected cause before revealing the new occurrence, and set only that cause's score. Every diagnosis inherits the declared alternative_cause qualification. The failure renewable has capacity two and no declared caveats.

Choosing requires a current positive selected-cause score. Each repair commitment uses the input repair code qualified directly by that cause's current renewable occurrence, retaining repair_unverified only on the decision. This preserves the selected occurrence as the sole evidence in commitment_grounds. Runtime lineage and relies_on edges may additionally record earlier renewal/predecessor dependencies as documented, while grounds remain exact. Failure reveals opposing evidence and reopens the current repair. Success adds no observation or reopening. Next resets current diagnoses and selected code while retaining graph, decision series and tokens. HUD frozen/revision read the real decision series.

## Runtime self-checks

All execution used the public runner. One distinct source version was submitted; no source revisions were needed. Eight of twelve allowed runtime checks were used:

1. validate: initial HUD, empty history and unobserved evidence.
2. two-failures.jsonl: both causes observed, invalid phase actions, two investigations, diagnosis reset, exhausted tokens, renewed bearing and failure, terminal second failure, six save/restores.
3. repeated-success.jsonl: three identical bearing readings produce bearing_check, bearing_check@2, bearing_check@3; fourth inspect rejects; choosing freezes repair code 1 despite score 2; first-investigation success is terminal; three save/restores.
4. cross-cause.jsonl: motor zero rejects choice; new motor score 1 permits repair code 2 based solely on motor_check@2; failed repair followed by new bearing diagnosis and repair code 1; second-investigation success; three save/restores.
5. alternative-preserved.jsonl: motor remains current through two bearing inspections and a rejected zero-score bearing choice; motor repair grounds exclude bearing; success; one save/restore.
6. zero-latest.jsonl: a new zero score replaces an old positive score and prevents choice; both causes can be zero; token exhaustion rejects more inspection; one save/restore.
7. admission.jsonl: unknown event, missing/extra parameters, lower/upper bounds, fractional values, numeric strings, boolean/null/array/object values, overflowed JSON number, invalid inputs around accepted choices and investigation reset; three save/restores.
8. motor-renewals.jsonl: same cause across investigations, three motor occurrences, second choice grounded only on motor_check@3, failure@2 reopening, no third investigation; two save/restores.

The seven replays contain 44 accepted events, 65 rejected events, and 19 successful resume checks. Runner resume checks compare complete restored and original snapshots. I additionally compared complete serialized snapshots for every rejection against its preceding snapshot: all 65 were unchanged. Every logged snapshot has exactly the eight required HUD properties and elapsed time zero. Inspected HUD transitions, renewable names/capacities, observed relation order, commitment grounds, and decision journals, including frozen values on reopening. All eight runner records have status completed.

## Uncertainties and limitations

No private verifier feedback was available or consulted. These are authored histories, not an exhaustive proof over every possible input sequence. The public runner parses and serializes JSON before dispatch, so the 1e400 test reaches the runtime as null and demonstrates rejection but does not independently exercise a raw nonfinite native dispatch. Nonnumeric/nonfinite admission also relies on the documented numeric event contract. No known behavioral limitation was found in the candidate.

The final source is the first submitted version, SHA-256 4ccdfdb96107c5b72d65463cdd04ae5aeb7be50c730e4ee34275cffa692fe1b3. The next action is the runner finish command, after which no run files will be edited.
