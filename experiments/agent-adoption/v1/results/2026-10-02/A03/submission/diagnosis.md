# Separate citation diagnostic

Installed version: 0.1.0-rc.7. This exercise is separate from first.cav and tracker.cav.

Invalid program (invalid-because.cav):

```caveat
claim ready;
evidence tool from "tool";
event observe;
on observe reveal tool;
bind result.value = qualified(85, tool) when observed(tool) because ready;

```

Exact installed CLI diagnostic (stderr, exit 2):

```text
<TRIAL_A03>\invalid-because.cav does not load: binding result.value because: ready is a declared claim, not a value citation; cite a state, a history read such as latest(STREAM), or an evidence-bearing expression that the value or its conditions actually read
```

A declared claim is not an evidence-bearing value citation. The corrected program cites the qualified expression actually used by the displayed value:

```caveat
claim ready;
evidence tool from "tool";
event observe;
on observe reveal tool;
bind result.value = qualified(85, tool) when observed(tool) because qualified(85, tool);

```

validate succeeds. After observe, explain shows result.value = 85 because tool. The guard delays the evidence-bearing value until neutral observation; it does not invent a support relation.

Before this binding exercise, I tried commit answer because ready using latest(checks), and then commit answer because latest(checks) using latest(checks). Both failed with unknown commit reason (receipts retained). Commit uses a reason such as enough; binding because takes value citations. I corrected the diagnostic exercise to test the intended claim/value-citation distinction.
