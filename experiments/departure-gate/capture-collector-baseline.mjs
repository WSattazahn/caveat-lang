// Reproduce the exact embedded-source old saves with a verified baseline WASM.
// Example: node experiments/departure-gate/capture-collector-baseline.mjs --runtime=PATH --out=test-results/reproduced-baseline.json
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {loadRuntimeFromDirectory} from '../../kit/lib/node.mjs';
const option=name=>process.argv.find(value=>value.startsWith(`--${name}=`))?.slice(name.length+3);
const location=option('runtime'),output=option('out');
assert(location&&output,'Require --runtime and --out');
assert(!existsSync(output),'Refuse to overwrite baseline evidence');
const accepted={
  'f5ec8294efe2be24705f234ef75e5f5459aa5e89':'daab2c71023777a3a9e5dc72f7882c653147c4acb445470368cecd76ed30bc59',
  '3a88ba0f80d563b4493840dc7bd7e195329b8302':'29edda09e93f61381049b5bb614a77d74bf22873ca37fef40da014dbe2fed577',
};
const hash=text=>createHash('sha256').update(text).digest('hex');
const runtime=await loadRuntimeFromDirectory(location);
assert(accepted[runtime.identity.revision],'Unexpected baseline source revision');
assert.equal(runtime.identity.reactiveWasmSha256,accepted[runtime.identity.revision]);
assert.equal(runtime.identity.clean,true);
// Embedded source preserves the recorded source identity across checkout line endings.
const template=JSON.parse(readFileSync(new URL('../../runtime/tests/fixtures/collector-baseline-f5ec829.json',import.meta.url),'utf8'));
const cases=[];
for(const {name,source,source_sha256,events} of template.cases) {
  assert.equal(hash(source),source_sha256);
  const session=runtime.open(source);
  try {
    for(const event of events) assert.equal(session.dispatchView(event,{}).outcome,'accepted');
    const save=session.save();
    cases.push({name,source,source_sha256,save,save_sha256:hash(save),events});
  } finally {session.close();}
}
writeFileSync(output,JSON.stringify({schema:'collector-baseline-save-fixtures/1',runtime:runtime.identity,cases},null,2)+'\n');
console.log(JSON.stringify({output,runtime:runtime.identity,cases:cases.map(({name,save_sha256})=>({name,save_sha256}))}));
