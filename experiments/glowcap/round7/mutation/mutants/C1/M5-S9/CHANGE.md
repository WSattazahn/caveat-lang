# C1 M5-S9

- File: `glowcap.cav`
- Line: 178
- Site: `bind journal.e3 ... because j3`: add to this because an evidence the value never reads

Before:

```
        when j3 > $m_entry_low and j3 < $m_entry_low + 10 because j3;
```

After:

```
        when j3 > $m_entry_low and j3 < $m_entry_low + 10 because j3, taste_pit;
```
