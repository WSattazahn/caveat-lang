//! `reading(STREAM, INDEX)`: a stream's earlier reading, named wherever
//! `occurrence(EVIDENCE, INDEX)` names one. See
//! spec/caveat-reading-index-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const LEDGER: &str = r#"
claim passing;
caveat flaky consequence material;
evidence runner from "the test runner";
evidence recheck from "a re-run";
readings checks from runner limit 8;
decisions ship limit 8;
state first = 0;
state second = 0;
state first_withdrawn = 0;
state first_flaky = 0;
state first_seen = 0;
event report v min 0 max 1;
event see_recheck;
event decide;
event flag i min 1 max 8;
event unflag i min 1 max 8;
event retract i min 1 max 8;
event revisit i min 1 max 8;
event recount;
event ask;
on report sample checks = v supports passing;
on see_recheck reveal recheck;
on decide commit ship because enough using latest(checks);
on flag qualify reading(checks, i) with flaky;
on unflag unqualify reading(checks, i) with flaky;
on retract withdraw reading(checks, i) because recheck;
on revisit when committed(ship) reopen ship because reading(checks, i);
on recount set first = history_at(checks, 0);
on recount set second = history_at(checks, 1);
on ask set first_withdrawn = if(withdrawn(reading(checks, 1)), 1, 0);
on ask set first_flaky = if(carries(reading(checks, 1), flaky), 1, 0);
on ask set first_seen = if(observed(reading(checks, 1)), 1, 0);
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("load: {error}"))
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn caveats(game: &ReactiveSession, state: &str) -> Vec<String> {
    snapshot(game)["qualified_values"][state]["provenance"]["caveats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|caveat| caveat.as_str().unwrap().to_string())
        .collect()
}

fn value(game: &ReactiveSession, state: &str) -> f64 {
    snapshot(game)["qualified_values"][state]["value"]
        .as_f64()
        .unwrap()
}

fn outcome(game: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap()).unwrap()
}

fn restores(source: &str, game: &ReactiveSession) {
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("restore: {error}"));
    assert_eq!(restored.save_json().unwrap(), save);
}

const TWO: [(&str, &str); 2] = [("report", r#"{"v":1}"#), ("report", r#"{"v":0}"#)];

#[test]
fn an_earlier_reading_can_be_withdrawn_after_later_ones_arrive() {
    let mut game = session(LEDGER);
    run(&mut game, &TWO);
    run(
        &mut game,
        &[
            ("see_recheck", "{}"),
            ("retract", r#"{"i":1}"#),
            ("recount", "{}"),
            ("ask", "{}"),
        ],
    );
    assert_eq!(value(&game, "first_withdrawn"), 1.0);
    assert!(caveats(&game, "first").contains(&"withdrawn".to_string()));
    assert!(!caveats(&game, "second").contains(&"withdrawn".to_string()));
    let withdrawals = &snapshot(&game)["withdrawals"];
    assert!(
        withdrawals.to_string().contains("checks@1"),
        "{withdrawals}"
    );
    restores(LEDGER, &game);
}

#[test]
fn an_earlier_reading_can_be_qualified_and_taken_back_alone() {
    let mut game = session(LEDGER);
    run(&mut game, &TWO);
    run(
        &mut game,
        &[
            ("flag", r#"{"i":1}"#),
            ("flag", r#"{"i":2}"#),
            ("recount", "{}"),
            ("ask", "{}"),
        ],
    );
    assert!(caveats(&game, "first").contains(&"flaky".to_string()));
    assert!(caveats(&game, "second").contains(&"flaky".to_string()));
    assert_eq!(value(&game, "first_flaky"), 1.0);
    restores(LEDGER, &game);
    run(
        &mut game,
        &[("unflag", r#"{"i":1}"#), ("recount", "{}"), ("ask", "{}")],
    );
    assert!(!caveats(&game, "first").contains(&"flaky".to_string()));
    assert!(caveats(&game, "second").contains(&"flaky".to_string()));
    assert_eq!(value(&game, "first_flaky"), 0.0);
    let taken: Vec<String> = snapshot(&game)["taken_back"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["evidence"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(taken, ["checks@1"]);
    restores(LEDGER, &game);
}

#[test]
fn a_decision_can_be_reopened_because_of_an_earlier_reading() {
    let mut game = session(LEDGER);
    run(&mut game, &TWO);
    run(&mut game, &[("decide", "{}"), ("revisit", r#"{"i":1}"#)]);
    let effects = snapshot(&game)["effects"].to_string();
    assert!(
        effects.contains(r#""kind":"reopen""#) && effects.contains("checks@1"),
        "{effects}"
    );
    restores(LEDGER, &game);
}

#[test]
fn an_index_that_names_no_held_reading_refuses_the_event() {
    let mut game = session(LEDGER);
    run(&mut game, &TWO);
    for (event, payload) in [("flag", r#"{"i":3}"#), ("retract", r#"{"i":3}"#)] {
        let shown = outcome(&mut game, event, payload);
        assert_eq!(shown["outcome"], "rejected", "{shown}");
        assert_eq!(shown["code"], "expression", "{shown}");
    }
    // Asked of a reading not taken yet, the predicates answer false.
    let mut fresh = session(LEDGER);
    run(&mut fresh, &[("ask", "{}")]);
    assert_eq!(value(&fresh, "first_seen"), 0.0);
    run(&mut fresh, &[("report", r#"{"v":1}"#), ("ask", "{}")]);
    assert_eq!(value(&fresh, "first_seen"), 1.0);
}

#[test]
fn a_retired_reading_cannot_be_named() {
    let source = LEDGER.replace(
        "readings checks from runner limit 8;",
        "readings checks from runner window 2;",
    );
    let mut game = session(&source);
    run(&mut game, &TWO);
    run(&mut game, &[("report", r#"{"v":1}"#)]);
    let shown = outcome(&mut game, "flag", r#"{"i":1}"#);
    assert_eq!(shown["outcome"], "rejected", "{shown}");
    run(&mut game, &[("flag", r#"{"i":2}"#)]);
    for step in 0..4 {
        run(
            &mut game,
            &[
                ("report", r#"{"v":0}"#),
                ("flag", &format!(r#"{{"i":{}}}"#, step + 4)),
            ],
        );
        restores(&source, &game);
    }
}

#[test]
fn reading_names_a_reading_stream_and_keeps_the_template_rule() {
    let with = |line: &str| format!("{LEDGER}{line}\n");
    assert!(!load_error(&with(
        "event x;\non x qualify reading(runner, 1) with flaky;"
    ))
    .is_empty());
    assert!(!load_error(&with(
        "event x;\non x withdraw reading(nothing, 1) because recheck;"
    ))
    .is_empty());
    let error = load_error(&with("event doubt;\non doubt qualify runner with flaky;"));
    assert!(error.contains("puts it on the template"), "{error}");
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_default()
}
