//! Withdrawing an observation. See spec/caveat-withdrawal-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};
use std::collections::BTreeMap;

const LEDGER: &str = r#"
claim safe;
claim misreading;
evidence ci from "the pull request's checks";
evidence recheck from "the agent re-read the log";
evidence second_look from "another re-read";
evidence gate from "a gate only a guard reads";
readings checks from ci limit 8;
decisions merge limit 4;
state estimate = 0;
state total = 0;
state first = 0;
state decided = 0;
state gated = 0;
event check result min 0 max 1;
event decide;
event decide_behind_gate;
event open_gate;
event misread;
event misread_gate;
event misread_again;
event recount;
on check sample checks = result supports safe;
on check set estimate = latest(checks);
on decide commit merge because enough using latest(checks);
on open_gate reveal gate supports safe;
on decide_behind_gate when observed(gate) commit merge because enough using latest(checks);
on misread when not observed(recheck) reveal recheck supports misreading;
on misread withdraw latest(checks) because recheck;
on misread_gate when not observed(recheck) reveal recheck supports misreading;
on misread_gate withdraw gate because recheck;
on misread_again when not observed(second_look) reveal second_look supports misreading;
on misread_again withdraw latest(checks) because second_look;
on recount set estimate = latest(checks);
on recount set total = fold_history(checks, 0, sum);
on recount set first = history_at(checks, 0);
on recount set decided = latest(merge);
bind status.rests = 0;
bind status.rests = 1 when rests_on_withdrawn(merge);
bind status.latest = 0;
bind status.latest = 1 when withdrawn(latest(checks));
bind status.gate = 0;
bind status.gate = 1 when withdrawn(gate);
fn sum(total, reading) = total + reading;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn send(game: &mut ReactiveSession, event: &str, payload: Value) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, &payload.to_string())
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn go(game: &mut ReactiveSession, event: &str) -> Value {
    send(game, event, json!({}))
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn caveats(provenance: &Value) -> Vec<String> {
    provenance["caveats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|caveat| caveat.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_withdrawal_adds_a_record_marks_what_is_current_and_changes_nothing_recorded() {
    let mut game = session(LEDGER);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "decide");
    let before = snapshot(&game);
    let outcome = go(&mut game, "misread");
    assert_eq!(outcome["outcome"], "accepted");
    let after = snapshot(&game);
    assert_eq!(
        after["withdrawals"],
        json!([{ "evidence": "checks@1", "because": "recheck", "sequence": 3, "event": "misread" }])
    );
    for path in [
        "commitment_grounds",
        "commitment_bases",
        "decision_journal",
        "reading_streams",
    ] {
        assert_eq!(after[path], before[path], "{path} changed");
    }
    // The observation and its relation to its claim stay; nothing opposes it.
    let relations = after["relations"].as_array().unwrap();
    assert!(relations
        .iter()
        .any(|r| r["from"] == "checks@1" && r["relation"] == "supports" && r["to"] == "safe"));
    assert!(!relations
        .iter()
        .any(|r| r["from"] == "checks@1" && r["relation"] == "opposes"));
    // What is current now shows the withdrawal.
    assert!(
        caveats(&after["qualified_values"]["estimate"]["provenance"])
            .contains(&"withdrawn".to_string())
    );
    assert!(caveats(&after["value_grounds"]["estimate"]).contains(&"withdrawn".to_string()));
    // Nothing reopened by itself.
    assert_eq!(after["decision_series"]["merge"]["current"], "merge@1");
    assert_eq!(after["decision_journal"].as_array().unwrap().len(), 1);
    assert_eq!(after["bindings"]["status"]["rests"], 1.0);
    assert_eq!(after["bindings"]["status"]["latest"], 1.0);
    assert!(after["effects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|effect| *effect
            == json!({
                "kind": "withdraw", "evidence": "checks@1", "because": "recheck"
            })));
}

#[test]
fn a_decision_reopens_only_when_the_program_says_so() {
    let source = format!(
        "{LEDGER}\non misread when committed(merge) and not reopened(merge) and rests_on_withdrawn(merge) reopen merge because recheck;"
    );
    let mut game = session(&source);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "decide");
    go(&mut game, "misread");
    let journal = snapshot(&game)["decision_journal"].clone();
    assert_eq!(journal[1]["change"], "reopened");
    assert_eq!(journal[1]["because"], json!(["recheck"]));
    assert_eq!(journal[0]["because"], json!(["checks@1"]));
}

