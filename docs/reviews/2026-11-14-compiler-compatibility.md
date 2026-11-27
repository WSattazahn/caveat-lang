# Compiler compatibility continuation

**Later owner decision:** preserving the successful 1.4 root capture below
has been superseded by the
[module-binding correction](2026-11-14-module-binding-correction.md).
Fresh sessions use corrected scope; ambiguous historical saves select their
known interpretation explicitly. The observations below remain the record
of the earlier repair.

This continues the reconstructed name-compatibility work merged in
[PR #477](https://github.com/WSattazahn/caveat-lang/pull/477), at
`5c6338bde62b560e6abd4d87b9f7b172e61d0313`. It is not a release record or
a claim that every item in the interrupted D084–D088 inventory is closed:
the original eleven reproducer programs and unpushed D084 patch were not
recovered.

## Observed failures and bounded repairs

Historical native builds used the tagged sources `v1.0.0`
(`e2a1350024352ca707e74491d9405507b166ca23`) and `v1.4.1` (`5e31e83b`).
The pre-fix comparison is the code merged by #477. These are synthetic
programs, not measurements of how often users encounter the failures.

| Case | 1.0.0 | 1.4.1 / #477 | Repair |
| --- | --- | --- | --- |
| Authored `only_if(x,y)` called with numeric arguments inside another function | Loads | Refused by intrinsic boolean inference | After failed ordinary inference, validate the actual authored expansion |
| Unused wrapper passing a boolean to authored `only_if` | Refused | Loads | Keep successful ordinary inference; do not reinterpret it |
| Module-local `hold` or `release` in `add( hold , because )`, with no root name | Loads | Refused because the module prefix is missing | Retry a refused reactive bundle with actual effect positions identified by the rule parser |
| Same module expression with a root `hold=11` and local `hold=7` | Computes 10 | Computes 14 | Retain successful existing loading and its value 14 |
| Authored `sum`, `any`, or `all`, with `for + 1` as an argument | Loads | 1.4.1 loads; #477 refuses it as an across-members header | Distinguish a kind header from continuation of an ordinary expression |

The function fallback keeps the original error if expansion or full validation
fails. It retains purity, argument types, arity, cycle and expansion limits.
Genuine historical saves, including caveated frozen commitment bases, are in
[`runtime/tests/fixtures/only-if-functions`](../../runtime/tests/fixtures/only-if-functions/).

The bundle fallback runs only after the entire ordinary reactive load fails.
Already accepted programs keep their prior interpretation. Both attempts bind
saves and caches to the original bundle bytes. A second failure keeps the
first diagnostic. Public profile-neutral `link::link`, sequential loading and
source-library extraction retain their existing contracts. Consequently,
manually flattening a newly rescued bundle through the legacy raw linker can
lose the module context the reactive retry needs. The CLI's `--link` emits
the original bundle, so that round trip retains the context. `check` continues
to refuse bundles as its existing contract requires.

## Remaining ambiguity in D088

The older silent root capture cannot be resolved from source and save alone.
With the same collision source, 1.0.0 and 1.4.1 emitted byte-identical initial
saves: source ID `fnv1a64:8cb496ca5fa15ec4:195`, source SHA-256
`b8f3d86d72c1db1d3ca8ed50967eac9934a1fec07d855c87af7c02cd1890b138`.
Continuing that saved text computes 10 under 1.0.0 and 14 under 1.4.1.
There is no runtime-version discriminator in that save. Preserving the
currently accepted result avoids introducing another change; it does not
restore both incompatible historical continuations. A complete resolution
needs an explicit compatibility choice or external version information.

The focused tests exercise older acceptance, current acceptance, real new
forms and effects, refused input, source identity, interface extraction,
cache reuse, save restoration and continuation. Node checks compare both
WebAssembly builds, and browser checks use the public kit API. This evidence
does not substitute for the unavailable original inventory or the historical
29-release, 2.67-million-save comparison; neither is claimed complete here.
