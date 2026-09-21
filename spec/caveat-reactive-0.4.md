# CAVEAT Reactive Profile 0.4 — source library and qualified text

This profile extends [Reactive Profile 0.3](caveat-reactive-0.3.md) with bounded
text expressions, lazy expression selection, and a standard library written in
Caveat. Numeric state, events, graph effects, and qualification metadata retain
their existing contracts. The additive snapshot schema remains
`caveat-reactive/0.1`.

The interpreter and primitive operations remain implemented in Rust. This is a
source standard library, not self-hosting, a module system, or arbitrary host
code execution.

## Text expressions and inferred function results

```caveat
fn current_label(speed) = "Current: " + number_text(speed);
fn positive(value) = value > 0;
fn reciprocal_or_zero(value) = if(value == 0, 0, 1 / value);

state elapsed = 0;
event tick dt min 0 max 0.1;
on tick set elapsed = elapsed + dt;
bind clock.text = time_text(90 - elapsed);
bind description.text = "Elapsed " + time_text(floor(elapsed));
```

Expression values are numbers, booleans, or text. The `+` operator adds two
numbers or concatenates two texts. It does not implicitly convert mixed types.
Equality and inequality compare operands of the same type, including exact text
contents. Ordered comparisons remain numeric. Text literals use the existing
Caveat quoting and escaping rules; whitespace, `when`, parentheses, semicolons,
and comment markers inside a literal remain literal text.

Pure functions keep the existing `fn name(parameters) = expression;` syntax.
Parameters are numeric. The compiler infers a number, boolean, or text return
type from the body and its callees, independent of declaration order. A function
returning text can be used in text concatenation or a binding. It cannot supply a
numeric state initializer, numeric event argument, numeric function parameter,
or a commitment's `using` expression. Function bodies cannot capture state or
query the graph; qualified numeric arguments carry their basis into the call.
Recursion, duplicate parameters, wrong arity, mixed result types, unknown names,
and intrinsic or standard-library shadowing are errors before execution.

## Lazy control expressions

```text
if(BOOLEAN_CONDITION, THEN_EXPRESSION, ELSE_EXPRESSION)
require(BOOLEAN_CONDITION, VALUE_EXPRESSION)
```

`if` evaluates its condition followed by exactly one branch. Both branches must
have the same inferred type. Both are validated, including branches not reached
at runtime. An unread branch contributes no value, runtime error, observation,
or qualification.

`require` evaluates its condition first. A false condition raises a requirement
error. A true condition evaluates and returns the value expression, which may be
numeric, boolean, or text. This is a computation precondition; it does not assert
that a Caveat claim is true or discharge a caveat.

Ordinary source-function arguments remain eager, including arguments ignored by
the body. Consequently `constant(1 / 0)` still errors even if `constant` returns a
literal, while `if(true, "ready", text(1 / 0))` returns `"ready"`.

## Source standard library

The runtime includes [runtime/prelude.cav](../runtime/prelude.cav) as source and
parses, validates, and expands its functions through the same machinery as
application functions. Applications automatically receive these definitions.
The Rust expression evaluator no longer contains algorithms for `abs`, `min`,
`max`, or `clamp`.

```caveat
fn abs(value) = if(value < 0, -value, value + 0);
fn min(left, right) = if(left < right, left, right);
fn max(left, right) = if(left > right, left, right);
fn clamp(value, lower, upper) = require(lower <= upper, min(max(value, lower), upper));
```

Reversed clamp bounds still reject the operation, and every supplied argument
still evaluates and carries its qualifications. Applications cannot override
these reserved definitions to bypass checks.

The source library also defines presentation policy:

| Function | Result |
| --- | --- |
| `number_text(value)` | Round to an integer and convert to text. |
| `percent_text(fraction)` | Multiply by 100, round, and append `%`. |
| `time_text(seconds)` | Clamp below at zero, round up to a whole second, and format `m:ss`. |
| `time_total_text(total)` | Internal formatting helper for a nonnegative whole-second total. |
| `time_second_text(seconds)` | Internal helper that prefixes seconds below 10 with `0`. |

All nine library function names, including the helpers, are reserved. Authors
can define additional functions under other names. For example,
`time_text(floor(elapsed))` displays elapsed time rounded down while the default
`time_text(remaining)` preserves countdown rounding up.

The generic primitives are `text(number)`, `floor(number)`, `ceil(number)`, and
`round(number)`, alongside existing arithmetic, `sqrt`, `sin`, `cos`, and
`atan2`. `round` rounds halfway values away from zero. `text` converts a finite
number to locale-independent decimal text, normalizing either signed zero to
`"0"`; it does not add padding, units, percentages, or clock punctuation. Those
policies are ordinary Caveat source. The browser copies the resulting binding
text into its display.

## Qualifications and atomicity

Text is evaluated as a value with qualifications. Concatenation unions both
operands' bases. Numeric formatting preserves the numeric operand's basis.
`if` includes its condition and selected branch; `require` includes its successful
condition and value. Function calls include every eagerly evaluated numeric
argument, even when the returned text is constant.

For example, if `speed` is a qualified sample, `current_label(speed)` carries its
evidence and caveats through `round`, `text`, and concatenation into
`binding_qualifications`. A qualified condition selecting a fixed warning label
also remains inspectable. Unselected future readings are never manufactured as
observations. Candidate binding guards continue to contribute even when they do
not win the binding selection.

These operations preserve computational dependence; formatted text is not a
truth assertion. They do not alter the meaning of `observed`, `examined`,
`committed`, or `reopened`, erase contradictory evidence, remove retained
caveats, or rewrite historical commitment bases.

Any runtime error, including a failed requirement or oversized text result,
rejects the complete event. Numeric values, graph effects, commitment bases,
cues, and all qualification metadata remain at the preceding snapshot.

## Bounds and compatibility

Each text literal or result is limited to 65,536 UTF-8 bytes. Concatenation checks
the resulting size before allocating it; excess text is rejected rather than
truncated. The existing expression-source limit of 65,536 bytes also applies.
Expression limits remain 1,024 tokens, 64 levels of nesting, and 4,096 expanded
nodes. Functions remain nonrecursive and bounded to 128 total definitions;
the nine standard-library definitions count toward that total, leaving 119
application definitions in this profile.

The existing per-value provenance limits remain 1,024 identifiers and 65,536
UTF-8 bytes of names. Existing numeric state, numeric event payloads, primitive
binding output, and historical qualification metadata remain compatible.
Earlier profiles describe their historical intrinsic implementation; this
profile replaces those four numeric algorithms with the source definitions
above while retaining their finite-number and argument-validation behavior.
