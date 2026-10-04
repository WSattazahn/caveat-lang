# C4 M5-S29

- File: `glowcap.cav`
- Line: 278
- Site: `bind journal.text_4 ... because journal_4`: add to this because an evidence the value never reads

Before:

```
bind journal.text_4 = journal_text(journal_4) because journal_4;
```

After:

```
bind journal.text_4 = journal_text(journal_4) because journal_4, taste_pit;
```
