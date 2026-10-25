//! `carries_any(EVIDENCE [, CONSEQUENCE])`: whether evidence carries any
//! caveat, or any of consequence at least a level. See
//! spec/caveat-carries-any-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// A report stream whose readings may be doubted three ways, and a decision
/// that may not rest on a doubted report. `asked` writes each form of the
/// question beside the hand-written list it replaces.
const CHECKS: &str = r#"
claim passing;
caveat flaky consequence material;
caveat wrong_config consequence high;
caveat style_unchecked consequence low;
evidence runner from "the test runner";
evidence manual from "a manual check";
evidence recheck from "a finding that a report was wrong";
renewable manual limit 4;
readings checks from runner limit 8;
decisions ship limit 4;
state any_latest = 0;
state listed_latest = 0;
state material_latest = 0;
state first = 0;
state first_material = 0;
state manual_any = 0;
state manual_first = 0;
event report v min 0 max 1;
event flag;
event misconfigure;
event style;
event unflag;
event retract;
event check_manually;
event recheck_manually;
event doubt_manual;
event ask;
event decide;
on report sample checks = v supports passing;
on flag qualify latest(checks) with flaky;
on misconfigure qualify latest(checks) with wrong_config;
on style qualify latest(checks) with style_unchecked;
on unflag unqualify latest(checks) with flaky;
on retract reveal recheck;
on retract withdraw latest(checks) because recheck;
on check_manually reveal manual supports passing;
on recheck_manually renew manual;
on doubt_manual qualify manual with flaky;
on ask set any_latest = if(carries_any(latest(checks)), 1, 0);
on ask set listed_latest = if(carries(latest(checks), withdrawn) or carries(latest(checks), flaky)
    or carries(latest(checks), wrong_config) or carries(latest(checks), style_unchecked), 1, 0);
on ask set material_latest = if(carries_any(latest(checks), material), 1, 0);
on ask set first = if(carries_any(reading(checks, 1)), 1, 0);
on ask set first_material = if(carries_any(reading(checks, 1), material), 1, 0);
on ask set manual_any = if(carries_any(manual), 1, 0);
on ask set manual_first = if(carries_any(occurrence(manual, 1)), 1, 0);
on decide when carries_any(latest(checks), material) reject "the latest report is in doubt";
on decide commit ship because enough using latest(checks);
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

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn value(game: &ReactiveSession, state: &str) -> f64 {
    snapshot(game)["qualified_values"][state]["value"]
        .as_f64()
        .unwrap()
}

const ASKED: [&str; 7] = [
    "any_latest",
    "listed_latest",
    "material_latest",
    "first",
    "first_material",
    "manual_any",
    "manual_first",
];

fn ask(game: &mut ReactiveSession) -> [f64; 7] {
    run(game, &[("ask", "{}")]);
    ASKED.map(|state| value(game, state))
}

fn load_error(source: &str) -> String {
    ReactiveSession::from_source(source)
        .err()
        .unwrap_or_default()
}

