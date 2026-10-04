# C1 M5-S7

- File: `glowcap.cav`
- Line: 174
- Site: `bind journal.e1 ... because j1`: add to this because an evidence the value never reads

Before:

```
        when j1 > $m_entry_low and j1 < $m_entry_low + 10 because j1;
```

After:

```
        when j1 > $m_entry_low and j1 < $m_entry_low + 10 because j1, taste_pit;
```
