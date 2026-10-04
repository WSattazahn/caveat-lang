# C3 M5-S7

- File: `glowcap.cav`
- Line: 210
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
        when $m_taste == 1 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
        when $m_taste == 1 and has_caveat($m_taste, tasted_in_dark) because $m_taste, absorb_$m;
```
