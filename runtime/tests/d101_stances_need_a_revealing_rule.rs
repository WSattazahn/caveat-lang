//! D101: restore accepted a `supports` or `opposes` relation that no `reveal`
//! or `sample` of the program can add. As the session runs, only a `reveal`
//! gives evidence a stance, the one it names, and only a `sample` gives a
//! reading one, the one it is taken with; the declarations give theirs as
//! the program loads, and a save holds none of them. Restore held a saved
//! stance to its ends' kinds and to the records beside it, such as the
//! evidence's observation record, never to the program's reveals. So a save
//! could give `sensor` a stance on `calm` that no rule gives it, which also
//! met D064's pairing beside a forged record, and a later read of `sensor`
//! carried `shaky`, which qualifies `calm`, where the live session carried
//! nothing. Restore now refuses a stance no `reveal` in a rule, or in a
//! procedure a rule reaches, gives that evidence, a reading's stance other
//! than the one it was taken with, a stance a declaration gives, and a
//! stance listed twice. Conditions are not evaluated. See
//! spec/caveat-save-0.1.md ("Restore validation", "Changes").
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

/// Plays `events`, each accepted. After each, the save restores and saves the
/// same bytes again, and every session restored so far answers the events
/// still to come as the live one does. Returns the save after each event.
fn plays_and_restores(source: &str, events: &[&str]) -> Vec<Value> {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    let mut saves = Vec::new();
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
        saves.push(serde_json::from_str(&save).unwrap());
    }
    saves
}

/// `save` with `relation` added to its graph. A save whose events added no
/// relation lists none.
fn with(save: &Value, relation: [&str; 3]) -> Value {
    let mut save = save.clone();
    let relations = &mut save["graph"]["relations"];
    if relations.is_null() {
        *relations = json!([]);
    }
    relations.as_array_mut().unwrap().push(json!(relation));
    save
}

/// The finding's program, with `extra` added: `sensor` is declared to
/// support `safe`, `shaky` qualifies `calm`, `peek` reveals `other`, once
/// `other` is observed `go` has `sensor` oppose `safe`, `read` takes a
/// reading that supports `safe`, `quiet` a neutral one, and `count` reads
/// `sensor`. No rule gives anything a stance on `calm`.
fn program(extra: &str) -> String {
    format!(
        "claim safe;
claim calm;
evidence sensor from \"sensor\";
evidence other from \"o\";
evidence gauge from \"g\";
sensor supports safe;
caveat shaky consequence material;
shaky qualifies calm;
readings level from gauge limit 4;
state seen = 0;
event peek;
event go;
event read;
event quiet;
event count;
on peek reveal other supports safe;
on go when observed(other) reveal sensor opposes safe;
on read sample level = 1 supports safe;
on quiet sample level = 2;
on count set seen = seen + qualified(1, sensor);
{extra}"
    )
}

const NO_REVEAL: &str =
    "relation sensor supports calm: no rule of this program, or procedure one calls, reveals \
     sensor supports calm";

/// The finding: a stance on `calm` no rule gives, on declared evidence with
/// its record, beside a forged record that it would pair with (D064), or on
/// other evidence with its record, is refused.
#[test]
fn a_stance_no_rule_reveals_is_refused() {
    let source = program("");
    let saves = plays_and_restores(&source, &["peek", "go"]);
    let [peeked, gone] = [&saves[0], &saves[1]];
    refused(
        &source,
        &with(gone, ["sensor", "supports", "calm"]),
        NO_REVEAL,
    );
    let mut paired = with(peeked, ["sensor", "supports", "calm"]);
    paired["observation_qualifications"]["sensor"] = json!({"evidence": ["other"]});
    refused(&source, &paired, NO_REVEAL);
    refused(
        &source,
        &with(peeked, ["other", "opposes", "calm"]),
        "relation other opposes calm: no rule of this program, or procedure one calls, \
         reveals other opposes calm",
    );
}

/// Every save the program writes restores and plays on alike: stances a
/// rule reveals, a reading's stance, a neutral reading.
#[test]
fn the_stances_events_give_restore() {
    let source = program("");
    let saves = plays_and_restores(
        &source,
        &[
            "read", "peek", "count", "quiet", "go", "count", "read", "go",
        ],
    );
    let relations = &saves[saves.len() - 1]["graph"]["relations"];
    assert_eq!(
        relations,
        &json!([
            ["level@1", "supports", "safe"],
            ["other", "supports", "safe"],
            ["sensor", "opposes", "safe"],
            ["level@3", "supports", "safe"],
        ]),
        "{relations}"
    );
}

