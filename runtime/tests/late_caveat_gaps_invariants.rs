//! A randomized check of C009's detector, `ReactiveSession::late_caveat_gaps`
//! (spec/caveat-check-0.1.md, C009; spec/caveat-caveat-sources-0.1.md).
//!
//! A model reads the snapshot: a state's value, its grounds and a shown
//! binding lack a gap exactly when, for every caveat a `qualify` effect
//! names that the value lacks, no evidence it lists has a live relation for
//! that caveat (more `C qualifies E` relations than take-back records). On a
//! session 1.1 played the model and the detector both find nothing. Then a
//! save is made 1.0-shaped by stripping a late caveat from random states, as
//! a 1.0.x runtime wrote a value set under a committed decision after the
//! `qualify`, and restored: the detector must report exactly what the model
//! reads, after the restore, after every later event, and across a further
//! save and restore, until an event sets the value again or a take-back
//! marks the relation.
use caveat_runtime::reactive::{LateCaveatGap, ReactiveSession};
use serde_json::Value;
use std::collections::BTreeSet;

const SOURCE: &str = r#"
claim c;
caveat flaky consequence material;
caveat shaky consequence low;
caveat dusty consequence low;
evidence e from "tool";
renewable e limit 64;
evidence probe from "probe";
dusty qualifies probe;
readings s from probe limit 64;
evidence op from "operator check";
decisions trust limit 64;
budget 100000;
state v = 0;
state recalled = 0;
state latest_trust = 0;
state first = 0;
state sampled = 0;
state mixed = 0;
event report;
event flag;
event unflag;
event shake;
event decide;
event doubt;
event recall;
event read;
event see_op;
event withdraw;
event flag_s;
event unflag_s;
event decide_s;
event mix;
on report when observed(e) renew e;
on report reveal e;
on report set v = qualified(1, e);
on flag when observed(e) qualify e with flaky;
on unflag unqualify e with flaky;
on shake when observed(e) qualify e with shaky;
on decide when observed(e) commit trust because enough using v;
on doubt when committed(trust) and observed(e) reopen trust because e;
on recall when committed(trust) set recalled = 1;
on recall when committed(trust) set latest_trust = latest(trust);
on recall when history_count(trust) > 0 set first = history_at(trust, 0);
on read when observed(e) sample s = 1 supports c;
on read when has_sample(s) set sampled = latest(s);
on see_op reveal op;
on withdraw when observed(op) and observed(e) withdraw e because op;
on flag_s when has_sample(s) qualify latest(s) with flaky;
on unflag_s unqualify latest(s) with flaky;
on decide_s when has_sample(s) commit trust because enough using latest(s);
on mix set mixed = recalled + sampled;
bind view.recalled = recalled;
bind view.reopened = if(reopened(trust), 1, 0);
bind view.first = first + latest_trust;
bind view.mixed = mixed + v;
"#;

/// The caveats a `qualify` effect in SOURCE names. `dusty` is declared only.
const LATE: [&str; 2] = ["flaky", "shaky"];

