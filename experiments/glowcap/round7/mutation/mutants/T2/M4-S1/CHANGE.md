# T2 M4-S1

- File: `glowcap.ts`
- Line: 236
- Site: recovery basis `s.supportSinceContradiction.slice(0, 2)` passed to commit: reverse it

Before:

```
      commit(s, s.supportSinceContradiction.slice(0, 2));
```

After:

```
      commit(s, s.supportSinceContradiction.slice(0, 2).reverse());
```