/// A `reveal` in a procedure a rule calls counts, and one in a procedure no
/// rule calls does not. A stance on renewable evidence's later occurrence
/// is held to the reveals of that evidence.
#[test]
fn reveals_count_through_the_procedures_rules_call() {
    let look = "proc look() {\n    when observed(other) reveal sensor supports calm;\n};\n";
    let uncalled = program(look);
    let saves = plays_and_restores(&uncalled, &["peek", "go"]);
    refused(
        &uncalled,
        &with(&saves[1], ["sensor", "supports", "calm"]),
        NO_REVEAL,
    );

    let called = program(&format!("{look}event again;\non again call look();\n"));
    let saves = plays_and_restores(&called, &["peek", "again", "count", "go"]);
    assert!(saves[3]["graph"]["relations"]
        .as_array()
        .unwrap()
        .contains(&json!(["sensor", "supports", "calm"])));

    let renewable =
        program("renewable sensor limit 4;\nevent refresh;\non refresh renew sensor;\n");
    let saves = plays_and_restores(&renewable, &["peek", "refresh", "go", "count"]);
    assert!(saves[3]["graph"]["relations"]
        .as_array()
        .unwrap()
        .contains(&json!(["sensor@2", "opposes", "safe"])));
    let mut forged = saves[3].clone();
    for relation in forged["graph"]["relations"].as_array_mut().unwrap() {
        if *relation == json!(["sensor@2", "opposes", "safe"]) {
            *relation = json!(["sensor@2", "supports", "calm"]);
        }
    }
    refused(
        &renewable,
        &forged,
        "relation sensor@2 supports calm: no rule of this program, or procedure one calls, \
         reveals sensor supports calm",
    );
}

/// A name the program declares is its own evidence, however it is spelled:
/// `sensor@2` declared beside renewable `sensor` is no occurrence of it, as
/// its renewals list shows, so the reveals of `sensor` give it no stance.
/// Since D102, restore reads it as itself throughout, so the record beside
/// the stance is refused first: no rule reveals `sensor@2`.
#[test]
fn a_declared_name_is_no_occurrence() {
    let source = program("renewable sensor limit 4;\nevidence sensor@2 from \"s2\";\n");
    let saves = plays_and_restores(&source, &["peek", "go", "count"]);
    let mut forged = with(&saves[2], ["sensor@2", "opposes", "safe"]);
    forged["observation_qualifications"]["sensor@2"] = json!({});
    refused(
        &source,
        &forged,
        "observation of sensor@2: no rule or procedure of this program reveals sensor@2",
    );
}

/// A reading holds the stance it was taken with, and no other: another
/// relation, or its own one changed, is refused.
#[test]
fn a_reading_holds_the_stance_it_was_taken_with() {
    let source = program("");
    let saves = plays_and_restores(&source, &["read", "peek"]);
    let save = &saves[1];
    assert!(save["graph"]["relations"]
        .as_array()
        .unwrap()
        .contains(&json!(["level@1", "supports", "safe"])));
    refused(
        &source,
        &with(save, ["level@1", "supports", "calm"]),
        "relation level@1 supports calm: reading level@1 was not taken with that stance",
    );
    let mut changed = save.clone();
    for relation in changed["graph"]["relations"].as_array_mut().unwrap() {
        if *relation == json!(["level@1", "supports", "safe"]) {
            *relation = json!(["level@1", "opposes", "safe"]);
        }
    }
    refused(
        &source,
        &changed,
        "relation level@1 opposes safe: reading level@1 was not taken with that stance",
    );
}

/// No event adds a stance its evidence has: a `reveal` of one adds nothing,
/// and a declaration's is the program's, never the save's. So a stance a
/// declaration gives, or one listed twice, is refused.
#[test]
fn a_stance_the_graph_holds_already_is_refused() {
    let source = program("on peek reveal sensor supports safe;\n");
    let saves = plays_and_restores(&source, &["peek", "go"]);
    refused(
        &source,
        &with(&saves[1], ["sensor", "supports", "safe"]),
        "relation sensor supports safe: a declaration gives it, and no event adds it again",
    );
    refused(
        &source,
        &with(&saves[1], ["other", "supports", "safe"]),
        "relation other supports safe is listed twice",
    );
}

/// The check runs after every older one, so a save they refuse keeps its
/// message: a stance no rule reveals, on declared evidence without its
/// record, is refused as D065 refuses it, and on other evidence without its
/// record as rc.13 does.
#[test]
fn older_checks_keep_their_messages() {
    let source = program("");
    let saves = plays_and_restores(&source, &["count"]);
    refused(
        &source,
        &with(&saves[0], ["sensor", "supports", "calm"]),
        "evidence sensor has no observation record, though it supports calm, a stance no \
         declaration gives it",
    );
    refused(
        &source,
        &with(&saves[0], ["other", "opposes", "calm"]),
        "observed evidence other has no observation record",
    );
}
