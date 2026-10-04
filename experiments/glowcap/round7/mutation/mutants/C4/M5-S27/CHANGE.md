# C4 M5-S27

- File: `glowcap.cav`
- Line: 276
- Site: `bind journal.text_2 ... because journal_2`: add to this because an evidence the value never reads

Before:

```
bind journal.text_2 = journal_text(journal_2) because journal_2;
```

After:

```
bind journal.text_2 = journal_text(journal_2) because journal_2, taste_pit;
```
