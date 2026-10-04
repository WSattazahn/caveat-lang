# C1 M5-S12

- File: `glowcap.cav`
- Line: 184
- Site: `bind journal.e6 ... because j6`: add to this because an evidence the value never reads

Before:

```
        when j6 > $m_entry_low and j6 < $m_entry_low + 10 because j6;
```

After:

```
        when j6 > $m_entry_low and j6 < $m_entry_low + 10 because j6, taste_pit;
```
