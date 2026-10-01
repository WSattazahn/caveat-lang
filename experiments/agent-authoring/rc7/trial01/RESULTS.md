# Fresh integration trial 01

This is a small authored policy for an assistant deciding whether supplied observations warrant answering **one user task**. The first candidate passed its five scenarios and seven real-server host tests. No candidate source, host, or scenario correction was needed. This does not establish broad reliability.

## Inputs and lifecycle chosen from ordinary documentation

Read the repository `AGENTS.md`, `docs/AI_AUTHORING.md`, `kit/docs/REFERENCE.md`, and `kit/examples/agent-evidence/README.md`, plus the ordinary linked worked example, scenario, state-caveat, and history guidance. Read the shipped `assessment.cav` and official `caller.py`; no other agent conversations, rc.7 evaluation reports, or Muse/Dot proposals informed the candidate.

- One `Task` object owns one server/session and one user-supplied task ID. A different task opens a fresh session. A saved envelope includes its task ID and source SHA-256; the host refuses restoration into a different task or source. These fields are host checks, not authentication.
- Each attempt requires an accepted observation, an accepted consultation event, and an accepted assessment. The official `Attempt` implementation prevents an old approval from covering a rejected required operation. The custom `permits` check additionally requires the current decision's grounds to be exactly the latest observation and the consulted reference, and to exclude `withdrawn`.
- The program uses `decisions assessment ... reopened by observations`, the documented alternate policy. Every new sample reopens the previous revision, including another supporting observation. The latest observation supersedes earlier observations for the current numeric policy; history remains inspectable.
- The host reports an observation's synthetic confidence from 0 to 100. An effective score of at least 70 is required. Confidence is supplied evidence, not inferred correctness or a calibrated probability.
- The reference is observed once per task; later consultation events are idempotent. Approval explicitly uses both the observation and reference via `qualified(...)`. The reference is evidence in the decision's grounds, not just a control dependency.
- Correction withdraws the latest observation with the observed `recheck` reason and replaces it. The old decision's grounds remain frozen. This example exposes no API for selecting an older observation for correction.
- Qualifying the consulted reference as `outdated` updates the live basis and explicitly reopens the current decision using `caveated(assessment_basis, outdated)`. A reassessment applies a policy-chosen 10-point penalty. The caveat remains attached to later decisions; it is not erased by another observation.
- Save/restore continues the same task and source. It does not make an earlier assessment a successful new attempt. A later retry receives its own immutable attempt result.

The threshold and penalty are example policy choices, not facts guaranteed by Caveat. The only intended result is permission under this authored policy to use the proposed answer; no external action is taken.

## Preservation and commands

Before validation or test execution, source, host, scenarios, and the unmodified official client were written to `first/` and hashed in `FIRST_MANIFEST.json`. The files remain byte-identical to those hashes. `final/` is an identical copy, with `FINAL_MANIFEST.json`; no repairs or policy changes separate first and final. Test-generated JSON receipts live separately under `first/receipts/`.

Observed environment: Windows, Node v24.11.1, Python 3.14.0; checkout HEAD `3eab7b37868b24586d739b6dc76fc682902bbe95`; kit package version `0.1.0-rc.7`. The parent was rebuilding diagnostic artifacts independently. This trial did not build or package anything, and did not record a first-run WASM hash; HEAD alone must not be read as an assertion of an unmodified runtime artifact. The independent rerun subsequently passed validation, strict checking, all five scenarios and all seven host tests against reactive WASM SHA-256 `c110650b0059ca7022f93197a42d195f1a138b1283ddbd6afb5ffba8d84e78ff`. Its [verification summary](../evidence/verification-summary.json) and four `fresh-*.txt` receipts preserve the results; that build included the combined uncommitted diagnostic changes.

Working directory for all commands below: `C:/Dev/caveat-lang`. Python used `CAVEAT_COMMAND=["node","C:/Dev/caveat-lang/kit/bin/caveat.mjs"]`.

| Command | Observed result | Receipt |
| --- | --- | --- |
| `node kit/bin/caveat.mjs validate experiments/agent-authoring/rc7/trial01/first/task.cav` | exit 0, source loads | `logs/first-validate.log` |
| `node kit/bin/caveat.mjs check experiments/agent-authoring/rc7/trial01/first/task.cav` | exit 0, no warnings | `logs/first-check.log` |
| `node kit/bin/caveat.mjs test experiments/agent-authoring/rc7/trial01/first/task.scenarios.json` | exit 0, 5 passed, 0 failed | `logs/first-scenarios.log` |
| `python -B experiments/agent-authoring/rc7/trial01/first/host.py` | exit 0, 7 passed | `logs/first-host.log` |

`FIRST_RUN.json` preserves the actual argument arrays and exit codes for these commands. The runner wrapper completed with exit 0; the listed subcommand exit codes, rather than the wrapper status, are the relevant results.

## Evidence observed

The scenario suite exercises 34 events, including seven expected rejections and two resume checkpoints. The runner also checks rejected-event atomicity, save/restore agreement, and grounds within lineage. The host suite checks repeated supporting observations, correction, a rejected required observation after approval, independent retry, same-task continuation, late qualification and penalty behavior, and different-task isolation.

Exact late-qualification case:

| Moment | Current decision | Ground evidence | Ground caveats | Frozen value |
| --- | --- | --- | --- | --- |
| Before qualification | `assessment@1` | `observations@1`, `reference` | none | 85 |
| After qualification | `assessment@1` reopened because `reference` | original decision unchanged | original decision unchanged | 85 |
| After reassessment | `assessment@2` | `observations@1`, `reference` | `outdated` | 75 |

The earlier grounds, basis, and first journal entry remain equal after qualification and reassessment. Receipts: `first/receipts/before-qualification.json`, `after-qualification.json`, `after-reassessment.json`, `explain.json`, and `dependents-reference.json`.

The rejected-observation case starts from an approval, submits confidence 250, receives `input/bound_exceeded`, and records operation states `rejected`, `not sent`, `not sent`. Its attempt fails even though the snapshot still displays the old approval; the save remains identical. A new attempt with confidence 92 succeeds with new grounds. The failed result remains failed.

## Misunderstandings, changes, and limits

No first-candidate misunderstanding was exposed by these tests, and no code or scenario changes were made after testing. In particular, the guide explicitly explained that the shipped opposing-only reopening policy would not permit a second supporting reassessment; the candidate chose every-observation reopening from the outset.

This is one author's small, synthetic integration and self-authored test set. It does not test adversarial save tampering, concurrent host calls, source authentication, external answer correctness, network behavior, arbitrary old-reading correction, exhaustive capacities, or the entire repository. The program has finite capacity (16 readings, 8 decision revisions). At capacity an attempt fails; the integration does not silently reset the same task or discard its history. No repository-wide build, publication, commit, push, or network request was performed by this trial.
