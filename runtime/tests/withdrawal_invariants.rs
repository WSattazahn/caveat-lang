// Withdrawal 0.1 under random events: a withdrawal is one authoritative
// record per occurrence, the first one kept; `withdrawn(E)` and
// `withdrawn(latest(S))` ask that record for the current occurrence or
// reading, so a renewal or a new reading starts clean; a current value
// carries `withdrawn` exactly when its grounds hold a withdrawn occurrence;
// and the graph and the records agree, each record with its
// `withdrawn qualifies E` relation and each relation with its record.
// Checked against the snapshot after every accepted event, with saves and
// restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

const SOURCE: &str = r#"
caveat flaky consequence material;
claim ok;
evidence e from "tool";
renewable e limit 5;
evidence f from "operator";
evidence h from "sensor";
readings s from h limit 6;
state v_e = 0;
state v_s = 0;
state v_h = 0;
state v_mix = 0;
event see_e;
event see_f;
event see_h;
event new_e;
event read;
event retract_e;
event retract_s;
event retract_h;
event set_ve;
event set_vs;
event set_vh;
event set_mix;
event doubt_e;
event undoubt_e;
on see_e reveal e supports ok;
on see_f reveal f;
on see_h reveal h;
on new_e renew e;
on read sample s = v_s + 1 supports ok;
on retract_e when observed(e) withdraw e because f;
on retract_s when has_sample(s) withdraw latest(s) because f;
on retract_h when observed(h) withdraw h because f;
on set_ve when observed(e) set v_e = qualified(1, e);
on set_vs when has_sample(s) set v_s = latest(s);
on set_vh when observed(h) set v_h = qualified(1, h);
on set_mix when observed(e) and has_sample(s) set v_mix = qualified(1, e) + latest(s);
on doubt_e when observed(e) qualify e with flaky;
on undoubt_e unqualify e with flaky;
bind view.wd_e = if(withdrawn(e), 1, 0);
bind view.wd_s = if(withdrawn(latest(s)), 1, 0) when has_sample(s);
bind view.wd_h = if(withdrawn(h), 1, 0);
bind view.hc_ve = if(has_caveat(v_e, withdrawn), 1, 0);
bind view.hc_vs = if(has_caveat(v_s, withdrawn), 1, 0);
bind view.hc_vh = if(has_caveat(v_h, withdrawn), 1, 0);
bind view.hc_mix = if(has_caveat(v_mix, withdrawn), 1, 0);
bind view.obs_e = if(observed(e), 1, 0);
bind view.obs_h = if(observed(h), 1, 0);
"#;

