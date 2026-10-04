//! Restore refuses a qualification table keyed by a name its kind of record
//! cannot belong to, and a withdrawal record naming an event that cannot
//! withdraw that evidence (findings 86, 89, 96, 118–120). See
//! spec/caveat-save-0.1.md and docs/RESTORE_TRUST_BOUNDARY.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// A program that fills each table: observations by reveal and by a guarded
/// renewal, an examination, a reopening, a skipped effect's predicate guard
/// and a withdrawal reached through a procedure.
const PROGRAM: &str = r#"
budget 5;
claim safe;
evidence sensor from "sensor";
evidence other from "other";
evidence reason from "reason";
evidence gauge from "gauge";
evidence bite from "bite";
caveat stale consequence low;
caveat never consequence low;
renewable bite limit 4;
decisions route limit 4;
decisions plan limit 2;
event go;
event look;
event pull;
event regrow;
event redo;
event skip;
proc retract(e evidence) { withdraw e because reason; };
on go reveal sensor supports safe;
on go reveal reason supports safe;
on go reveal gauge supports safe;
on go when not committed(route) commit route because enough using qualified(1, sensor);
on go when not committed(plan) commit plan because enough using qualified(1, gauge);
on look when not examined(stale) examine stale cost 1;
on pull call retract(sensor);
on regrow when observed(sensor) renew bite;
on redo when committed(route) reopen route because gauge;
on skip when not observed(sensor) reveal other supports safe;
evidence never_seen from "unseen";
"#;

const EVENTS: [&str; 7] = ["go", "look", "regrow", "redo", "skip", "pull", "go"];

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
    for event in EVENTS {
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

// The genuine save holds an entry in every table, restores as the session it
// was, plays on as it would have, and saves and restores again.
#[test]
fn a_genuine_save_with_every_table_restores() {
    let mut original = played();
    let save = serde_json::to_value(original.save().unwrap()).unwrap();
    for (table, key) in [
        ("observation_qualifications", "sensor"),
        ("observation_qualifications", "bite@2"),
        ("examination_qualifications", "stale"),
        ("reopening_qualifications", "route@1"),
    ] {
        assert!(
            save[table].get(key).is_some(),
            "{table} {key}: {}",
            save[table]
        );
    }
    assert!(save["predicate_qualifications"]["observed"]
        .get("other")
        .is_some());
    assert_eq!(save["withdrawals"][0]["event"], "pull");
    let mut resumed = ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(resumed.snapshot(), original.snapshot());
    for event in ["regrow", "redo", "go"] {
        send(&mut original, event);
        send(&mut resumed, event);
        assert_eq!(resumed.snapshot(), original.snapshot(), "after {event}");
    }
    let again = ReactiveSession::restore_json(PROGRAM, &resumed.save_json().unwrap()).unwrap();
    assert_eq!(again.snapshot(), original.snapshot());
}

// An observation qualification keyed by evidence nothing observed. Only an
// observed evidence, or an occurrence a renewal created, has one.
#[test]
fn an_observation_record_of_unobserved_evidence_is_refused() {
    let mut save = saved();
    save["observation_qualifications"]["never_seen"] = json!({"evidence": ["sensor"]});
    refused(
        &save,
        "observation of never_seen: never_seen is not observed evidence or a renewed occurrence",
    );
}

// An examination qualification keyed by a caveat the save does not leave
// examined.
#[test]
fn an_examination_record_of_an_unexamined_caveat_is_refused() {
    let mut save = saved();
    save["examination_qualifications"]["never"] = json!({"caveats": ["never"]});
    refused(
        &save,
        "examination of never: never is not an examined caveat",
    );
}

// A reopening qualification keyed by a commitment nothing reopens.
#[test]
fn a_reopening_record_of_an_unreopened_commitment_is_refused() {
    let mut save = saved();
    save["reopening_qualifications"]["plan@1"] = json!({"evidence": ["gauge"]});
    refused(
        &save,
        "reopening of plan@1: plan@1 is not a commitment a reopens relation names",
    );
}

// A predicate guard's target must be of the kind its predicate reads.
#[test]
fn a_predicate_guard_on_a_name_of_the_wrong_kind_is_refused() {
    for (kind, target, expected) in [
        ("observed", "safe", "observed(safe): safe is not evidence"),
        (
            "examined",
            "sensor",
            "examined(sensor): sensor is not a caveat",
        ),
        (
            "committed",
            "sensor",
            "committed(sensor): sensor is not a decision this program makes",
        ),
        (
            "reopened",
            "stale",
            "reopened(stale): stale is not a decision this program makes",
        ),
    ] {
        let mut save = saved();
        save["predicate_qualifications"][kind][target] = json!({});
        refused(&save, expected);
    }
}

// A withdrawal record whose event cannot reach a `withdraw` of that evidence
// for that reason.
#[test]
fn a_withdrawal_by_an_event_that_cannot_withdraw_it_is_refused() {
    let mut save = saved();
    save["withdrawals"][0]["event"] = "go".into();
    refused(
        &save,
        "the withdrawal of sensor by go: no withdraw of that event withdraws sensor because reason",
    );
}

// Restore checks what the source could have recorded, not what it did. A
// predicate guard on a name of the right kind restores though no skipped
// effect recorded it, and so does a save with its withdrawal removed with its
// relation (round 7's F263): neither is shown never to have happened, and
// restore does not authenticate a save's history.
#[test]
fn records_the_source_could_have_left_restore_without_proof_they_did() {
    let mut save = saved();
    save["predicate_qualifications"]["committed"]["route"] = json!({});
    save["predicate_qualifications"]["observed"]["gauge"] = json!({});
    ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    let mut save = saved();
    save["withdrawals"] = json!([]);
    let relations = save["graph"]["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|relation| relation[0] != "withdrawn")
        .cloned()
        .collect::<Vec<_>>();
    save["graph"]["relations"] = Value::Array(relations);
    let restored = ReactiveSession::restore_json(PROGRAM, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        serde_json::to_value(restored.snapshot()).unwrap()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|relation| relation["from"] != "withdrawn")
    );
}
