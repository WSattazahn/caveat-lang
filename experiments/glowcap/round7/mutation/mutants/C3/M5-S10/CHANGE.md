# C3 M5-S10

- File: `glowcap.cav`
- Line: 215
- Site: `bind $m.label ... because $m_taste`: add to this because an evidence the value never reads

Before:

```
        when $m_taste == 1 and has_caveat($m_taste, taste_faded) because $m_taste;
```

After:

```
        when $m_taste == 1 and has_caveat($m_taste, taste_faded) because $m_taste, absorb_$m;
```