#[test]
fn carries_any_is_the_list_of_every_caveat_written_out() {
    let mut game = session(CHECKS);
    // No reading and no observation: false, and nothing is refused.
    assert_eq!(ask(&mut game), [0.0; 7]);
    run(&mut game, &[("report", r#"{"v":1}"#)]);
    assert_eq!(ask(&mut game)[..3], [0.0, 0.0, 0.0]);
    // A low caveat is a caveat, but not one of consequence material.
    run(&mut game, &[("style", "{}")]);
    assert_eq!(ask(&mut game)[..5], [1.0, 1.0, 0.0, 1.0, 0.0]);
    run(&mut game, &[("flag", "{}")]);
    assert_eq!(ask(&mut game)[..5], [1.0, 1.0, 1.0, 1.0, 1.0]);
    // A new reading carries nothing the last one was qualified with.
    run(&mut game, &[("report", r#"{"v":1}"#)]);
    assert_eq!(ask(&mut game)[..5], [0.0, 0.0, 0.0, 1.0, 1.0]);
    run(&mut game, &[("misconfigure", "{}")]);
    assert_eq!(ask(&mut game)[..3], [1.0, 1.0, 1.0]);
}

#[test]
fn a_take_back_and_a_withdrawal_change_the_answer() {
    let mut game = session(CHECKS);
    run(&mut game, &[("report", r#"{"v":1}"#), ("flag", "{}")]);
    assert_eq!(ask(&mut game)[..3], [1.0, 1.0, 1.0]);
    run(&mut game, &[("unflag", "{}")]);
    assert_eq!(ask(&mut game)[..3], [0.0, 0.0, 0.0]);
    // `withdrawn` is a caveat of consequence material.
    run(&mut game, &[("retract", "{}")]);
    assert_eq!(ask(&mut game)[..3], [1.0, 1.0, 1.0]);
}

#[test]
fn carries_any_of_evidence_asks_about_its_current_occurrence() {
    let mut game = session(CHECKS);
    run(
        &mut game,
        &[("check_manually", "{}"), ("doubt_manual", "{}")],
    );
    assert_eq!(ask(&mut game)[5..], [1.0, 1.0]);
    // A renewal is a new observation: the current occurrence is clean, the
    // first still carries what qualified it.
    run(&mut game, &[("recheck_manually", "{}")]);
    assert_eq!(ask(&mut game)[5..], [0.0, 1.0]);
}

#[test]
fn a_guard_on_carries_any_refuses_a_decision_on_a_doubted_report() {
    let mut game = session(CHECKS);
    run(&mut game, &[("report", r#"{"v":1}"#), ("style", "{}")]);
    // The low caveat does not reach the guard's level.
    run(&mut game, &[("decide", "{}")]);
    run(
        &mut game,
        &[("report", r#"{"v":1}"#), ("misconfigure", "{}")],
    );
    let error = dispatch(&mut game, "decide", "{}").unwrap_err();
    assert!(error.contains("the latest report is in doubt"), "{error}");
    let save = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(CHECKS, &save)
        .unwrap_or_else(|error| panic!("restore: {error}"));
    assert_eq!(restored.save_json().unwrap(), save);
}

#[test]
fn its_lineage_is_the_lineage_of_the_list_it_replaces() {
    // The same question, as carries_any and as the list, gives the same value
    // with the same evidence and caveats, at every step.
    let mut game = session(CHECKS);
    for step in [
        ("ask", "{}"),
        ("report", r#"{"v":1}"#),
        ("style", "{}"),
        ("flag", "{}"),
        ("report", r#"{"v":0}"#),
        ("misconfigure", "{}"),
        ("unflag", "{}"),
        ("retract", "{}"),
    ] {
        run(&mut game, &[step, ("ask", "{}")]);
        let values = snapshot(&game)["qualified_values"].clone();
        let mut any = values["any_latest"].clone();
        let mut listed = values["listed_latest"].clone();
        for value in [&mut any, &mut listed] {
            if let Some(object) = value.as_object_mut() {
                object.remove("name");
            }
        }
        assert_eq!(any, listed, "after {step:?}");
    }
}

#[test]
fn a_program_without_caveats_loads_and_answers_false() {
    let mut game = session(
        r#"
claim ready;
evidence review from "a review";
state doubted = 1;
event go;
on go reveal review supports ready;
on go set doubted = if(carries_any(review), 1, 0);
"#,
    );
    run(&mut game, &[("go", "{}")]);
    assert_eq!(value(&game, "doubted"), 0.0);
}

#[test]
fn a_procedure_passes_the_evidence_it_asks_about() {
    let source = r#"
claim ready;
caveat stale consequence material;
evidence east from "the east sensor";
evidence west from "the west sensor";
event look;
event doubt;
event use_east;
event use_west;
on look reveal east supports ready;
on look reveal west supports ready;
on doubt qualify west with stale;
proc use_sensor(e evidence) {
    when carries_any(e) reject "That sensor is in doubt.";
    reveal e supports ready;
};
on use_east call use_sensor(east);
on use_west call use_sensor(west);
"#;
    let mut game = session(source);
    run(
        &mut game,
        &[("look", "{}"), ("doubt", "{}"), ("use_east", "{}")],
    );
    let error = dispatch(&mut game, "use_west", "{}").unwrap_err();
    assert!(error.contains("That sensor is in doubt."), "{error}");
}

#[test]
fn a_program_that_defines_carries_any_keeps_its_function() {
    // A 1.3 program may define `fn carries_any`; its calls keep their meaning.
    let mut game = session(
        r#"
state x = 2;
state y = 0;
fn carries_any(v) = v * 10;
event go;
on go set y = carries_any(x);
"#,
    );
    run(&mut game, &[("go", "{}")]);
    assert_eq!(value(&game, "y"), 20.0);
}

#[test]
fn carries_any_names_evidence_and_a_consequence() {
    let with = |line: &str| format!("{CHECKS}event x;\non x when {line} reject \"no\";\n");
    for (form, expected) in [
        ("carries_any(passing)", "passing"),
        ("carries_any(latest(nothing))", "latest(nothing)"),
        ("carries_any(latest(checks), flaky)", "a consequence"),
        (
            "carries_any(latest(checks), material, high)",
            "carries_any expects",
        ),
        ("carries_any()", "carries_any expects"),
        ("carries_any(1)", "carries_any expects"),
        ("carries_any(any_latest + 1)", "carries_any expects"),
        (
            "carries_any(reading(checks))",
            "takes evidence and an index",
        ),
    ] {
        let error = load_error(&with(form));
        assert!(error.contains(expected), "{form}: {error}");
    }
}

#[test]
fn a_source_function_cannot_ask_it() {
    let error = load_error(&format!(
        "{CHECKS}fn doubted(v) = if(carries_any(runner), v, 0);\nevent y;\non y set first = doubted(1);\n"
    ));
    assert!(error.contains("cannot query graph predicates"), "{error}");
}

#[test]
fn a_binding_follows_the_caveats_and_an_examined_caveat_still_counts() {
    let mut game = session(
        r#"
budget 10;
claim ready;
caveat stale consequence material;
caveat partial consequence material;
evidence review from "a review";
event look;
event doubt;
event inspect;
event narrow;
event undoubt;
on look reveal review supports ready;
on doubt qualify review with stale;
on narrow qualify review with partial;
on inspect examine partial cost 1;
on undoubt unqualify review with stale;
bind hud.review = "clean";
bind hud.review = "in doubt" when carries_any(review, material);
"#,
    );
    let shown = |game: &ReactiveSession| snapshot(game)["bindings"]["hud"]["review"].clone();
    run(&mut game, &[("look", "{}")]);
    assert_eq!(shown(&game), "clean");
    run(&mut game, &[("doubt", "{}")]);
    assert_eq!(shown(&game), "in doubt");
    run(&mut game, &[("undoubt", "{}")]);
    assert_eq!(shown(&game), "clean");
    run(&mut game, &[("narrow", "{}"), ("inspect", "{}")]);
    assert_eq!(shown(&game), "in doubt");
}
