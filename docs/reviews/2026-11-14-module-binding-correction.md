# Correct module bindings and preserve historical save choices

This supersedes the decision in the earlier
[compiler compatibility review](2026-11-14-compiler-compatibility.md) to keep
successful 1.4 root capture. The owner explicitly requested that the bug be
fixed, rather than preserved as the default. PR #479's draft `unset` repair
was merged separately at `1c6b81241f97b2109675049faa4f868fbf28c5f3`.
This correction is unreleased. The reconstructed root-capture/save-ambiguity
finding is recorded as D091, without asserting that it is identical to any
unrecovered D084–D088 entry. The ledger records its observation in published
release builds; it does not record approval to merge this follow-up.

## The defect and the unavoidable ambiguity

A module-local `hold=7` used in `add( hold , because )`, with `because=3`,
should produce 10. The old linker mistook the reference for an effect
keyword and left it bare. A root `hold=11` then made the same program
produce 14. The same error affects `release`, authored functions and
procedure calls. The corrected linker qualifies references and leaves only
actual effect keywords bare. Fresh reactive loads and public `link::link`
use it directly. They do not fall back to a successful wrong interpretation.
A source that loaded only because it captured a wrong root declaration may
now correctly fail validation.

The collision fixture's published 1.0.0 and 1.4.1 initial saves are identical
bytes, but their next event produces 10 and 14 respectively. Both have
`source_id=fnv1a64:8cb496ca5fa15ec4:195` and source SHA-256
`b8f3d86d72c1db1d3ca8ed50967eac9934a1fec07d855c87af7c02cd1890b138`.
There is no reliable version or state heuristic that can choose both
histories correctly.

## Restore contract

- Affected state, named replay and positional replay saves record
  `module_bindings`: `scoped` or `legacy-hold-capture`.
