//! D041: restore refused a save the runtime itself wrote, with "outcome of
//! d@1: runs@1 must name a declared evidence", once a record that an
//! outcome's lineage named had departed. An outcome's lineage is frozen, as a
//! reopening's basis is (Outcomes 0.1), and a frozen lineage replaces a
//! departed record's name with its history's marker (Lineage Compaction 0.1),
//! but departure never compacted an outcome's lineage. It kept the departed
//! names, a state that read the outcome copied them, and no later save of the
//! session restored. Departure now compacts an outcome's lineage as it does a
//! basis, the archive publishes the membership of its markers, and restore
//! checks them as it checks every other provenance's. See
//! spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeSet;

fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// Sends each event and returns the provenance nodes the archive published.
fn send_draining(session: &mut ReactiveSession, events: &[(&str, &str)]) -> BTreeSet<String> {
    let mut published = BTreeSet::new();
    for (event, payload) in events {
        send(session, event, payload);
        for item in session.drain_archive() {
            let item = serde_json::to_value(&item).unwrap();
            if item["kind"] == "provenance" {
                published.insert(item["id"].as_str().unwrap().to_string());
            }
        }
    }
    published
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
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]["departed"][0]
        ["through"] = serde_json::json!(4);
    let error = ReactiveSession::restore_json(FOLD, &forged.to_string())
        .expect_err("an outcome's marker over a live record is refused");
    assert!(error.contains("outcome of d@1"), "{error}");
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

#[test]
fn a_session_restored_after_the_departure_plays_on_alike() {
    let mut live = ReactiveSession::from_source(LATER).unwrap();
    for (event, payload) in [
        ("go", "{}"),
        ("reading", r#"{"v":1}"#),
        ("close", "{}"),
        ("reading", r#"{"v":2}"#),
        ("reading", r#"{"v":3}"#),
    ] {
        send(&mut live, event, payload);
        // Every save along the way restores, before the departure and after.
        ReactiveSession::restore_json(LATER, &live.save_json().unwrap())
            .unwrap_or_else(|error| panic!("the save after {event} was refused: {error}"));
    }
    let mut restored = ReactiveSession::restore_json(LATER, &live.save_json().unwrap()).unwrap();
    for (event, payload) in [("copy", "{}"), ("reading", r#"{"v":4}"#), ("copy", "{}")] {
        let answer = send(&mut live, event, payload);
        assert_eq!(
            send(&mut restored, event, payload),
            answer,
            "{event} after restore"
        );
        let saved = live.save_json().unwrap();
        assert_eq!(restored.save_json().unwrap(), saved, "save after {event}");
        ReactiveSession::restore_json(LATER, &saved)
            .unwrap_or_else(|error| panic!("the save after {event} was refused: {error}"));
    }
}
