# C2 M1-S9

- File: `glowcap.cav`
- Line: 278
- Site: `bind belief.observations = ... because supported, contradicted;` (caveat-only binding read for belief.caveats): remove the `supported` citation argument

Before:

```
bind belief.observations = supported + contradicted because supported, contradicted;
```

After:

```
bind belief.observations = supported + contradicted because contradicted;
```
