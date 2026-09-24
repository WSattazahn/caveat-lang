// What the tests expect of the grammar, written independently of
// scripts/build-grammar.mjs. Each group says which words it covers, where they
// mean what they mean (a probe: `@` stands for the word), and the scope they
// must get there. words.test.mjs checks the groups account for every word the
// runtime lists and that each probe holds; corpus.test.mjs uses them to judge
// real programs.

export const groups = [
  {
    name: 'statement heads of the other profiles',
    words: ['action', 'choice', 'select', 'resolve', 'investigate', 'inspect', 'converge', 'infer', 'require', 'observe',
      'open', 'operate', 'move', 'stay', 'relate', 'place', 'connect', 'position', 'camera', 'route', 'overview',
      'start_at', 'program', 'origin', 'rule'],
    probe: '@ x;',
    scope: 'keyword.other.statement',
    // Reactive programs use these as names: `event observe`, `retaining camera`.
    notKeyword: ['on @ when x;', 'commit x because enough retaining @;'],
  },
  {
    name: 'declarations',
    words: ['scene', 'claim', 'evidence', 'caveat', 'state', 'event', 'readings', 'decisions', 'renewable', 'define',
      'fn', 'proc', 'bind', 'cue', 'control', 'clock', 'budget', 'entity', 'display', 'module', 'use'],
    probe: '@ x;',
    scope: 'storage.type',
  },
  { name: 'control', words: ['on', 'when', 'for', 'as', 'if', 'then', 'otherwise', 'when_committed'], probe: 'x @ y;', scope: 'keyword.control' },
  {
    name: 'effects',
    words: ['set', 'sample', 'reveal', 'examine', 'commit', 'reopen', 'emit', 'call', 'reject', 'qualify', 'renew', 'defer'],
    probe: 'x @ y;',
    scope: 'keyword.other.effect',
  },
  { name: 'relations', words: ['supports', 'opposes', 'qualifies'], probe: 'a @ b;', scope: 'keyword.operator.relation' },
  { name: 'logical operators', words: ['and', 'or', 'not'], probe: 'x = a @ b;', scope: 'keyword.operator.logical' },
  { name: 'booleans', words: ['true', 'false'], probe: 'x = @;', scope: 'constant.language.boolean' },
  {
    name: 'clauses',
    words: ['from', 'limit', 'because', 'using', 'retaining', 'after', 'with', 'cost', 'consequence', 'kind', 'at', 'to',
      'via', 'through', 'toward'],
    probe: 'x @ y;',
    scope: 'keyword.other',
  },
  { name: 'stop reasons', words: ['enough', 'budget', 'deadline'], probe: 'commit x because @;', scope: 'constant.language.stop-reason' },
  {
    name: 'uncited bindings',
    words: ['nothing'],
    probe: 'bind a.b = 1 because @;',
    scope: 'constant.language.nothing',
    notKeyword: ['state @ = 1;'],
  },
  {
    name: 'consequence levels',
    words: ['negligible', 'low', 'material', 'high', 'catastrophic'],
    probe: 'caveat x consequence @;',
    scope: 'constant.language.consequence',
    notKeyword: ['state @ = 1;'],
  },
  {
    name: 'cue kinds',
    words: ['sound', 'toast', 'flash', 'ring'],
    probe: 'cue c @ 440 0.1 0.1;',
    scope: 'keyword.other.cue-kind',
    // `bind toast.visible`, `entity marker kind ring`.
    notKeyword: ['bind @.visible = 1;', 'entity e kind @ at p;', 'bind e.@.color = "#fff";'],
  },
  { name: 'clock steps', words: ['every'], probe: 'clock tick @ 1;', scope: 'keyword.other', notKeyword: ['state @ = 1;'] },
  { name: 'control resets', words: ['reset'], probe: 'control c = e @;', scope: 'keyword.other', notKeyword: ['state @ = 1;'] },
  {
    name: 'bounds',
    words: ['min', 'max'],
    probe: 'state s = 0 @ -5;',
    scope: 'keyword.other.bound',
    alsoProbe: ['set x = @(a, b);', 'support.function.builtin'],
    notKeyword: ['set x = @;'],
  },
  {
    name: 'sequential clauses',
    words: ['options', 'requires_open', 'steps'],
    probe: 'action a from x to y @ z;',
    scope: 'keyword.other',
    notKeyword: ['state @ = 1;'],
  },
  {
    name: 'action steps',
    words: ['inspect', 'operate', 'open', 'through', 'move', 'observe'],
    probe: 'action a from x to y steps move y, @ z;',
    scope: 'keyword.other.statement',
  },
  {
    name: 'epistemic predicates and functions',
    words: ['observed', 'committed', 'reopened', 'examined', 'has_caveat', 'caveated', 'carries', 'qualified', 'latest',
      'has_sample', 'history_at', 'history_count', 'fold_history', 'elapsed', 'require', 'text', 'abs', 'atan2', 'ceil',
      'floor', 'round', 'sqrt', 'sin', 'cos', 'clamp', 'wrap', 'number_text', 'percent_text', 'time_second_text',
      'time_total_text', 'time_text', 'min', 'max'],
    probe: 'set x = @(y);',
    scope: 'support.function.builtin',
    notKeyword: ['state @ = 1;'],
  },
];

// Words the runtime reserves that are not source syntax: graph relations and
// origins it writes into views and snapshots (spec/caveat-reactive-0.3.md,
// spec/game-session-0.1.md). Reserving them keeps module rewriting safe; no
// parser reads them, so the grammar leaves them alone.
export const notSourceWords = {
  words: ['live', 'relies_on', 'reopens', 'retains', 'in_context'],
  notKeyword: ['x @ y;'],
};

const group = name => groups.find(entry => entry.name === name);
export const profileHeads = group('statement heads of the other profiles').words;
const builtinWords = new Set(group('epistemic predicates and functions').words);
export const isBuiltin = word => builtinWords.has(word);

// Every word the grammar may give a keyword, storage or language-constant scope.
export const highlightedWords = new Set(groups
  .filter(entry => !entry.scope.startsWith('support.'))
  .flatMap(entry => entry.words));

// The scope a statement head must have, if it is a keyword.
const headOrder = ['statement heads of the other profiles', 'declarations', 'control', 'effects', 'logical operators'];
export function expectedHeadScope(word) {
  for (const name of headOrder) if (group(name).words.includes(word)) return group(name).scope;
  return null;
}
