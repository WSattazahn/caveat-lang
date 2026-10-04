# C4 M5-S31

- File: `glowcap.cav`
- Line: 280
- Site: `bind journal.text_6 ... because journal_6`: add to this because an evidence the value never reads

Before:

```
bind journal.text_6 = journal_text(journal_6) because journal_6;
```

After:

```
bind journal.text_6 = journal_text(journal_6) because journal_6, taste_pit;
```
