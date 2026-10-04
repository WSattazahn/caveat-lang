# C2 M5-S21

- File: `glowcap.cav`
- Line: 237
- Site: `bind $m.why_absorb ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_absorb = "Known duskcap" when $m_taste == 2 because $m_taste;
```

After:

```
    bind $m.why_absorb = "Known duskcap" when $m_taste == 2 because $m_taste, absorb_$m;
```
