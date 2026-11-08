//! D041: restore refused a save the runtime itself wrote, with "outcome of
//! d@1: runs@1 must name a declared evidence", once a record that an
//! outcome's lineage named had departed. An outcome's lineage is frozen, as a
//! reopening's basis is (Outcomes 0.1), and a frozen lineage replaces a
//! departed record's name with its history's marker (Lineage Compaction 0.1),
//! but departure never compacted an outcome's lineage. It kept the departed
//! names, a state that read the outcome copied them, and no later save of the
//! session restored while they stayed. Departure now compacts an outcome's
//! lineage as it does a
//! basis, the archive publishes the membership of its markers, and restore
//! checks them as it checks every other provenance's. See
//! spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// Sends each event and returns the records the archive took and the
/// provenance nodes it published.
fn play(session: &mut ReactiveSession, events: &[(&str, &str)]) -> (Vec<Value>, BTreeSet<String>) {
    let mut records = Vec::new();
    let mut published = BTreeSet::new();
    for (event, payload) in events {
        send(session, event, payload);
        for item in session.drain_archive() {
            let item = serde_json::to_value(&item).unwrap();
            if item["kind"] == "provenance" {
                published.insert(item["id"].as_str().unwrap().to_string());
            } else {
                records.push(item);
            }
        }
    }
    (records, published)
}

/// Sends each event and returns the provenance nodes the archive published.
fn send_draining(session: &mut ReactiveSession, events: &[(&str, &str)]) -> BTreeSet<String> {
    play(session, events).1
}

/// The refusal restore gives `save`.
fn refused(source: &str, save: &Value) -> String {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => error,
        Ok(_) => panic!("accepted a forged save"),
    }
}

/// The lineage of revision `index`'s outcome in `save`.
fn outcome_lineage(save: &mut Value, index: usize) -> &mut Value {
    &mut save["decision_series"]["d"]["revisions"][index]["outcome"]["lineage"]
}

