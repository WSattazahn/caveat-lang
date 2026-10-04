# C4 M5-S6

- File: `glowcap.cav`
- Line: 212
- Site: `bind $m.label ... because $m_known`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Glowcap" when $m_known == 1 because $m_known;
```

After:

```
    bind $m.label = "Glowcap" when $m_known == 1 because $m_known, absorb_$m;
```
