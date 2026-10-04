# C1 M5-S21

- File: `glowcap.cav`
- Line: 199
- Site: `bind $m.why.taste ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.taste = "Out of reach" when $m_out_of_reach because nothing;
```

After:

```
    bind $m.why.taste = "Out of reach" when $m_out_of_reach because taste_pit;
```
