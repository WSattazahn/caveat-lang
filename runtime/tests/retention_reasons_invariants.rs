// Retention Reasons 0.1 under random events: `retaining C because E`
// resolves E to its current occurrence, which must be observed or the event
// refuses as evaluation/unobserved_evidence; the reason joins the decision's
// grounds with the caveats it carries and the lineage; the commit records
// each retained caveat with its reason's occurrence, frozen with the
// commitment; withdrawing the reason makes rests_on_withdrawn true; the
// caveat is retained exactly as without `because`; and a program that names
// no reason has no `retention_reasons`. Checked after every event, with
// saves and restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const SOURCE: &str = r#"
caveat gaps consequence material;
caveat drift consequence low;
caveat flaky consequence material;
claim ok;
evidence note from "scope note";
renewable note limit 4;
evidence memo from "memo";
evidence e from "tool";
evidence op from "operator";
state v = 0;
decisions d limit 8;
decisions plain limit 8;
event see_note;
event see_memo;
event see_e;
event see_op;
event new_note;
event flag_note;
event flag_memo;
event unflag_memo;
event retract_note;
event set_v;
event decide;
event decide_two;
event decide_plain;
event reconsider;
event reconsider_plain;
on see_note reveal note;
on see_memo reveal memo;
on see_e reveal e supports ok;
on see_op reveal op;
on new_note renew note;
on flag_note when observed(note) qualify note with flaky;
on flag_memo when observed(memo) qualify memo with flaky;
on unflag_memo unqualify memo with flaky;
on retract_note when observed(note) and observed(op) withdraw note because op;
on set_v when observed(e) set v = qualified(1, e);
on decide when observed(e) commit d because enough using v retaining gaps because note;
on decide_two when observed(e) commit d because enough using v retaining gaps because note, drift because memo;
on decide_plain when observed(e) commit plain because enough using v retaining gaps;
on reconsider when committed(d) reopen d because e;
on reconsider_plain when committed(plain) reopen plain because e;
bind view.stale_d = if(rests_on_withdrawn(d), 1, 0) when committed(d);
"#;

