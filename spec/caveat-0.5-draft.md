# CAVEAT Language Specification — Draft 0.5

This draft adds modules. Until now a program is one file, and the only library
is `runtime/prelude.cav`, which is compiled into the runtime binary by
`include_str!` and restricted to pure function definitions. An author cannot
write a second one. `game/light_the_way.cav` is 888 lines because there is
nowhere else to put them.

The governing rule of this draft is one sentence:

> **Names are scoped. The epistemic graph is not.**

A module boundary controls which identifiers a file can *write*. It never
controls what is *true about history*. Everything below follows from that, and
from the invariants in [CAVEAT_ESSENCE.md](../docs/CAVEAT_ESSENCE.md).

## 1. Modules declare; programs execute

```caveat
module weather;

claim low_visibility;
evidence fog_bank from "lookout report, 06:40";
caveat fog_refraction consequence material;
fog_bank supports low_visibility;

fn visibility_scale(metres) = clamp(metres / 400, 0, 1);
```

A module file begins with `module NAME;`. It may contain only *declarations*:
`claim`, `evidence`, `caveat`, relation statements, `rule`, `fn`, effect
procedures, `display`, and the presentation declarations. It may **not**
contain statements that execute: `budget`, `examine`, `defer`, `infer`,
`select`, `converge`, `commit`, `reopen`, `inspect`, or `start_at`.

The reason is not tidiness. Draft 0.4 says conditions "read only the executed
prefix", and Draft 0.3's world checks depend on a single ordered program. If a
module could execute, the order in which two independent imports ran would
decide which caveats were examined and which commitments existed — so the
meaning of a program would depend on the order its author happened to write the
`use` lines. Declarations are order-independent; effects are not. Modules get
the half that is.

The prelude becomes an ordinary module under this rule rather than a special
case in `reactive.rs`.

## 2. Importing is declaration, never observation

```caveat
use weather;
```

`use` makes a module's declared names visible to the importing file. That is
all it does. It does not:

- make any evidence observed — `observed(weather::fog_bank)` is false until
  execution actually reaches a `supports` or `opposes` edge from it;
- make any caveat examined — `examined(weather::fog_refraction)` is false until
  some execution pays for it out of the program's budget;
- assert any claim, or attach the importer as a source of the imported
  evidence. `evidence ... from "lookout report"` keeps that provenance in every
  file that can see it.

This preserves "Declaring evidence does not make it observed" across the new
boundary. Importing is a strictly weaker act than declaring, and declaring was
already not observing.

## 3. Qualified names

Imported names are written `module::symbol`:

```caveat
use weather;

budget 12;
examine weather::fog_refraction cost 3;
commit hold_course because enough retaining weather::fog_refraction;
```

A file's own declarations stay bare. There is no wildcard import and no
aliasing in this draft.

Both omissions are deliberate. If two modules could contribute a bare
`low_visibility`, the linker would have to either merge them into one graph node
or pick one. Merging invents a relation nobody authored: every edge attached to
either claim would silently attach to the other, and a `supports` edge from one
author's evidence would start supporting a different author's claim. Picking one
makes the graph depend on import order. A qualified name cannot do either.

`::` rather than `.` because `.` is already taken: presentation positions are
registered as `target.axis` constants (`ferry.x`), so `weather.fog_refraction`
would be reaching into an existing namespace.

## 4. One graph, one budget, one transaction

Three properties are explicitly *not* scoped by modules.

**One graph.** `observed(x)` asks whether execution reached an edge from `x`.
That is a fact about history, not about the asking file. If an effect declared
in module B creates the edge, an importer sees `observed(B::x)` as true. The
alternative — module-local observation — would make the same edge observed and
unobserved at once, which is not a persistent epistemic graph.

**One budget.** `budget` is a program statement and is rejected inside a module.
Per-module budgets would mean that splitting a file buys attention: the same
examinations would become affordable by refactoring. "Examination spends the
program's attention budget" is a single budget, and moving text between files is
not a way to earn more of it.

**One transaction.** Reactive 0.7 already holds that nested calls share one
event transaction and work budget, and that any late failure rolls back history,
graph changes, cues and occurrence IDs. A call that crosses a module boundary is
an ordinary call. It does not open a nested transaction, cannot partially
commit, and cannot roll back only its own module's effects.

An imported caveat that some execution examined reports `examined` to every file
that can name it. Nobody received examination for free; the importer is only
permitted to observe that it was paid for.

## 5. Linking and program identity

Modules are a **link-time** construct. The runtime continues to receive one
program and one source identity, so Draft 0.4 section 5 saves and byte-pinned
provenance receipts keep working unchanged.

A multi-module program ships as a **bundle**: the module sources concatenated
with explicit headers, which is still a single text file.

```
#caveat-bundle 1
#module weather 214
module weather;
...
#module main 1801
use weather;
...
```

Each header gives a module name and the exact byte length of the source that
follows, so splitting a bundle back into modules is exact and requires no
re-parsing. The bundle is the artifact: `source_id` is computed over the bundle
bytes exactly as it is computed over a single file today, saves store the bundle
bytes, and restore compares them for equality. A single-file program is a
bundle with no headers, so every existing program and every existing save
remains valid.

Linking resolves each `use` against the bundle's module names, checks the rules
in section 6, and rewrites every qualified name to a flat identifier. A module
named in two different import paths is linked **once**, by module name, so a
diamond import yields one set of graph nodes rather than one per path.

Flattened names are reserved: source may not declare an identifier containing
`__`, so a rewritten name can never collide with a hand-written one.

## 6. Errors

Linking rejects, before any execution:

- a `use` of a module not present in the bundle;
- a cycle in the import graph;
- a qualified name whose module is not imported by that file;
- a qualified name that the named module does not declare;
- an executing statement in a module (section 1), naming the statement and file;
- `budget` in a module (section 4);
- duplicate `module` names in one bundle;
- a declared identifier containing `__` (section 5);
- a bundle header whose byte length does not match the text that follows.

These are static errors. None of them can be reached at runtime, and none of
them consume attention or create graph state.

## 7. Honest boundaries

Modules do not make Caveat self-hosting. Parsing, transactions, graph storage,
primitive numeric and text operations, and WebAssembly remain Rust. The linker
is a pre-pass over parsed declarations; it adds no evaluation semantics, and the
evaluator is unchanged by this draft.

A module system is a reuse mechanism, not evidence of epistemic correctness.
Sharing a vocabulary of claims between two games does not establish that the
shared vocabulary is well chosen, that either game's policy is warranted, or
that an imported caveat's consequence level is honest. It establishes only that
two programs are talking about the same graph node instead of two accidentally
identical names.

Versioning, module visibility, separate compilation, and a package registry are
out of scope. So is any notion of a trusted or signed module: a bundle's
provenance is exactly the provenance of its bytes, which is what the receipt
already pins.
