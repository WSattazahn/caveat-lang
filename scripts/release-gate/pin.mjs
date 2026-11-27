// The kit pin in pin.json, checked, for the workflows' shell steps:
//   eval "$(node scripts/release-gate/pin.mjs --shell)"
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const pin = JSON.parse(await readFile(new URL('./pin.json', import.meta.url), 'utf8'));
assert.match(pin.version, /^\d+\.\d+\.\d+$/, 'The gate kit pin must name a released version.');
assert.equal(pin.tarball, `https://registry.npmjs.org/caveat-lang/-/caveat-lang-${pin.version}.tgz`, 'The gate kit pin must name that version on the public registry.');
assert.match(pin.sha256, /^[0-9a-f]{64}$/, 'The gate kit pin must carry the tarball SHA256.');
if (process.argv[2] === '--shell') {
  console.log(`GATE_KIT_TARBALL='${pin.tarball}'\nGATE_KIT_SHA256='${pin.sha256}'`);
} else {
  console.log(JSON.stringify(pin));
}
