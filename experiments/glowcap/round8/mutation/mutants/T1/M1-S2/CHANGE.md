# T1 M1 S2

Site: canRetaste: includes(FADED) test -> false

File: `glowcap.ts`
Line: 189 (line numbers are those of the original file)

Line 189, before:

```
  return (s.evidenceCaveats[evidenceId('taste', id, m.life)] ?? []).includes(FADED);
```

Line 189, after:

```
  return false;
```

Choice: A caveat is removed from the list or push that attaches it (or the statement attaching it is removed); an `.includes(<caveat>)` test is replaced by `false` (as if the caveat were absent).
