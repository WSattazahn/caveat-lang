# C1 M5-S8

- File: `glowcap.cav`
- Line: 176
- Site: `bind journal.e2 ... because j2`: add to this because an evidence the value never reads

Before:

```
        when j2 > $m_entry_low and j2 < $m_entry_low + 10 because j2;
```

After:

```
        when j2 > $m_entry_low and j2 < $m_entry_low + 10 because j2, taste_pit;
```
