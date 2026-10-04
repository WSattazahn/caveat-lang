# C1 M5-S17

- File: `glowcap.cav`
- Line: 195
- Site: `bind $m.why.absorb ... because nothing`: add to this because an evidence the value never reads

Before:

```
    bind $m.why.absorb = "Out of reach" when $m_out_of_reach because nothing;
```

After:

```
    bind $m.why.absorb = "Out of reach" when $m_out_of_reach because taste_pit;
```
