# C1 M5-S10

- File: `glowcap.cav`
- Line: 180
- Site: `bind journal.e4 ... because j4`: add to this because an evidence the value never reads

Before:

```
        when j4 > $m_entry_low and j4 < $m_entry_low + 10 because j4;
```

After:

```
        when j4 > $m_entry_low and j4 < $m_entry_low + 10 because j4, taste_pit;
```
