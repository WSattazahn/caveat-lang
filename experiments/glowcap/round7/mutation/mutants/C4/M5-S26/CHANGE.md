# C4 M5-S26

- File: `glowcap.cav`
- Line: 275
- Site: `bind journal.text_1 ... because journal_1`: add to this because an evidence the value never reads

Before:

```
bind journal.text_1 = journal_text(journal_1) because journal_1;
```

After:

```
bind journal.text_1 = journal_text(journal_1) because journal_1, taste_pit;
```
