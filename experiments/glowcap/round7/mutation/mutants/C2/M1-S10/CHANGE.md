# C2 M1-S10

- File: `glowcap.cav`
- Line: 278
- Site: `bind belief.observations = ... because supported, contradicted;` (caveat-only binding read for belief.caveats): remove the `contradicted` citation argument

Before:

```
bind belief.observations = supported + contradicted because supported, contradicted;
```

After:

```
bind belief.observations = supported + contradicted because supported;
```