- This runtime marks every affected save, so an unmarked one was written by
  a release: before 1.2.0 with module scope, or from 1.2.0 to 1.4.1 with
  root capture. Restore uses the interpretation of the releases that could
  have written the save, as they linked the source (below): those that could
  load the source and wrote the save's format. Every release wrote state
  saves, from rc.17 on with `source_sha256`; Replay Save 0.1 came in 1.1.0
  and 0.2 in 1.2.0 ([D095](#d095-a-saves-writers-are-the-releases-that-wrote-its-format)).
  Restore requires an explicit choice only when releases of both kinds could
  have written the save and their readings differ. Where none could, it uses
  the interpretation that loads, and requires a choice only when both load
  and read the source differently. Saved state validation is never used to
  pick a convenient interpretation.
- A saved marker selects the interpretation. An explicit conflicting option
  is refused. The selected program supplies the initial state, bounds and
  capabilities before saved state is applied or events replayed. Cache keys
  include the interpretation.
- Source identity still covers the original raw bytes. When reading a
  module's `hold` and `release` either way links the same source, saves omit
  the marker and preserve their bytes.
- Selection continues historical state; it does not rewrite frozen decisions
  or migrate past state to corrected semantics. It authenticates nothing.

The [save specification](../../spec/caveat-save-0.1.md#module-binding-interpretation)
defines the native, WASM and kit APIs. Serve accepts an optional per-request
choice and a startup flag for an existing state file. Successful explicit
startup restore checkpoints the marked save atomically before ready, without
waiting for another event. Refused restore leaves the prior file/session.

`savedEvents` and `carrySave` read event definitions from the restored
session's snapshot: freshly loading corrected source can legitimately fail
for a legacy-only program. This adds the cost of one full snapshot to these
helpers; it avoids introducing another public interface API. Sessions close
on success and errors. Raw save text is not parsed and reserialized by the
production restore adapter, preserving signed zero.

## Evidence

The ten raw saves in
[`runtime/tests/fixtures/module-bindings`](../../runtime/tests/fixtures/module-bindings/)
were generated through each published package's own public Node loader and
shipped WASM after checking tarball size and SHA-256 against its release
record. `provenance.json` records package and save hashes. The fixtures cover
identical initial saves, state after an event, both replay encodings, and a
legacy-only source whose corrected function call has the wrong arity.

Native regressions cover explicit selection, all three writers, conflicting
and malformed markers/options, cache interleaving, raw source identity,
unknown-state validation and subsequent events. The existing genuine rc.11
grounds fixture remains unchanged. An older D083 unit fixture depended on
root capture; it now asserts corrected fresh refusal and explicitly checks
the historical restore classification under the legacy interpretation.

Public kit tests cover both WASM builds, legacy-only save helpers, strict
options, signed-zero forwarding, session closure, serve state checkpointing
  and retention of the actual restore error when a fresh source load refuses.
Final command results and remote checks are recorded on the pull request.

## D088: each bundle read as the releases that loaded it

The original D084–D088 reproducers were then run on this branch with release
builds of every release from rc.1 to 1.4.1. D084, D085 and D087 were fixed by
#477 to #479, and D086 by #481. D088's remaining programs loaded on older
minors and stopped at 1.1.0, 1.2.0 or 1.4.0, because those releases linked a
module's names another way
([Caveat 0.5](../../spec/caveat-0.5-draft.md)):

- A module that declares `irreversible` (rc.1 to 1.1.0 load it), and, found
  by the same check over every reserved word, one that declares
  `forecloses`. From 1.2.0 the linker reserves both.
- A module that declares `only_if` as a state or define and gives
  `only_if(…)` a number for its condition (four programs; rc.1 to 1.3.0
  load them). Before 1.4.0 the name was the module's, so the call reached
  the bundle program's `fn MODULE__only_if`; from 1.4.0 it is the form.
- `reopened by changed ,r` beside a module's stream `changed` (rc.3 to 1.0.3
  load it). From 1.1.0 the linker ended a trigger only at a word that ends
  in `,` or `;`, so `changed` stayed bare there and named the program's
  stream `changed`.

The third review below found two more rules of this kind, which main and
#480 applied to every bundle too:

- A module's `fn resolve (y) = …`, with a space before its parameters, or
  its `event resolve revision` (rc.1 to 1.2.0 load them). From 1.3.0 the
  linker kept `resolve` bare before a name and `=`, or before `revision`,
  for outcomes.
- A module's `retaining gap because once ;`, with a space before its end
  (1.1.0 to 1.3.0 load it), or a later step's `because once , x` in a
  procedure that retains a caveat (rc.1 to 1.3.0). From 1.4.0 the linker
  kept `once` bare there, for `because once`, naming the program's `once`.

Each line of releases' linking is now a `link::Reading`: `NOW` (this
runtime: 1.4.1's linking, corrected), `V1_4` (1.4.0 and 1.4.1), `V1_3`
(1.3.x), `V1_2` (1.2.0), `V1_1` (1.1.0, and 1.2.0 corrected) and `V1_0`
(rc.1 to 1.0.3). Each keeps by position only the clause words that the
minors up to its line added, and `Reading::scoped` reads a line with a
module's `hold` and `release` its own. Which releases could load what a
reading links is found from the program as it loads (`release_bounds` in
`runtime/src/reactive_releases.rs`): from the first release that reads every
form it keeps, to the last release before one that refused one of its
function bodies, a body that calls the program's own function named like a
form that release added and read as the form there (D084, and D093 for
`only_if`). A bundle is read as the newest release that could load it linked
it, with a module's `hold` and `release` its own; where that release loaded
it only through root capture, as the newest older one whose reading loads
with module scope; and a bundle that no release could load so is read as
this runtime links it, which refuses a bundle that loaded only through
capture, as #480 does. `link::link` keeps 1.4's linking, corrected, and so
still refuses a module that declares `irreversible` or `forecloses`, as
1.4.1 does. D088's eight programs all load, and so do the third review's,
and each one's saves equal those of the newest release that loaded it.

The same check over the review's 20,328 generated `only_if` bundles found
one more case, a finding of its own (D093). Seven bundles declare `only_if`
in a module, `fn only_if` and `fn MODULE__only_if` in the program, and give
`only_if(…)` a number for its condition in a function's body. rc.1 to 1.3.0
load them, the module's call reaching `fn MODULE__only_if`; 1.4.0 and 1.4.1
refuse them. Main, since #478 let a function's body call the program's own
`fn only_if`, read the module's call as that function's, which no release
did, and so did #480. Such a body is one 1.4 refused, so the newest release
that could load these bundles is 1.3.0, and the seven compute its saves.

An unmarked save of an affected bundle restores as the releases that could
have written it read it, as the [Restore contract](#restore-contract) above
says. A bundle that only releases from 1.2.0 to 1.4.1 loaded, through root
capture, is refused fresh, but its saves restore with root capture and no
choice. Where releases before 1.2.0 could also have written the save, and
read the bundle differently, the save needs a choice. So
`only_if( hold , because )` in a module that declares `only_if` and `hold`,
beside the program's `define hold` and `fn MODULE__only_if`, loads fresh as
1.1.0 read it, the call of `fn MODULE__only_if` with the module's number,
where 1.4 read the form with the program's define as its condition; an
unmarked 1.4.1 state or named replay save of it needs `legacy-hold-capture`,
and of 1.1.0 `scoped`, while a positional one, which only releases from
1.2.0 wrote, restores with root capture and no choice (D095).

### Reviews

An adversarial review of a first version of these changes found three
problems, which the second version fixed. It split the triggers at every
comma in both readings, so it read
`changed` in other places than 1.1.0 to 1.4.1 did: among 7,968 generated
bundles it refused 34 that those releases load and changed the saves of 26,
and it continued some 1.1.0 saves under a reading no release had. It read
the new clause words' offsets with the older rules, whose offsets move after
text that is not ASCII earlier in a module, so a module could read a
program's name in place of its own. And it told a save of another program,
or of an edited source, to choose a reading, where 1.4.1 refuses it as such;
restore now checks whose save it is first. The older rules' offsets still
move so, as in every release since each rule came; correcting them would
change programs 1.4.1 loads, so that is a finding of its own (D092). The
reading by release line above adds no positional rule, so the second
problem has no place in it.

A second independent review, of a version that read a bundle that did not
load as linked now with the first older reading that loaded, latest first,
found fifteen problems with that rule. It gave bundles only 1.4.0 and 1.4.1
load the pre-1.4 call, a computation no release had, and made their 1.4.1
saves need a choice; gave bundles only rc.3 to 1.0.3 load the 1.1.0 trigger
reading and refused their saves; loaded a module that declares
`irreversible` or `forecloses` with 1.4's `only_if` reading, which none of
the releases that load it used, and asked a choice for the older saves of
one beside a hold capture, offering a reading no release had; loaded
bundles no release loads by mixing an older linking with later forms; and
kept D093's misreading where 1.4 refused a bundle for another reason. Its
split of the triggers at every comma was not the reading before 1.1.0
either, a D088 test used a bundle no release loads, and this record, the
spec and the code said that nothing 1.4 loads is read another way, which was
not so. The reading by release line above replaces that rule.

A third independent review, of the version before this one, found that it
kept two later minors' rules in every reading: 1.3.0's, which kept `resolve`
bare before a name and `=` or before `revision`, and 1.4.0's, which kept
`once` bare after `because`. So it refused a module's `fn resolve (y)` and
`event resolve revision`, which rc.1 to 1.2.0 load, and a module's `once`
after a retained caveat's `because`, which releases up to 1.3.0 load, with
their saves, or read them otherwise: the event as the program's `resolve`
where 1.1.0 named it `m__resolve`, and the program's `once` where those
releases read the module's. Each reading now keeps only the rules of the
minors up to its line, and `V1_3` is new for the rules of 1.3.x.

It also found two problems in restore. Asked for `legacy-hold-capture`, a
bundle that only releases before 1.2.0 could load was read as 1.4 links it,
which changed its `changed` triggers and a module's `only_if(…)` calls as
well as `hold` and `release`; and where the module's `hold` and `release`
read the same either way, the session's saves carried no marker, so without
the choice this runtime refused them or continued them otherwise. No release
read such a bundle with root capture: the choice now reads it as those
releases did where the program's `hold` and `release` would change nothing,
and is refused where they would. And a save of another program, or of an
edited source, that carried a marker or was restored with a choice was
refused for a conflict with its marker or for the chosen reading's load
error, not as another program's save; restore now checks whose save it is
before it uses a marker or a choice. The review's other findings were in
wording: a D088 test gave D093 as the reason 1.4 refused a bundle, where it
was the bundle's function body (D084); the Checks below labelled the bundle
sets wrongly, so that the parts added to more than the total; and a commit
message said that every older save of the restore set restores, where 12
1.0.3 saves (six of one program, which the set holds twice) are refused, as
1.3.0 refuses them. Each is corrected.

A fourth independent check, of those fixes, found both restore problems
fixed, over 3,100 more generated bundles, the release saves of the restore set
and 241,972 chains of restores and saves, and five things said wrongly:
Save 0.1 said that a fresh session reads a module's names as its own, which
holds only for its `hold` and `release`, and that a choice is needed only
where releases of both kinds could have written a save, leaving out a source
no release could load; `caveat serve` reported an unmarked save that needs a choice as a file
that is not a save of the program; the new refusal named the host's
`moduleBindings` where the choice came from the save's marker; and the last
bullet below, like two counts before it, called bundle files programs, and
gave capture as the reason for every bundle it refuses. Each is corrected,
and the refusal now names neither the option nor the marker.

### Checks

Release builds of every release from rc.1 to 1.4.1 were the reference, over
43,146 bundles: the probes of this branch and its first two reviews (654, the
second review's 622 among them), the D083 and D084 probes (296) and 42,196
generated bundles. Which
of 1.0.0, 1.0.3, 1.1.0, 1.2.0, 1.3.0, 1.4.0 and 1.4.1 load each bundle is
predicted from its readings exactly, apart from one probe, an agent declared
in a module with a relation, that 1.4 refused while parsing (D083); it is
read as this runtime reads it either way.

- Of the 9,192 bundles a 1.x release loads, 9,130 compute the saves of the
  newest release that loads them (4 runs of 30 seeded events, the marker
  aside). The other 62 are where a module's `hold` or `release` before
  `because` named a declaration of the program: 34 loaded only through that
  capture and are refused, 27 compute 1.1.0's saves, and one, which uses an
  irreversible series, computes the corrected value no release had (10 where
  1.4.1 gives 14, as in the collision fixture above).
- Of the 33,954 bundles no 1.x release loads, 32 load, with #480's saves,
  and 2,175 that the previous version of this branch loaded are refused, as
  #480 and main refuse them.
- Every save 1.0.3, 1.1.0, 1.2.0, 1.3.0 and 1.4.1 write of 8,746 of these
  bundles (2 runs of 10 events, 219,648 saves) was restored and continued with
  the writer's next events. Of 1.4.1's 96,888 saves, 96,206 restore and
  continue byte for byte as 1.4.1 continued them, and 682 (of 31 bundles, 22 programs) need a
  choice. Of the older releases' 122,760 saves, 121,098 restore, and all
  continue as their writer did except those of 23 programs that a later
  release linked differently, 1.4.0 a module's `only_if(…)` or 1.1.0 `reopened
  by changed ,r`: 22 continue byte for byte as 1.4.1 continues them, and one
  that 1.4 refused as 1.3.0 does, which refuses the same six of its 1.0.3
  saves. 1,650 saves (of 29, 32, 7 and 7 bundles; 20, 23, 7 and 7 programs) need a choice. With the
  choice that names the writer's reading, every save that needs one restores
  and continues as its writer did. #480 refused 84,206 of the older releases'
  saves, mostly of D088 programs it does not load.
- After the third review's fix, these bundles give the same loads, saves and
  restores as before it, and so do the review's own 7,152 (1,788 forms in four
  module templates) loads and saves. Its 53 probes and 320 bundles generated
  around the two rules were checked the same way. Which releases load each is
  predicted exactly. The 261 that a release loads all load: 205 compute the
  saves of the newest release that loads them, and 56 are where a module's
  `hold` before `because` named the program's, of which 32 compute 1.1.0's
  saves and 24, which only releases from 1.3.0 load, the corrected value. The
  other 112 are refused, as #480 and main refuse them. Every save 1.0.3,
  1.1.0, 1.2.0, 1.3.0 and 1.4.1 write of them (13,750) restores and continues
  as its writer did, or needs a choice (2,706) and does with it, except those
  of one probe that 1.4.0 linked differently, a module's
  `retaining gap because once ;` beside the program's own `once`. Of each
  older writer's 22 saves of it, 4 continue as 1.4.1 continues them and 18
  are refused, as main refuses them since D054 and 1.4.1 did not: they carry
  the module's `once` into a decision, which no rule does as 1.4 links it.
- After the restore fixes, the 43,146 bundles and the review's 7,152 give the
  same loads and saves, markers included, and every release's save of the
  restore set continues as before, with the D091 choice and without it.
  Asked for `legacy-hold-capture`, the 59,620 saves 1.0.3 and 1.1.0 write of
  that set continue as before wherever a release from 1.2.0 on loads the
  bundle. Of the bundles only earlier releases load, 93 for each writer (82
  and 90 programs: the set holds some programs more than once) now continue
  every save as their writer did, where the previous version refused or
  changed them, and 42 of 1.0.3's and 46 of 1.1.0's (36 and 40 programs) are
  refused with the new message: releases from 1.2.0 to 1.4.1 would have read
  their module's `hold` or `release` before `because` as the program's, which
  in 5 and 9 of them the program does not declare, so that those releases
  refused them. Without a choice their saves continue as their writer did.
  The review's own cases now continue as their writers did with the choice,
  and its saves of another program or of an edited source are refused as
  such, with or without a marker or a choice (78 restores).

The cases are in `runtime/tests/d088_module_names_older_releases_loaded.rs`,
`runtime/tests/d093_module_only_if_beside_a_program_only_if.rs`,
`runtime/tests/module_binding_restore.rs` and the tests of
`runtime/src/reactive_releases.rs`.

## D095: a save's writers are the releases that wrote its format

Restore counted every release that could load an affected bundle as a
possible writer of an unmarked save, whatever the save's format, and so
asked for a choice where only one kind of release wrote that format. No
release before 1.2.0 wrote Replay Save 0.2, so an unmarked positional save
of a bundle every release loads is now read with root capture, as the
releases from 1.2.0 to 1.4.1 that wrote it read the bundle, with no choice.
A named replay save of it still needs one, since 1.1.0 wrote that format
too. A state save without `source_sha256`, which only release candidates
before rc.17 wrote (rc.16 writes the collision fixture's 1.0.0 saves without
it), is read with module scope, as they read it. Where none of the releases
that wrote a save's format could load its source, a choice is needed only
where both readings load and read it differently.

As everywhere above, a bundle is read as the newest release that could load
it linked it. Where 1.3.0 or 1.4.0 linked a bundle differently from 1.2.0,
a positional save that 1.2.0 or 1.3.0 wrote continues as the newest such
release would have continued it, or is refused where it holds an event that
release would not accept, as a state save of such a bundle that only
releases from 1.2.0 could load already did; no choice reaches the older
reading.

1.1.0, 1.2.0, 1.3.0 and 1.4.1 each wrote replay saves of the 8,746 bundles
above (2 runs of 10 seeded events), and main (`e00a2bc6`) and this change
restored and continued every one with the writer's events. Main
asked for a choice on 990 positional saves (154 from 1.2.0, 154 from 1.3.0,
682 from 1.4.1); this change restores all 990 with no choice, and each
continues byte for byte as its writer did. Every other replay save restores
as before, including the 1,674 named ones that still need a choice.

An independent review of this change found that it still asked a choice for
a state save without `source_sha256`, that no test pinned 1.2.0 as the
first release that wrote positional saves, and that this record, Replay
Save 0.2 and two D088 test comments still gave the earlier rule. Each is
corrected; the cases are in `runtime/tests/module_binding_restore.rs`.

## Scope limits

No release, publication, broad performance improvement or new full
historical release sweep is claimed. The bug is corrected for fresh
sessions; retaining legacy semantics is an explicit historical continuation,
not the default behavior.
