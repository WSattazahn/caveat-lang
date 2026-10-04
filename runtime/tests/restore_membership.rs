//! Restore refuses a membership the graph records without the record every
//! event that adds it leaves: observed evidence without an observation
//! record, an examined caveat without an examination record, a commitment a
//! `reopens` relation names without a reopening record (findings
//! F317-F322). See spec/caveat-save-0.1.md and docs/RESTORE_TRUST_BOUNDARY.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// Observations by stance and by neutral reveal, an examination and a
/// reopening, with `initial` observed by declaration.
const PROGRAM: &str = r#"
budget 5;
claim safe;
evidence initial from "initial";
evidence sensor from "sensor";
evidence gauge from "gauge";
evidence note from "note";
caveat stale consequence low;
initial supports safe;
decisions route limit 4;
event go;
event look;
event redo;
on go reveal sensor supports safe;
on go reveal gauge supports safe;
on go reveal note;
on go when not committed(route) commit route because enough using qualified(1, sensor);
on look when not examined(stale) examine stale cost 1;
on redo when committed(route) reopen route because gauge;
"#;

fn send(game: &mut ReactiveSession, event: &str) {
    let outcome = serde_json::to_value(
        game.dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

fn played() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in ["go", "look", "redo"] {
        send(&mut game, event);
    }
    game
}

fn saved() -> Value {
    serde_json::to_value(played().save().unwrap()).unwrap()
}

fn refused(save: &Value, expected: &str) {
    match ReactiveSession::restore_json(PROGRAM, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted: {expected}"),
    }
}

fn without(save: &Value, table: &str, key: &str) -> Value {
    let mut save = save.clone();
    assert!(
        save[table].as_object_mut().unwrap().remove(key).is_some(),
        "{table} {key}"
    );
    save
}

// The genuine save holds a record for each membership an event added, none
// for `initial`, which the program observes as it loads, and restores.
#[test]
fn a_genuine_save_has_a_record_for_every_membership_and_restores() {
    let save = saved();
    assert_eq!(
        save["observations"],
        json!(["initial", "sensor", "gauge", "note"])
    );
    for (table, key) in [
        ("observation_qualifications", "sensor"),
        ("observation_qualifications", "gauge"),
        ("observation_qualifications", "note"),
        ("examination_qualifications", "stale"),
        ("reopening_qualifications", "route@1"),
    ] {
        assert!(save[table].get(key).is_some(), "{table} {key}");
    }
    assert!(save["observation_qualifications"].get("initial").is_none());
    let original = played();
    let resumed = ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(resumed.snapshot(), original.snapshot());
}

// F317-F322: each record removed while the graph keeps its membership.
#[test]
fn f317_a_membership_without_its_record_is_refused() {
    let save = saved();
    for (table, key, expected) in [
        (
            "observation_qualifications",
            "sensor",
            "observed evidence sensor has no observation record",
        ),
        (
            "observation_qualifications",
            "note",
            "observed evidence note has no observation record",
        ),
        (
            "examination_qualifications",
            "stale",
            "examined caveat stale has no examination record",
        ),
        (
            "reopening_qualifications",
            "route@1",
            "reopened commitment route@1 has no reopening record",
        ),
    ] {
        refused(&without(&save, table, key), expected);
    }
}

// The boundary: a record removed together with its membership restores, as
// a save the program could have written had the event not added it. Restore
// does not establish which events happened.
#[test]
fn a_record_removed_with_its_membership_restores() {
    let mut save = without(&saved(), "examination_qualifications", "stale");
    save["graph"]["attention"] = json!({});
    if let Err(error) = ReactiveSession::restore_json(PROGRAM, &save.to_string()) {
        panic!("{error}");
    }
}
