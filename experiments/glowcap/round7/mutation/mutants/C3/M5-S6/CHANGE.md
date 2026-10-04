# C3 M5-S6

- File: `glowcap.cav`
- Line: 208
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Glowcap" when $m_taste == 1 because $m_taste;
```

After:

```
    bind $m.label = "Glowcap" when $m_taste == 1 because $m_taste, absorb_$m;
```
