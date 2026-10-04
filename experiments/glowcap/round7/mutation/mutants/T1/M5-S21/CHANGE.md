# T1 M5-S21

- File: `glowcap.ts`
- Line: 266
- Site: journalTrust: journal entry because [...h.because]: add one evidence id the value never reads

Before:

```
  pushBounded(s.journal, { text: TRUST_TEXT[h.change], because: [...h.because] });
```

After:

```
  pushBounded(s.journal, { text: TRUST_TEXT[h.change], because: [...h.because, 'taste_pit'] });
```
