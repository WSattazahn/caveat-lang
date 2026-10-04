//! Restore refuses a `qualifies` relation or a pending scheduled
//! qualification that no mechanism of the loaded source can make (F95, F260).
//! See spec/caveat-save-0.1.md and docs/RESTORE_TRUST_BOUNDARY.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// One qualification by each mechanism that can make one: a rule, a
/// procedure specialized for the names it is passed, a reading inheriting
/// its template's caveat, a renewal carrying its evidence's caveat, a
/// withdrawal, and a delayed `qualify ... after`.
const PATHS: &str = r#"
claim safe;
evidence direct from "direct";
evidence viaproc from "procedure";
evidence template from "template";
evidence renewed from "renewed";
evidence pulled from "pulled";
evidence delayed from "delayed";
evidence reason from "reason";
caveat c_direct consequence low;
caveat c_proc consequence low;
caveat c_template consequence low;
caveat c_renewed consequence low;
caveat c_delayed consequence low;
caveat never consequence low;
c_template qualifies template;
c_renewed qualifies renewed;
readings stream from template limit 4;
renewable renewed limit 3;
event tick dt min 0 max 0.1;
event go;
event read x min 0 max 1;
event regrow;
event pull;
proc stamp(e evidence, c caveat) { qualify e with c; };
on go reveal direct supports safe;
on go reveal viaproc supports safe;
on go reveal renewed supports safe;
on go reveal pulled supports safe;
on go reveal delayed supports safe;
on go reveal reason supports safe;
on go qualify direct with c_direct;
on go call stamp(viaproc, c_proc);
on go qualify delayed with c_delayed after 0.05;
on read sample stream = x supports safe;
on regrow renew renewed;
on pull withdraw pulled because reason;
"#;

