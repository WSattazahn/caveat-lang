//! D063: restore accepted a renewals list naming an occurrence of renewable
//! evidence that no rule renews. Only a `renew` makes an occurrence, and the
//! renewals check held the list to its first entry, its order, its kind and
//! the declared capacity, never to the program's rules, so a save could list
//! `sensor@2` (with the graph node to match) in a program with no `renew`.
//! The restored session then read that occurrence as current: a later
//! `qualified(1, sensor)` was refused as `unobserved_evidence` where the
//! live session accepted it. Restore now refuses an occurrence past the
//! evidence itself unless a rule, or a procedure a rule reaches, renews the
//! evidence, with a guard or without. Conditions are not evaluated, so a
//! listed occurrence of evidence some rule renews restores whether or not an
//! event made it, as before. See spec/caveat-save-0.1.md ("Restore
//! validation", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

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
        Ok(_) => panic!("accepted {}", save["renewals"]),
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

/// The finding's program, with `renews` added: `sensor` is renewable and
/// supports `safe`, `peek` reveals `other`, `see` reveals `sensor`'s current
/// occurrence, and `count` reads it.
fn program(renews: &str) -> String {
    format!(
        "claim safe;
evidence sensor from \"sensor\";
evidence other from \"o\";
renewable sensor limit 4;
sensor supports safe;
state seen = 0;
event peek;
event count;
event regrow;
event see;
on peek reveal other supports safe;
on see reveal sensor supports safe;
on count set seen = seen + qualified(1, sensor);
{renews}"
    )
}

/// The save after `peek`, listing `sensor@2` as if a renewal had made it.
fn forged(source: &str) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    assert_eq!(answer(&mut live, "peek")["outcome"], "accepted");
    let mut save = saved(&live);
    assert_eq!(save["renewals"], Value::Null, "{}", save["renewals"]);
    save["graph"]["nodes"] = json!([{"kind": "occurrence", "name": "sensor@2"}]);
    save["renewals"] = json!({"sensor": ["sensor", "sensor@2"]});
    save
}

/// No rule renews `sensor`, or only a procedure no rule calls does, so no
/// session lists `sensor@2`: restore refuses it.
#[test]
fn an_occurrence_no_rule_renews_is_refused() {
    for renews in ["", "proc again() { renew sensor; };\n"] {
        let source = program(renews);
        refused(
            &source,
            &forged(&source),
            "renewable sensor occurrence sensor@2: no rule or procedure of this program renews sensor",
        );
    }
}

/// A rule that renews `sensor`, guarded or not, directly or through a
/// procedure, lets the renewals list its occurrences: every save the session
/// writes restores and plays on alike, and the forged list restores too,
/// whether or not an event made it.
#[test]
fn an_occurrence_a_rule_renews_restores() {
    for renews in [
        "on regrow renew sensor;\n",
        "on regrow when observed(other) renew sensor;\n",
        "proc again() { renew sensor; };\non regrow call again();\n",
    ] {
        let source = program(renews);
        let save = plays_and_restores(
            &source,
            &[
                "peek", "regrow", "see", "count", "regrow", "see", "count", "peek",
            ],
        );
        assert_eq!(
            save["renewals"],
            json!({"sensor": ["sensor", "sensor@2", "sensor@3"]}),
            "{renews}"
        );
        let forged = forged(&source);
        ReactiveSession::restore_json(&source, &forged.to_string())
            .unwrap_or_else(|error| panic!("{renews}: {error}"));
    }
}

/// Conditions are not evaluated: the forged list restores under a renewing
/// rule whose guard is false in the save, and the restored session, reading
/// the guard as the save left it, renews nothing on a later `regrow`.
#[test]
fn an_occurrence_restores_though_its_guard_is_false() {
    let source = program("on regrow when seen > 5 renew sensor;\n");
    let forged = forged(&source);
    let mut restored = ReactiveSession::restore_json(&source, &forged.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    answer(&mut restored, "regrow");
    assert_eq!(saved(&restored)["renewals"], forged["renewals"]);
}

/// The check runs after every older one, so a save one of them refuses
/// keeps its message: here the history-reach check (D054, D061) refuses
/// `seen`'s lineage citing `other`, which `count` never reads, before this
/// check reaches `sensor@2`.
#[test]
fn a_save_an_older_check_refuses_keeps_its_message() {
    let source = program("");
    let mut save = forged(&source);
    save["states"]["seen"] = json!({"value": 0.0, "lineage": {"evidence": ["other"]}});
    refused(
        &source,
        &save,
        "state seen: no rule or declaration of this program carries records of other into it",
    );
}

/// Under a window the renewals list keeps only the occurrences still held:
/// `sensor@3` departs, and `sensor@2` stays, pinned by `seen`'s lineage.
/// Those restore as before.
#[test]
fn a_windowed_renewal_restores_after_its_occurrences_depart() {
    let source = program("on regrow renew sensor;\n")
        .replace("renewable sensor limit 4;", "renewable sensor window 1;");
    let save = plays_and_restores(
        &source,
        &[
            "peek", "regrow", "see", "count", "regrow", "regrow", "see", "count",
        ],
    );
    assert_eq!(
        save["renewals"],
        json!({"sensor": ["sensor", "sensor@2", "sensor@4"]})
    );
}

/// Each renewable evidence is held to the rules that renew it, not to any
/// rule that renews something.
#[test]
fn a_renewal_of_other_evidence_does_not_count() {
    let source = program(
        "evidence probe from \"p\";
renewable probe limit 4;
on regrow renew probe;
",
    );
    plays_and_restores(&source, &["peek", "regrow", "count"]);
    refused(
        &source,
        &forged(&source),
        "renewable sensor occurrence sensor@2: no rule or procedure of this program renews sensor",
    );
}
