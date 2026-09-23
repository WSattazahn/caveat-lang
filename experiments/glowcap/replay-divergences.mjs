// Replays every event in a differential --dump file, comparing accept/reject
// and views after each event (including each tick). Adjacent identical events
// may be stored with `repeat` to keep checked-in regression fixtures small.
// Run after npm run build; the default fixture is the five pre-fix failures.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createPolicy as createTs } from './ts/glowcap.ts';
import { createPolicy as createCaveat, ready } from './caveat5/adapter.mjs';
import { compareViews } from './compare-views.mjs';

await ready;
const file = process.argv[2] ?? new URL('./fixtures/round6-float-divergences.json', import.meta.url);
const cases = JSON.parse(await readFile(file, 'utf8'));
assert.ok(cases.length > 0, 'A regression replay must contain cases');
function attempt(policy, event) {
  try { policy.dispatch(event); return 'accepted'; } catch { return 'rejected'; }
}

let events = 0;
let resumes = 0;
for (const { sequence, history } of cases) {
  let ts = createTs();
  let caveat = createCaveat();
  let index = 0;
  try {
    for (const { repeat = 1, ...event } of history) {
      assert.ok(Number.isSafeInteger(repeat) && repeat > 0, 'Invalid repeat count');
      for (let n = 0; n < repeat; n += 1) {
        const context = `sequence ${sequence}, event ${index}: ${JSON.stringify(event)}`;
        if (event.type === 'resume') {
          ts = createTs(JSON.parse(JSON.stringify(ts.save())));
          const resumed = createCaveat(JSON.parse(JSON.stringify(caveat.save())));
          caveat.free();
          caveat = resumed;
          resumes += 1;
        } else {
          assert.equal(attempt(caveat, event), attempt(ts, event), `accept/reject: ${context}`);
          events += 1;
        }
        assert.deepEqual(compareViews(ts.view(), caveat.view()), null, `view: ${context}`);
        index += 1;
      }
    }
    console.log(`sequence ${sequence}: ${index} events/resumes agree`);
  } finally {
    caveat.free();
  }
}
console.log(`${cases.length} saved divergences: ${events} events and ${resumes} resumes agree event by event`);
