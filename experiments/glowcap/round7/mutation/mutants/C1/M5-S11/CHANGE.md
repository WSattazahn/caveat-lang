# C1 M5-S11

- File: `glowcap.cav`
- Line: 182
- Site: `bind journal.e5 ... because j5`: add to this because an evidence the value never reads

Before:

```
        when j5 > $m_entry_low and j5 < $m_entry_low + 10 because j5;
```

After:

```
        when j5 > $m_entry_low and j5 < $m_entry_low + 10 because j5, taste_pit;
```
