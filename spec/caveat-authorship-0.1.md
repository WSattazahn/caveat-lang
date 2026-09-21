# CAVEAT Authorship 0.1

[caveat-0.5](caveat-0.5-draft.md) gave the language modules, and flattened
`weather::fog_bank` to `weather__fog_bank` so that one program comes out the
other side. That left the graph answering "which part asserted this?" with a
naming convention: you could recover the answer only by parsing a symbol's
name and knowing what `__` meant.

For a language whose premise is that provenance belongs in the record rather
than in a habit, that was the wrong shape. This profile makes it structural.

## 1. Two provenances, asked separately

```caveat
evidence fog_bank from "lookout report, 06:40";
```

`from` says **where the claim came from in the world**. It is unchanged by this
profile, and importing a module never rewrites it: an imported evidence keeps
its own `from` in every file that can see it.

*Origin* says **where the assertion came from in the program** — which part of
the source declared this node, or asserted this edge. The graph could not
answer that before.

They are different questions and both are recorded. A snapshot symbol carries
`source` and `origin` side by side.

## 2. The statement

```caveat
origin weather;
```

Everything declared after an `origin` belongs to that part, until the next one.
The linker emits one before each part it concatenates, so a bundle carries the
record with no work from the author.

The marker is an ordinary statement in the linked text, not a side channel.
That is deliberate: a reader of the artifact — or of a save, or of anything
pinned by a receipt — sees the same attribution the graph reports, and nothing
has to be trusted to a parallel data structure that could drift from the text.

## 3. What is not recorded

**A single-file program records nothing.** No `origin` statement means no
attribution, and the graph says so: `origin(node)` is `None` and the origins
map is empty. An invented default — "main", or the filename — would be a
confident answer to a question nobody answered.

**An edge a running effect reveals is unattributed.** Declarations are over by
the time an event dispatches, so there is no part still being read. The obvious
shortcut, letting the edge inherit whichever part was read last, would attach a
specific and wrong name to it. The nodes such an edge connects keep their own
origins, so "evidence declared by `glow` now supports a claim declared by
`glow`" is still answerable; what the graph does not claim to know is which
part's rule was executing.

Attributing runtime effects means carrying an origin on procedures and rules
through execution. That is a real extension and it is not in this profile.

## 4. Honest boundary

A recorded origin is what the source says, not an authentication. A
hand-written `origin survey;` is recorded as `survey` with no check that any
part called `survey` exists, exactly as
[source-library-0.1](source-library-0.1.md) holds that "caller-supplied
provenance is not automatically authenticated evidence".

What the record establishes is that the graph and the linked text agree about
where an assertion was written. It does not establish that the part is what it
claims to be, that its author was entitled to assert it, or that the assertion
is warranted. Those are not questions a name can answer.
