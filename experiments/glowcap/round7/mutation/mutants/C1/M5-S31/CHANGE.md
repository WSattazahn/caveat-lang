# C1 M5-S31

- File: `glowcap.cav`
- Line: 212
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
        when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
        when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste, absorb_$m;
```
