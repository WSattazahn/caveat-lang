# C2 M5-S1

- File: `glowcap.cav`
- Line: 41
- Site: `bind journal_$index.text ... because journal_cite_$index`: add to this because an evidence the value never reads

Before:

```
    bind journal_$index.text = journal_text(journal_cite_$index, journal_who_$index) because journal_cite_$index;
```

After:

```
    bind journal_$index.text = journal_text(journal_cite_$index, journal_who_$index) because journal_cite_$index, taste_pit;
```
