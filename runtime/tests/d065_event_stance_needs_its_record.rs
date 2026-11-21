//! D065: restore accepted declared evidence with a stance no declaration
//! gives it but no observation record. Every event that gives evidence such
//! a stance leaves its record in the same step, and restore pairs the two
//! on any other evidence ("observed evidence E has no observation record"),
//! but it skipped every piece of evidence a declaration relates to a claim,
//! with whatever stances the save gave it (rc.13, F317–F322). So a save
//! could keep `sensor opposes safe` and drop `sensor`'s record, and a later
//! `count` carried `sensor` alone into `seen` where the live session
//! carried `other` with it. Restore now refuses a stance on declared
//! evidence that no declaration gives it unless the evidence has its
//! record. The stances the declarations give need none. D064 holds the
//! record to the stance the other way. See spec/caveat-save-0.1.md
//! ("Restore validation", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {}", save["graph"]["relations"]),
    }
}

/// Plays `events`, each accepted, and returns the save after each, every one
/// of which restores and saves the same bytes again.
fn saves(source: &str, events: &[&str]) -> Vec<Value> {
    let mut live = ReactiveSession::from_source(source).unwrap();
    events
        .iter()
        .map(|event| {
            let outcome = answer(&mut live, event);
            assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
            let save = live.save_json().unwrap();
            let again = ReactiveSession::restore_json(source, &save)
                .unwrap_or_else(|error| panic!("{event}: {error}"));
            assert_eq!(again.save_json().unwrap(), save, "{event}");
            serde_json::from_str(&save).unwrap()
        })
        .collect()
}

/// `save` without `sensor`'s observation record.
fn unrecorded(save: &Value) -> Value {
    let mut save = save.clone();
    let records = save["observation_qualifications"].as_object_mut().unwrap();
    assert!(records.remove("sensor").is_some(), "{records:?}");
    save
}

/// The finding's program: `sensor` is declared to support `safe`, `peek`
/// reveals `other`, once `other` is observed `go` has `sensor` oppose `safe`
/// and `again` has it support `calm`, and `count` reads `sensor`.
const PROGRAM: &str = "claim safe;
claim calm;
evidence sensor from \"sensor\";
evidence other from \"o\";
sensor supports safe;
state seen = 0;
event peek;
event go;
event again;
event count;
on peek reveal other supports safe;
on go when observed(other) reveal sensor opposes safe;
on again when observed(other) reveal sensor supports calm;
on count set seen = seen + qualified(1, sensor);
";

/// The finding: after `go`, `sensor opposes safe` without `sensor`'s record
/// is refused, and so is `sensor supports calm` after `again` alone. With
/// both, the first in the save's order is named.
#[test]
fn a_stance_no_declaration_gives_needs_its_record() {
    let [_, opposed, both] = saves(PROGRAM, &["peek", "go", "again"]).try_into().unwrap();
    refused(
        PROGRAM,
        &unrecorded(&opposed),
        "evidence sensor has no observation record, though it opposes safe, a stance no \
         declaration gives it",
    );
    refused(
        PROGRAM,
        &unrecorded(&both),
        "evidence sensor has no observation record, though it opposes safe, a stance no \
         declaration gives it",
    );
    let [_, calm] = saves(PROGRAM, &["peek", "again"]).try_into().unwrap();
    refused(
        PROGRAM,
        &unrecorded(&calm),
        "evidence sensor has no observation record, though it supports calm, a stance no \
         declaration gives it",
    );
}

/// The stance the declaration gives `sensor` comes with the program, not
/// the save, and needs no record: the saves before `go` hold no stance of
/// `sensor` and no record of it, and restore. Those after hold the stances
/// the events gave it, with the record `go` left.
#[test]
fn the_declared_stance_needs_no_record() {
    let all = saves(PROGRAM, &["count", "peek", "count", "go", "count", "again"]);
    for save in &all[..3] {
        assert!(
            save["graph"]["relations"]
                .as_array()
                .is_none_or(|relations| relations.iter().all(|r| r[0] != "sensor")),
            "{}",
            save["graph"]["relations"]
        );
        assert_eq!(save["observation_qualifications"].get("sensor"), None);
    }
    for save in &all[3..] {
        assert_eq!(
            save["observation_qualifications"]["sensor"],
            json!({"evidence": ["other"]})
        );
    }
    assert_eq!(
        all[5]["graph"]["relations"],
        json!([
            ["other", "supports", "safe"],
            ["sensor", "opposes", "safe"],
            ["sensor", "supports", "calm"],
        ])
    );
}
