//! Random histories over `reading(STREAM, N)`, `in_basis(…)` and
//! `sample … noting ID` (Reading Index 0.1, In Basis 0.1, Reading Notes 0.1)
//! on a stream whose window departs readings, checked against a model read
//! from the snapshot, and against a session restored from the live save
//! before every event.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

const SOURCE: &str = r#"
claim passing;
caveat flaky consequence material;
evidence runner from "the test runner";
evidence recheck from "a re-run";
readings checks from runner window 4;
identifiers limit 64;
decisions ship limit 64;
state w = 0;
state c = 0;
state o = 0;
state b = 0;
state bl = 0;
state bk = 0;
event report v min 0 max 3;
event noted v min 0 max 3, tag id;
event see_recheck;
event decide;
event flag i min 1 max 12;
event unflag i min 1 max 12;
event retract i min 1 max 12;
event revisit i min 1 max 12;
event ask k min 1 max 12, i min 1 max 12;
on report sample checks = v supports passing;
on noted sample checks = v supports passing noting tag;
on see_recheck when not observed(recheck) reveal recheck;
on decide when not committed(ship) or reopened(ship) commit ship because enough using latest(checks);
on flag qualify reading(checks, i) with flaky;
on unflag unqualify reading(checks, i) with flaky;
on retract when observed(recheck) and not withdrawn(reading(checks, i)) withdraw reading(checks, i) because recheck;
on revisit when committed(ship) and not reopened(ship) reopen ship because reading(checks, i);
on ask set w = if(withdrawn(reading(checks, i)), 1, 0);
on ask set c = if(carries(reading(checks, i), flaky), 1, 0);
on ask set o = if(observed(reading(checks, i)), 1, 0);
on ask set b = if(in_basis(ship, reading(checks, i)), 1, 0);
on ask set bl = if(in_basis(ship, latest(checks)), 1, 0);
on ask set bk = if(in_basis(revision(ship, k), reading(checks, i)), 1, 0);
"#;

