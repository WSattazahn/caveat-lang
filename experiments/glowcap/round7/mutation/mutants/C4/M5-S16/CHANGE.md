# C4 M5-S16

- File: `glowcap.cav`
- Line: 232
- Site: `bind $m.why_absorb ... because $m_known`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Known duskcap" when $m_known == 2 because $m_known;
```

After:

```
    bind $m.why_absorb = "Known duskcap" when $m_known == 2 because $m_known, absorb_$m;
```
