# Review: CR17–CR20 for the Glowcap beat

`BEAT.md` describes a small game feature built up by sixteen change requests.
`existing-scenarios.mjs` (base to CR12) and `existing-scenarios-r7.mjs` (CR13–CR16) hold its acceptance scenarios. A designer has written
four more requests, `REQUESTS.md`, with scenarios in `scenarios-r8.mjs`. Before
anyone implements them, check them. Nobody has implemented them yet.

Check, and report each problem with the request or scenario id and step:

1. **Every expectation follows from the requests and BEAT.md.** Trace each
   scenario by hand, step by step: timers, counts, labels, evidence ids,
   caveats, ordering, accepted and rejected events. Name each expectation you
   compute differently and show your trace.
2. **Ambiguity.** A point where two careful implementers could reasonably
   produce different views or accept different events.
3. **Bounds.** Anything that grows without a stated bound, or a bound without
   a stated behaviour at the limit.
4. **Conflicts** with earlier requests or with existing scenarios that are
   still in force (an existing scenario applies through its `until`, or to the
   end when it has none).
5. **Format.** The module imports nothing, ids continue from S51, `since` is
   cr17–cr20, steps use the documented forms.

Do not edit REQUESTS.md or scenarios-r8.mjs. Write `REVIEW.md` in this
directory: a numbered list of problems, each with what you found and the
smallest fix you would suggest, or "No problems found" for a check that passed.

Your final answer: the number of problems found per check.
