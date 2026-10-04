# C2 M5-S13

- File: `glowcap.cav`
- Line: 226
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
    bind $m.label = "Probably a duskcap (taste has faded)" when $m_taste == 2 and has_caveat($m_taste, taste_faded) because $m_taste;
```

After:

```
    bind $m.label = "Probably a duskcap (taste has faded)" when $m_taste == 2 and has_caveat($m_taste, taste_faded) because $m_taste, absorb_$m;
```
