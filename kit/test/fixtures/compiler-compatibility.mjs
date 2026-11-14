// Shared by Node and the real-browser kit checks. The module save below
// was emitted by the tagged v1.0.0 runtime (e2a13500), after one m__go.
const moduleSource = 'module m;\nstate hold = 7;\nstate v = 0;\nfn add(a,b) = a+b;\nevent go;\non go set v = v + add( hold , because );\n';
const rootSource = 'use m;\nstate because = 3;\n';
const bundle = root => `#caveat-bundle 1\n#module m ${moduleSource.length}\n${moduleSource}#module main ${root.length}\n${root}`;

export const moduleHold = {
  name: 'module hold identifier', source: bundle(rootSource), event: 'm__go', state: 'm__v', first: 10, next: 20,
  saved: '{"schema":"caveat-reactive-save/0.1","source_id":"fnv1a64:720e76e5fd6168d0:182","source_sha256":"a0b7db01fa116cea31f6e08acd23f0a37cea46f2a7c33584c930bf4edd63134f","sequence":1,"last_event":"m__go","elapsed":0.0,"states":{"m__v":{"value":10.0}},"graph":{}}',
};

const unsetModule = 'module m;\n# é\nstate unset = 2;\nstate value unset;\nstate total = 0;\nproc clear() { unset value; };\nevent go;\non go set value = unset;\non go call clear();\non go when not is_set(value) set total = total + unset;\n';
const unsetRoot = 'use m;\nstate unset = 99;\n';

export const compilerCompatibility = [
  moduleHold,
  { name: 'existing module root capture', source: bundle(`${rootSource}state hold = 11;\n`), event: 'm__go', state: 'm__v', first: 14, next: 28 },
  {
    name: 'module unset procedure effect',
    source: `#caveat-bundle 1\n#module m ${new TextEncoder().encode(unsetModule).length}\n${unsetModule}#module main ${unsetRoot.length}\n${unsetRoot}`,
    event: 'm__go', state: 'm__total', first: 2, next: 4,
  },
  {
    name: 'authored only_if wrapper',
    source: 'fn only_if(x,y) = x+y;\nfn wrapped(x) = only_if(x,1);\nstate n = 0;\nevent go;\non go set n = wrapped(n);\n',
    event: 'go', state: 'n', first: 1, next: 2,
  },
  ...['sum', 'any', 'all'].map(name => ({
    name: `authored ${name} with for identifier`,
    source: `fn ${name}(x) = x;\nstate for = 1;\nstate n = ${name}(for + 1);\nevent go;\non go set n = ${name}(for + n);\n`,
    event: 'go', state: 'n', first: 3, next: 4,
  })),
];
