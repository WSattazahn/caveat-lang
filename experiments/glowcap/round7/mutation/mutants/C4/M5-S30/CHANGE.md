# C4 M5-S30

- File: `glowcap.cav`
- Line: 279
- Site: `bind journal.text_5 ... because journal_5`: add to this because an evidence the value never reads

Before:

```
bind journal.text_5 = journal_text(journal_5) because journal_5;
```

After:

```
bind journal.text_5 = journal_text(journal_5) because journal_5, taste_pit;
```