const EVENTS: [&str; 14] = [
    "see_e",
    "see_f",
    "see_h",
    "new_e",
    "read",
    "retract_e",
    "retract_s",
    "retract_h",
    "set_ve",
    "set_vs",
    "set_vh",
    "set_mix",
    "doubt_e",
    "undoubt_e",
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn bound(shown: &Value, name: &str) -> Option<f64> {
    shown["bindings"]["view"][name].as_f64()
}

fn withdrawn_set(shown: &Value) -> BTreeSet<String> {
    shown["withdrawals"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|record| record["evidence"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn current_e(shown: &Value) -> String {
    shown["renewals"]["e"]["occurrences"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

fn latest_s(shown: &Value) -> Option<String> {
    shown["reading_streams"]["s"]["current"]
        .as_str()
        .map(str::to_string)
}

fn check(shown: &Value, at: &str) {
    let withdrawn = withdrawn_set(shown);
    // Records and graph agree.
    let relations = shown["relations"].as_array().unwrap();
    let qualified_withdrawn = relations
        .iter()
        .filter(|edge| edge["from"] == "withdrawn" && edge["relation"] == "qualifies")
        .map(|edge| edge["to"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        qualified_withdrawn, withdrawn,
        "{at}: the graph's withdrawn relations and the records disagree"
    );
    let records = shown["withdrawals"].as_array().map(Vec::len).unwrap_or(0);
    assert_eq!(
        records,
        withdrawn.len(),
        "{at}: an occurrence is withdrawn twice: {}",
        shown["withdrawals"]
    );
    // The predicates ask about the current occurrence and reading.
    let flag = |truth: bool| Some(if truth { 1.0 } else { 0.0 });
    assert_eq!(
        bound(shown, "wd_e"),
        flag(withdrawn.contains(&current_e(shown))),
        "{at}: withdrawn(e) with {withdrawn:?}"
    );
    assert_eq!(
        bound(shown, "wd_h"),
        flag(withdrawn.contains("h")),
        "{at}: withdrawn(h)"
    );
    match latest_s(shown) {
        Some(reading) => assert_eq!(
            bound(shown, "wd_s"),
            flag(withdrawn.contains(&reading)),
            "{at}: withdrawn(latest(s)) for {reading}"
        ),
        None => assert_eq!(
            bound(shown, "wd_s"),
            None,
            "{at}: withdrawn(latest(s)) shown with no reading"
        ),
    }
    // A current value carries `withdrawn` exactly when its grounds hold a
    // withdrawn occurrence.
    for (state, binding) in [
        ("v_e", "hc_ve"),
        ("v_s", "hc_vs"),
        ("v_h", "hc_vh"),
        ("v_mix", "hc_mix"),
    ] {
        let grounds = &shown["value_grounds"][state];
        let evidence = grounds["evidence"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|item| item.as_str().unwrap().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let carries = grounds["caveats"]
            .as_array()
            .is_some_and(|list| list.iter().any(|caveat| caveat == "withdrawn"));
        let rests_on_withdrawn = evidence
            .iter()
            .any(|occurrence| withdrawn.contains(occurrence));
        assert_eq!(
            carries, rests_on_withdrawn,
            "{at}: {state} grounds {grounds} against withdrawals {withdrawn:?}"
        );
        assert_eq!(
            bound(shown, binding),
            flag(carries),
            "{at}: has_caveat({state}, withdrawn)"
        );
    }
}

#[test]
fn the_withdrawal_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn withdrawals_keep_one_record_and_reach_exactly_what_rests_on_them() {
    let mut seed: u64 = 0x9b05_688c_2b3e_6c1f;
    let (mut accepted, mut withdrawals, mut repeats) = (0, 0, 0);
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        check(&before, &format!("run {run} start"));
        for step in 0..45 {
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
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            check(&shown, &at);
            let was = withdrawn_set(&before);
            let now = withdrawn_set(&shown);
            // A retract whose guard held withdrew its target, or found it
            // withdrawn already; one whose guard did not hold did nothing.
            let target = match event {
                "retract_e" if bound(&before, "obs_e") == Some(1.0) => Some(current_e(&before)),
                "retract_s" => latest_s(&before),
                "retract_h" if bound(&before, "obs_h") == Some(1.0) => Some("h".to_string()),
                _ => None,
            };
            if let Some(target) = target {
                if was.contains(&target) {
                    // Already withdrawn: the first record stays, nothing else happens.
                    repeats += 1;
                    assert_eq!(
                        shown["withdrawals"], before["withdrawals"],
                        "{at}: a repeated withdrawal changed the records"
                    );
                } else {
                    withdrawals += 1;
                    let mut expected = was.clone();
                    expected.insert(target.clone());
                    assert_eq!(
                        now, expected,
                        "{at}: withdrawals after withdrawing {target}"
                    );
                    let record = shown["withdrawals"].as_array().unwrap().last().unwrap();
                    assert_eq!(record["evidence"], target.as_str(), "{at}");
                    assert_eq!(record["because"], "f", "{at}: the reason");
                    assert_eq!(record["event"], event, "{at}");
                    assert_eq!(record["sequence"], shown["sequence"], "{at}");
                    let effects = shown["effects"].as_array().unwrap();
                    assert!(
                        effects.iter().any(|effect| effect["kind"] == "withdraw"
                            && effect["evidence"] == target.as_str()),
                        "{at}: no withdraw effect: {effects:?}"
                    );
                }
            } else {
                assert_eq!(now, was, "{at}: {event} changed the withdrawals");
            }
            // A renewal or a new reading starts clean.
            if event == "new_e" {
                assert_eq!(
                    bound(&shown, "wd_e"),
                    Some(0.0),
                    "{at}: a fresh occurrence is withdrawn"
                );
                assert_eq!(
                    bound(&shown, "obs_e"),
                    Some(0.0),
                    "{at}: a fresh occurrence is observed"
                );
            }
            if event == "read" {
                assert_eq!(
                    bound(&shown, "wd_s"),
                    Some(0.0),
                    "{at}: a new reading is withdrawn"
                );
            }
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
    eprintln!("{accepted} accepted events, {withdrawals} withdrawals, {repeats} repeated");
    assert!(
        withdrawals > 400 && repeats > 300,
        "the alphabet barely withdrew: {withdrawals} / {repeats}"
    );
}