/// Every `archive_ref` a save holds, with where it is.
fn archive_refs(value: &Value, path: &str, out: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                let here = format!("{path}/{key}");
                if key == "archive_ref" {
                    if let Some(id) = inner.as_str() {
                        out.push((here.clone(), id.to_string()));
                    }
                }
                archive_refs(inner, &here, out);
            }
        }
        Value::Array(items) => {
            for (index, inner) in items.iter().enumerate() {
                archive_refs(inner, &format!("{path}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// A reading the outcome's value read departs once two newer readings retire
/// it. The outcome pins only its `because`, `e`.
const FOLD: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 2;
decisions d limit 4;
state w = 0;
event go;
event reading v min 0 max 9;
event close;
event copy;
on go reveal e supports c;
on go commit d because enough using 1;
on reading sample runs = v;
on close resolve d = history_at(runs, 0) + history_at(runs, 1) because e;
on copy when resolved(d) set w = outcome(d);
";

#[test]
fn an_outcome_lineage_compacts_when_a_record_it_names_departs() {
    let mut session = ReactiveSession::from_source(FOLD).unwrap();
    // runs@1 and runs@2 retire, and depart: nothing pins them.
    let published = send_draining(
        &mut session,
        &[
            ("go", "{}"),
            ("reading", r#"{"v":1}"#),
            ("reading", r#"{"v":2}"#),
            ("close", "{}"),
            ("reading", r#"{"v":3}"#),
            ("reading", r#"{"v":4}"#),
        ],
    );
    let snapshot = session.snapshot();
    let lineage = &snapshot.decision_series["d"].revisions[0]
        .outcome
        .as_ref()
        .expect("an outcome")
        .lineage;
    assert_eq!(
        lineage
            .evidence
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["e"]
    );
    let marker = &lineage.departed["runs"];
    assert_eq!((marker.read, marker.from, marker.through), (2, 1, 2));
    let id = serde_json::to_value(&marker.archive_ref).unwrap();
    assert!(
        published.contains(id.as_str().unwrap()),
        "{id} is not in the archive"
    );
    // A later reader of the outcome takes the marker, not the names.
    send_draining(&mut session, &[("copy", "{}")]);
    let saved = session.save_json().unwrap();
    let save: Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(
        save["states"]["w"]["lineage"]["evidence"],
        serde_json::json!(["e"])
    );
    assert_eq!(
        save["states"]["w"]["lineage"]["departed"][0]["history"],
        "runs"
    );
    let restored = ReactiveSession::restore_json(FOLD, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(restored.snapshot(), session.snapshot());
    assert_eq!(restored.save_json().unwrap(), saved);
    // A marker in an outcome's lineage is checked as any other's: runs@4 has
    // not departed.
    let mut forged = save.clone();
    outcome_lineage(&mut forged, 0)["departed"][0]["through"] = json!(4);
    assert_eq!(
        refused(FOLD, &forged),
        "cannot restore save: outcome of d@1: runs's departed range is not of departed records"
    );
    // So are its inherited names: it holds no `zz`.
    let mut forged = save.clone();
    outcome_lineage(&mut forged, 0)["inherited"] = json!(["zz"]);
    assert_eq!(
        refused(FOLD, &forged),
        "cannot restore save: outcome of d@1 inherited zz, which it does not hold"
    );
}

/// Two states each hold a marker for a departed reading, through guards they
/// read; the outcome's value joins the two markers as it resolves.
const JOIN: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 1;
decisions d limit 4;
state a = 0;
state b = 0;
event go;
event reading v min 0 max 9;
event take_a;
event take_b;
event close;
on go reveal e supports c;
on go commit d because enough using 1;
on reading sample runs = v;
on take_a when latest(runs) > 5 set a = 1;
on take_b when latest(runs) > 5 set b = 1;
on close resolve d = a + b because e;
";

#[test]
fn an_outcome_marker_joined_as_it_resolves_is_published() {
    let mut session = ReactiveSession::from_source(JOIN).unwrap();
    let published = send_draining(
        &mut session,
        &[
            ("go", "{}"),
            ("reading", r#"{"v":1}"#),
            ("take_a", "{}"),
            ("reading", r#"{"v":2}"#),
            ("take_b", "{}"),
            ("reading", r#"{"v":3}"#),
            ("close", "{}"),
        ],
    );
    let saved = session.save_json().unwrap();
    let save: Value = serde_json::from_str(&saved).unwrap();
    let mut refs = Vec::new();
    archive_refs(&save, "", &mut refs);
    assert!(
        refs.iter()
            .any(|(path, _)| path.starts_with("/decision_series/d/revisions/0/outcome")),
        "the outcome holds no marker: {refs:?}"
    );
    let unpublished = refs
        .iter()
        .filter(|(_, id)| !published.contains(id))
        .collect::<Vec<_>>();
    assert!(
        unpublished.is_empty(),
        "the archive never published {unpublished:?}"
    );
    ReactiveSession::restore_json(JOIN, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
}

/// The occurrence of a renewable evidence that the outcome's guard read
/// departs at the next renewal.
const RENEWED: &str = "claim c;
evidence e from \"e\";
evidence r from \"r\";
renewable r window 1;
decisions d limit 4;
event go;
event again;
event close;
on go reveal e supports c;
on go commit d because enough using 1;
on again renew r;
on again reveal r supports c;
on close when observed(r) resolve d = 1 because e;
";

#[test]
fn an_outcome_whose_guard_read_a_departed_occurrence_restores() {
    let mut session = ReactiveSession::from_source(RENEWED).unwrap();
    send_draining(
        &mut session,
        &[
            ("go", "{}"),
            ("again", "{}"),
            ("close", "{}"),
            ("again", "{}"),
        ],
    );
    let saved = session.save_json().unwrap();
    let save: Value = serde_json::from_str(&saved).unwrap();
    let lineage = &save["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"];
    assert_eq!(lineage["departed"][0]["history"], "r", "{lineage}");
    let restored = ReactiveSession::restore_json(RENEWED, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(restored.save_json().unwrap(), saved);
}

/// The outcome reads one reading, which departs two readings later.
const LATER: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 2;
decisions d limit 4;
state w = 0;
event go;
event reading v min 0 max 9;
event close;
event copy;
on go reveal e supports c;
on go commit d because enough using 1;
on reading sample runs = v;
on close resolve d = latest(runs) because e;
on copy when resolved(d) set w = outcome(d);
";

/// `close` resolves on runs@1, which departs at the reading of 3.
const LATER_EVENTS: [(&str, &str); 8] = [
    ("go", "{}"),
    ("reading", r#"{"v":1}"#),
    ("close", "{}"),
    ("reading", r#"{"v":2}"#),
    ("reading", r#"{"v":3}"#),
    ("copy", "{}"),
    ("reading", r#"{"v":4}"#),
    ("copy", "{}"),
];

#[test]
fn a_session_restored_before_or_after_the_departure_plays_on_alike() {
    let mut live = ReactiveSession::from_source(LATER).unwrap();
    let mut restored = Vec::new();
    for (index, (event, payload)) in LATER_EVENTS.into_iter().enumerate() {
        let answer = send(&mut live, event, payload);
        let saved = live.save_json().unwrap();
        let save: Value = serde_json::from_str(&saved).unwrap();
        let departed =
            &save["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]["departed"];
        assert_eq!(departed.is_null(), index < 4, "after {event} {index}");
        // A session restored earlier plays on as the live one does.
        for (from, session) in &mut restored {
            assert_eq!(
                send(session, event, payload),
                answer,
                "{event} {index}, restored after event {from}"
            );
            assert_eq!(
                session.save_json().unwrap(),
                saved,
                "save after {event} {index}, restored after event {from}"
            );
        }
        // Every save restores, before the departure and after it. Play on the
        // sessions restored after `close`, before runs@1 departs, and after
        // the reading that departs it.
        let session = ReactiveSession::restore_json(LATER, &saved)
            .unwrap_or_else(|error| panic!("the save after {event} {index} was refused: {error}"));
        if index == 2 || index == 4 {
            restored.push((index, session));
        }
    }
}

/// Three revisions: d@1 and d@3 resolve on `e`, and d@2, between them, on
/// what runs@1 and runs@2 say. `extra` adds rules.
fn three_revisions(extra: &str) -> String {
    format!(
        "claim c;
evidence e from \"e\";
evidence why from \"why\";
evidence feed from \"feed\";
readings runs from feed window 2;
decisions d limit 4;
event go;
event again;
event reading v min 0 max 9;
event late;
event fixed;
on go reveal e supports c;
on go commit d because enough using 1;
on again reveal why;
on again reopen d because why;
on reading sample runs = v;
on late when history_at(runs, 0) + history_at(runs, 1) > 0 resolve d = 1 because e;
on fixed resolve d = 1 because e;
{extra}"
    )
}

/// d@1 resolves, d@2 resolves on runs@1 and runs@2 and is superseded by d@3,
/// which resolves; then runs@1 and runs@2 depart.
const THREE_REVISIONS: [(&str, &str); 12] = [
    ("go", "{}"),
    ("fixed", "{}"),
    ("again", "{}"),
    ("go", "{}"),
    ("reading", r#"{"v":1}"#),
    ("reading", r#"{"v":2}"#),
    ("late", "{}"),
    ("again", "{}"),
    ("go", "{}"),
    ("fixed", "{}"),
    ("reading", r#"{"v":3}"#),
    ("reading", r#"{"v":4}"#),
];

#[test]
fn the_middle_revisions_outcome_compacts_names_its_holder_is_published_and_is_checked() {
    // A state reads the same two readings: the departed records name both
    // holders, and the outcome holds the state's marker.
    let source = three_revisions(
        "state x = 0;
on late when history_at(runs, 0) + history_at(runs, 1) > 0 set x = 1;
",
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let (records, _) = play(&mut session, &THREE_REVISIONS);
    assert_eq!(records.len(), 2, "{records:?}");
    for entry in &records {
        assert_eq!(
            entry["holders"],
            json!([
                {"kind": "series", "name": "d@2", "in": "outcome"},
                {"kind": "state", "name": "x", "in": "lineage"},
            ]),
            "{}",
            entry["record"]
        );
    }
    let saved = session.save_json().unwrap();
    let mut save: Value = serde_json::from_str(&saved).unwrap();
    assert_eq!(save["decision_series"]["d"]["current"], "d@3");
    let plain = json!({"caveats": [], "evidence": ["e"]});
    assert_eq!(outcome_lineage(&mut save, 0), &plain);
    assert_eq!(outcome_lineage(&mut save, 2), &plain);
    assert_eq!(outcome_lineage(&mut save, 1)["evidence"], json!(["e"]));
    let marker = outcome_lineage(&mut save, 1)["departed"].clone();
    assert_eq!(marker, save["states"]["x"]["lineage"]["departed"]);
    let restored = ReactiveSession::restore_json(&source, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    assert_eq!(restored.save_json().unwrap(), saved);
    outcome_lineage(&mut save, 1)["departed"][0]["through"] = json!(3);
    assert_eq!(
        refused(&source, &save),
        "cannot restore save: outcome of d@2: runs's departed range is not of departed records"
    );

    // Without the state only the outcome holds the marker's root, so only
    // its settling publishes it.
    let source = three_revisions("");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let (_, published) = play(&mut session, &THREE_REVISIONS);
    let mut save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    let root = outcome_lineage(&mut save, 1)["departed"][0]["archive_ref"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(published.contains(&root), "{root} was never published");
}

/// No window, so nothing departs: `d` resolves on `e`, and no take-back
/// carries `flaky`.
const PLAIN: &str = "claim c;
evidence e from \"e\";
caveat flaky consequence material;
decisions d limit 4;
event go;
event close;
on go reveal e supports c;
on go commit d because enough using 1;
on close resolve d = 1 because e;
";

/// The save after `d` resolves in `PLAIN`, checked to restore.
fn resolved_plain() -> Value {
    let mut session = ReactiveSession::from_source(PLAIN).unwrap();
    send(&mut session, "go", "{}");
    send(&mut session, "close", "{}");
    let saved = session.save_json().unwrap();
    ReactiveSession::restore_json(PLAIN, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    serde_json::from_str(&saved).unwrap()
}

#[test]
fn restore_checks_an_outcome_lineage_in_a_program_without_a_window() {
    for (field, forged, expected) in [
        ("inherited", json!(["e"]), "cannot record inherited names"),
        (
            "kept",
            json!(["flaky"]),
            "keeps flaky, which is not a take-back caveat it carries",
        ),
        (
            "departed",
            json!([{"history": "e", "read": 1, "from": 1, "through": 1, "departed_at": 1}]),
            "holds departed records of e, which has no window",
        ),
    ] {
        let mut save = resolved_plain();
        outcome_lineage(&mut save, 0)[field] = forged;
        assert_eq!(
            refused(PLAIN, &save),
            format!("cannot restore save: outcome of d@1 {expected}"),
            "{field}"
        );
    }
}

/// With two defects, restore names the one first in the save's order: an
/// outcome's lineage is checked with its series, before the observation
/// qualifications.
#[test]
fn an_outcome_lineage_is_checked_in_its_series_place() {
    let mut save = resolved_plain();
    outcome_lineage(&mut save, 0)["kept"] = json!(["flaky"]);
    save["observation_qualifications"]["e"]["kept"] = json!(["flaky"]);
    assert_eq!(
        refused(PLAIN, &save),
        "cannot restore save: outcome of d@1 keeps flaky, which is not a take-back caveat it \
         carries"
    );
}

/// `close` resolves the revision `k` names, so two revisions hold outcomes.
const REVISED: &str = "claim c;
evidence e from \"e\";
evidence why from \"why\";
caveat flaky consequence material;
decisions d limit 4;
event go;
event again;
event close k min 1 max 4;
on go reveal e supports c;
on go commit d because enough using 1;
on again reveal why;
on again reopen d because why;
on close resolve revision(d, k) = 1 because e;
";

/// With two defects, restore names the one first in the save's order: a
/// series' selection qualifications, then its outcomes by revision.
#[test]
fn restore_checks_a_series_selection_then_its_outcomes_by_revision() {
    let mut session = ReactiveSession::from_source(REVISED).unwrap();
    for (event, payload) in [
        ("go", "{}"),
        ("close", r#"{"k":1}"#),
        ("again", "{}"),
        ("go", "{}"),
        ("close", r#"{"k":2}"#),
    ] {
        send(&mut session, event, payload);
    }
    let saved = session.save_json().unwrap();
    ReactiveSession::restore_json(REVISED, &saved)
        .unwrap_or_else(|error| panic!("restore refused the runtime's own save: {error}"));
    let save: Value = serde_json::from_str(&saved).unwrap();
    // `kept` is forged into each place `targets` names.
    let refusal = |targets: &[&str]| {
        let mut forged = save.clone();
        for target in targets {
            let place = match *target {
                "selection" => &mut forged["decision_series"]["d"]["selection_qualifications"],
                "d@1" => outcome_lineage(&mut forged, 0),
                _ => outcome_lineage(&mut forged, 1),
            };
            place["kept"] = json!(["flaky"]);
        }
        refused(REVISED, &forged)
    };
    assert_eq!(
        refusal(&["d@2", "d@1"]),
        "cannot restore save: outcome of d@1 keeps flaky, which is not a take-back caveat it \
         carries"
    );
    assert_eq!(
        refusal(&["d@1", "selection"]),
        "cannot restore save: decision series d keeps flaky, which is not a take-back caveat \
         it carries"
    );
}

/// Two histories with a window of one: `both` samples each, so `a@1` and
/// `b@1` depart in one event. `close_a` and `close_b` resolve a revision on
/// the latest reading of one history; `try` is a guarded commit whose guard
/// reads `b` and is false, so the series' selection qualifications name the
/// reading it read.
const TWO_HISTORIES: &str = "claim c;
evidence e from \"e\";
evidence why from \"why\";
evidence fa from \"fa\";
evidence fb from \"fb\";
readings a from fa window 1;
readings b from fb window 1;
decisions d limit 4;
event go;
event again;
event try;
event sa v min 0 max 9;
event sb v min 0 max 9;
event both v min 0 max 9;
event close_a k min 1 max 4;
event close_b k min 1 max 4;
on go reveal e supports c;
on go commit d because enough using 1;
on again reveal why;
on again reopen d because why;
on try when latest(b) > 5 commit d because enough using 1;
on sa sample a = v;
on sb sample b = v;
on both sample a = v;
on both sample b = v;
on close_a resolve revision(d, k) = latest(a) because e;
on close_b resolve revision(d, k) = latest(b) because e;
";

/// Plays `events`, draining after each, and returns the records and the
/// provenance nodes the last one drained, in the order the archive gave them.
fn last_drain(events: &[(&str, &str)]) -> (Vec<String>, Vec<String>) {
    let mut session = ReactiveSession::from_source(TWO_HISTORIES).unwrap();
    let mut records = Vec::new();
    let mut nodes = Vec::new();
    for (event, payload) in events {
        send(&mut session, event, payload);
        records.clear();
        nodes.clear();
        for item in session.drain_archive() {
            let item = serde_json::to_value(&item).unwrap();
            let name = item["record"].as_str().unwrap_or("").to_string();
            if item["kind"] == "provenance" {
                nodes.push(format!("{}:{name}", item["operation"].as_str().unwrap()));
            } else {
                records.push(name);
            }
        }
    }
    (records, nodes)
}

/// The archive publishes a series' nodes in the save's order: its selection
/// qualifications', then each outcome's by revision, all before the anchors
/// of records no provenance holds.
#[test]
fn the_archive_publishes_outcome_nodes_in_the_saves_order() {
    // The selection qualifications hold b@1 and d@1's outcome holds a@1.
    let (records, nodes) = last_drain(&[
        ("go", "{}"),
        ("sa", r#"{"v":1}"#),
        ("sb", r#"{"v":1}"#),
        ("close_a", r#"{"k":1}"#),
        ("again", "{}"),
        ("try", "{}"),
        ("both", r#"{"v":2}"#),
    ]);
    assert_eq!(records, ["a@1", "b@1"]);
    assert_eq!(
        nodes,
        ["record:b@1", "record:a@1"],
        "selection, then outcome"
    );
    // d@1's outcome holds a@1 and d@2's holds b@1.
    let (records, nodes) = last_drain(&[
        ("go", "{}"),
        ("sa", r#"{"v":1}"#),
        ("sb", r#"{"v":1}"#),
        ("close_a", r#"{"k":1}"#),
        ("again", "{}"),
        ("go", "{}"),
        ("close_b", r#"{"k":2}"#),
        ("both", r#"{"v":2}"#),
    ]);
    assert_eq!(records, ["a@1", "b@1"]);
    assert_eq!(nodes, ["record:a@1", "record:b@1"], "d@1, then d@2");
    // d@1's outcome holds b@1 and d@2's holds a@1.
    let (records, nodes) = last_drain(&[
        ("go", "{}"),
        ("sb", r#"{"v":1}"#),
        ("sa", r#"{"v":1}"#),
        ("close_b", r#"{"k":1}"#),
        ("again", "{}"),
        ("go", "{}"),
        ("close_a", r#"{"k":2}"#),
        ("both", r#"{"v":2}"#),
    ]);
    assert_eq!(records, ["a@1", "b@1"]);
    assert_eq!(nodes, ["record:b@1", "record:a@1"], "d@1, then d@2");
    // Only d@1's outcome holds b@1; nothing holds a@1, whose anchor the
    // archive publishes after every holder's nodes.
    let (records, nodes) = last_drain(&[
        ("go", "{}"),
        ("sb", r#"{"v":1}"#),
        ("sa", r#"{"v":1}"#),
        ("close_b", r#"{"k":1}"#),
        ("both", r#"{"v":2}"#),
    ]);
    assert_eq!(records, ["a@1", "b@1"]);
    assert_eq!(nodes, ["record:b@1", "record:a@1"], "outcome, then anchor");
}
