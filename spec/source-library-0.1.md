# Source function library API 0.1

The library makes existing pure Caveat functions callable from Rust and
WebAssembly. It adds no new function syntax and creates no reactive session.
A host compiles functions once, then invokes them with numeric inputs. The
ordinary evaluator supplies type checks, bounded work, and qualification
propagation.

## Native API

```rust
use caveat_runtime::source_library::{FunctionValue, SourceLibrary};
let library = SourceLibrary::from_module_source(
    "fn travel_time(distance) = clamp(distance / 3, 0.45, 2.8);"
)?;
let result = library.call_numbers("travel_time", &[6.0])?;
assert_eq!(result.value, FunctionValue::Number(2.0));
```

`from_module_source` accepts only pure function declarations. `from_source`
parses a complete program and extracts its functions. Both add the standard
prelude, reject duplicate or invalid definitions (including unused ones), and
compile the functions. Extraction does not validate other program declarations:
a game must still pass its corresponding session/runtime validation.

`call(name, &[Tracked<f64>])` returns `Result<Tracked<FunctionValue>, String>`.
`FunctionValue` is `Number`, `Bool`, or `Text`. `call_numbers` accepts plain
numbers and returns the same tracked result. `signatures()` returns function
names, ordered numeric parameters, and result types. Libraries are immutable
and cheaply cloned through shared compiled storage.

All inputs are checked before evaluating the function. Unknown names, wrong
arity, nonfinite or out-of-range numbers, invalid provenance, failed `require`,
division by zero, and exceeded work limits return errors. Every accepted
argument contributes its provenance, including arguments ignored by the body.
Boolean and text results retain that metadata too. A failed call has no mutable
library state to publish.

Functions may call pure functions and calculate numbers, booleans, and text.
They cannot read globals, mutate state, query the graph or histories, execute
procedures, or recurse.

## WebAssembly API

```js
import init, { WebSourceLibrary } from './pkg/caveat_runtime.js';
await init();
const library = new WebSourceLibrary(source);
const result = JSON.parse(library.call('travel_time', '[6]'));
// { value: 2, provenance: { evidence: [], caveats: [] } }
library.free();
```

The constructor extracts functions like native `from_source`. `call` accepts a
JSON numeric array and returns JSON containing `value` and `provenance`.
`signatures()` returns JSON describing the callable functions. Invalid calls
throw the bridge's existing Rust error. The caller owns and frees the handle.

This bridge accepts plain numbers, not evidence declarations. Native tracked
calls preserve supplied provenance and check its bounds, but cannot
authenticate caller-supplied labels. Authoritative graph qualifications remain
the graph runtime's responsibility.

## Limits

| Resource | Bound |
| --- | --- |
| Supplied source | 1 MiB |
| Functions including prelude | 128 |
| Parameters per function | 32 |
| Retained compiled nodes across the library | 32,768 |
| Retained compiled literal/name storage | 1 MiB |
| Numeric arguments and results | Finite, absolute value at most 1e12 |
| WebAssembly argument JSON | 4,096 bytes |

Existing expression expansion, evaluation, text, and provenance limits also
apply. Compilation counts expanded storage across the whole library, including
duplicated expansions. A bounded function does not permit unbounded repetition
of its compiled body elsewhere in the library.

## Application contracts

The Door compiles motion policy from its supplied scenario. Caveat chooses
movement duration, facing/animation thresholds, signed angle wrapping, turn
durations, and opening angle/duration. Rust retains coordinate/path geometry
and typed animation command construction. The adapter checks required function
signatures and rejects invalid durations. `Web3DSession` stages the complete
action, including source calls, before publication.

The Last Beacon passes factual flags/counts from the committed game snapshot
to source functions. Caveat chooses feedback category, headings, watch number,
and continuation label. These results are display-only. A feedback failure
occurs after the game action committed and must say so accurately; it cannot
claim that the action rolled back or repeat it to repair the display.

These are different host transaction contracts using the same pure library.
The library cannot make an external host operation atomic on its own. Source
edits can change policy without changing the adapters; existing material and
shader handling remains with the renderer.
