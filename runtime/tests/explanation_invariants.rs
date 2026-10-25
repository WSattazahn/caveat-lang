// Explanations 0.1 and 0.2 under random events: a shown binding's explanation
// is the union of its citations' grounds; without `because` it is the whole
// lineage; `because nothing` is empty; every evidence and caveat it cites is in
// the binding's lineage (selective, never fabricated); only shown bindings
// have one. Checked against the snapshot after every accepted event, with
// saves and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const SOURCE: &str = r#"
caveat flaky consequence material;
caveat dim consequence low;
claim ok;
evidence e from "tool";
renewable e limit 5;
evidence f from "operator";
evidence g from "third";
dim qualifies g;
state a = 0;
state b = 0;
state c = 0;
event see_e;
event see_f;
event see_g;
event new_e;
event more_a;
event more_b;
event more_c;
event mix;
event doubt_e;
event undoubt_e;
event doubt_f;
event retract_f;
on see_e reveal e supports ok;
on see_f reveal f;
on see_g reveal g;
on new_e renew e;
on more_a when observed(e) set a = qualified(a + 1, e);
on more_b when observed(f) set b = qualified(b + 1, f);
on more_c set c = c + 1;
on mix when observed(g) set c = qualified(c, g) + a;
on doubt_e when observed(e) qualify e with flaky;
on undoubt_e unqualify e with flaky;
on doubt_f when observed(f) qualify f with flaky;
on retract_f when observed(f) withdraw f because g;
bind view.whole = a + b + c;
bind view.none = a + b + c because nothing;
bind view.only_a = a + b + c because a;
bind view.both = a + b + c because a, b;
bind view.with_c = a + c because c;
bind view.cond = a when observed(e) because a;
bind view.cond_whole = a + b when observed(e) and observed(f);
bind view.late = qualified(a, e) when observed(e) because qualified(a, e);
bind view.gated = b when c > 2;
bind view.gated = a when c > 4;
"#;