const EVENTS: [&str; 14] = [
    "report", "flag", "unflag", "shake", "decide", "doubt", "recall", "read", "see_op", "withdraw",
    "flag_s", "unflag_s", "decide_s", "mix",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn strings(list: &Value) -> Vec<String> {
    list.as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `caveat qualifies occurrence` is live: more relations than
/// take-back records (Take-back 0.1).
fn live(shown: &Value, caveat: &str, occurrence: &str) -> bool {
    let relations = shown["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| {
            edge["relation"] == "qualifies" && edge["from"] == caveat && edge["to"] == occurrence
        })
        .count();
    let marked = shown["taken_back"]
        .as_array()
        .map(|records| {
            records
                .iter()
                .filter(|record| record["caveat"] == caveat && record["evidence"] == occurrence)
                .count()
        })
        .unwrap_or(0);
    relations > marked
}

/// The gaps the model reads in one provenance.
fn gaps_in(
    shown: &Value,
    place: &'static str,
    name: &str,
    provenance: &Value,
) -> Vec<LateCaveatGap> {
    let caveats = strings(&provenance["caveats"]);
    let evidence = strings(&provenance["evidence"]);
    let mut found = Vec::new();
    for caveat in LATE {
        if caveats.iter().any(|carried| carried == caveat) {
            continue;
        }
        let reached: Vec<String> = evidence
            .iter()
            .filter(|occurrence| live(shown, caveat, occurrence))
            .cloned()
            .collect();
        if !reached.is_empty() {
            found.push(LateCaveatGap {
                place,
                name: name.to_string(),
                caveat: caveat.to_string(),
                evidence: reached,
            });
        }
    }
    found
}

/// Every gap the model reads: each state's value and grounds, each shown
/// binding, in the detector's order (states by name, value then grounds;
/// bindings by target and property).
fn model(shown: &Value) -> Vec<LateCaveatGap> {
    let mut found = Vec::new();
    let states = shown["qualified_values"].as_object().unwrap();
    for (name, state) in states {
        found.extend(gaps_in(shown, "state", name, &state["provenance"]));
        found.extend(gaps_in(
            shown,
            "grounds",
            name,
            &shown["value_grounds"][name],
        ));
    }
    if let Some(targets) = shown["binding_qualifications"].as_object() {
        for (target, properties) in targets {
            for (property, lineage) in properties.as_object().unwrap() {
                found.extend(gaps_in(
                    shown,
                    "binding",
                    &format!("{target}.{property}"),
                    lineage,
                ));
            }
        }
    }
    found
}

fn key(gap: &LateCaveatGap) -> (String, String, String, Vec<String>) {
    (
        gap.place.to_string(),
        gap.name.clone(),
        gap.caveat.clone(),
        gap.evidence.clone(),
    )
}

fn same(found: &[LateCaveatGap], expected: &[LateCaveatGap], at: &str) {
    let found_keys: BTreeSet<_> = found.iter().map(key).collect();
    let expected_keys: BTreeSet<_> = expected.iter().map(key).collect();
    assert_eq!(
        found_keys, expected_keys,
        "{at}: the detector and the model differ"
    );
    assert_eq!(
        found.len(),
        found_keys.len(),
        "{at}: the detector repeats a gap"
    );
}

/// Strip `caveat` from a state's lineage and grounds in a save, as a 1.0.x
/// runtime saved a value it set under a committed decision after the
/// `qualify`. Returns whether anything was removed.
fn strip(save: &mut Value, state: &str, caveat: &str) -> bool {
    let mut removed = false;
    let Some(record) = save["states"].get_mut(state) else {
        return false;
    };
    for part in ["lineage", "grounds"] {
        let Some(part) = record.get_mut(part).and_then(Value::as_object_mut) else {
            continue;
        };
        for list in ["caveats", "kept"] {
            if let Some(items) = part.get_mut(list).and_then(Value::as_array_mut) {
                let before = items.len();
                items.retain(|item| item != caveat);
                removed |= items.len() < before;
                if items.is_empty() {
                    part.remove(list);
                }
            }
        }
    }
    removed
}

#[test]
fn the_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn the_detector_reports_exactly_the_values_a_1_0_save_left_without_a_late_caveat() {
    let mut seed: u64 = 0x3c6e_f372_fe94_f82b;
    let (
        mut accepted,
        mut clean_checks,
        mut strips,
        mut reported,
        mut on_grounds,
        mut on_bindings,
        mut closed_by_set,
        mut closed_by_take_back,
        mut round_trips,
    ) = (0, 0, 0, 0, 0, 0, 0, 0, 0);
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail: Vec<String> = Vec::new();
        // Whether the session holds a value a strip left without its caveat.
        let mut stripped = false;
        let mut open_before: Vec<LateCaveatGap> = Vec::new();
        for _ in 0..50 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 3);
            let at = |trail: &[String], what: &str| format!("run {run} after {trail:?} at {what}");
            if pick == EVENTS.len() {
                // A save and restore in place keeps every gap: no repair on load.
                let before = game.late_caveat_gaps();
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(SOURCE, &saved)
                    .unwrap_or_else(|error| panic!("{}: {error}", at(&trail, "save")));
                trail.push("save".into());
                same(
                    &game.late_caveat_gaps(),
                    &before,
                    &at(&trail, "the restored save"),
                );
                round_trips += 1;
            } else if pick > EVENTS.len() {
                // A 1.0-shaped save: a late caveat stripped from some states.
                let mut save: Value = serde_json::from_str(&game.save_json().unwrap()).unwrap();
                let names: Vec<String> = save["states"]
                    .as_object()
                    .map(|states| states.keys().cloned().collect())
                    .unwrap_or_default();
                let mut what = Vec::new();
                for name in names {
                    seed = seed
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1_442_695_040_888_963_407);
                    let choice = (seed >> 33) % 3;
                    if choice == 2 {
                        continue;
                    }
                    let caveat = LATE[choice as usize];
                    if strip(&mut save, &name, caveat) {
                        what.push(format!("{name}-{caveat}"));
                    }
                }
                if what.is_empty() {
                    continue;
                }
                trail.push(format!("strip {}", what.join(",")));
                game = ReactiveSession::restore_json(SOURCE, &save.to_string())
                    .unwrap_or_else(|error| panic!("{}: {error}", at(&trail, "strip")));
                stripped = true;
                strips += 1;
            } else {
                let event = EVENTS[pick];
                trail.push(event.into());
                if game.dispatch_json(event, "{}").is_err() {
                    continue;
                }
                accepted += 1;
                // Every event's save restores, to the same snapshot (D021).
                let saved_game = game.save_json().unwrap();
                let restored_game = ReactiveSession::restore_json(SOURCE, &saved_game)
                    .unwrap_or_else(|error| {
                        panic!("run {run}: {trail:?}: restore after the event: {error}")
                    });
                assert_eq!(
                    restored_game.save_json().unwrap(),
                    saved_game,
                    "run {run}: {trail:?}: save round trip"
                );
                assert_eq!(
                    snapshot(&restored_game),
                    snapshot(&game),
                    "run {run}: {trail:?}: snapshot after restore"
                );
            }
            let shown = snapshot(&game);
            let expected = model(&shown);
            let found = game.late_caveat_gaps();
            let here = at(&trail, "the check");
            if !stripped {
                // A session 1.1 played holds no gap: the invariant of Caveat
                // Sources 0.1 read from the snapshot, and the detector agrees.
                assert_eq!(
                    expected,
                    [],
                    "{here}: the model found a gap in a 1.1 session"
                );
                assert_eq!(
                    found,
                    [],
                    "{here}: the detector found a gap in a 1.1 session"
                );
                clean_checks += 1;
            } else {
                same(&found, &expected, &here);
                if !found.is_empty() {
                    reported += 1;
                }
                on_grounds += found.iter().filter(|gap| gap.place == "grounds").count();
                on_bindings += found.iter().filter(|gap| gap.place == "binding").count();
                let last = trail.last().unwrap();
                for gap in &open_before {
                    if found.iter().any(|now| key(now) == key(gap)) {
                        continue;
                    }
                    if last == "unflag" || last == "unflag_s" {
                        closed_by_take_back += 1;
                    } else if EVENTS.contains(&last.as_str()) {
                        closed_by_set += 1;
                    }
                }
            }
            open_before = found;
        }
    }
    eprintln!(
        "{accepted} accepted events, {clean_checks} clean checks, {strips} strips, {reported} checks with gaps ({on_grounds} on grounds, {on_bindings} on bindings), {closed_by_set} gaps closed by an event, {closed_by_take_back} by a take-back, {round_trips} save round trips"
    );
    assert!(
        clean_checks > 2000
            && strips > 200
            && reported > 400
            && on_grounds > 50
            && on_bindings > 100
            && closed_by_set > 50
            && closed_by_take_back > 20
            && round_trips > 300,
        "{clean_checks} / {strips} / {reported} / {on_grounds} / {on_bindings} / {closed_by_set} / {closed_by_take_back} / {round_trips}"
    );
}
