# T2 M1-S2

- File: `glowcap.ts`
- Line: 318
- Site: taste: `observe(s, tasteId, kind, s.glow > 0 ? [] : ['tasted_in_dark'])`: remove 'tasted_in_dark' from the caveat map entry

Before:

```
      observe(s, tasteId, kind, s.glow > 0 ? [] : ['tasted_in_dark']);
```

After:

```
      observe(s, tasteId, kind, []);
```
