# CAVEAT-RS — Bootstrap Systems Profile 0.1

**Status:** executable bootstrap experiment.

CAVEAT-RS asks whether Rust-like systems programming becomes more useful when provisional commitment, retained uncertainty, and reopening are first-class program semantics rather than comments or ad-hoc application state.

It is intentionally bootstrapped by transpiling to Rust. Rust remains the memory-safety and code-generation substrate; CAVEAT-RS adds epistemic correctness constraints above it. The experiment should be judged by whether those added semantics prove useful, not by whether the surface syntax looks novel.

## 1. Design rule

CAVEAT-RS is not "Rust with different punctuation."

A construct belongs in this profile only when it expresses information ordinary Rust does not preserve directly: what a value was committed to despite, what uncertainty was retained, and why a prior commitment became open again.

Ordinary ownership, borrowing, lifetimes, algebraic data types, control flow, and machine-level representation remain Rust responsibilities during bootstrap.

## 2. Bootstrap pipeline

```
.cvr source
  -> CAVEAT-RS semantic pass
  -> generated Rust
  -> rustc / borrow checker / LLVM / WASM or native target
```

The first compiler is therefore a semantic preprocessor/transpiler, not a Rust replacement compiler.

## 3. First native type

```
provisional<T>
```

A provisional value contains:

- the committed value `T`;
- the caveats explicitly retained at commitment time;
- whether the commitment has subsequently been reopened;
- the causes that reopened it.

Generated Rust uses `Provisional<T>`.

A provisional value is deliberately not the same as `Option<T>`, `Result<T, E>`, or a confidence score. A value can exist and be acted on while unresolved qualifications remain attached to it.

## 4. Function-local caveats

```
caveat stale_sensor;
caveat incomplete_coverage;
```

A caveat declaration creates a symbolic qualification available to commitments in that function. Bootstrap 0.1 caveats are function-scoped and carry identity, not yet consequence/attention metadata.

Declaring the same caveat twice in one function is a compile-time CAVEAT-RS error.

## 5. Commitment

Direct function commitment:

```
fn choose(ok: bool) -> provisional<bool> {
    caveat stale_sensor;
    commit ok retaining stale_sensor;
}
```

Binding a provisional commitment for later revision:

```
fn choose(ok: bool) -> provisional<bool> {
    caveat stale_sensor;
    let decision = commit ok retaining stale_sensor;
    return decision;
}
```

Rules:

- `commit` is valid only in a function whose return type is `provisional<...>`;
- every retained caveat must already be declared in that function;
- a provisional-returning function must construct at least one commitment;
- ordinary Rust expressions remain the value expression during bootstrap.

## 6. Reopening

```
reopen decision because stale_sensor;
```

Rules:

- the cause must be a declared caveat;
- the target must be a provisional binding created by `let NAME = commit ...`;
- reopening does not erase or replace the committed value;
- reopening appends the cause to history and marks the commitment open.

This preserves the distinction between "the value was never chosen" and "the value was chosen, then later evidence made that choice questionable."

## 7. Rust-compatible core

Bootstrap 0.1 deliberately passes ordinary Rust statements and expressions through to generated Rust. That means the Rust compiler continues to enforce:

- ownership and moves;
- borrowing and aliasing;
- lifetimes;
- type checking;
- exhaustiveness;
- thread-safety traits;
- target/code-generation constraints.

CAVEAT-RS adds checks before rustc; it does not weaken Rust's checks.

## 8. Example

```
fn route(sensor_ok: bool) -> provisional<bool> {
    caveat stale_sensor;
    caveat camera_gap;

    let decision = commit sensor_ok retaining stale_sensor, camera_gap;
    reopen decision because camera_gap;

    return decision;
}
```

Conceptual generated Rust:

```rust
fn route(sensor_ok: bool) -> Provisional<bool> {
    let mut decision =
        Provisional::new(sensor_ok, &["stale_sensor", "camera_gap"]);
    decision.reopen("camera_gap");
    return decision;
}
```

## 9. Static errors required in 0.1

The CAVEAT-RS pass must reject:

1. duplicate caveat declarations;
2. commitment in a non-provisional function;
3. retained undeclared caveats;
4. reopening because of an undeclared caveat;
5. reopening a name not created by a provisional commitment binding;
6. provisional-returning functions that never construct a commitment;
7. malformed CAVEAT-RS extension statements.

These errors occur before rustc.

## 10. Deliberate limitations of bootstrap 0.1

- free functions only;
- one-line function signatures;
- caveats are function-scoped;
- no native claim/evidence graph syntax yet;
- no effect system yet;
- no async-specific semantics yet;
- no independent ownership checker;
- generated Rust is the executable artifact.

These are staging constraints, not the intended endpoint.

## 11. Next earned extensions

Only after 0.1 is executable and tested:

- typed evidence/provenance values;
- `supports`, `opposes`, and `qualifies` relationships in systems code;
- consequence/attention on caveats;
- effect annotations for observations and external state;
- reopening triggers driven by observations;
- world/resource invariants;
- a compiler IR shared with ordinary CAVEAT;
- source maps and diagnostics;
- eventually, evaluation of whether CAVEAT-RS warrants its own backend.

## 12. Success criterion

CAVEAT-RS earns further work if a systems program can preserve and statically check reasoning history that plain Rust would normally encode manually, while still inheriting Rust's memory/type safety.

If the profile reduces to a thin wrapper around `Result`, metadata structs, or comments without providing useful static guarantees, record that result rather than pretending it is a new language.