#[test]
fn reading_withdrawn_history_again_keeps_the_withdrawal() {
    let mut game = session(LEDGER);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "decide");
    go(&mut game, "misread");
    go(&mut game, "recount");
    let shot = snapshot(&game);
    for state in ["estimate", "total", "first", "decided"] {
        assert!(
            caveats(&shot["qualified_values"][state]["provenance"])
                .contains(&"withdrawn".to_string()),
            "{state} lost the withdrawal: {}",
            shot["qualified_values"][state]
        );
        assert!(
            caveats(&shot["value_grounds"][state]).contains(&"withdrawn".to_string()),
            "{state} grounds"
        );
    }
    // The archive still holds the reading as it was taken.
    let reading = &shot["reading_streams"]["checks"]["occurrences"][0];
    assert!(!caveats(&reading["provenance"]).contains(&"withdrawn".to_string()));
    assert_eq!(reading["value"], 1.0);

    // A new reading is not withdrawn; a fold over both still rests on the old one.
    send(&mut game, "check", json!({ "result": 0 }));
    go(&mut game, "recount");
    let shot = snapshot(&game);
    assert!(
        !caveats(&shot["qualified_values"]["estimate"]["provenance"])
            .contains(&"withdrawn".to_string())
    );
    assert!(caveats(&shot["qualified_values"]["total"]["provenance"])
        .contains(&"withdrawn".to_string()));
    assert_eq!(shot["bindings"]["status"]["latest"], 0.0);
}

#[test]
fn evidence_only_a_guard_read_is_not_what_a_decision_rests_on() {
    let mut game = session(LEDGER);
    go(&mut game, "open_gate");
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "decide_behind_gate");
    let shot = snapshot(&game);
    assert_eq!(
        shot["commitment_grounds"]["merge@1"]["evidence"],
        json!(["checks@1"])
    );
    assert!(
        shot["commitment_bases"]["merge@1"]["provenance"]["evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("gate"))
    );
    go(&mut game, "misread_gate");
    let shot = snapshot(&game);
    assert_eq!(shot["bindings"]["status"]["gate"], 1.0);
    assert_eq!(
        shot["bindings"]["status"]["rests"], 0.0,
        "a guard's evidence counted as grounds"
    );
    go(&mut game, "misread_again");
    assert_eq!(snapshot(&game)["bindings"]["status"]["rests"], 1.0);
}

#[test]
fn a_fresh_occurrence_is_not_withdrawn_because_an_earlier_one_was() {
    let source = r#"
        claim safe; claim misreading;
        evidence taste from "a taste"; evidence recheck from "a second opinion";
        renewable taste limit 4;
        decisions trust limit 2;
        state fresh = 0;
        event eat; event doubt; event regrow; event eat_again;
        on eat reveal taste supports safe;
        on eat commit trust because enough using qualified(1, taste);
        on doubt reveal recheck supports misreading;
        on doubt withdraw taste because recheck;
        on regrow renew taste;
        on eat_again reveal taste supports safe;
        on eat_again set fresh = qualified(1, taste);
        bind status.current = 0;
        bind status.current = 1 when withdrawn(taste);
        bind status.rests = 0;
        bind status.rests = 1 when rests_on_withdrawn(trust);
    "#;
    let mut game = session(source);
    for event in ["eat", "doubt"] {
        go(&mut game, event);
    }
    assert_eq!(snapshot(&game)["bindings"]["status"]["current"], 1.0);
    for event in ["regrow", "eat_again"] {
        go(&mut game, event);
    }
    let shot = snapshot(&game);
    assert_eq!(
        shot["withdrawals"][0]["evidence"], "taste",
        "the withdrawal stays with the first occurrence"
    );
    assert_eq!(shot["bindings"]["status"]["current"], 0.0);
    assert_eq!(shot["bindings"]["status"]["rests"], 1.0);
    assert!(!caveats(&shot["qualified_values"]["fresh"]["provenance"])
        .contains(&"withdrawn".to_string()));
}

#[test]
fn a_refused_event_withdraws_nothing() {
    let source = format!(
        "{LEDGER}\nevent misread_refused;\non misread_refused when not observed(recheck) reveal recheck supports misreading;\non misread_refused withdraw latest(checks) because recheck;\non misread_refused when committed(merge) and not reopened(merge) reopen merge because recheck;\non misread_refused reject \"not now\";"
    );
    let mut game = session(&source);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "decide");
    let before = (snapshot(&game), game.save_json().unwrap());
    let outcome = go(&mut game, "misread_refused");
    assert_eq!(
        (outcome["outcome"].as_str(), outcome["origin"].as_str()),
        (Some("rejected"), Some("policy"))
    );
    assert_eq!((snapshot(&game), game.save_json().unwrap()), before);
    assert!(snapshot(&game).get("withdrawals").is_none());
}

#[test]
fn withdrawing_again_keeps_the_first_record() {
    let mut game = session(LEDGER);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "misread");
    let outcome = go(&mut game, "misread_again");
    let shot = snapshot(&game);
    assert_eq!(shot["withdrawals"].as_array().unwrap().len(), 1);
    assert_eq!(shot["withdrawals"][0]["because"], "recheck");
    assert!(!outcome["snapshot"]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|effect| effect["kind"] == "withdraw"));
}

