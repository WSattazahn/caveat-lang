# Independent PR176 matrix reuse

`independent-wasm-check.mjs` reuses the generator and substantive test body from
the user-supplied **Caveat_PR176_Independent_Review.zip**. Its original authorship,
hashes, case counts and adapter-only changes are recorded in
`independent-review-provenance.json`. The original bundle stays outside this
repository. The adapted driver checks that its retained test-body bytes match
the supplied portable harness.

Run with Node.js 22 or later, after independently building and freezing the
desired artifacts. This script performs no build, installation, native profile,
network request, source modification or Git operation:

```text
node experiments/departure-gate/withdrawal-batch/independent-wasm-check.mjs --before=FROZEN_C1/pkg-reactive --candidate=dist/pkg-reactive --before-revision=c1d8fea1164e9d89edc8ff33af5a3bb608bc37d3 --candidate-revision=FULL_CANDIDATE_REVISION --output=FRESH_DIRECTORY_OUTSIDE_CHECKOUT
```

Paths resolve from the calling working directory. Both full revision arguments
must equal the actual clean, compiled build-info records. Each runtime directory
contains `caveat_runtime.js` and `caveat_runtime_bg.wasm`; `build-info.json` must
be in that directory or its parent. Their bytes must match the reactive artifact
hashes in the build record. The unchanged C1 Node/session loader is separately
hash-pinned. Inputs are checked again before reporting success.

The output directory must not exist and must be outside the checkout, keeping
generated programs and event traces out of frozen source inputs. It contains
11 generated programs, 22 exact event traces and an incremental `result.json`.
Invalid preflight inputs exit nonzero before creating output; a later test
failure exits nonzero and records its details. A rerun requires a new directory.

The unchanged matrix covers 4,096 logical steps on four sessions, 396 rejected
steps, 604 restored sessions requiring accepted no-op continuations, and 906
independently calculated retained-record closure checks. It compares outcomes,
full snapshot/view text, exact save strings and ordered archives across runtime
versions and immediate/delayed draining. It preserves the original sources,
seeds, refusal/rollback checks and archive assertions.

Reuse against C1 and a future candidate is a new artifact comparison using the
same externally authored cases, **not additional case coverage**. These finite
checks establish no universal equivalence, native timing/allocation result,
artifact authenticity, merge permission or release readiness.
