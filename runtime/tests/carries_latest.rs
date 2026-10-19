//! `carries(latest(NAME), CAVEAT)`: whether the current reading or
//! occurrence carries a caveat. See spec/caveat-latest-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const CHECKS: &str = r#"
claim passing;
caveat flaky consequence material;
evidence runner from "the test runner";
evidence manual from "a manual check";
readings checks from runner limit 8;
state flagged = 0;
state manual_flagged = 0;
state flags = 0;
event report v min 0 max 1;
event flag;
event unflag;
event flag_manual;
event ask;
on report sample checks = v supports passing;
on flag when carries(latest(checks), flaky) reject "already flagged";
on flag qualify latest(checks) with flaky;
on flag set flags = flags + 1;
on unflag unqualify latest(checks) with flaky;
on flag_manual reveal manual;
on flag_manual qualify manual with flaky;
on ask set flagged = if(carries(latest(checks), flaky), 1, 0);
on ask set manual_flagged = if(carries(latest(manual), flaky), 1, 0);
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("load: {error}"))
}

fn dispatch(game: &mut ReactiveSession, event: &str, payload: &str) -> Result<(), String> {
    game.dispatch_json(event, payload).map(|_| ())
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        dispatch(game, event, payload).unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn value(game: &ReactiveSession, state: &str) -> f64 {
    let snapshot: Value = serde_json::to_value(game.snapshot()).unwrap();
    snapshot["qualified_values"][state]["value"]
        .as_f64()
        .unwrap()
}

fn ask(game: &mut ReactiveSession) -> [f64; 2] {
    run(game, &[("ask", "{}")]);
    ["flagged", "manual_flagged"].map(|state| value(game, state))
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_default()
}

#[test]
fn carries_latest_asks_about_the_current_reading_only() {
    let mut game = session(CHECKS);
    // No reading yet: false, and nothing is refused.
    assert_eq!(ask(&mut game), [0.0, 0.0]);
    run(&mut game, &[("report", r#"{"v":1}"#), ("flag", "{}")]);
    assert_eq!(ask(&mut game), [1.0, 0.0]);
    // The guard reads the flag on the current reading.
    assert!(dispatch(&mut game, "flag", "{}").is_err());
    assert_eq!(value(&game, "flags"), 1.0);
    // checks@2 copies nothing from checks@1's qualification.
    run(&mut game, &[("report", r#"{"v":1}"#)]);
    assert_eq!(ask(&mut game), [0.0, 0.0]);
    run(&mut game, &[("flag", "{}"), ("unflag", "{}")]);
    assert_eq!(ask(&mut game), [0.0, 0.0]);
    assert_eq!(value(&game, "flags"), 2.0);
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(CHECKS, &save)
        .unwrap_or_else(|error| panic!("restore: {error}"));
    assert_eq!(restored.save_json().unwrap(), save);
}

#[test]
fn carries_latest_of_evidence_asks_about_its_current_occurrence() {
    let mut game = session(CHECKS);
    run(&mut game, &[("flag_manual", "{}")]);
    assert_eq!(ask(&mut game), [0.0, 1.0]);
}

#[test]
fn carries_latest_names_a_stream_or_evidence_and_a_caveat() {
    let with = |line: &str| format!("{CHECKS}event x;\non x when {line} reject \"no\";\n");
    for (form, expected) in [
        ("carries(latest(nothing), flaky)", "latest(nothing)"),
        ("carries(latest(passing), flaky)", "latest(passing)"),
        (
            "carries(latest(checks), passing)",
            "passing must name a declared caveat",
        ),
        (
            "has_caveat(latest(checks), flaky)",
            "carries(latest(NAME), CAVEAT)",
        ),
    ] {
        let error = load_error(&with(form));
        assert!(error.contains(expected), "{form}: {error}");
    }
}