#[test]
fn withdrawing_unobserved_evidence_or_an_empty_stream_fails_the_event() {
    let source = format!("{LEDGER}\nevent early;\non early withdraw latest(checks) because recheck;\nevent unfounded;\non unfounded withdraw gate because recheck;");
    let mut game = session(&source);
    let before = (snapshot(&game), game.save_json().unwrap());
    for event in ["early", "unfounded"] {
        let error = game.dispatch_outcome_json(event, "{}").unwrap_err();
        assert_eq!(error.code, "unclassified", "{event}");
        assert_eq!(
            (snapshot(&game), game.save_json().unwrap()),
            before,
            "{event}"
        );
    }
}

#[test]
fn only_withdraw_applies_the_withdrawn_caveat() {
    for (extra, expected) in [
        ("caveat withdrawn consequence low;", "cannot declare it"),
        ("state withdrawn = 0;", "cannot declare it"),
        (
            "event taint;\non taint qualify ci with withdrawn;",
            "qualify cannot apply withdrawn",
        ),
        (
            "event taint;\non taint set estimate = qualified(1, ci, withdrawn);",
            "qualified cannot apply withdrawn",
        ),
        (
            "event hold;\non hold commit merge because enough retaining withdrawn;",
            "retaining cannot apply withdrawn",
        ),
        (
            "budget 3;\nevent look;\non look examine withdrawn cost 1;",
            "examine cannot apply withdrawn",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{LEDGER}\n{extra}"))
            .err()
            .unwrap();
        assert!(error.contains(expected), "{extra}: {error}");
    }
    // Reading it is allowed.
    session(&format!(
        "{LEDGER}\nbind status.flag = has_caveat(estimate, withdrawn);"
    ));
    // A program that never withdraws may use the name as before.
    session("caveat withdrawn consequence low;\nstate x = 0;\nevent go;\non go set x = 1;");
}

#[test]
fn procedures_can_withdraw_a_stream_they_are_given() {
    let source = format!(
        "{LEDGER}\nproc retract(s readings, r evidence) {{ withdraw latest(s) because r; }};\nevent retracted;\non retracted when not observed(second_look) reveal second_look supports misreading;\non retracted call retract(checks, second_look);"
    );
    let mut game = session(&source);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "retracted");
    assert_eq!(
        snapshot(&game)["withdrawals"][0],
        json!({
            "evidence": "checks@1", "because": "second_look", "sequence": 2, "event": "retracted"
        })
    );
}

#[test]
fn save_and_restore_keep_withdrawals_and_refuse_inconsistent_ones() {
    let mut game = session(LEDGER);
    send(&mut game, "check", json!({ "result": 1 }));
    go(&mut game, "decide");
    go(&mut game, "misread");
    let saved = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(LEDGER, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    for session in [&mut game, &mut restored] {
        go(session, "recount");
        send(session, "check", json!({ "result": 0 }));
    }
    assert_eq!(snapshot(&restored), snapshot(&game));

    let parsed: Value = serde_json::from_str(&saved).unwrap();
    let with = |change: &dyn Fn(&mut Value)| {
        let mut save = parsed.clone();
        change(&mut save);
        save.to_string()
    };
    let record = parsed["withdrawals"][0].clone();
    let cases: Vec<(String, &str)> = vec![
        (
            with(&|save| {
                save.as_object_mut().unwrap().remove("withdrawals");
            }),
            "disagree with the graph",
        ),
        (
            with(&|save| save["withdrawals"][0]["evidence"] = json!("gate")),
            "disagree with the graph",
        ),
        (
            with(&|save| save["withdrawals"][0]["evidence"] = json!("nowhere")),
            "unknown evidence",
        ),
        (
            with(&|save| save["withdrawals"] = json!([record.clone(), record.clone()])),
            "withdrawn twice",
        ),
        (
            with(&|save| save["withdrawals"][0]["sequence"] = json!(99)),
            "out of sequence",
        ),
        (
            with(&|save| save["withdrawals"][0]["event"] = json!("nothing")),
            "unknown event",
        ),
    ];
    for (text, expected) in cases {
        let error = ReactiveSession::restore_json(LEDGER, &text).err().unwrap();
        assert!(error.contains(expected), "{expected}: {error}");
    }
    let plain = "state x = 0;\nevent go;\non go set x = 1;";
    let mut other = session(plain);
    other.apply("go", &BTreeMap::new()).unwrap();
    let mut save: Value = serde_json::from_str(&other.save_json().unwrap()).unwrap();
    save["withdrawals"] = json!([record]);
    let error = ReactiveSession::restore_json(plain, &save.to_string())
        .err()
        .unwrap();
    assert!(error.contains("does not withdraw"), "{error}");
}
