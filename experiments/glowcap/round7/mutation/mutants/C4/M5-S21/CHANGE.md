# C4 M5-S21

- File: `glowcap.cav`
- Line: 237
- Site: `bind $m.why_taste ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.why_taste = "Out of reach" when $m_out_of_reach because nothing;
```

After:

```
    bind $m.why_taste = "Out of reach" when $m_out_of_reach because taste_pit;
```
