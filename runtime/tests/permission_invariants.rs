// Permission 0.1 under random events: a `permitted by` commit is refused as
// `policy/not_permitted` exactly when the grant is missing, withdrawn, or,
// with `for X`, not equal to X, resolving named evidence to its current
// occurrence and `latest(STREAM)` to the current reading; the refusal rolls
// the event back; a permitted commitment records a frozen permission
// record (grant, scope, caveats then) that later grants, withdrawals and
// changes to X never rewrite, named by the journal's `permitted_by`; the
// grant is in the commitment's lineage and not in its grounds;
// `permission_withdrawn(D)` asks about the recorded grant, not the latest
// one. Checked against the snapshot after every event, with saves and
// restores in place.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

const SOURCE: &str = r#"
caveat flaky consequence material;
claim ok;
evidence e from "tool";
evidence g from "approver";
renewable g limit 5;
evidence h from "head";
readings heads from h limit 6;
readings approvals from h limit 6;
decisions plain limit 6;
decisions scoped limit 6;
decisions by_reading limit 6;
state a = 0;
state head = 0;
event see_e;
event see_g;
event new_g;
event doubt_g;
event retract_g;
event read_head v min 0 max 9;
event approve v min 0 max 9;
event retract_approval;
event decide_plain;
event decide_scoped;
event decide_reading;
event reopen_plain;
event reopen_scoped;
event reopen_reading;
on see_e reveal e supports ok;
on see_g reveal g;
on new_g renew g;
on doubt_g when observed(g) qualify g with flaky;
on retract_g when observed(g) and observed(e) withdraw g because e;
on read_head sample heads = v supports ok;
on read_head set head = v;
on approve sample approvals = v supports ok;
on retract_approval when has_sample(approvals) and observed(e) withdraw latest(approvals) because e;
on decide_plain when observed(e) commit plain because enough using qualified(a, e) permitted by g;
on decide_scoped when observed(e) commit scoped because enough using qualified(a, e) permitted by latest(approvals) for head;
on decide_reading when observed(e) commit by_reading because enough using qualified(a, e) permitted by latest(approvals);
on reopen_plain when committed(plain) reopen plain because e;
on reopen_scoped when committed(scoped) reopen scoped because e;
on reopen_reading when committed(by_reading) reopen by_reading because e;
bind view.revoked_plain = if(permission_withdrawn(plain), 1, 0) when committed(plain);
bind view.revoked_scoped = if(permission_withdrawn(scoped), 1, 0) when committed(scoped);
bind view.revoked_reading = if(permission_withdrawn(by_reading), 1, 0) when committed(by_reading);
"#;

