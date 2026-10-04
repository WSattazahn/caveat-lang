# C3 M5-S24

- File: `glowcap.cav`
- Line: 264
- Site: `bind $j.text ... because $j_cite`: add to this because an evidence the value never reads

Before:

```
    bind $j.text = "Absorbed the " + $j_name + " mushroom: it gave light." when $j_what == 1 and $j_cite > 0 because $j_cite;
```

After:

```
    bind $j.text = "Absorbed the " + $j_name + " mushroom: it gave light." when $j_what == 1 and $j_cite > 0 because $j_cite, taste_pit;
```
