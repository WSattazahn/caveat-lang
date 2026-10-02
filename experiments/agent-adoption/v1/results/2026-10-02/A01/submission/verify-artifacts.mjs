import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
const manifest=JSON.parse(readFileSync('first-manifest.json'));
const hash=f=>createHash('sha256').update(readFileSync(f)).digest('hex');
assert.equal(hash('first.cav'),manifest.sourceSha256);
assert.equal(hash('first.scenarios.json'),manifest.scenariosSha256);
assert.equal(JSON.parse(readFileSync('first.scenarios.json')).source,'first.cav');
assert.equal(JSON.parse(readFileSync('tracker.scenarios.json')).source,'tracker.cav');
assert.equal(readFileSync('tracker.cav').equals(readFileSync('first.cav')),true);
for(const f of readdirSync('receipts').filter(f=>/^\d+\.json$/.test(f))) {
 const r=JSON.parse(readFileSync('receipts/'+f));
 writeFileSync('receipts/'+f.replace('.json','.stdout.txt'),r.stdout);
 writeFileSync('receipts/'+f.replace('.json','.stderr.txt'),r.stderr);
}
let result=readFileSync('RESULTS.md','utf8');
result=result.replace('Early shell discovery calls happened before the receipt helper existed; their complete raw stdout/stderr were not saved to disk.',
'Shell discovery and some document-inspection calls were made directly through the tool API; their complete raw stdout/stderr were not all saved to disk. The initial discovery calls also preceded the receipt helper.');
writeFileSync('RESULTS.md',result);
console.log('First-file hashes verified, source identical, scenario source names correct, raw streams exported.');
