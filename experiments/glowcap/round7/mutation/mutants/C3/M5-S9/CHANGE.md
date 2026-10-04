# C3 M5-S9

- File: `glowcap.cav`
- Line: 213
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
        when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste;
```

After:

```
        when $m_taste == 2 and has_caveat($m_taste, tasted_in_dark) because $m_taste, absorb_$m;
```
