# Compatibility follow-up, 2026-11-14

These bounded checks found no additional refusal of an accepted historical
program. They extend the [compiler compatibility review](2026-11-14-compiler-compatibility.md),
not the unavailable original inventory or a release-readiness claim. The
follow-up `unset` linker repair described below has its own local validation
record at the end. Required remote checks are tracked on
[PR #479](https://github.com/WSattazahn/caveat-lang/pull/479).

## Published-package function comparison

The 399 synthetic programs exercise 20 function spellings, arities 0–3,
numeric/boolean/text results, nested calls, argument and global name
collisions, unused definitions, and bounded depth/width cases. Each package
was loaded through its own public Node loader and shipped WASM. Tarball byte
counts and SHA-256 matched the release records before extraction or execution.

| Published artifact | Bytes | Tarball SHA-256 |
| --- | ---: | --- |
| [`caveat-lang@1.0.0`](../releases/v1.0.0.md) | 1,215,669 | `56860ea8a712c7eed052811d910fff239c1e5da06f4581d7b428aeb281dccddf` |
| [`caveat-lang@1.4.1`](../releases/v1.4.1.md) | 1,488,301 | `ecb535b181f5f33836f4730ed8c1c0c8ac9a1622eb55e3d43e3e3c3492f281f5` |

The embedded build revisions are respectively
`e2a1350024352ca707e74491d9405507b166ca23` and
`5e31e83beb097499212965a142207cc800ff46b5`. Their loaded reactive WASM
SHA-256 values are respectively
`3c894023941efebe2367b97c5f5890eb3fc523d73e03d322cf3daecece88fdc6` and
`7e8bd09bcf5e7b0e0792f88b0058ed048fa361859871fb56f715bf078cfc4c6c`.

Candidate artifacts were built cleanly at
`c71c5e7f165bb89f987eb86cf103b0b447a19a79`, with runtime source fingerprint
`7a582f80025ebb1a844bb9cd27421d6041220f552475fe1b5958d2d5b8119139`.
The full WASM SHA-256 is
`ce4883622550ae3669165d6425d5e560ce103d4a51e51e4e123045330795a8b6`;
reactive WASM is
`0997f0ffb6ccfdcc921d189c6b41fac7f2e6291dd3612867525bb57d1e97b84a`.
The source is unchanged by the later `21fd92ab` corpus classification and
`efe3d55c` ledger commit. These are development artifacts, not a published 1.5.

| Runtime | Inputs | Load and dispatch accepted |
| --- | ---: | ---: |
| Published 1.0.0 | 399 | 339 |
| Published 1.4.1 | 399 | 333 |
| Candidate reactive WASM | 399 | 363 |
| Candidate full WASM | 399 | 363 |

All **1,344 comparisons** passed: each historical accepted input against
both candidate builds, comparing initial snapshots, complete first-event
outcomes, exact save bytes, and the next event. Restoring each historical
save in each candidate also reproduced the saved snapshot and next event.
No accepted historical input was refused or changed in this matrix. This
does not cover arbitrary source, all event histories, or every release.

Native release mode also accepted 363 inputs and matched every successful
native historical control. Windows native debug builds exhausted the default
stack on some deep call chains: current debug could load chains of 61/62
functions but crashed during dispatch; 1.4.1 debug already crashed loading
them. Both candidate WASM builds and native release computed the expected
value 5. Debug crashes are recorded as host/build limitations, not successful
refusals or evidence that the shipped packages reject those programs.

## Both complete tracked source corpora

The native load audit compared v1.4.1 with runtime code
`21fd92ab3803e9968b04ad7a37f38473de2f8afd`, using every tracked `.cav` path
from both the v1.4.1 checkout and the current checkout at
`efe3d55c333de2432352df9538203e29395f6912`. Runtime source was identical between
`21fd92ab` and `efe3d55c`. Counts below were independently recomputed from the
saved rows.

| Source bytes | Files | Both load | Both refuse | Older loads / candidate refuses | Older refuses / candidate loads |
| --- | ---: | ---: | ---: | ---: | ---: |
| v1.4.1 checkout | 212 | 182 | 30 | 0 | 0 |
| Current checkout | 223 | 189 | 30 | 0 | 4 |

Both CLIs resolved imports beside each exact source path. All jointly
accepted inputs had identical initial numeric `snapshot.values`; there
were no timeouts or transport failures. The four newly accepted current
files are `kit/examples/intel-betrayal/intel.cav` (newer across syntax) and
the three `v1.0.0-{boolean,numeric,text}.cav` function recovery fixtures.
The corpora overlap, include standalone module and intentionally refused
fixtures, and differ in six shared files' exact bytes. No historical tracked
path is missing from the current corpus. This audit checks loading and
initial numeric values, not event histories or saves.

## Unresolved boundary and draft follow-up

The interrupted handoff's original **eleven reproducer programs** and
unpushed D084 patch remain unavailable. These checks cannot establish that
every original D084–D088 item is closed, and do not repeat the reported
29-release, 2.67-million-save sweep.

The module-name ambiguity remains as recorded in the earlier review:
1.0.0 and 1.4.1 can emit the same initial save bytes for the same source,
then continue to 10 and 14 respectively. The save contains no version
discriminator. Recovery preserves the already accepted 1.4.1 interpretation;
it cannot reproduce both conflicting continuations from those bytes alone.

The separate repair concerns **unpublished draft 1.5 `unset`
syntax**: parser-derived keyword positions must preserve actual declarations
and effects, including procedure steps, and keep UTF-8 byte offsets after
comments. It also corrects an already accepted draft input: a module with
a comment containing 19 `é` characters, local `local = 7`,
`result = local`, and `state value unset`, beside root `define local = 11`,
previously bound the result to root 11 because the shifted keyword offset
exempted the local reference from module scoping. The repair restores the
module-local result 7. This is an intentional draft semantic correction;
stable 1.4.1 cannot load that `unset` declaration. On Windows with Rust
1.98.1 and `RUST_MIN_STACK=16777216`, the follow-up passed 329 native library
and affected integration tests, 137 reactive-only integration tests, formatting,
and Clippy on all targets with both default and reactive-only features.
The earlier passing audits do not certify this repair; the following
separate checks exercise its actual artifacts.

## Local validation of the unset repair

The clean full/reactive WebAssembly build at
`dbbd2050cae523eab3fd4b10eacab7be3e1cd27d` passed `npm run build`, using
Rust 1.98.1, wasm-bindgen 0.2.104 and Node 24.11.1 on Windows.
Its runtime source fingerprint is
`800ab0a0d7b281c2672372cffdabc564510e37688b1610800f289de63eb7a51f`.
Full WASM SHA-256 is
`98b7f215c8f0a98e374519423652715964230b3a7ee259251a62ce6adb887992`;
reactive WASM is
`89d1944c02315632438e65b23ce6fe6960f750a96bed1b8e8c003eef70441eae`.

- `npm run test:kit`: 699 passed, including the new shared unset fixture in
  both WASM builds, historical saves and continuation.
- `npm run test:ledger`: seven unit tests, strict check, ten scenarios and
  all 444 recorded events passed, including the D090 finding events.
- `node scripts/test-interface.mjs`: 193 programs had identical interfaces
  across native and both WASM builds; 30 refused alike.
- `PLAYWRIGHT_CHANNEL=chrome node scripts/test-kit-browser.mjs`: passed in
  Chrome 157.0.8103.51, including the same unset fixture with UTF-8 source.
- The same 399-input published-package matrix was rerun against these new
  artifacts: all 1,344 comparisons passed, including exact historical saves
  and restored continuation. Candidate acceptance remained 363/399 in both
  builds. Historical tarball and loaded artifact digests were rechecked.
- The same two complete tracked corpora were rerun against an explicitly
  rebuilt native CLI: all 435 source paths and bytes matched the earlier
  audit. Both rows of the table above were unchanged, with no historical
  acceptance regression, initial numeric difference or timeout.

Independent source/test review found no blocking issue. Local receipt audits
accepted all five native and four public-consumer checks against the current
input and artifact hashes, with no problems. This establishes only the named
local checks; it neither substitutes for required remote CI nor closes the
unavailable original inventory. Repeated matrix summary SHA-256 is
`0bb0b6921a9d1e65164e19edb0f30d13f50e3c407b93a60fcc9cc4ba31082cac`;
its separate harness is
`16027d5da2cad89d4c908e7541ab9bda13c476e36aa73552b60157159e12fd8d`.

## Reproduction outline and retained evidence

Use isolated directories; download exact versions without installing or
running package scripts, then verify the above hashes before loading:

```sh
npm pack caveat-lang@1.0.0 --ignore-scripts --pack-destination AUDIT_DIR
npm pack caveat-lang@1.4.1 --ignore-scripts --pack-destination AUDIT_DIR
cargo build --manifest-path runtime/Cargo.toml --release --bin caveat-reactive --no-default-features
git -C RELEASE_CHECKOUT ls-files '*.cav'
git -C CANDIDATE_CHECKOUT ls-files '*.cav'
BASELINE_BIN validate SOURCE.cav
CANDIDATE_BIN validate SOURCE.cav
```

For the function matrix, load each extracted package's `lib/node.mjs`, use
its `loadRuntimeFromDirectory()`, and load both candidate `dist/pkg` and
`dist/pkg-reactive` through the candidate loader. For each exact source,
open, snapshot, dispatch `go` with `{}`, retain the original `save()` text,
dispatch `go` again, then restore each historical save in both candidates
and compare its snapshot and continued dispatch. Close every session.

The local handoff retains `cases.json`, `published-matrix.mjs`, artifact
receipts, full observations and comparisons, plus both source-corpus rows.
They are not committed as multi-megabyte report attachments. Matrix input
SHA-256 is `b95f6ae1b7a81afa96370a9960d97c3c09c662b2ddced57cd22200c9017f35a4`;
harness SHA-256 is `2d25a02c4153cae715e2d1db525e4c16681ace5f347fbb970e68336e98bf5e78`.
The corpus summary SHA-256 is
`088cde32e76693a30e1fa2fe885f9d640463ef1a3464551cc099c27a2e9025fd`.
The command outline alone does not reconstruct the retained matrix inputs.
