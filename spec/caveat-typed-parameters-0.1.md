# CAVEAT Typed Parameters 0.1

Event parameters were numbers with bounds. A host that wanted to say "the
slime absorbed the pool mushroom, and it was a duskcap" had to know a
per-mushroom event name and encode the kind as `0` or `1`. The source, in
turn, needed one event per mushroom.

```caveat
event absorb target kind mushroom, sort in glowcap duskcap;

for mushroom as $m {
    on absorb when target == $index and sort == sort.duskcap set heavy = 20;
};
```

```json
{"target": "pool", "sort": "duskcap"}
```

## Syntax

```text
NAME kind KIND      -- an entity of that kind, sent by name
NAME in A B C...    -- one of these names, sent as text
NAME min N max M    -- a number, as before
```

## Meaning

A typed parameter is still a number inside the program: the position of the
name, counted from 1.

- `NAME kind KIND` accepts the name of any entity declared `kind KIND`. Its
  value is that entity's position in declaration order, the same count that
  `$index` gives inside a `for KIND` block. Declaring the parameter is an
  error if no entity has that kind.
- `NAME in A B C` accepts one of the listed names. Its value is the name's
  position in the list.
- For every typed parameter, `NAME.MEMBER` is a numeric constant in the
  source: `sort.duskcap` is `2` above, and `target.pool` is `2`. Two events
  may declare the same parameter with the same members; the same name with a
  different value is an error.

A JSON payload may send the name or the position. The name is resolved
against the event's signature before dispatch; an unknown name is rejected
with the accepted names listed, and the event has no effect. A number for a
typed parameter, or a name for a numeric one, follows the same rule: numbers
are bounds-checked as before, and a name for a numeric parameter is rejected.
The native `dispatch` takes positions.

## Snapshot

Each parameter in `events[].parameters` gains a `domain` when it is typed:

```json
{"name": "target", "min": 1, "max": 3, "domain": {"entity": {"kind": "mushroom", "members": ["cave", "pool", "ruin"]}}}
{"name": "sort", "min": 1, "max": 2, "domain": {"member": {"members": ["glowcap", "duskcap"]}}}
```

Numeric parameters are unchanged, so existing hosts read what they read
before.
