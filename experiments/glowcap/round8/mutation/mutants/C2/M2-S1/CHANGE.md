# C2 M2 S1

Site: decision.caveats: read the basis evidence's current caveats instead of the frozen grounds caveats

File: `adapter.mjs`
Line: 120 (line numbers are those of the original file)

Line 120, before:

```
    out.decision.caveats = caveatNames(grounds?.caveats ?? []);
```

Line 120, after:

```
    out.decision.caveats = caveatNames([...new Set(Object.values(ex).flatMap((props) => Object.values(props)).filter((x) => (x.evidence ?? []).some((n) => (grounds?.evidence ?? []).includes(n))).flatMap((x) => x.caveats ?? []))]);
```

Choice: Choice (same in C1-C3): decision.caveats is computed from live state instead of the commitment's frozen grounds: the union of the caveats of every current binding explanation that cites any of the basis evidence. This is the closest live reading the view offers, since it has no per-evidence caveats.
