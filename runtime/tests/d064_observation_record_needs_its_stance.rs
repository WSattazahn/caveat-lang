//! D064: restore accepted an observation record on declared evidence
//! without the stance that leaves it. Evidence a declaration relates to a
//! claim is observed as the program loads, with no observation record
//! (rc.13, F317–F322). A `reveal` that gives it a stance no declaration
//! gives it adds that relation and leaves the record in the same step, and
//! D052 held a record's name to the reveals a rule reaches, never to the
//! save's relations. So a save could hold `sensor`'s record without
//! `sensor opposes safe`, and the restored session read `sensor` through a
//! record no session of the program holds: a later `count` carried `other`
//! into `seen` where the live session carried `sensor` alone. Restore now
//! refuses a record on declared evidence that has no stance but those its
//! declarations give it. D065 holds the stance to the record the other way.
//! See spec/caveat-save-0.1.md ("Restore validation", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const NO_STANCE: &str =
    "observation of sensor: sensor has no stance but those its declarations give it";

fn answer(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn saved(session: &ReactiveSession) -> Value {
    serde_json::from_str(&session.save_json().unwrap()).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {}", save["observation_qualifications"]),
    }
}

/// Plays `events`, each accepted. After each, the save restores and saves the
/// same bytes again, and every session restored so far answers the events
/// still to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[&str]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, event) in events.iter().enumerate() {
        let outcome = answer(&mut live, event);
        assert_eq!(
            outcome["outcome"], "accepted",
            "{index}: {event}: {outcome}"
        );
        for run in &mut restored {
            assert_eq!(answer(run, event), outcome, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    saved(&live)
}

/// The finding's program, with `go`'s effect given and `extra` added:
/// `sensor` is declared to support `safe`, `peek` reveals `other`, and
/// `count` reads `sensor`.
fn program(go: &str, extra: &str) -> String {
    format!(
        "claim safe;
claim calm;
evidence sensor from \"sensor\";
evidence other from \"o\";
sensor supports safe;
state seen = 0;
event peek;
event go;
event count;
on peek reveal other supports safe;
on go {go};
on count set seen = seen + qualified(1, sensor);
{extra}"
    )
}

/// Once `other` is observed, `go` has `sensor` oppose `safe`, a stance of its
/// own, which leaves `sensor`'s record.
const OPPOSES: &str = "when observed(other) reveal sensor opposes safe";

/// `save` without the relation `[from, relation, to]`.
fn without(save: &Value, relation: [&str; 3]) -> Value {
    let mut save = save.clone();
    let relations = save["graph"]["relations"].as_array_mut().unwrap();
    let before = relations.len();
    relations.retain(|held| *held != json!(relation));
    assert_eq!(relations.len(), before - 1, "{relation:?}");
    save
}

/// The finding: after `peek`, a record on `sensor` beside no stance of its
/// own, empty or listing what `go`'s guard reads, is refused.
#[test]
fn a_record_without_its_stance_is_refused() {
    let source = program(OPPOSES, "");
    let mut live = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(answer(&mut live, "peek")["outcome"], "accepted");
    for record in [json!({"evidence": ["other"]}), json!({})] {
        let mut save = saved(&live);
        save["observation_qualifications"]["sensor"] = record;
        refused(&source, &save, NO_STANCE);
    }
}

/// The record `go` leaves restores beside its stance, before and after
/// `count` reads through it, and every save the session writes restores and
/// plays on alike. With the stance taken out of the save, the record is
/// refused.
#[test]
fn a_record_beside_its_stance_restores() {
    let source = program(OPPOSES, "");
    let save = plays_and_restores(
        &source,
        &["peek", "count", "go", "count", "go", "peek", "count"],
    );
    assert_eq!(
        save["observation_qualifications"],
        json!({"other": {}, "sensor": {"evidence": ["other"]}})
    );
    refused(
        &source,
        &without(&save, ["sensor", "opposes", "safe"]),
        NO_STANCE,
    );
}

/// A stance on another claim, or one a procedure gives, is the evidence's
/// own as well, and each holds the record alike.
#[test]
fn any_stance_no_declaration_gives_holds_the_record() {
    for (go, extra, relation) in [
        (
            "when observed(other) reveal sensor supports calm",
            "",
            ["sensor", "supports", "calm"],
        ),
        (
            "call look()",
            "proc look() {\n    when observed(other) reveal sensor opposes safe;\n};\n",
            ["sensor", "opposes", "safe"],
        ),
    ] {
        let source = program(go, extra);
        let save = plays_and_restores(&source, &["peek", "go", "count", "go"]);
        assert_eq!(
            save["observation_qualifications"],
            json!({"other": {}, "sensor": {"evidence": ["other"]}}),
            "{go}"
        );
        refused(&source, &without(&save, relation), NO_STANCE);
    }
}

/// Once `refresh` renews `sensor`, `go` reveals its current occurrence and
/// leaves the record on `sensor@2`. That occurrence's stance is not
/// `sensor`'s own, so a record on `sensor` beside it is refused.
#[test]
fn an_occurrences_stance_is_not_the_evidences_own() {
    let source = program(
        OPPOSES,
        "renewable sensor limit 4;\nevent refresh;\non refresh renew sensor;\n",
    );
    let save = plays_and_restores(&source, &["peek", "refresh", "go", "count"]);
    assert_eq!(
        save["observation_qualifications"],
        json!({"other": {}, "sensor@2": {"evidence": ["other"]}})
    );
    let mut forged = save.clone();
    forged["observation_qualifications"]["sensor"] = json!({"evidence": ["other"]});
    refused(&source, &forged, NO_STANCE);
}

/// Nothing removes either: a qualification, its take-back and a withdrawal
/// keep both, and `sensor` itself never departs, even once a renewal under
/// a window retires it, while its later occurrences depart with their
/// records and stances. Every save on the way restores.
#[test]
fn nothing_removes_the_record_or_its_stance() {
    let source = program(
        OPPOSES,
        "caveat doubted consequence material;
event doubt;
event undoubt;
event drop;
on doubt qualify sensor with doubted;
on undoubt unqualify sensor with doubted;
on drop withdraw sensor because other;
",
    );
    let save = plays_and_restores(
        &source,
        &["peek", "go", "doubt", "count", "undoubt", "count", "drop"],
    );
    assert_eq!(
        save["observation_qualifications"]["sensor"]["evidence"],
        json!(["other"])
    );
    assert!(save["graph"]["relations"]
        .as_array()
        .unwrap()
        .contains(&json!(["sensor", "opposes", "safe"])));
    let source = program(
        OPPOSES,
        "renewable sensor window 1;\nevent refresh;\non refresh renew sensor;\n",
    );
    let save = plays_and_restores(
        &source,
        &["peek", "go", "refresh", "go", "refresh", "go", "count"],
    );
    assert_eq!(save["renewals"], json!({"sensor": ["sensor", "sensor@3"]}));
    assert_eq!(save["retired"]["sensor"], json!(3));
    assert_eq!(
        save["observation_qualifications"],
        json!({
            "other": {},
            "sensor": {"evidence": ["other"]},
            "sensor@3": {"evidence": ["other"]},
        })
    );
    assert_eq!(
        save["graph"]["relations"],
        json!([
            ["other", "supports", "safe"],
            ["sensor", "opposes", "safe"],
            ["sensor@3", "opposes", "safe"],
        ])
    );
}
