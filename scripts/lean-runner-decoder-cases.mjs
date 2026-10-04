import { SCHEMA } from './lean-conformance-cases.mjs';
import { LATE_SCHEMA } from './lean-late-qualification-cases.mjs';
// Raw negative inputs for the actual compiled Lean decoder. These are protocol
// refusals, not semantic mutants; a crash or the wrong diagnostic is a failure.
export function leanRunnerDecoderCases() {
  const valid = {
    schema: SCHEMA, id: 'decoder-control',
    seed: { a: 2, b: -2, g: 1 },
    steps: [{ actions: [{ target: 'x', body: ['a', 'b'], guard: null, citations: null }] }],
  };
  const encode = mutate => {
    const value = structuredClone(valid);
    mutate(value);
    return JSON.stringify(value);
  };
  const raw = JSON.stringify(valid);
  const conditional = () => ({ condition: 'g', then: ['a'], else: ['b'] });
  const branchInput = change => encode(v => {
    const body = conditional(); change(body); v.steps[0].actions[0].body = body;
  });
  const branchRaw = branchInput(() => {});
  const tracked = (value, evidence, caveats) => ({ value, lineage: { evidence, caveats }, grounds: { evidence, caveats } });
  const late = {
    schema: LATE_SCHEMA, id: 'late-decoder-control',
    before: {
      states: { a: tracked(3, ['ea'], ['ca']), g: tracked(1, ['eg'], ['cg']) },
      commitments: { plan: { basis: { value: 3, evidence: ['ea'], caveats: ['ca'] }, grounds: { evidence: ['ea'], caveats: ['ca'] } } },
      observations: ['ea', 'eg'],
    },
    qualification: { evidence: 'ea', caveat: 'late', qualifiers: ['meta'], guard: 'g' },
  };
  const lateInput = mutate => { const value = structuredClone(late); mutate(value); return JSON.stringify(value); };
  return [
    { id: 'missing-guard', input: encode(v => { delete v.steps[0].actions[0].guard; }), diagnostic: 'missing field: guard' },
    { id: 'missing-citations', input: encode(v => { delete v.steps[0].actions[0].citations; }), diagnostic: 'missing field: citations' },
    { id: 'unknown-field', input: encode(v => { v.seed.extra = 0; }), diagnostic: 'unknown field: extra' },
    { id: 'unknown-reference', input: encode(v => { v.steps[0].actions[0].body = ['z']; }), diagnostic: 'unknown state reference: z' },
    { id: 'wrong-schema', input: encode(v => { v.schema = 'unsupported'; }), diagnostic: 'unsupported schema' },
    { id: 'invalid-id', input: encode(v => { v.id = 'Uppercase'; }), diagnostic: 'id requires lowercase ASCII' },
    { id: 'seed-outside-domain', input: encode(v => { v.seed.a = 1001; }), diagnostic: 'seed requires a,b' },
    { id: 'empty-steps', input: encode(v => { v.steps = []; }), diagnostic: 'steps requires 1..8 entries' },
    { id: 'empty-actions', input: encode(v => { v.steps[0].actions = []; }), diagnostic: 'actions requires 1..4 entries' },
    { id: 'too-many-total-actions', input: encode(v => {
      const action = v.steps[0].actions[0];
      v.steps = Array.from({ length: 3 }, () => ({ actions: [action, action, action] }));
    }), diagnostic: 'at most eight total actions' },
    { id: 'negative-zero', input: raw.replace('"a":2', '"a":-0'), diagnostic: 'signed zero is outside' },
    { id: 'fraction-token', input: raw.replace('"a":2', '"a":2.0'), diagnostic: 'non-integer numeric token' },
    { id: 'exponent-token', input: raw.replace('"a":2', '"a":2e0'), diagnostic: 'non-integer numeric token' },
    { id: 'duplicate-root-field', input: raw.replace('"id":', '"id":"first-id","id":'), diagnostic: 'duplicate field' },
    { id: 'duplicate-seed-field', input: raw.replace('"a":2', '"a":999,"a":2'), diagnostic: 'duplicate field' },
    { id: 'duplicate-action-field', input: raw.replace('"guard":null', '"guard":"a","guard":null'), diagnostic: 'duplicate field' },
    { id: 'escaped-duplicate-field', input: raw.replace('"a":2', '"\\u0061":999,"a":2'), diagnostic: 'duplicate field' },
    { id: 'legacy-schema', input: encode(v => { v.schema = 'caveat-guard-citation/0.1'; }), diagnostic: 'unsupported schema' },
    ...['condition', 'then', 'else'].map(field => ({ id: 'missing-branch-' + field,
      input: branchInput(body => { delete body[field]; }), diagnostic: 'missing field: ' + field })),
    { id: 'unknown-branch-field', input: branchInput(body => { body.extra = true; }), diagnostic: 'unknown field: extra' },
    { id: 'invalid-condition', input: branchInput(body => { body.condition = null; }), diagnostic: 'conditional condition requires a state reference' },
    { id: 'unknown-condition', input: branchInput(body => { body.condition = 'z'; }), diagnostic: 'unknown state reference: z' },
    ...['then', 'else'].flatMap(field => [
      { id: 'nonarray-branch-' + field, input: branchInput(body => { body[field] = conditional(); }), diagnostic: 'conditional ' + field + ' requires an array' },
      { id: 'oversized-branch-' + field, input: branchInput(body => { body[field] = Array(5).fill('a'); }), diagnostic: 'conditional ' + field + ' requires 0..4 entries' },
      { id: 'nested-branch-' + field, input: branchInput(body => { body[field] = [conditional()]; }), diagnostic: 'conditional ' + field + ' requires state references' },
    ]),
    { id: 'duplicate-branch-condition', input: branchRaw.replace('"condition":"g"', '"condition":"a","condition":"g"'), diagnostic: 'duplicate field' },
    { id: 'escaped-duplicate-branch-condition', input: branchRaw.replace('"condition":"g"', '"\\u0063ondition":"a","condition":"g"'), diagnostic: 'duplicate field' },
    { id: 'oversized-stdin', input: raw + ' '.repeat(65537), diagnostic: 'request exceeds 65536 UTF-8 bytes' },
    { id: 'late-missing-qualification', input: lateInput(v => { delete v.qualification; }), diagnostic: 'missing field: qualification' },
    { id: 'late-unknown-field', input: lateInput(v => { v.before.journal = []; }), diagnostic: 'unknown field: journal' },
    { id: 'late-missing-guard', input: lateInput(v => { delete v.qualification.guard; }), diagnostic: 'missing field: guard' },
    { id: 'late-unknown-guard', input: lateInput(v => { v.qualification.guard = 'z'; }), diagnostic: 'unknown guard state: z' },
    { id: 'late-grounds-exceed-lineage', input: lateInput(v => { v.before.states.a.grounds.caveats = ['ca', 'invented']; }),
      diagnostic: 'state grounds exceed its lineage' },
    { id: 'late-invalid-name', input: lateInput(v => { v.qualification.caveat = 'Late'; }), diagnostic: 'caveat requires a lowercase identifier' },
    { id: 'late-missing-basis-value', input: lateInput(v => { delete v.before.commitments.plan.basis.value; }), diagnostic: 'missing field: value' },
    { id: 'late-value-outside-domain', input: lateInput(v => { v.before.states.a.value = 1000000001; }), diagnostic: 'state value outside' },
    { id: 'late-fraction-token', input: lateInput(() => {}).replace('"value":3', '"value":3.5'), diagnostic: 'non-integer numeric token' },
    { id: 'late-too-many-commitments', input: lateInput(v => {
      for (const name of ['b', 'c', 'd', 'e']) v.before.commitments[name] = structuredClone(v.before.commitments.plan);
    }), diagnostic: 'commitments requires at most 4 entries' },
    { id: 'late-duplicate-field', input: lateInput(() => {}).replace('"evidence":"ea"', '"evidence":"eb","evidence":"ea"'), diagnostic: 'duplicate field' },
  ];
}