fn send(game: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = serde_json::to_value(
        game.dispatch_outcome_json(event, payload)
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

/// A session that has made a qualification by every path, and holds one
/// delayed qualification still pending.
fn played() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(PATHS).unwrap();
    for (event, payload) in [
        ("go", "{}"),
        ("read", r#"{"x": 1}"#),
        ("regrow", "{}"),
        ("pull", "{}"),
        ("tick", r#"{"dt": 0.1}"#),
        ("go", "{}"),
    ] {
        send(&mut game, event, payload);
    }
    game
}

fn saved() -> Value {
    serde_json::to_value(played().save().unwrap()).unwrap()
}

fn refused(save: &Value, expected: &str) {
    match ReactiveSession::restore_json(PATHS, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(game) => panic!(
            "accepted with relations {}",
            serde_json::to_value(game.snapshot()).unwrap()["relations"]
        ),
    }
}

// The six paths' genuine save restores with its relations, occurrences and
// schedule intact, plays on as the original, and saves and restores again.
#[test]
fn f260_a_qualification_from_each_source_path_restores() {
    let mut original = played();
    let save = serde_json::to_value(original.save().unwrap()).unwrap();
    let qualifies = save["graph"]["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|relation| relation[1] == "qualifies")
        .cloned()
        .collect::<Vec<_>>();
    for relation in [
        json!(["c_direct", "qualifies", "direct"]),
        json!(["c_proc", "qualifies", "viaproc"]),
        json!(["c_template", "qualifies", "stream@1"]),
        json!(["c_renewed", "qualifies", "renewed@2"]),
        json!(["withdrawn", "qualifies", "pulled"]),
        json!(["c_delayed", "qualifies", "delayed"]),
    ] {
        assert!(qualifies.contains(&relation), "{relation} in {qualifies:?}");
    }
    assert_eq!(
        save["scheduled_qualifications"][0]["caveat"], "c_delayed",
        "a delayed qualification is pending"
    );
    let mut resumed = ReactiveSession::restore_json(PATHS, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(resumed.snapshot(), original.snapshot());
    for (event, payload) in [
        ("tick", r#"{"dt": 0.1}"#),
        ("read", r#"{"x": 0}"#),
        ("regrow", "{}"),
    ] {
        send(&mut original, event, payload);
        send(&mut resumed, event, payload);
        assert_eq!(resumed.snapshot(), original.snapshot(), "after {event}");
    }
    let again = ReactiveSession::restore_json(PATHS, &resumed.save_json().unwrap()).unwrap();
    assert_eq!(again.snapshot(), original.snapshot());
}

// F260: an injected or retargeted `qualifies` relation restored and was read
// as live, though nothing in the program could relate that pair.
#[test]
fn f260_a_qualification_no_source_mechanism_makes_is_refused() {
    let no_mechanism = |caveat: &str, evidence: &str| {
        format!(
            "relation {caveat} qualifies {evidence}: no rule, declaration, reading, renewal or \
             withdrawal of this program qualifies {evidence} with {caveat}"
        )
    };
    let relations = |save: &mut Value| save["graph"]["relations"].as_array_mut().unwrap().clone();
    // An unrelated declared caveat added to observed evidence.
    let mut added = saved();
    added["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .push(json!(["never", "qualifies", "direct"]));
    refused(&added, &no_mechanism("never", "direct"));
    // A genuine relation retargeted to other evidence.
    let mut retargeted = saved();
    let mut moved = relations(&mut retargeted);
    let at = moved
        .iter()
        .position(|relation| *relation == json!(["c_direct", "qualifies", "direct"]))
        .unwrap();
    moved[at] = json!(["c_direct", "qualifies", "viaproc"]);
    retargeted["graph"]["relations"] = Value::Array(moved);
    refused(&retargeted, &no_mechanism("c_direct", "viaproc"));
    // An occurrence named by its stream or renewable evidence takes only what
    // that source can qualify it with.
    for (caveat, evidence, source) in [
        ("c_renewed", "stream@1", "stream"),
        ("c_template", "renewed@2", "renewed"),
    ] {
        let mut save = saved();
        save["graph"]["relations"]
            .as_array_mut()
            .unwrap()
            .push(json!([caveat, "qualifies", evidence]));
        refused(
            &save,
            &format!(
                "relation {caveat} qualifies {evidence}: no rule, declaration, reading, renewal \
                 or withdrawal of this program qualifies {source} with {caveat}"
            ),
        );
    }
}

// F95: a pending scheduled qualification retagged to another caveat or other
// evidence restored and later fired. Only a `qualify ... after` schedules one.
#[test]
fn f95_a_scheduled_qualification_no_rule_schedules_is_refused() {
    for (field, value, evidence, caveat) in [
        ("caveat", "c_direct", "delayed", "c_direct"),
        ("caveat", "never", "delayed", "never"),
        ("evidence", "direct", "direct", "c_delayed"),
    ] {
        let mut save = saved();
        save["scheduled_qualifications"][0][field] = value.into();
        refused(
            &save,
            &format!(
                "scheduled qualification of {evidence} with {caveat}: no qualify ... after of \
                 this program schedules it"
            ),
        );
    }
}

// The check is of what the source can make, not of what an event made: a
// relation a guarded rule could add restores though its guard never held.
// Hosts that need the account itself to be authentic keep the save text
// intact and trusted (docs/RESTORE_TRUST_BOUNDARY.md).
#[test]
fn f260_a_possible_qualification_restores_without_proof_its_guard_held() {
    let source = format!("{PATHS}\ncaveat c_guarded consequence low;\non go when observed(never_seen) qualify direct with c_guarded;\nevidence never_seen from \"unseen\";\n");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    send(&mut game, "go", "{}");
    let mut save = serde_json::to_value(game.save().unwrap()).unwrap();
    save["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .push(json!(["c_guarded", "qualifies", "direct"]));
    let restored = ReactiveSession::restore_json(&source, &save.to_string())
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(
        serde_json::to_value(restored.snapshot()).unwrap()["relations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|relation| relation["from"] == "c_guarded" && relation["origin"] == "live")
    );
}