const EVENTS: [&str; 12] = [
    "see_e",
    "see_f",
    "see_g",
    "new_e",
    "more_a",
    "more_b",
    "more_c",
    "mix",
    "doubt_e",
    "undoubt_e",
    "doubt_f",
    "retract_f",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn names(record: &Value, key: &str) -> BTreeSet<String> {
    record[key]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn union(records: &[&Value]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut evidence = BTreeSet::new();
    let mut caveats = BTreeSet::new();
    for record in records {
        evidence.extend(names(record, "evidence"));
        caveats.extend(names(record, "caveats"));
    }
    (evidence, caveats)
}

fn check(shown: &Value, at: &str) {
    let bindings = &shown["bindings"]["view"];
    let explanations = &shown["binding_explanations"]["view"];
    let lineages = &shown["binding_qualifications"]["view"];
    let grounds = |state: &str| shown["value_grounds"][state].clone();
    for property in [
        "whole",
        "none",
        "only_a",
        "both",
        "with_c",
        "cond",
        "cond_whole",
        "late",
        "gated",
    ] {
        let shown_now = bindings.get(property).is_some();
        let explained = explanations.get(property).is_some();
        assert_eq!(
            explained, shown_now,
            "{at}: {property} shown {shown_now} but explained {explained}"
        );
        if !shown_now {
            continue;
        }
        let explanation = &explanations[property];
        let lineage = &lineages[property];
        // Selective, never fabricated.
        assert!(
            names(explanation, "evidence").is_subset(&names(lineage, "evidence")),
            "{at}: {property} cites evidence it never read: {explanation} not in {lineage}"
        );
        assert!(
            names(explanation, "caveats").is_subset(&names(lineage, "caveats")),
            "{at}: {property} cites a caveat it never carried: {explanation} not in {lineage}"
        );
        let expected = match property {
            "whole" | "cond_whole" | "gated" => {
                (names(lineage, "evidence"), names(lineage, "caveats"))
            }
            "none" => (BTreeSet::new(), BTreeSet::new()),
            "only_a" | "cond" => union(&[&grounds("a")]),
            "both" => union(&[&grounds("a"), &grounds("b")]),
            "with_c" => union(&[&grounds("c")]),
            "late" => {
                // The current occurrence of e and the caveats it carries now,
                // the ones qualified(a, e) brings, together with a's grounds.
                let current = shown["renewals"]["e"]["occurrences"]
                    .as_array()
                    .unwrap()
                    .last()
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                let live = shown["relations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|edge| {
                        edge["relation"] == "qualifies" && edge["to"] == current.as_str()
                    })
                    .map(|edge| edge["from"].as_str().unwrap().to_string())
                    .filter(|caveat| {
                        let relations = shown["relations"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter(|edge| {
                                edge["relation"] == "qualifies"
                                    && edge["to"] == current.as_str()
                                    && edge["from"] == caveat.as_str()
                            })
                            .count();
                        let taken = shown["taken_back"].as_array().map_or(0, |list| {
                            list.iter()
                                .filter(|record| {
                                    record["evidence"] == current.as_str()
                                        && record["caveat"] == caveat.as_str()
                                })
                                .count()
                        });
                        relations > taken
                    })
                    .collect::<BTreeSet<_>>();
                let (mut evidence, mut caveats) = union(&[&grounds("a")]);
                evidence.insert(current);
                caveats.extend(live);
                (evidence, caveats)
            }
            _ => unreachable!(),
        };
        let actual = (
            names(explanation, "evidence"),
            names(explanation, "caveats"),
        );
        assert_eq!(actual, expected, "{at}: explanation of {property}: {explanation}\nlineage {lineage}\ngrounds a {} b {} c {}", grounds("a"), grounds("b"), grounds("c"));
    }
    // The last winning declaration supplies `gated`.
    let c = shown["values"]["c"]
        .as_f64()
        .or_else(|| shown["qualified_values"]["c"]["value"].as_f64())
        .unwrap_or(0.0);
    let a = shown["values"]["a"]
        .as_f64()
        .or_else(|| shown["qualified_values"]["a"]["value"].as_f64())
        .unwrap_or(0.0);
    let b = shown["values"]["b"]
        .as_f64()
        .or_else(|| shown["qualified_values"]["b"]["value"].as_f64())
        .unwrap_or(0.0);
    let gated = bindings.get("gated").map(|value| value.as_f64().unwrap());
    let expected = if c > 4.0 {
        Some(a)
    } else if c > 2.0 {
        Some(b)
    } else {
        None
    };
    assert_eq!(
        gated,
        expected,
        "{at}: gated with a {a} b {b} c {c}: {}",
        json!(bindings)
    );
}

#[test]
fn the_explanation_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn explanations_cite_what_their_citations_read_and_nothing_else() {
    let mut seed: u64 = 0x510e_527f_ade6_82d1;
    let mut accepted = 0;
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        check(&before, &format!("run {run} start"));
        for step in 0..40 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push("save");
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(SOURCE, &saved).unwrap_or_else(|error| {
                    panic!("run {run} step {step} restore: {error}: {trail:?}")
                });
                assert_eq!(
                    snapshot(&game),
                    before,
                    "run {run} step {step} after restore: {trail:?}"
                );
                continue;
            }
            let event = EVENTS[pick];
            trail.push(event);
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            if outcome["outcome"] != "accepted" {
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            check(
                &shown,
                &format!("run {run} step {step} after {event}: {trail:?}"),
            );
            // Every event's save restores, to the same save and snapshot (D021).
            let saved_after = game.save_json().unwrap();
            let restored_after = ReactiveSession::restore_json(SOURCE, &saved_after)
                .unwrap_or_else(|error| {
                    panic!("run {run} step {step}: restore after the event: {error}")
                });
            assert_eq!(
                restored_after.save_json().unwrap(),
                saved_after,
                "run {run} step {step}: save round trip after the event"
            );
            assert_eq!(
                snapshot(&restored_after),
                shown,
                "run {run} step {step}: snapshot after restore"
            );
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events");
    assert!(accepted > 5000);
}