const EVENTS: [(&str, &str); 44] = [
    ("report", r#"{"v": 1}"#),
    ("report", r#"{"v": 2}"#),
    ("noted", r#"{"v": 3, "tag": "r3"}"#),
    ("noted", r#"{"v": 0, "tag": "r0"}"#),
    ("noted", r#"{"v": 1, "tag": "again"}"#),
    ("see_recheck", "{}"),
    ("decide", "{}"),
    ("decide", "{}"),
    ("decide", "{}"),
    ("flag", r#"{"i": 1}"#),
    ("flag", r#"{"i": 2}"#),
    ("flag", r#"{"i": 3}"#),
    ("flag", r#"{"i": 4}"#),
    ("flag", r#"{"i": 6}"#),
    ("flag", r#"{"i": 9}"#),
    ("unflag", r#"{"i": 1}"#),
    ("unflag", r#"{"i": 2}"#),
    ("unflag", r#"{"i": 3}"#),
    ("unflag", r#"{"i": 5}"#),
    ("unflag", r#"{"i": 8}"#),
    ("retract", r#"{"i": 1}"#),
    ("retract", r#"{"i": 2}"#),
    ("retract", r#"{"i": 3}"#),
    ("retract", r#"{"i": 4}"#),
    ("retract", r#"{"i": 7}"#),
    ("revisit", r#"{"i": 1}"#),
    ("revisit", r#"{"i": 2}"#),
    ("revisit", r#"{"i": 3}"#),
    ("revisit", r#"{"i": 5}"#),
    ("revisit", r#"{"i": 11}"#),
    ("ask", r#"{"k": 1, "i": 1}"#),
    ("ask", r#"{"k": 1, "i": 2}"#),
    ("ask", r#"{"k": 2, "i": 2}"#),
    ("ask", r#"{"k": 2, "i": 3}"#),
    ("ask", r#"{"k": 3, "i": 3}"#),
    ("ask", r#"{"k": 3, "i": 4}"#),
    ("ask", r#"{"k": 4, "i": 5}"#),
    ("ask", r#"{"k": 5, "i": 6}"#),
    ("ask", r#"{"k": 6, "i": 7}"#),
    ("ask", r#"{"k": 8, "i": 8}"#),
    ("ask", r#"{"k": 12, "i": 9}"#),
    ("ask", r#"{"k": 1, "i": 12}"#),
    ("ask", r#"{"k": 12, "i": 1}"#),
    ("ask", r#"{"k": 2, "i": 1}"#),
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

/// The readings `checks` holds, with their notes, in order.
fn readings(shown: &Value) -> Vec<(String, Option<String>)> {
    shown["reading_streams"]["checks"]["occurrences"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| {
                    (
                        item["id"].as_str().unwrap().to_string(),
                        item.get("note").and_then(Value::as_str).map(str::to_string),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `reading(checks, i)` names a reading now: held and not retired.
fn named(shown: &Value, i: usize) -> Option<String> {
    let id = format!("checks@{i}");
    let held = readings(shown).iter().any(|(held, _)| *held == id);
    (held && !shown["retired"][&id].is_number()).then_some(id)
}

fn latest(shown: &Value) -> Option<String> {
    readings(shown).last().map(|(id, _)| id.clone())
}

fn withdrawn(shown: &Value, id: &str) -> bool {
    shown["withdrawals"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|record| record["evidence"] == id)
}

fn revisions(shown: &Value) -> Vec<String> {
    shown["decision_series"]["ship"]["revisions"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item["id"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn current(shown: &Value) -> Option<String> {
    shown["decision_series"]["ship"]["current"]
        .as_str()
        .map(str::to_string)
}

fn open(shown: &Value, revision: &str) -> bool {
    shown["commitments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["action"] == revision)
        .map(|item| item["open"] == true)
        .unwrap_or(true)
}

fn grounds(shown: &Value, revision: &str) -> Vec<String> {
    strings(&shown["commitment_grounds"][revision]["evidence"])
}

fn in_basis(shown: &Value, revision: Option<&str>, reference: Option<&str>) -> bool {
    match (revision, reference) {
        (Some(revision), Some(reference)) => {
            grounds(shown, revision).contains(&reference.to_string())
        }
        _ => false,
    }
}

fn value(shown: &Value, state: &str) -> f64 {
    shown["qualified_values"][state]["value"].as_f64().unwrap()
}

fn payload_index(payload: &str, key: &str) -> usize {
    let value: Value = serde_json::from_str(payload).unwrap();
    value[key].as_u64().unwrap_or(0) as usize
}

#[test]
fn the_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn readings_named_by_index_answer_as_the_snapshot_says() {
    let mut seed: u64 = 0x5e1f_7ead_1234_abcd;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed >> 33
    };
    let mut accepted = 0usize;
    let mut refused = 0usize;
    let mut flagged = 0usize;
    let mut unflagged = 0usize;
    let mut withdrawn_count = 0usize;
    let mut reopened = 0usize;
    let mut decided = 0usize;
    let mut basis_true = 0usize;
    let mut past_true = 0usize;
    let mut departed_named = 0usize;
    let mut noted = 0usize;
    let mut round_trips = 0usize;
    let mut carried = 0usize;
    for run in 0..160 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        // Readings whose flaky relation is live (edges minus take-backs).
        let mut live: BTreeMap<String, bool> = BTreeMap::new();
        // What every revision was made on, pinned the step it was made.
        let mut made_on: BTreeMap<String, Vec<String>> = BTreeMap::new();
        // The note each reading was taken with.
        let mut notes: BTreeMap<String, Option<String>> = BTreeMap::new();
        // Whether recheck has been revealed, which the program does once.
        let mut recheck_seen = false;
        let mut before = snapshot(&game);
        for step in 0..60 {
            let at = format!("run {run} step {step}");
            let pick = (next() as usize) % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                let save = game.save_json().unwrap();
                let restored = ReactiveSession::restore_json(SOURCE, &save)
                    .unwrap_or_else(|error| panic!("{at}: restore: {error}"));
                assert_eq!(restored.save_json().unwrap(), save, "{at}: save round trip");
                assert_eq!(snapshot(&restored), before, "{at}: restored snapshot");
                game = restored;
                round_trips += 1;
                continue;
            }
            let (event, payload) = EVENTS[pick];
            let save_before = game.save_json().unwrap();
            let mut mirror = ReactiveSession::restore_json(SOURCE, &save_before).unwrap();
            let outcome: Value =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap()).unwrap();
            let mirrored: Value =
                serde_json::to_value(mirror.dispatch_outcome_json(event, payload).unwrap())
                    .unwrap();
            assert_eq!(
                mirrored, outcome,
                "{at}: the restored session ran differently"
            );

            let i = payload_index(payload, "i");
            let k = payload_index(payload, "k");
            let target = named(&before, i);
            let latest_before = latest(&before);
            let current_before = current(&before);
            let committed = current_before.is_some();
            let reopened_before = current_before
                .as_deref()
                .is_some_and(|revision| open(&before, revision));
            let decide_fires = !committed || reopened_before;
            let retract_fires =
                recheck_seen && !target.as_deref().is_some_and(|id| withdrawn(&before, id));
            let revisit_fires = committed && !reopened_before;
            let expected: Option<(&str, &str)> = match event {
                "decide" if decide_fires && latest_before.is_none() => {
                    Some(("evaluation", "expression"))
                }
                "flag" | "unflag" if target.is_none() => Some(("evaluation", "expression")),
                "retract" if retract_fires && target.is_none() => {
                    Some(("evaluation", "expression"))
                }
                "revisit" if revisit_fires && target.is_none() => {
                    Some(("evaluation", "expression"))
                }
                _ => None,
            };
            if let Some((origin, code)) = expected {
                assert_eq!(
                    (outcome["origin"].as_str(), outcome["code"].as_str()),
                    (Some(origin), Some(code)),
                    "{at}: {event} {payload}: {outcome}"
                );
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refusal changed the save"
                );
                refused += 1;
                continue;
            }
            assert_eq!(
                outcome["outcome"], "accepted",
                "{at}: {event} {payload}: {outcome}"
            );
            accepted += 1;
            let shown = snapshot(&game);
            assert_eq!(
                snapshot(&mirror),
                shown,
                "{at}: the restored session's snapshot differs"
            );
            let effects = shown["effects"].as_array().unwrap();
            let qualified: Vec<String> = effects
                .iter()
                .filter(|effect| effect["kind"] == "qualify")
                .map(|effect| effect["evidence"].as_str().unwrap().to_string())
                .collect();
            let unqualified: Option<Vec<String>> = effects
                .iter()
                .find(|effect| effect["kind"] == "unqualify")
                .map(|effect| strings(&effect["occurrences"]));
            match event {
                "report" | "noted" => {
                    let taken = latest(&shown).unwrap();
                    assert_ne!(Some(&taken), latest_before.as_ref(), "{at}: no new reading");
                    let note = if event == "noted" {
                        let text: Value = serde_json::from_str(payload).unwrap();
                        noted += 1;
                        Some(text["tag"].as_str().unwrap().to_string())
                    } else {
                        None
                    };
                    let sample = effects
                        .iter()
                        .find(|effect| effect["kind"] == "sample" && effect["id"] == taken)
                        .unwrap_or_else(|| panic!("{at}: no sample effect for {taken}"));
                    assert_eq!(
                        sample
                            .get("note")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        note,
                        "{at}: the sample effect's note"
                    );
                    notes.insert(taken, note);
                }
                "see_recheck" => {
                    recheck_seen = true;
                }
                "decide" if decide_fires => {
                    let revision = current(&shown).unwrap();
                    assert!(
                        !made_on.contains_key(&revision),
                        "{at}: {revision} made twice"
                    );
                    let on = latest_before.clone().unwrap();
                    assert!(
                        grounds(&shown, &revision).contains(&on),
                        "{at}: {revision} made using {on}: {:?}",
                        grounds(&shown, &revision)
                    );
                    made_on.insert(revision, grounds(&shown, &current(&shown).unwrap()));
                    decided += 1;
                }
                "flag" => {
                    let id = target.clone().unwrap();
                    assert_eq!(qualified, vec![id.clone()], "{at}: qualify reported");
                    if !live.get(&id).copied().unwrap_or(false) {
                        flagged += 1;
                    }
                    live.insert(id, true);
                }
                "unflag" => {
                    let id = target.clone().unwrap();
                    let marked = if live.get(&id).copied().unwrap_or(false) {
                        live.insert(id.clone(), false);
                        unflagged += 1;
                        vec![id]
                    } else {
                        vec![]
                    };
                    // A take-back reports exactly what it marked, or nothing.
                    assert_eq!(
                        unqualified.clone().unwrap_or_default(),
                        marked,
                        "{at}: take-back reported {unqualified:?}"
                    );
                    assert_eq!(
                        unqualified.is_some(),
                        !marked.is_empty(),
                        "{at}: {unqualified:?}"
                    );
                }
                "retract" => {
                    let id = target.clone().unwrap_or_default();
                    assert_eq!(
                        withdrawn(&shown, &id),
                        retract_fires || withdrawn(&before, &id),
                        "{at}: withdrawal of {id}"
                    );
                    if retract_fires {
                        withdrawn_count += 1;
                    }
                }
                "revisit" => {
                    let revision = current_before.clone().unwrap_or_default();
                    if revisit_fires {
                        assert!(open(&shown, &revision), "{at}: {revision} not reopened");
                        reopened += 1;
                    }
                }
                "ask" => {
                    let id = target.clone();
                    let expect_w = id.as_deref().is_some_and(|id| withdrawn(&before, id));
                    let expect_c = id
                        .as_deref()
                        .is_some_and(|id| live.get(id).copied().unwrap_or(false));
                    let expect_o = id.is_some();
                    let expect_b = in_basis(&before, current_before.as_deref(), id.as_deref());
                    let expect_bl =
                        in_basis(&before, current_before.as_deref(), latest_before.as_deref());
                    let past = revisions(&before).get(k - 1).cloned();
                    let expect_bk = in_basis(&before, past.as_deref(), id.as_deref());
                    for (state, expected) in [
                        ("w", expect_w),
                        ("c", expect_c),
                        ("o", expect_o),
                        ("b", expect_b),
                        ("bl", expect_bl),
                        ("bk", expect_bk),
                    ] {
                        assert_eq!(
                            value(&shown, state),
                            if expected { 1.0 } else { 0.0 },
                            "{at}: ask k={k} i={i}: {state} (reading {id:?}, current {current_before:?}, revisions {:?})",
                            revisions(&before)
                        );
                    }
                    basis_true += usize::from(expect_b);
                    past_true += usize::from(expect_bk && past != current_before);
                    carried += usize::from(expect_c);
                }
                _ => {
                    assert!(
                        qualified.is_empty() && unqualified.is_none(),
                        "{at}: {event} touched flaky's relations"
                    );
                }
            }
            // Notes ride with the readings that took them, and no other.
            for (id, note) in readings(&shown) {
                assert_eq!(notes.get(&id), Some(&note), "{at}: note on {id}");
            }
            // Every revision keeps the grounds it was made on.
            for (revision, evidence) in &made_on {
                assert_eq!(
                    &grounds(&shown, revision),
                    evidence,
                    "{at}: {revision}'s grounds changed"
                );
            }
            // Flaky's relations and take-backs name held readings only.
            let held: Vec<String> = readings(&shown).into_iter().map(|(id, _)| id).collect();
            for edge in shown["relations"].as_array().unwrap() {
                if edge["from"] == "flaky" {
                    let to = edge["to"].as_str().unwrap().to_string();
                    assert!(
                        held.contains(&to),
                        "{at}: flaky qualifies {to}, which departed"
                    );
                }
            }
            for (id, is_live) in &live {
                if *is_live && !held.contains(id) {
                    departed_named += 1;
                }
            }
            live.retain(|id, _| held.contains(id));
            notes.retain(|id, _| held.contains(id));
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
    eprintln!("{accepted} accepted events, {refused} refused, {flagged} readings flagged, {unflagged} live take-backs, {withdrawn_count} withdrawals, {reopened} reopenings, {decided} decisions, {basis_true} in_basis true, {past_true} on a past revision, {carried} carries true, {departed_named} flagged readings departed, {noted} noted, {round_trips} round trips");
    assert!(
        accepted > 3500
            && refused > 1000
            && flagged > 160
            && unflagged > 35
            && withdrawn_count > 75
            && reopened > 90
            && decided > 140
            && basis_true > 100
            && past_true > 10
            && carried > 100
            && departed_named > 35
            && noted > 300
            && round_trips > 190,
        "coverage"
    );
}
