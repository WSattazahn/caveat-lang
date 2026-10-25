# Retracted studies

When a study is retracted, every conclusion drawn from it should be flagged,
and every decision already made on it should still show what it rested on.
This example replays a real case: the two Surgisphere papers retracted by the
Lancet and the New England Journal of Medicine on 4 June 2020.

- `retraction.cav`: the studies, the conclusions drawn from them and the
  actions taken on them.
- `retraction.scenarios.json`: four scenarios that state its behavior.
- `retraction.events.jsonl`: the timeline below, in order.

## The case

| Date (2020) | What happened | Event |
| --- | --- | --- |
| 1 May | NEJM publishes Mehra et al.: in 8,910 patients from 169 hospitals, ACE inhibitors were not linked to higher in-hospital death | `published nejm` |
| 22 May | The Lancet publishes Mehra et al.: in 96,032 patients, hydroxychloroquine (HCQ) was linked to higher in-hospital death | `published lancet` |
| 25 May | WHO pauses the HCQ arm of its Solidarity trial after the Lancet paper | `pause_announced` |
| 2 and 3 June | NEJM, then the Lancet, publish expressions of concern | `concern nejm`, `concern lancet` |
| 3 June | WHO resumes the arm: its data safety monitoring committee found no reason to change the protocol | `dsmb_review` |
| 4 June | Both papers are retracted: the authors could not audit the registry's data | `retracted lancet`, `retracted nejm` |
| 5 June | The randomized RECOVERY trial reports no mortality benefit from HCQ | `recovery_reported` |

Two kinds of decision stand in for people, and are not historical records.
`harm_signal`, `acei_continue` and `no_benefit` are the conclusions a reader
drew from each study. `guidance` is an illustrative hospital, not a real one,
that stopped giving HCQ to inpatients on the Lancet paper and later based the
same guidance on RECOVERY.

## Run it

In a project with this package installed, copy the folder out of
`node_modules/caveat-lang/examples/retractions` and run:

```sh
npx --no-install caveat-lang test retraction.scenarios.json
npx --no-install caveat-lang explain retraction.cav retraction.events.jsonl
npx --no-install caveat-lang dependents retraction.cav lancet_registry retraction.events.jsonl
```

## What the language does

| Need | How | Scenario |
| --- | --- | --- |
| A retraction flags every conclusion that rested on the study | `withdraw lancet_registry because lancet_retraction`, then `reopen D` for each decision where `rests_on_withdrawn(D)` | R01, R03 |
| A flagged conclusion keeps the grounds it was made on | `harm_signal@1` stays "based on lancet_registry", and `explain` adds "lancet_registry has since been withdrawn at #10 because lancet_retraction" | R01 |
| A decision already reversed for another reason still shows what it rested on | WHO's pause was reopened by the monitoring committee before the retraction; `rests_on_withdrawn(revision(pause_arm, 1))` still finds the retracted study | R01 |
| An expression of concern adds doubt without retracting anything | `qualify lancet_registry with data_unverified`; the conclusion stays in force and the display says it is in doubt | R01 |
| What didn't rest on the study stands | `no_benefit@1` rests only on `recovery_hcq`; `guidance@2` reaches the same answer on new grounds, and `guidance@1` keeps its Lancet basis | R01 |
| A retracted study supports no new decision | every commit rule that reads a study first checks `withdrawn(...)` | R02, R04 |
| Retracting one study touches only what rested on it | after `retracted nejm`, the HCQ conclusion is still in force | R03 |

`dependents retraction.cav lancet_registry retraction.events.jsonl` lists
everything the Lancet paper reached: the claim it supported, the three
decisions based on it and the values and displays that cite it.

## What the author writes by hand

`npx --no-install caveat-lang check --withdrawals retraction.cav` reports the
first two of these when a rule is missing: C010 for a commit that nothing
guards, C011 for a decision no withdrawal reopens
([check](../../docs/reference/spec/caveat-check-0.1.md)).

- **A guard on every commit.** Withdrawn evidence still has a value, and a
  commit rule that reads it records a decision whose grounds carry the
  `withdrawn` caveat. Each rule here checks `withdrawn(...)` first (R04
  refuses a new pause after the retraction).
- **A reopen rule for each decision.** A withdrawal reopens nothing by
  itself. The `retracted` event has one `reopen` rule per decision, guarded
  by `rests_on_withdrawn`, and a decision added later needs its own. WHO's
  pause has one too, although the monitoring committee reopened it first.
- **Grounds versus lineage.** `guidance@2` is based only on `recovery_hcq`,
  but its guard read `reopened(guidance)`, so `explain` lists the retracted
  study as something that "could also have been influenced" it. A report of
  what a decision rests on should read the grounds, not the lineage.

See [Caveat on one page](../../docs/REFERENCE.md) for each form.

## Sources

- Mehra MR, Desai SS, Ruschitzka F, Patel AN. Hydroxychloroquine or
  chloroquine with or without a macrolide for treatment of COVID-19: a
  multinational registry analysis. Lancet 2020 (retracted).
  https://doi.org/10.1016/S0140-6736(20)31180-6. Expression of concern:
  https://doi.org/10.1016/S0140-6736(20)31290-3. Retraction:
  https://doi.org/10.1016/S0140-6736(20)31324-6.
- Mehra MR, Desai SS, Kuy S, Henry TD, Patel AN. Cardiovascular disease,
  drug therapy, and mortality in Covid-19. N Engl J Med 2020 (retracted).
  https://doi.org/10.1056/NEJMoa2007621. Retraction:
  https://doi.org/10.1056/NEJMc2021225.
- RECOVERY Collaborative Group. Effect of hydroxychloroquine in hospitalized
  patients with Covid-19. N Engl J Med 2020;383:2030-2040.
  https://doi.org/10.1056/NEJMoa2022926. Chief investigators' statement of
  5 June 2020:
  https://www.ox.ac.uk/news/2020-06-05-no-clinical-benefit-use-hydroxychloroquine-hospitalised-patients-covid-19
- Retraction Watch, 4 June 2020, on both retractions:
  https://retractionwatch.com/2020/06/04/lancet-retracts-controversial-hydroxychloroquine-study/
