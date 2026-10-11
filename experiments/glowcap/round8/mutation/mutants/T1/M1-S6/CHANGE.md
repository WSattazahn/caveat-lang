# T1 M1 S6

Site: tick fade: drop FADED from the caveat map entry ([...cav, FADED] -> [...cav])

File: `glowcap.ts`
Line: 379 (line numbers are those of the original file)

Line 379, before:

```
          if (!cav.includes(FADED)) s.evidenceCaveats[ev] = [...cav, FADED];
```

Line 379, after:

```
          if (!cav.includes(FADED)) s.evidenceCaveats[ev] = [...cav];
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
