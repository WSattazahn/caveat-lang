# C1 M5-S29

- File: `glowcap.cav`
- Line: 208
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Duskcap — avoid" when $m_taste == 2 because $m_taste;
```

After:

```
    bind $m.label = "Duskcap — avoid" when $m_taste == 2 because $m_taste, absorb_$m;
```
