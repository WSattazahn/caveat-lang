//! `retaining C because E`: a caveat kept on purpose, with its reason. See
//! spec/caveat-retention-reasons-0.1.md and finding D006.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};

/// D006's deferral: two caveats left for two different reasons.
const DEFERRAL: &str = r#"
claim pass_can_wait;
evidence scope_note from "the release plan: the game does not gate 1.0";
evidence d1_only from "draft 179 only matters for option B";
evidence game from "the game as it is";
evidence operator from "the operator";
caveat presentation_gaps consequence low;
caveat pr179_drift consequence low;
caveat secondhand consequence low;
secondhand qualifies d1_only;
decisions defer_pass limit 4;
decisions plain_defer limit 4;
state seen = 0;
event triage;
event defer;
event defer_plain;
event defer_early;
event retract;
event ask;
on triage reveal scope_note supports pass_can_wait;
on triage reveal d1_only supports pass_can_wait;
on triage reveal game;
on defer when reopened(defer_pass) or not committed(defer_pass)
  commit defer_pass because enough
  retaining presentation_gaps because scope_note, pr179_drift because d1_only;
on defer_plain commit plain_defer because enough retaining presentation_gaps;
on defer_early commit defer_pass because enough retaining presentation_gaps because operator;
on retract reveal operator;
on retract withdraw scope_note because operator;
on retract when committed(defer_pass) and rests_on_withdrawn(defer_pass)
  reopen defer_pass because scope_note;
on ask set seen = if(rests_on_withdrawn(defer_pass), 1, 0);
"#;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn deferred() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(DEFERRAL).unwrap();
    run(&mut game, &[("triage", "{}"), ("defer", "{}")]);
    game
}

#[test]
fn d006_each_retained_caveat_keeps_its_own_reason() {
    let shown = snapshot(&deferred());
    assert_eq!(
        shown["retention_reasons"],
        json!({"defer_pass@1": {"presentation_gaps": "scope_note", "pr179_drift": "d1_only"}})
    );
    // Both reasons are grounds, with the caveat the second carries.
    assert_eq!(
        shown["commitment_grounds"]["defer_pass@1"],
        json!({
            "evidence": ["d1_only", "scope_note"],
            "caveats": ["pr179_drift", "presentation_gaps", "secondhand"]
        })
    );
}

#[test]
fn a_program_that_names_no_reason_has_no_retention_reasons() {
    let mut game = ReactiveSession::from_source(DEFERRAL).unwrap();
    run(&mut game, &[("triage", "{}"), ("defer_plain", "{}")]);
    let shown = snapshot(&game);
    assert!(shown.get("retention_reasons").is_none(), "{shown}");
    assert_eq!(
        shown["commitment_grounds"]["plain_defer@1"],
        json!({"evidence": [], "caveats": ["presentation_gaps"]})
    );
    assert!(!game.save_json().unwrap().contains("retention_reasons"));
}

#[test]
fn withdrawing_a_reason_reaches_the_decision_and_the_record_stays() {
    let mut game = deferred();
    run(&mut game, &[("retract", "{}"), ("ask", "{}")]);
    let shown = snapshot(&game);
    let journal = shown["decision_journal"].as_array().unwrap();
    assert_eq!(journal.last().unwrap()["change"], "reopened", "{shown}");
    assert_eq!(
        shown["retention_reasons"]["defer_pass@1"]["presentation_gaps"],
        "scope_note"
    );
    // The next revision cannot cite the withdrawn reason's occurrence as
    // anything but withdrawn; it still records the pairing.
    run(&mut game, &[("defer", "{}")]);
    assert_eq!(
        snapshot(&game)["retention_reasons"]["defer_pass@2"]["pr179_drift"],
        "d1_only"
    );
}

#[test]
fn an_unobserved_reason_refuses_the_event() {
    let mut session = WebReactiveSession::new(DEFERRAL).unwrap();
    let accepted = json(&session.dispatch_outcome("triage", "{}").unwrap());
    assert_eq!(accepted["outcome"], "accepted");
    let before = json(&session.save().unwrap());
    let result = json(&session.dispatch_outcome("defer_early", "{}").unwrap());
    assert_eq!(result["outcome"], "rejected", "{result}");
    assert_eq!(result["code"], "unobserved_evidence", "{result}");
    assert_eq!(json(&session.save().unwrap()), before);
}

#[test]
fn reasons_save_and_restore_and_restore_checks_them() {
    let mut game = deferred();
    run(&mut game, &[("retract", "{}"), ("defer", "{}")]);
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(DEFERRAL, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    assert_eq!(restored.save_json().unwrap(), saved);

    let tampered = |edit: &dyn Fn(&mut Value)| {
        let mut save = json(&saved);
        edit(&mut save);
        match ReactiveSession::restore_json(DEFERRAL, &save.to_string()) {
            Ok(_) => panic!("a tampered save restored"),
            Err(error) => error,
        }
    };
    for (edit, expected) in [
        (
            Box::new(|save: &mut Value| {
                save["retention_reasons"]["nowhere@1"] = json!({"presentation_gaps": "scope_note"})
            }) as Box<dyn Fn(&mut Value)>,
            "unknown commitment",
        ),
        (
            Box::new(|save: &mut Value| {
                save["retention_reasons"]["defer_pass@1"]["secondhand_typo"] = json!("scope_note")
            }),
            "do not retain",
        ),
        (
            Box::new(|save: &mut Value| {
                save["retention_reasons"]["defer_pass@1"]["presentation_gaps"] = json!("game")
            }),
            "not observed evidence in its grounds",
        ),
        (
            Box::new(|save: &mut Value| {
                save["retention_reasons"]["defer_pass@1"]["presentation_gaps"] = json!("ghost")
            }),
            "unknown evidence",
        ),
    ] {
        let error = tampered(&*edit);
        assert!(error.contains(expected), "{expected}: {error}");
    }
}

#[test]
fn load_rules() {
    for (line, expected) in [
        (
            "on defer commit defer_pass because enough retaining presentation_gaps because;",
            "retaining expects",
        ),
        (
            "on defer commit defer_pass because enough retaining presentation_gaps because seen;",
            "names evidence",
        ),
        (
            "on defer commit defer_pass because enough retaining presentation_gaps because secondhand;",
            "names evidence",
        ),
        (
            "on defer commit defer_pass because enough retaining presentation_gaps because latest(x);",
            "",
        ),
        (
            "on defer commit defer_pass because enough retaining presentation_gaps because a b;",
            "retaining expects",
        ),
        (
            "on defer commit defer_pass because enough retaining presentation_gaps because scope_note, presentation_gaps;",
            "duplicate",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{DEFERRAL}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
}

#[test]
fn a_procedure_names_its_evidence_parameter_as_a_reason() {
    let source = r#"
evidence note from "a note";
caveat gap consequence low;
decisions keep limit 2;
event go;
proc defer(e evidence) {
    reveal e;
    commit keep because enough retaining gap because e;
};
on go call defer(note);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(&mut game, &[("go", "{}")]);
    assert_eq!(
        snapshot(&game)["retention_reasons"],
        json!({"keep@1": {"gap": "note"}})
    );
}
