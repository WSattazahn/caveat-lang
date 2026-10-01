// The common mistakes on the one-page reference that came from an agent
// integration: each reproduction fails to load with the diagnostic the page
// quotes, and the fix the page gives loads.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { CaveatError } from '../lib/session.mjs';
import { real } from './helpers.mjs';

const reference = (await readFile(new URL('../docs/REFERENCE.md', import.meta.url), 'utf8')).replace(/\s+/g, ' ');

const MISTAKES = [
  {
    name: 'an event of its own named tick',
    source: [
      'scene "Built-in tick carries a special time-step limit.";',
      'event tick dt min 0 max 86400;',
      'clock tick every 1;',
    ],
    diagnostic: 'tick must declare only dt with bounds inside 0..0.1 seconds',
    fixed: [
      'scene "Built-in tick carries a special time-step limit.";',
      'event advance dt min 0 max 86400;',
      'clock advance every 1;',
    ],
    page: ['clock advance every 1'],
  },
  {
    name: 'one parameter name with a member at different positions',
    source: [
      'scene "Enum names are shared by parameter name.";',
      'event first origin in tool_observation direct_user_report external_text;',
      'event second origin in direct_user external_text;',
    ],
    diagnostic: 'origin.external_text already names a different value',
    fixed: [
      'scene "Enum names are shared by parameter name.";',
      'event first origin in tool_observation direct_user_report external_text;',
      'event second requester_origin in direct_user external_text;',
    ],
    page: ['requester_origin'],
  },
  {
    name: 'an event parameter named like a state',
    source: [
      'scene "Event parameters cannot shadow state.";',
      'state limitations = 0;',
      'event configure limitations min 0 max 1;',
    ],
    diagnostic: 'event configure parameter limitations shadows state or a coordinate',
    fixed: [
      'scene "Event parameters cannot shadow state.";',
      'state limitations = 0;',
      'event configure limit min 0 max 1;',
    ],
    page: [],
  },
];

for (const mistake of MISTAKES) {
  test(`the reference's mistake "${mistake.name}" fails as it says, and its fix loads`, () => {
    assert.throws(() => real.open(mistake.source.join('\n')),
      error => error instanceof CaveatError && error.kind === 'load' && error.message.includes(mistake.diagnostic),
      `the runtime does not refuse it with "${mistake.diagnostic}"`);
    real.open(mistake.fixed.join('\n')).close();
    for (const text of [mistake.diagnostic.replace(/^event configure /, ''), ...mistake.page]) {
      assert.ok(reference.includes(text), `REFERENCE.md does not say "${text}"`);
    }
  });
}

// The page's claim is about constants, not names: two lists under one name
// load when every member they share has the same position.
test('a parameter name shared by events loads when their shared members agree', () => {
  real.open('event first origin in tool_observation external_text;\nevent second origin in tool_observation;').close();
  assert.throws(() => real.open('event first origin in a b;\nevent second origin in b a;'),
    error => error instanceof CaveatError && /origin\.[ab] already names a different value/.test(error.message));
});
