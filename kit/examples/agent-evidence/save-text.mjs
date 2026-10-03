// Copy beside an installed caveat-lang package and run:
//   node save-text.mjs checkpoint.caveat-save.json
// The checkpoint contains the exact text returned by save(). Keep it with
// this exact source; JSON.parse/JSON.stringify can change numeric behavior.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';

export const source = `
state x = 0 min -9 max 9;
decisions direction limit 1;
event compute v min -9 max 9;
event decide;
on compute set x = v * 0;
on decide when atan2(x, -1) < 0 commit direction because enough using -1;
on decide when atan2(x, -1) >= 0 commit direction because enough using 1;
bind hud.angle = atan2(x, -1);
`;

export async function saveTextExample(runtime, filename) {
  const session = runtime.open(source);
  let resumed;
  try {
    // -2 * 0 produces negative zero inside Caveat. JSON event payloads do not
    // preserve a host's negative zero, so sending -0 would not test this.
    assert.equal(session.dispatch('compute', { v: -2 }).outcome, 'accepted');
    const savedText = session.save();
    await writeFile(filename, savedText, 'utf8');
    const loadedText = await readFile(filename, 'utf8');
    assert.equal(loadedText, savedText);
    resumed = runtime.restore(source, loadedText);
    assert.deepEqual(resumed.snapshot(), session.snapshot());
    const negativeZero = Object.is(resumed.snapshot().values.x, -0);
    const angle = resumed.view().bindings.hud.angle;
    assert.equal(negativeZero, true);
    assert.equal(angle, -Math.PI);
    assert.equal(resumed.dispatch('decide').outcome, 'accepted');
    const decision = resumed.snapshot().commitment_bases['direction@1'].value;
    assert.equal(decision, -1);
    return { savedText, loadedText, negativeZero, angle, decision };
  } finally {
    session.close();
    resumed?.close();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv.length !== 3) throw new Error('usage: node save-text.mjs CHECKPOINT_FILE');
  const runtime = await loadRuntimeFromDirectory();
  const { negativeZero, angle, decision } = await saveTextExample(runtime, process.argv[2]);
  console.log(JSON.stringify({ negativeZero, angle, decision }));
}
