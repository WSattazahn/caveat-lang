# C4 M5-S7

- File: `glowcap.cav`
- Line: 213
- Site: `bind $m.label ... because $m_known`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Duskcap — avoid" when $m_known == 2 because $m_known;
```

After:

```
    bind $m.label = "Duskcap — avoid" when $m_known == 2 because $m_known, absorb_$m;
```
