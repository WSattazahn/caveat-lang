# The wrong-delivery fixture, decided by Caveat

The shared false-completion test from
[openai/codex#36718](https://github.com/openai/codex/issues/36718): one approved
JSON artifact, one SHA-256, one declared destination, three deliveries and a
pre-frozen control. Here a [Caveat](https://github.com/WSattazahn/caveat-lang)
program makes the accept or reject decision and keeps what it rested on.

Synthetic local fixtures only. No real delivery, provider, agent or API.

## What is fixed

- `task-spec.json` holds the task, the checks, the contract (approved SHA-256
  and destination), the fixture bytes and each case's expected outcome. The
  bytes are copied unchanged from `examples/wrong_delivery.py` in
  [Grit-77/done-is-a-claim](https://github.com/Grit-77/done-is-a-claim/blob/968b2f04198cbc44f334198e454580020381f9e5/examples/wrong_delivery.py) at commit 968b2f0 (Apache-2.0), with the shared-digest reading
  agreed in the thread.
- The approved artifact is `{"version":2,"reviewed":true}\n`, SHA-256
  `690369840efdeb49c63bdef7d922221dd32a42cb92587a51bc716fb9569f6a3d`. The
  destination is `delivered/artifact.json`.
- In every case the agent reports the delivery done before anything is read
  back.

## Run it

Node 20 or later, from this folder. The program runs on the published package, not on this checkout.

```sh
npm install
node run.mjs            # the fixture, on real temporary files
node run.mjs --json     # the same, as observations
npx --no-install caveat-lang test delivery.scenarios.json
npx --no-install caveat-lang explain delivery.cav correct.events.jsonl
```

`run.mjs` makes a temporary directory per case, writes the fixtures, copies the
case's source to the declared destination, reads the destination back, hashes
it and tries to parse it. It sends four events to a fresh Caveat session:
`freeze_contract`, `agent_reports_done`, `read_back` and `judge`. Its exit code
is 0 only when every case ran and matched its pre-frozen expected outcome. That
is not the acceptance of any delivery.

## Results (caveat-lang 1.2.0, Node 22.22.0)

| Case | Digest | JSON | Verdict | Expected |
| --- | --- | --- | --- | --- |
| correct | matches | parses | accept | accept |
| stale | does not match | parses | reject | reject |
| invalid_json | does not match | does not parse | reject | reject |
| control: matching digest, invalid JSON | matches | does not parse | reject | reject |

Each verdict rests on exactly two things, `checks@1` (the read-back, with its
payload) and `contract`. The agent's own report is recorded and appears in no
verdict's grounds. `results.txt`, `observations.json`, `scenarios.txt` and
`explain-correct.txt` are the outputs of the commands above.

The control does its job. With the JSON condition removed from the program, the
control is accepted and the runner exits 1.

## What the ledger adds beyond the shared contract

These are extra cases, not part of the agreed fixture:

- **A report without a read-back.** `judge` is refused with "No read-back of
  the declared destination." No verdict exists.
- **A read-back of another path.** The correct bytes read back from
  `elsewhere/artifact.json` are refused as evidence, so they cannot ground a
  verdict.
- **Overwritten after acceptance.** The destination is accepted, then
  overwritten with the stale bytes and read back again. The changed read-back
  reopens the verdict; a new verdict rejects. `verdict@1` still rests on its
  original read-back and contract, and both revisions stay inspectable after a
  save and resume.

## Limits

- Caveat does not read files, hash bytes or parse JSON. The host (`run.mjs`)
  does, and Caveat records what the host reports. It compares the reported
  digest with the frozen one and decides. A host that lies about its read-back
  defeats it; Caveat records evidence, it does not authenticate it.
- The JSON check is Node's `JSON.parse` on strict UTF-8. Python's `json.loads`
  accepts some inputs it refuses (`NaN`, for example). The fixture bytes behave
  the same under both.
- Re-reading after acceptance happens only when the host reads again. Nothing
  here watches the destination.
- This is not a Codex integration. Capturing a real agent's delivery and its
  completion report needs host hooks.
