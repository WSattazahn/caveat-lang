// What the tests expect of the grammar, written independently of
// scripts/build-grammar.mjs. Each group says which words it covers, where they
// mean what they mean (probes: `@` stands for the word), and the scope they
// must get there. words.test.mjs checks the groups account for every word the
// runtime lists and that each probe holds; corpus.test.mjs uses them to judge
// real programs.

export const groups = [
  {
    name: 'statement heads of the other profiles',
    words: ['action', 'choice', 'select', 'resolve', 'investigate', 'inspect', 'converge', 'infer', 'require', 'observe',
      'open', 'operate', 'move', 'stay', 'relate', 'place', 'connect', 'position', 'camera', 'route', 'overview',
      'start_at', 'program', 'origin', 'rule'],
    // At a statement start, including in a `for` body.
    probe: ['@ x;', 'y; @ x;', 'for k as $r {\n  @ $r;\n};', 'for k as $r { @ $r_x; };'],
    scope: 'keyword.other.statement',
    // Reactive programs use these as names: `event observe`, `retaining
    // camera`, and evidence in a relation, `camera supports door_open`.
    notKeyword: ['on @ when x;', 'commit x because enough retaining @;', '@ supports x;', 'for k as $r {\n  @ qualifies $r;\n};'],
  },
  {
    name: 'declarations',
    words: ['scene', 'claim', 'evidence', 'caveat', 'state', 'event', 'readings', 'decisions', 'renewable', 'identifiers',
      'define', 'fn', 'proc', 'bind', 'cue', 'control', 'clock', 'budget', 'entity', 'display', 'module', 'use'],
    probe: '@ x;',
    scope: 'storage.type',
  },
  { name: 'control', words: ['on', 'when', 'for', 'as', 'if', 'then', 'otherwise', 'when_committed'], probe: 'x @ y;', scope: 'keyword.control' },
  {
    name: 'effects',
    words: ['set', 'sample', 'reveal', 'examine', 'commit', 'reopen', 'emit', 'call', 'reject', 'qualify', 'renew', 'withdraw', 'defer'],
    probe: 'x @ y;',
    scope: 'keyword.other.effect',
  },
  // Relations only in the forms the parsers read (see relations() in
  // reference.mjs); anywhere else the three words are names.
  {
    name: 'relation statements',
    words: ['supports', 'opposes', 'qualifies'],
    probe: ['a @ b;', 'glow::a @ glow::b;', 'x = 1; a @ b;', 'for k as $r {\n  $r_a @ b_$r;\n};',
      // Wrapped over lines, and the last statement without its `;`.
      'a\n  @ b;', 'a @\n  b;', 'a @ b\n;', 'x = 1;\na @ b', 'glow::a\n  @ b;',
      // FROM may be a word that begins other statements.
      'camera @ glow::door;', 'camera @ b\n;', 'x = 1;\ncamera @ b',
      // A template at column 0 and straight after `{`.
      'for k as $r {\n$r_a @ b_$r;\n};', 'for k as $r {$r_a @ b_$r;};', 'for k as $r {\n  $r_a\n    @ b_$r;\n};'],
    scope: 'keyword.operator.relation',
    notKeyword: ['claim @;', 'place @ kind dock;', 'state @ = 1;', 'event @;', 'a @ b c;', 'claim\n  @;',
      'for k as $r {\n  claim @;\n};', 'for k as $r {\n  place @ kind $r;\n};', 'for k as $r {\nclaim @;\n};'],
  },
  // A wrapped relation may begin with any name, keywords included: the
  // runtime accepts `evidence place from "sensor";` and a relation on it.
  {
    name: 'wrapped relations from keyword-named evidence',
    words: ['supports', 'opposes', 'qualifies'],
    probe: ['place\n  @ c;', 'claim\n  @ c;', 'supports\n  @ c;', 'observe # note\n  @ c;', 'for k as $r {\n  place\n    @ $r;\n};',
      'for k as $r {\n  claim // note\n    @ $r;\n};',
      // `sample` names a caveat here, not the sampling effect.
      'sample\n  @ c;', 'sample # note\n  @ c;', 'sample @\n  c;', 'for k as $r {\n  sample\n    @ $r;\n};'],
    scope: 'keyword.operator.relation',
    notKeyword: ['@\n  supports c;'],
  },
  // A comment may end any line a relation is wrapped across.
  {
    name: 'relations wrapped across comments',
    words: ['supports', 'opposes'],
    probe: ['sensor # note\n  @ ready;', 'sensor // note\n  @ ready;', 'sensor @ # note\n  ready;', 'sensor @ ready # note\n;',
      'camera @ ready // note\n;', 'on inspect reveal sensor @ ready # note\n;', 'on inspect reveal sensor @ ready // note\n;',
      'on e sample s = a + b // note\n  @ c;', 'when_committed act a # note\n  @ b;'],
    scope: 'keyword.operator.relation',
  },
  {
    name: 'relations in effects',
    words: ['supports', 'opposes'],
    probe: ['on e when x reveal a @ b;', 'reveal c then a @ b;', 'when_committed act a @ b;', 'on e sample s = x + 1 @ c;',
      'proc p() {\n  reveal a @ b;\n};', 'for k as $r {\n  on e reveal $r_a @ c;\n};', 'for k as $r {\n  on e sample s_$r = $index @ c;\n};',
      // Wrapped over lines.
      'on e reveal a\n  @ b;', 'on e reveal a @\n  b;', 'reveal c then\n  a @ b;', 'when_committed act a\n  @ b;',
      'on e sample s = a +\n  b\n  @ c;', 'for k as $r {\n  on e sample s_$r = $index\n    @ c;\n};',
      // The keyword alone on its line: the effect's region carries on.
      'proc p() {\n  reveal\n    a @ b;\n};', 'reveal\n  c then a @ b;', 'when_committed\n  act a @ b;', 'on e sample\n  s = a @ c;',
      'for k as $r {\n  reveal\n    $r_a @ c;\n};', 'for k as $r {\n  sample\n    s_$r = 1 @ c;\n};'],
    scope: 'keyword.operator.relation',
    notKeyword: ['on e reveal a @ b c;', 'on e sample s = @;', 'reveal c then a @;'],
  },
  {
    name: 'qualification is a statement',
    words: ['qualifies'],
    scope: 'keyword.operator.relation',
    notKeyword: ['on e reveal a @ b;', 'reveal c then a @ b;', 'on e sample s = 1 @ c;'],
  },
  { name: 'logical operators', words: ['and', 'or', 'not'], probe: 'x = a @ b;', scope: 'keyword.operator.logical' },
  { name: 'booleans', words: ['true', 'false'], probe: 'x = @;', scope: 'constant.language.boolean' },
  {
    name: 'clauses',
    words: ['from', 'limit', 'because', 'using', 'retaining', 'after', 'with', 'cost', 'consequence', 'kind', 'id', 'at', 'to',
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
    probe: ['action a from x to y @ z;', 'for k as $r {\n  action a_$r from x to y @ $r;\n};'],
    scope: 'keyword.other',
    notKeyword: ['state @ = 1;'],
  },
  {
    name: 'action steps',
    words: ['inspect', 'operate', 'open', 'through', 'move', 'observe'],
    probe: ['action a from x to y steps move y, @ z;', 'for k as $r {\n  action a from x to y steps @ $r, move $r;\n};'],
    scope: 'keyword.other.statement',
  },
  {
    name: 'epistemic predicates and functions',
    words: ['observed', 'committed', 'reopened', 'examined', 'has_caveat', 'caveated', 'carries', 'qualified', 'latest',
      'has_sample', 'history_at', 'history_count', 'fold_history', 'elapsed', 'require', 'text', 'abs', 'atan2', 'ceil',
      'floor', 'round', 'sqrt', 'sin', 'cos', 'clamp', 'wrap', 'number_text', 'percent_text', 'time_second_text',
      'time_total_text', 'time_text', 'min', 'max', 'id_text', 'withdrawn', 'rests_on_withdrawn'],
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