const EVENTS: [(&str, &str); 20] = [
    ("see_e", "{}"),
    ("see_g", "{}"),
    ("new_g", "{}"),
    ("doubt_g", "{}"),
    ("retract_g", "{}"),
    ("read_head", r#"{"v": 3}"#),
    ("read_head", r#"{"v": 5}"#),
    ("approve", r#"{"v": 3}"#),
    ("approve", r#"{"v": 5}"#),
    ("approve", r#"{"v": 7}"#),
    ("retract_approval", "{}"),
    ("decide_plain", "{}"),
    ("decide_scoped", "{}"),
    ("decide_reading", "{}"),
    ("reopen_plain", "{}"),
    ("reopen_scoped", "{}"),
    ("reopen_reading", "{}"),
    ("decide_plain", "{}"),
    ("decide_scoped", "{}"),
    ("decide_reading", "{}"),
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

fn withdrawn(shown: &Value, occurrence: &str) -> bool {
    shown["withdrawals"]
        .as_array()
        .is_some_and(|list| list.iter().any(|record| record["evidence"] == occurrence))
}

fn observed(shown: &Value, occurrence: &str) -> bool {
    shown["observations"]
        .as_array()
        .is_some_and(|list| list.iter().any(|item| item == occurrence))
}

fn current_g(shown: &Value) -> String {
    strings(&shown["renewals"]["g"]["occurrences"])
        .last()
        .unwrap()
        .clone()
}

fn latest_approval(shown: &Value) -> Option<String> {
    shown["reading_streams"]["approvals"]["current"]
        .as_str()
        .map(str::to_string)
}

fn value_of_reading(shown: &Value, reading: &str) -> f64 {
    shown["reading_streams"]["approvals"]["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == reading)
        .unwrap()["value"]
        .as_f64()
        .unwrap()
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

/// What the spec says the commit's permission check finds, from the
/// snapshot before the event: Some(reason) for a denial, None for a grant.
fn denial(before: &Value, series: &str) -> Option<&'static str> {
    match series {
        "plain" => {
            let grant = current_g(before);
            if !observed(before, &grant) {
                Some("missing")
            } else if withdrawn(before, &grant) {
                Some("withdrawn")
            } else {
                None
            }
        }
        "scoped" | "by_reading" => {
            let Some(reading) = latest_approval(before) else {
                return Some("missing");
            };
            if withdrawn(before, &reading) {
                Some("withdrawn")
            } else if series == "scoped"
                && value_of_reading(before, &reading)
                    != before["values"]["head"].as_f64().unwrap_or(0.0)
            {
                Some("scope")
            } else {
                None
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn the_permission_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn permitted_commits_are_granted_refused_and_recorded_as_the_spec_says() {
    let mut seed: u64 = 0x2b99_2ddf_a232_49d6;
    let mut denials = BTreeMap::<String, usize>::new();
    let (mut grants, mut accepted) = (0, 0);
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
                trail.push(("save", ""));
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
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let save_before = game.save_json().unwrap();
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            let series = match event {
                "decide_plain" => Some("plain"),
                "decide_scoped" => Some("scoped"),
                "decide_reading" => Some("by_reading"),
                _ => None,
            };
            if let Some(series) = series {
                // The rule's own guard first; then in-force; then permission.
                let guard = observed(&before, "e");
                if guard && in_force(&before, series) {
                    assert_eq!(outcome["code"], "decision_in_force", "{at}: {outcome}");
                } else if guard {
                    match denial(&before, series) {
                        Some(reason) => {
                            *denials.entry(format!("{series}/{reason}")).or_default() += 1;
                            assert_eq!(
                                (outcome["origin"].as_str(), outcome["code"].as_str()),
                                (Some("policy"), Some("not_permitted")),
                                "{at}: expected a {reason} denial: {outcome}"
                            );
                            assert_eq!(
                                game.save_json().unwrap(),
                                save_before,
                                "{at}: a denial changed the save"
                            );
                        }
                        None => {
                            grants += 1;
                            assert_eq!(
                                outcome["outcome"], "accepted",
                                "{at}: expected a grant: {outcome}"
                            );
                            let shown = snapshot(&game);
                            let current = shown["decision_series"][series]["current"]
                                .as_str()
                                .unwrap()
                                .to_string();
                            let record = &shown["commitment_permissions"][&current];
                            let expected_grant = if series == "plain" {
                                current_g(&before)
                            } else {
                                latest_approval(&before).unwrap()
                            };
                            assert_eq!(record["grant"], expected_grant.as_str(), "{at}: {record}");
                            if series == "scoped" {
                                let head = before["values"]["head"].as_f64().unwrap_or(0.0);
                                assert_eq!(
                                    record["scope"],
                                    serde_json::json!({"granted": head, "required": head}),
                                    "{at}: {record}"
                                );
                            } else {
                                assert!(record.get("scope").is_none(), "{at}: {record}");
                            }
                            // The caveats the grant carried then.
                            let carried = before["relations"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .filter(|edge| {
                                    edge["relation"] == "qualifies"
                                        && edge["to"] == expected_grant.as_str()
                                })
                                .map(|edge| edge["from"].as_str().unwrap().to_string())
                                .collect::<Vec<_>>();
                            let mut carried_sorted = carried.clone();
                            carried_sorted.sort();
                            let mut recorded = strings(&record["caveats"]);
                            recorded.sort();
                            assert_eq!(recorded, carried_sorted, "{at}: the record's caveats");
                            // The journal names it; the grant is lineage, not grounds.
                            let entry = shown["decision_journal"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .rev()
                                .find(|entry| {
                                    entry["commitment"] == current.as_str()
                                        && entry["change"] == "committed"
                                })
                                .unwrap();
                            assert_eq!(
                                entry["permitted_by"],
                                expected_grant.as_str(),
                                "{at}: {entry}"
                            );
                            assert!(
                                !strings(&shown["commitment_grounds"][&current]["evidence"])
                                    .contains(&expected_grant),
                                "{at}: the grant entered the grounds"
                            );
                            assert!(
                                strings(
                                    &shown["commitment_bases"][&current]["provenance"]["evidence"]
                                )
                                .contains(&expected_grant),
                                "{at}: the grant is not in the lineage"
                            );
                            records.insert(current, record.clone());
                        }
                    }
                }
            }
            if outcome["outcome"] != "accepted" {
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            // Frozen records, and permission_withdrawn over the recorded grant.
            for (commitment, record) in &records {
                assert_eq!(
                    &shown["commitment_permissions"][commitment], record,
                    "{at}: the permission record of {commitment} changed"
                );
            }
            for (series, binding) in [
                ("plain", "revoked_plain"),
                ("scoped", "revoked_scoped"),
                ("by_reading", "revoked_reading"),
            ] {
                let revoked = shown["bindings"]["view"][binding].as_f64();
                match shown["decision_series"][series]["current"].as_str() {
                    Some(current) => {
                        let grant = records
                            .get(current)
                            .map(|record| record["grant"].as_str().unwrap().to_string())
                            .unwrap_or_else(|| {
                                shown["commitment_permissions"][current]["grant"]
                                    .as_str()
                                    .unwrap()
                                    .to_string()
                            });
                        assert_eq!(
                            revoked,
                            Some(if withdrawn(&shown, &grant) { 1.0 } else { 0.0 }),
                            "{at}: permission_withdrawn({series}) with grant {grant}"
                        );
                    }
                    None => assert_eq!(revoked, None, "{at}"),
                }
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
    eprintln!("{accepted} accepted events, {grants} permitted commits, denials {denials:?}");
    for site in [
        "plain/missing",
        "plain/withdrawn",
        "scoped/missing",
        "scoped/withdrawn",
        "scoped/scope",
        "by_reading/missing",
        "by_reading/withdrawn",
    ] {
        assert!(
            denials.get(site).copied().unwrap_or(0) > 10,
            "the alphabet barely reached {site}: {denials:?}"
        );
    }
}
