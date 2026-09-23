//! The reactive clock is readable as a number without a parallel state timer.
use caveat_runtime::reactive::{BindingValue, Provenance, ReactiveSession};
use std::collections::BTreeSet;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn number(game: &ReactiveSession, property: &str) -> f64 {
    match game.snapshot().bindings["hud"][property] {
        BindingValue::Number(value) => value,
        ref value => panic!("hud.{property} should be numeric, got {value:?}"),
    }
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn trace(actual: &Provenance, evidence: &[&str], caveats: &[&str]) {
    assert_eq!(actual.evidence, names(evidence));
    assert_eq!(actual.caveats, names(caveats));
}

#[test]
fn starts_at_zero_and_only_the_selected_clock_event_advances_it() {
    let mut game = session(
        r#"
        state captured = elapsed();
        event advance dt min 0 max 2;
        clock advance every 0.25;
        event tick dt min 0 max 0.1;
        event other dt min 0 max 2;
        on advance set captured = elapsed();
        bind hud.now = elapsed();
        "#,
    );
    assert_eq!(number(&game, "now"), 0.0);
    assert_eq!(game.snapshot().values["captured"], 0.0);
    for (event, payload) in [("tick", r#"{"dt":0.0625}"#), ("other", r#"{"dt":2}"#)] {
        game.dispatch_json(event, payload).unwrap();
        assert_eq!(number(&game, "now"), 0.0);
    }
    let shown = game.dispatch_json("advance", r#"{"dt":0.75}"#).unwrap();
    assert_eq!(shown.elapsed, 0.75);
    assert_eq!(shown.values["captured"], shown.elapsed);
    assert_eq!(number(&game, "now"), shown.elapsed);
}

#[test]
fn reserved_tick_counts_fractional_time_without_a_clock_declaration() {
    let mut game = session(
        r#"
        event tick dt min 0 max 0.1;
        event idle;
        bind hud.now = elapsed();
        bind hud.remaining = 1.5 - elapsed();
        bind hud.ready = elapsed() >= 0.09375;
        "#,
    );
    for (dt, expected) in [(0.0625, 0.0625), (0.03125, 0.09375), (0.0078125, 0.1015625)] {
        let shown = game
            .dispatch_json("tick", &format!(r#"{{"dt":{dt}}}"#))
            .unwrap();
        assert_eq!(shown.elapsed, expected);
        assert_eq!(number(&game, "now"), expected);
        assert_eq!(number(&game, "remaining"), 1.5 - expected);
        assert_eq!(
            shown.bindings["hud"]["ready"],
            BindingValue::Bool(expected >= 0.09375)
        );
    }
    game.dispatch_json("idle", "{}").unwrap();
    assert_eq!(number(&game, "now"), 0.1015625);
}

#[test]
fn without_a_time_event_the_read_stays_zero() {
    let mut game = session("event advance dt min 0 max 3; bind hud.now = elapsed();");
    for _ in 0..3 {
        let shown = game.dispatch_json("advance", r#"{"dt":3}"#).unwrap();
        assert_eq!(shown.elapsed, 0.0);
        assert_eq!(number(&game, "now"), 0.0);
    }
}

#[test]
fn defines_guards_procedures_and_pure_function_arguments_read_current_time() {
    let mut game = session(
        r#"
        state direct = 0;
        state argument = 0;
        event advance dt min 0 max 1;
        clock advance every 0.25;
        fn doubled(value) = value * 2;
        define ready = elapsed() >= 0.5;
        define scale = doubled(elapsed());
        proc capture(value) {
            when ready set direct = elapsed();
            when elapsed() >= 0.5 set argument = value;
        };
        on advance when ready call capture(scale);
        bind hud.now = scale;
        "#,
    );
    game.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap();
    assert_eq!(game.snapshot().values["direct"], 0.0);
    assert_eq!(number(&game, "now"), 0.5);
    let shown = game.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap();
    assert_eq!(shown.values["direct"], 0.5);
    assert_eq!(shown.values["argument"], 1.0);
    assert_eq!(number(&game, "now"), 1.0);
}

#[test]
fn clock_only_changes_invalidate_numeric_and_conditional_bindings() {
    let mut game = session(
        r#"
        event tick dt min 0 max 0.1;
        define passed = elapsed() >= 0.125;
        bind hud.now = elapsed();
        bind hud.text = "waiting";
        bind hud.text = "ready" when passed;
        "#,
    );
    assert!(game.snapshot().values.is_empty());
    game.dispatch_json("tick", r#"{"dt":0.0625}"#).unwrap();
    assert_eq!(number(&game, "now"), 0.0625);
    assert_eq!(
        game.snapshot().bindings["hud"]["text"],
        BindingValue::Text("waiting".into())
    );
    game.dispatch_json("tick", r#"{"dt":0.0625}"#).unwrap();
    assert_eq!(number(&game, "now"), 0.125);
    assert_eq!(
        game.snapshot().bindings["hud"]["text"],
        BindingValue::Text("ready".into())
    );
}

const TIMED: &str = r#"
    claim safe;
    evidence sight from "the observation";
    caveat stale consequence material;
    state at_boundary = 0;
    event observe;
    event advance dt min 0 max 1;
    clock advance every 0.125;
    on observe reveal sight supports safe;
    on observe qualify sight with stale after 0.375;
    on advance when elapsed() >= 0.375 and carries(sight, stale) set at_boundary = elapsed();
    on advance when elapsed() >= 0.375 and not committed(route)
        commit route because enough using elapsed();
    bind hud.now = elapsed();
    bind hud.stale = carries(sight, stale);
"#;

#[test]
fn timed_qualification_rules_journal_and_restore_share_one_clock() {
    let mut game = session(TIMED);
    game.dispatch_json("observe", "{}").unwrap();
    game.dispatch_json("advance", r#"{"dt":0.125}"#).unwrap();
    assert_eq!(game.snapshot().values["at_boundary"], 0.0);
    assert_eq!(
        game.snapshot().bindings["hud"]["stale"],
        BindingValue::Bool(false)
    );
    let save = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(TIMED, &save).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    assert_eq!(number(&restored, "now"), 0.125);
    let shown = game.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap();
    assert_eq!(
        restored.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap(),
        shown
    );
    assert_eq!(shown.values["at_boundary"], 0.375);
    assert!(shown.scheduled_qualifications.is_empty());
    assert_eq!(shown.bindings["hud"]["stale"], BindingValue::Bool(true));
    assert_eq!(shown.decision_journal[0].elapsed, Some(0.375));
    assert_eq!(shown.decision_journal[0].value, Some(0.375));
    let roundtrip = ReactiveSession::restore_json(TIMED, &restored.save_json().unwrap()).unwrap();
    assert_eq!(roundtrip.snapshot(), shown);
}

#[test]
fn elapsed_adds_no_provenance_but_surrounding_values_and_control_keep_theirs() {
    let mut game = session(
        r#"
        claim safe;
        evidence sight from "sighting";
        caveat uncertain consequence low;
        uncertain qualifies sight;
        sight supports safe;
        state plain = 0;
        state guarded = 0;
        state mixed = 0;
        event tick dt min 0 max 0.1;
        on tick set plain = elapsed();
        on tick when qualified(1, sight) > 0 and elapsed() > 0 set guarded = elapsed();
        on tick set mixed = qualified(2, sight) + elapsed();
        bind hud.now = elapsed();
        bind hud.reason = "time passed" when elapsed() > 0 and qualified(1, sight) > 0;
        "#,
    );
    let shown = game.dispatch_json("tick", r#"{"dt":0.0625}"#).unwrap();
    trace(&shown.qualified_values["plain"].provenance, &[], &[]);
    trace(&shown.value_grounds["plain"], &[], &[]);
    trace(&shown.binding_qualifications["hud"]["now"], &[], &[]);
    trace(&shown.binding_explanations["hud"]["now"], &[], &[]);
    trace(
        &shown.qualified_values["guarded"].provenance,
        &["sight"],
        &["uncertain"],
    );
    trace(&shown.value_grounds["guarded"], &[], &[]);
    trace(
        &shown.qualified_values["mixed"].provenance,
        &["sight"],
        &["uncertain"],
    );
    trace(&shown.value_grounds["mixed"], &["sight"], &["uncertain"]);
    trace(
        &shown.binding_qualifications["hud"]["reason"],
        &["sight"],
        &["uncertain"],
    );
}

#[test]
fn rejected_payloads_and_event_effects_do_not_advance_time() {
    let source = format!("{TIMED}\non advance when elapsed() >= 0.5 set at_boundary = 1 / 0;");
    let mut game = session(&source);
    game.dispatch_json("observe", "{}").unwrap();
    game.dispatch_json("advance", r#"{"dt":0.125}"#).unwrap();
    let before = game.snapshot();
    let saved = game.save_json().unwrap();
    for payload in ["{}", r#"{"dt":2}"#, r#"{"dt":"bad"}"#, r#"{"dt":0.5}"#] {
        assert!(game.dispatch_json("advance", payload).is_err(), "{payload}");
        assert_eq!(game.snapshot(), before, "{payload}");
        assert_eq!(game.save_json().unwrap(), saved, "{payload}");
    }
    // The failed transaction crossed the qualification and commitment boundary.
    // A later accepted event must still cross it exactly once.
    let shown = game.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap();
    assert_eq!(shown.elapsed, 0.375);
    assert_eq!(shown.decision_journal.len(), 1);
    assert!(shown.scheduled_qualifications.is_empty());
}

#[test]
fn a_failed_binding_rolls_back_the_clock_and_keeps_the_previous_view() {
    let mut game = session(
        "event tick dt min 0 max 0.1; bind hud.now = elapsed(); bind hud.ratio = 1 / (0.125 - elapsed());",
    );
    game.dispatch_json("tick", r#"{"dt":0.0625}"#).unwrap();
    let before = game.snapshot();
    assert!(game.dispatch_json("tick", r#"{"dt":0.0625}"#).is_err());
    assert_eq!(game.snapshot(), before);
    game.dispatch_json("tick", r#"{"dt":0.03125}"#).unwrap();
    assert_eq!(number(&game, "now"), 0.09375);
}

#[test]
fn a_state_named_elapsed_does_not_shadow_or_write_the_clock() {
    let mut game = session(
        r#"
        state elapsed = 7;
        event tick dt min 0 max 0.1;
        event reset;
        on tick set elapsed = elapsed + 1;
        on reset set elapsed = 0;
        bind hud.now = elapsed();
        bind hud.state = elapsed;
        "#,
    );
    game.dispatch_json("tick", r#"{"dt":0.0625}"#).unwrap();
    assert_eq!(number(&game, "now"), 0.0625);
    assert_eq!(number(&game, "state"), 8.0);
    game.dispatch_json("reset", "{}").unwrap();
    assert_eq!(number(&game, "now"), 0.0625);
    assert_eq!(number(&game, "state"), 0.0);
}

#[test]
fn event_and_procedure_parameters_named_elapsed_coexist_with_the_clock_read() {
    let mut game = session(
        r#"
        state event_value = 0;
        state procedure_value = 0;
        event advance dt min 0 max 1;
        clock advance every 0.25;
        event measure elapsed min 0 max 10;
        proc capture(elapsed) { set procedure_value = elapsed + elapsed(); };
        on measure set event_value = elapsed + elapsed();
        on measure call capture(elapsed * 2);
        bind hud.now = elapsed();
        "#,
    );
    game.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap();
    let shown = game.dispatch_json("measure", r#"{"elapsed":3}"#).unwrap();
    assert_eq!(shown.values["event_value"], 3.25);
    assert_eq!(shown.values["procedure_value"], 6.25);
    assert_eq!(number(&game, "now"), 0.25);
}

#[test]
fn unused_pure_functions_cannot_capture_ambient_elapsed_time() {
    for definition in [
        "fn ambient() = elapsed();",
        "fn ambient(value) = value + elapsed();",
    ] {
        let source = format!("event idle; {definition}");
        assert!(
            ReactiveSession::from_source(&source).is_err(),
            "accepted unused ambient clock capture: {definition}"
        );
        assert!(caveat_runtime::source_library::SourceLibrary::from_source(&source).is_err());
    }
}

#[test]
fn signed_zero_clock_changes_invalidate_bindings() {
    let source = r#"
        event advance dt min 0 max 1;
        clock advance every 0.25;
        bind hud.angle = atan2(elapsed(), -1);
    "#;
    // Manufacture a valid signed-zero runtime clock, whose sign remains
    // observable through atan2 even though -0.0 == 0.0 compares true.
    let mut manufactured = session(source).save().unwrap();
    manufactured.elapsed = -0.0;
    let mut restored = ReactiveSession::restore(source, &manufactured).unwrap();
    assert_eq!(restored.snapshot().elapsed.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(number(&restored, "angle"), -std::f64::consts::PI);
    restored.dispatch_json("advance", r#"{"dt":0}"#).unwrap();
    assert_eq!(restored.snapshot().elapsed.to_bits(), 0.0_f64.to_bits());
    assert_eq!(number(&restored, "angle"), std::f64::consts::PI);
}

#[test]
fn elapsed_preserves_an_explicit_clocks_permitted_negative_dt() {
    let source =
        "event advance dt min -1 max 1; clock advance every 0.25; bind hud.now = elapsed();";
    let mut game = session(source);
    game.dispatch_json("advance", r#"{"dt":-0.25}"#).unwrap();
    assert_eq!(number(&game, "now"), -0.25);
    let mut restored = ReactiveSession::restore_json(source, &game.save_json().unwrap()).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    restored.dispatch_json("advance", r#"{"dt":0.5}"#).unwrap();
    assert_eq!(number(&restored, "now"), 0.25);
}

#[test]
fn a_manufactured_valid_save_exposes_runtime_time_beyond_the_state_number_cap() {
    let source = r#"
        state copied = 0;
        event advance dt min 0 max 1;
        clock advance every 0.25;
        event copy;
        on copy set copied = elapsed();
        bind hud.now = elapsed();
        bind hud.offset = elapsed() - 1000000000000;
    "#;
    // This deliberately manufactures the save's real runtime clock. It does
    // not claim to simulate a trillion seconds by dispatching ordinary events.
    let mut manufactured = session(source).save().unwrap();
    manufactured.elapsed = 1_000_000_000_000.25;
    let mut restored = ReactiveSession::restore(source, &manufactured).unwrap();
    assert_eq!(restored.snapshot().elapsed, manufactured.elapsed);
    assert_eq!(number(&restored, "now"), manufactured.elapsed);
    assert_eq!(number(&restored, "offset"), 0.25);
    restored.dispatch_json("advance", r#"{"dt":0.25}"#).unwrap();
    assert_eq!(restored.snapshot().elapsed, 1_000_000_000_000.5);
    assert_eq!(number(&restored, "now"), restored.snapshot().elapsed);
    assert_eq!(number(&restored, "offset"), 0.5);
    let before = restored.snapshot();
    assert!(restored.dispatch_json("copy", "{}").is_err());
    assert_eq!(
        restored.snapshot(),
        before,
        "ordinary state retains its numeric range limit"
    );
    let roundtrip = ReactiveSession::restore_json(source, &restored.save_json().unwrap()).unwrap();
    assert_eq!(roundtrip.snapshot(), before);
}

#[test]
fn bad_arity_types_writes_and_ambient_function_reads_fail_at_load() {
    for body in [
        "bind hud.now = elapsed(1);",
        "bind hud.now = elapsed(1, 2);",
        "bind hud.now = elapsed() and true;",
        "bind hud.now = elapsed() + \"seconds\";",
        "on tick when elapsed() set value = 1;",
        "on tick set elapsed = 1;",
        "on tick set elapsed() = 1;",
        "fn ambient() = elapsed(); bind hud.now = ambient();",
        "fn ambient(value) = value + elapsed(); bind hud.now = ambient(1);",
        "fn elapsed() = 7; bind hud.now = elapsed();",
    ] {
        let source = format!("state value = 0; event tick dt min 0 max 0.1; {body}");
        assert!(
            ReactiveSession::from_source(&source).is_err(),
            "accepted: {body}"
        );
    }
}
