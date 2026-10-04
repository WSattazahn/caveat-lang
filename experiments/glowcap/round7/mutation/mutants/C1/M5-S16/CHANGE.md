# C1 M5-S16

- File: `glowcap.cav`
- Line: 194
- Site: `bind $m.why.absorb ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.absorb = "Known duskcap" when $m_taste == 2 because $m_taste;
```

After:

```
    bind $m.why.absorb = "Known duskcap" when $m_taste == 2 because $m_taste, absorb_$m;
```
