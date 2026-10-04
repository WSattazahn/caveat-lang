# T1 M4-S1

- File: `glowcap.ts`
- Line: 296
- Site: recovery basis `s.sinceContradiction.slice(0, 2)`: reverse it

Before:

```
    const basis = s.sinceContradiction.slice(0, 2);
```

After:

```
    const basis = s.sinceContradiction.slice(0, 2).reverse();
```