const EVENTS: [&str; 18] = [
    "see_note",
    "see_memo",
    "see_e",
    "see_op",
    "new_note",
    "flag_note",
    "flag_memo",
    "unflag_memo",
    "retract_note",
    "set_v",
    "decide",
    "decide_two",
    "decide_plain",
    "reconsider",
    "reconsider_plain",
    "decide",
    "reconsider",
    "set_v",
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

fn observed(shown: &Value, occurrence: &str) -> bool {
    shown["observations"]
        .as_array()
        .is_some_and(|list| list.iter().any(|item| item == occurrence))
}

fn current_note(shown: &Value) -> String {
    strings(&shown["renewals"]["note"]["occurrences"])
        .last()
        .unwrap()
        .clone()
}

fn live_caveats(shown: &Value, occurrence: &str) -> Vec<String> {
    let relations = shown["relations"].as_array().unwrap();
    let mut caveats = relations
        .iter()
        .filter(|edge| edge["relation"] == "qualifies" && edge["to"] == occurrence)
        .map(|edge| edge["from"].as_str().unwrap().to_string())
        .filter(|caveat| {
            let count = relations
                .iter()
                .filter(|edge| {
                    edge["relation"] == "qualifies"
                        && edge["to"] == occurrence
                        && edge["from"] == caveat.as_str()
                })
                .count();
            let taken = shown["taken_back"].as_array().map_or(0, |list| {
                list.iter()
                    .filter(|record| {
                        record["evidence"] == occurrence && record["caveat"] == caveat.as_str()
                    })
                    .count()
            });
            count > taken
        })
        .collect::<Vec<_>>();
    caveats.sort();
    caveats.dedup();
    caveats
}

fn in_force(shown: &Value, series: &str) -> bool {
    shown["decision_series"][series]["current"]
        .as_str()
        .is_some_and(|current| {
            shown["commitments"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["action"] == current && item["open"] == false)
        })
}

#[test]
fn the_retention_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn retention_reasons_join_the_grounds_and_are_recorded_frozen() {
    let mut seed: u64 = 0x4a7d_1ed4_14d0_4f49;
    let (mut accepted, mut recorded_commits, mut refused_unobserved, mut stale_through_reason) =
        (0, 0, 0, 0);
    let mut records: BTreeMap<String, Value> = BTreeMap::new();
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        records.clear();
        for step in 0..50 {
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
            let save_before = game.save_json().unwrap();
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, "{}").unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            if matches!(event, "decide" | "decide_two")
                && observed(&before, "e")
                && !in_force(&before, "d")
            {
                let note = current_note(&before);
                let reasons: Vec<(&str, String)> = if event == "decide" {
                    vec![("gaps", note.clone())]
                } else {
                    vec![("gaps", note.clone()), ("drift", "memo".to_string())]
                };
                let unobserved = reasons
                    .iter()
                    .find(|(_, occurrence)| !observed(&before, occurrence));
                if let Some((_, occurrence)) = unobserved {
                    refused_unobserved += 1;
                    assert_eq!(
                        (outcome["origin"].as_str(), outcome["code"].as_str()),
                        (Some("evaluation"), Some("unobserved_evidence")),
                        "{at}: reason {occurrence} unobserved: {outcome}"
                    );
                    assert_eq!(
                        game.save_json().unwrap(),
                        save_before,
                        "{at}: a refused reason changed the save"
                    );
                    continue;
                }
                assert_eq!(outcome["outcome"], "accepted", "{at}: {outcome}");
                recorded_commits += 1;
                let shown = snapshot(&game);
                let current = shown["decision_series"]["d"]["current"]
                    .as_str()
                    .unwrap()
                    .to_string();
                let grounds = &shown["commitment_grounds"][&current];
                let basis = &shown["commitment_bases"][&current]["provenance"];
                let mut expected_record = serde_json::Map::new();
                for (caveat, occurrence) in &reasons {
                    assert!(
                        strings(&grounds["evidence"]).contains(occurrence),
                        "{at}: reason {occurrence} is not in the grounds {grounds}"
                    );
                    assert!(
                        strings(&basis["evidence"]).contains(occurrence),
                        "{at}: reason {occurrence} is not in the lineage {basis}"
                    );
                    for carried in live_caveats(&before, occurrence) {
                        assert!(
                            strings(&grounds["caveats"]).contains(&carried),
                            "{at}: the reason's caveat {carried} is not in the grounds {grounds}"
                        );
                    }
                    assert!(
                        strings(&grounds["caveats"]).contains(&caveat.to_string()),
                        "{at}: retained {caveat} is not in the grounds {grounds}"
                    );
                    expected_record.insert(caveat.to_string(), json!(occurrence));
                }
                assert_eq!(
                    shown["retention_reasons"][&current],
                    Value::Object(expected_record.clone()),
                    "{at}: {}",
                    shown["retention_reasons"]
                );
                // Retained exactly as without a reason: the commitment lists it.
                let commitment = shown["commitments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|item| item["action"] == current)
                    .unwrap();
                for (caveat, _) in &reasons {
                    assert!(
                        strings(&commitment["retained"]).contains(&caveat.to_string()),
                        "{at}: {commitment}"
                    );
                }
                records.insert(current, Value::Object(expected_record));
            }
            if event == "decide_plain" && outcome["outcome"] == "accepted" {
                // No reason named: no record for it. The rule fires only
                // once e is observed; an accepted event before that commits nothing.
                let shown = snapshot(&game);
                if let Some(current) = shown["decision_series"]["plain"]["current"].as_str() {
                    assert!(
                        shown["retention_reasons"].get(current).is_none(),
                        "{at}: {}",
                        shown["retention_reasons"]
                    );
                }
            }
            if outcome["outcome"] != "accepted" {
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            for (commitment, record) in &records {
                assert_eq!(
                    &shown["retention_reasons"][commitment], record,
                    "{at}: the record of {commitment} changed"
                );
            }
            // Withdrawing the reason makes the revision rest on withdrawn evidence.
            if let Some(current) = shown["decision_series"]["d"]["current"].as_str() {
                let withdrawn = shown["withdrawals"]
                    .as_array()
                    .map(|list| {
                        list.iter()
                            .map(|record| record["evidence"].as_str().unwrap().to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let grounds = strings(&shown["commitment_grounds"][current]["evidence"]);
                let stale = grounds.iter().any(|name| withdrawn.contains(name));
                assert_eq!(
                    shown["bindings"]["view"]["stale_d"].as_f64(),
                    Some(if stale { 1.0 } else { 0.0 }),
                    "{at}: rests_on_withdrawn(d)"
                );
                if stale
                    && records.get(current).is_some_and(|record| {
                        record
                            .as_object()
                            .unwrap()
                            .values()
                            .any(|reason| withdrawn.contains(&reason.as_str().unwrap().to_string()))
                    })
                {
                    stale_through_reason += 1;
                }
            }
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events, {recorded_commits} commits with reasons, {refused_unobserved} refused for an unobserved reason, {stale_through_reason} readings stale through a withdrawn reason");
    assert!(
        recorded_commits > 200 && refused_unobserved > 100 && stale_through_reason > 100,
        "{recorded_commits} / {refused_unobserved} / {stale_through_reason}"
    );
}
