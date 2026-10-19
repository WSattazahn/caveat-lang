//! `in_basis(DECISION, EVIDENCE)`: whether a decision's grounds, current or
//! of a past revision, hold the evidence named. See
//! spec/caveat-in-basis-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const LEDGER: &str = r#"
claim passing;
evidence runner from "the test runner";
evidence manual from "a manual check";
readings checks from runner limit 8;
decisions ship limit 8;
state latest_counts = 0;
state reading_counts = 0;
state revision_counts = 0;
state manual_counts = 0;
event report v min 0 max 1;
event see_manual;
event decide;
event approve;
event revisit i min 1 max 8;
event ask k min 1 max 8, i min 1 max 8;
on report sample checks = v supports passing;
on see_manual reveal manual;
on decide commit ship because enough using latest(checks);
on approve commit go because enough using qualified(1, manual);
on revisit when committed(ship) reopen ship because reading(checks, i);
on ask set latest_counts = if(in_basis(ship, latest(checks)), 1, 0);
on ask set reading_counts = if(in_basis(ship, reading(checks, i)), 1, 0);
on ask set revision_counts = if(in_basis(revision(ship, k), occurrence(checks, i)), 1, 0);
on ask set manual_counts = if(in_basis(go, manual), 1, 0);
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

fn value(game: &ReactiveSession, state: &str) -> f64 {
    let snapshot: Value = serde_json::to_value(game.snapshot()).unwrap();
    snapshot["qualified_values"][state]["value"]
        .as_f64()
        .unwrap()
}

/// `ask {k, i}`, then the four answers in declaration order.
fn ask(game: &mut ReactiveSession, k: u32, i: u32) -> [f64; 4] {
    run(game, &[("ask", &format!(r#"{{"k":{k},"i":{i}}}"#))]);
    [
        "latest_counts",
        "reading_counts",
        "revision_counts",
        "manual_counts",
    ]
    .map(|state| value(game, state))
}

fn restores(source: &str, game: &ReactiveSession) {
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &save)
        .unwrap_or_else(|error| panic!("restore: {error}"));
    assert_eq!(restored.save_json().unwrap(), save);
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_default()
}

#[test]
fn a_later_reading_is_not_in_the_basis_of_the_decision_in_force() {
    let mut game = session(LEDGER);
    assert_eq!(ask(&mut game, 1, 1), [0.0; 4]);
    run(&mut game, &[("report", r#"{"v":1}"#), ("decide", "{}")]);
    assert_eq!(ask(&mut game, 1, 1), [1.0, 1.0, 1.0, 0.0]);
    run(&mut game, &[("report", r#"{"v":0}"#)]);
    // The current reading is checks@2, which ship@1 did not rest on.
    assert_eq!(ask(&mut game, 1, 1), [0.0, 1.0, 1.0, 0.0]);
    assert_eq!(ask(&mut game, 1, 2), [0.0, 0.0, 0.0, 0.0]);
    restores(LEDGER, &game);
}

#[test]
fn a_past_revision_keeps_its_own_basis() {
    let mut game = session(LEDGER);
    run(
        &mut game,
        &[
            ("report", r#"{"v":1}"#),
            ("decide", "{}"),
            ("report", r#"{"v":1}"#),
            ("revisit", r#"{"i":2}"#),
            ("decide", "{}"),
        ],
    );
    // ship@2 is in force and rests on checks@2; ship@1 rested on checks@1.
    assert_eq!(ask(&mut game, 1, 1), [1.0, 0.0, 1.0, 0.0]);
    assert_eq!(ask(&mut game, 2, 2), [1.0, 1.0, 1.0, 0.0]);
    assert_eq!(ask(&mut game, 1, 2), [1.0, 1.0, 0.0, 0.0]);
    // A revision not made yet answers false.
    assert_eq!(ask(&mut game, 3, 2)[2], 0.0);
    restores(LEDGER, &game);
}

#[test]
fn a_reopened_revision_keeps_its_basis() {
    let mut game = session(LEDGER);
    run(
        &mut game,
        &[
            ("report", r#"{"v":1}"#),
            ("decide", "{}"),
            ("revisit", r#"{"i":1}"#),
        ],
    );
    // As `rests_on_withdrawn(ship)` reads it: the current revision, reopened
    // or not. `committed(ship) and in_basis(…)` asks about one in force.
    assert_eq!(ask(&mut game, 1, 1), [1.0, 1.0, 1.0, 0.0]);
    restores(LEDGER, &game);
}

#[test]
fn a_commitment_and_declared_evidence_can_be_asked_about() {
    let mut game = session(LEDGER);
    run(&mut game, &[("see_manual", "{}")]);
    assert_eq!(ask(&mut game, 1, 1)[3], 0.0);
    run(&mut game, &[("approve", "{}")]);
    assert_eq!(ask(&mut game, 1, 1)[3], 1.0);
    restores(LEDGER, &game);
}

#[test]
fn in_basis_names_a_decision_and_evidence() {
    let with =
        |line: &str| format!("{LEDGER}state x = 0;\nevent x;\non x set x = if({line}, 1, 0);\n");
    for (form, expected) in [
        ("in_basis(nothing, manual)", "names no decision"),
        (
            "in_basis(revision(go, 1), manual)",
            "names no decision series",
        ),
        ("in_basis(ship, latest(nothing))", "latest(nothing)"),
        ("in_basis(ship, reading(runner, 1))", "renewable"),
        ("in_basis(ship, passing)", "evidence"),
    ] {
        let error = load_error(&with(form));
        assert!(error.contains(expected), "{form}: {error}");
    }
}

#[test]
fn a_source_function_named_in_basis_keeps_its_meaning() {
    let source = r#"
fn in_basis(a, b) = a + b;
state a = 2;
state b = 3;
state named = 0;
state literal = 0;
event go;
on go set named = in_basis(a, b);
on go set literal = in_basis(1, 2);
"#;
    let mut game = session(source);
    run(&mut game, &[("go", "{}")]);
    assert_eq!(value(&game, "named"), 5.0);
    assert_eq!(value(&game, "literal"), 3.0);
}
