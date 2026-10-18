//! `qualify latest(NAME)` and `unqualify latest(NAME)`: a reading stream's
//! current reading, or evidence's current occurrence. See
//! spec/caveat-latest-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const LEDGER: &str = r#"
claim passing;
caveat flaky consequence material;
evidence runner from "the test runner";
readings checks from runner limit 8;
decisions ship limit 8;
state last = 0;
state first = 0;
event report v min 0 max 1;
event flag;
event flag_later;
event unflag;
event decide;
event recount;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on report sample checks = v supports passing;
on report set last = latest(checks);
on flag qualify latest(checks) with flaky;
on flag_later qualify latest(checks) with flaky after 0.2;
on unflag unqualify latest(checks) with flaky;
on decide commit ship because enough using latest(checks);
on recount set last = latest(checks);
on recount set first = history_at(checks, 0);
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

fn carries(game: &ReactiveSession, state: &str, caveat: &str) -> bool {
    snapshot(game)["qualified_values"][state]["provenance"]["caveats"]
        .as_array()
        .unwrap()
        .iter()
        .any(|name| name == caveat)
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_default()
}

#[test]
fn qualify_latest_reaches_the_current_reading_only() {
    let mut game = session(LEDGER);
    run(&mut game, &[("report", r#"{"v":1}"#), ("flag", "{}")]);
    assert!(carries(&game, "last", "flaky"));
    let effects = &snapshot(&game)["effects"];
    assert!(
        effects
            .as_array()
            .unwrap()
            .iter()
            .any(|effect| effect["kind"] == "qualify"
                && effect["evidence"] == "checks@1"
                && effect["caveat"] == "flaky"),
        "{effects}"
    );
    // A new reading is a new observation: it does not carry the caveat.
    run(&mut game, &[("report", r#"{"v":1}"#)]);
    assert!(!carries(&game, "last", "flaky"));
    // The flagged reading still does, read again through history.
    run(&mut game, &[("recount", "{}")]);
    assert!(carries(&game, "first", "flaky"));
    assert!(!carries(&game, "last", "flaky"));
}

#[test]
fn qualify_latest_of_a_stream_with_no_reading_is_refused() {
    let mut game = session(LEDGER);
    let outcome = serde_json::to_value(game.dispatch_outcome_json("flag", "{}").unwrap()).unwrap();
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    assert_eq!(outcome["code"], "unobserved_evidence", "{outcome}");
}

#[test]
fn unqualify_latest_takes_back_only_the_current_readings_caveat() {
    let mut game = session(LEDGER);
    run(
        &mut game,
        &[
            ("report", r#"{"v":1}"#),
            ("flag", "{}"),
            ("decide", "{}"),
            ("report", r#"{"v":0}"#),
            ("flag", "{}"),
        ],
    );
    assert!(carries(&game, "last", "flaky"));
    run(&mut game, &[("unflag", "{}")]);
    assert!(!carries(&game, "last", "flaky"));
    let shown = snapshot(&game);
    let taken: Vec<&str> = shown["taken_back"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["evidence"].as_str().unwrap())
        .collect();
    assert_eq!(taken, ["checks@2"]);
    assert!(
        shown["effects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|effect| effect["kind"] == "unqualify"
                && effect["evidence"] == "checks"
                && effect["occurrences"] == serde_json::json!(["checks@2"])),
        "{}",
        shown["effects"]
    );
    // The first reading's caveat stands, and the decision made on it keeps
    // what it was made on.
    run(&mut game, &[("recount", "{}")]);
    assert!(carries(&game, "first", "flaky"));
    let basis = &snapshot(&game)["commitment_bases"]["ship@1"]["provenance"]["caveats"];
    assert!(
        basis.as_array().unwrap().iter().any(|name| name == "flaky"),
        "{basis}"
    );
    // Taking back again finds nothing live, and so does a stream with no
    // reading.
    run(&mut game, &[("unflag", "{}")]);
    assert_eq!(snapshot(&game)["taken_back"].as_array().unwrap().len(), 1);
    let mut fresh = session(LEDGER);
    run(&mut fresh, &[("unflag", "{}")]);
    assert!(snapshot(&fresh)["taken_back"]
        .as_array()
        .is_none_or(|records| records.is_empty()));
}

#[test]
fn a_scheduled_qualification_stays_on_the_reading_current_when_it_was_made() {
    let mut game = session(LEDGER);
    run(
        &mut game,
        &[
            ("report", r#"{"v":1}"#),
            ("flag_later", "{}"),
            ("report", r#"{"v":1}"#),
        ],
    );
    for _ in 0..3 {
        run(&mut game, &[("tick", r#"{"dt":0.1}"#)]);
    }
    run(&mut game, &[("recount", "{}")]);
    assert!(carries(&game, "first", "flaky"));
    assert!(!carries(&game, "last", "flaky"));
}

#[test]
fn a_save_restores_with_qualifications_and_take_backs_on_readings() {
    let mut game = session(LEDGER);
    run(
        &mut game,
        &[
            ("report", r#"{"v":1}"#),
            ("flag", "{}"),
            ("report", r#"{"v":0}"#),
            ("flag", "{}"),
            ("unflag", "{}"),
            ("flag_later", "{}"),
        ],
    );
    let save = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(LEDGER, &save).unwrap();
    for game in [&mut game, &mut restored] {
        run(
            game,
            &[
                ("tick", r#"{"dt":0.1}"#),
                ("tick", r#"{"dt":0.1}"#),
                ("tick", r#"{"dt":0.1}"#),
                ("unflag", "{}"),
                ("recount", "{}"),
            ],
        );
    }
    assert_eq!(game.save_json().unwrap(), restored.save_json().unwrap());
    assert!(carries(&restored, "first", "flaky"));
    assert!(!carries(&restored, "last", "flaky"));
}

/// Evidence's current occurrence: `unqualify latest(E)` marks only it, where
/// `unqualify E` marks every held occurrence.
#[test]
fn unqualify_latest_of_renewable_evidence_marks_only_the_current_occurrence() {
    let source = r#"
claim passing;
caveat flaky consequence material;
evidence tests from "test runner";
renewable tests limit 8;
state last = 0;
event report;
event flag;
event unflag;
event unflag_all;
on report when observed(tests) renew tests;
on report reveal tests supports passing;
on report set last = qualified(1, tests);
on flag qualify latest(tests) with flaky;
on unflag unqualify latest(tests) with flaky;
on unflag_all unqualify tests with flaky;
"#;
    let mut game = session(source);
    run(
        &mut game,
        &[
            ("report", "{}"),
            ("flag", "{}"),
            ("report", "{}"),
            ("flag", "{}"),
            ("unflag", "{}"),
        ],
    );
    let shown = snapshot(&game);
    let taken: Vec<&str> = shown["taken_back"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["evidence"].as_str().unwrap())
        .collect();
    assert_eq!(taken, ["tests@2"]);
    run(&mut game, &[("unflag_all", "{}")]);
    let shown = snapshot(&game);
    let taken: Vec<&str> = shown["taken_back"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["evidence"].as_str().unwrap())
        .collect();
    assert_eq!(taken, ["tests@2", "tests"]);
    let save = game.save_json().unwrap();
    ReactiveSession::restore_json(source, &save).unwrap();
}

#[test]
fn latest_names_a_reading_stream_or_evidence() {
    let with = |line: &str| format!("{LEDGER}{line}\n");
    assert!(
        load_error(&with("on flag qualify latest(nothing) with flaky;"))
            .contains("must name a reading stream or declared evidence")
    );
    assert!(load_error(&with("on flag unqualify latest(ship) with flaky;")).contains("ship"));
    // A stream is named only through latest(…).
    assert!(!load_error(&with("on flag qualify checks with flaky;")).is_empty());
    assert!(!load_error(&with("on flag unqualify checks with flaky;")).is_empty());
    assert!(!load_error(&with("on flag qualify latest(checks) flaky;")).is_empty());
    assert!(!load_error(&with("on flag qualify latest(checks with flaky;")).is_empty());
}

#[test]
fn a_caveat_taken_back_from_readings_is_never_put_on_the_template() {
    let error = load_error(&format!(
        "{LEDGER}event doubt_runner;\non doubt_runner qualify runner with flaky;\n"
    ));
    assert!(
        error.contains("unqualify latest(checks) with flaky"),
        "{error}"
    );
    // Withdrawal's caveat is never taken back.
    let error = load_error(&format!(
        "{LEDGER}evidence recheck from \"a re-read\";\nevent misread;\non misread withdraw latest(checks) because recheck;\non misread unqualify latest(checks) with withdrawn;\n"
    ));
    assert!(error.contains("nothing undoes a withdrawal"), "{error}");
}

/// With a window, a flagged reading retires as later ones arrive; the session
/// keeps going and a save restores at every step.
#[test]
fn flagged_readings_retire_with_their_window() {
    let source = LEDGER.replace(
        "readings checks from runner limit 8;",
        "readings checks from runner window 2;",
    );
    let mut game = session(&source);
    for step in 0..8 {
        let v = if step % 2 == 0 { "1" } else { "0" };
        run(
            &mut game,
            &[("report", &format!(r#"{{"v":{v}}}"#)), ("flag", "{}")],
        );
        if step % 3 == 0 {
            run(&mut game, &[("unflag", "{}")]);
        }
        let save = game.save_json().unwrap();
        let restored = ReactiveSession::restore_json(&source, &save)
            .unwrap_or_else(|error| panic!("step {step}: {error}"));
        assert_eq!(restored.save_json().unwrap(), save, "step {step}");
    }
    assert!(carries(&game, "last", "flaky"));
}
