use caveat_runtime::reactive::{ReactiveSession, REACTIVE_PRELUDE_SOURCE};
use std::collections::BTreeSet;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

#[test]
fn source_prelude_preserves_numeric_operations_and_owns_display_policy() {
    let game = session(
        r#"
        state a = abs(-8);
        state b = min(3, -2);
        state c = max(3, -2);
        state d = clamp(12, 0, 10);
        event step;
        bind total.text = number_text(12.6);
        bind progress.text = percent_text(.456);
        bind clock.text = time_text(60.1);
        bind elapsed.text = time_text(floor(60.9));
        bind empty.text = time_text(-8);
    "#,
    );
    let snapshot = game.snapshot();
    assert_eq!(snapshot.values["a"], 8.0);
    assert_eq!(snapshot.values["b"], -2.0);
    assert_eq!(snapshot.values["c"], 3.0);
    assert_eq!(snapshot.values["d"], 10.0);
    for (target, expected) in [
        ("total", "13"),
        ("progress", "46%"),
        ("clock", "1:01"),
        ("elapsed", "1:00"),
        ("empty", "0:00"),
    ] {
        assert_eq!(snapshot.bindings[target]["text"].as_str(), Some(expected));
    }
    assert!(REACTIVE_PRELUDE_SOURCE.contains("fn clamp("));
    assert!(REACTIVE_PRELUDE_SOURCE.contains("fn time_text("));
}

#[test]
fn text_functions_infer_returns_and_preserve_literal_syntax() {
    let mut game = session(
        r#"
        fn title(value) = "Reading: " + number_text(value);
        fn positive(value) = value > 0;
        fn safe_ratio(value) = if(value == 0, 0, 1 / value);
        state counter = if("two  spaces" == "two spaces", 9, 0);
        state ratio = safe_ratio(0);
        event step;
        on step when " ( set ; when # " == " ( set ; when # " set counter = if("a  b" == "a b", 99, 7);
        bind label.text = "when (" + title(counter) + ")  #;" when positive(counter);
        bind unicode.text = "海\n\"light\"";
    "#,
    );
    let initial = game.snapshot();
    assert_eq!(initial.values["counter"], 0.0);
    assert_eq!(initial.values["ratio"], 0.0);
    assert!(!initial.bindings.contains_key("label"));
    let after = game.dispatch_json("step", "{}").unwrap();
    assert_eq!(after.values["counter"], 7.0);
    assert_eq!(
        after.bindings["label"]["text"].as_str(),
        Some("when (Reading: 7)  #;")
    );
    assert_eq!(
        after.bindings["unicode"]["text"].as_str(),
        Some("海\n\"light\"")
    );
}

#[test]
fn formatting_carries_guard_and_selected_branch_qualifications() {
    let game = session(
        r#"
        claim route;
        evidence chart from "forecast";
        evidence gauge from "beam sample";
        evidence unread from "future sample";
        caveat old consequence high;
        caveat noisy consequence low;
        chart supports route;
        gauge opposes route;
        old qualifies chart;
        noisy qualifies gauge;
        state planned = qualified(1, chart);
        event step;
        bind current.text = if(planned > 0, "Current " + number_text(qualified(2.6, gauge)), number_text(qualified(99, unread)));
    "#,
    );
    let snapshot = game.snapshot();
    assert_eq!(
        snapshot.bindings["current"]["text"].as_str(),
        Some("Current 3")
    );
    let trace = &snapshot.binding_qualifications["current"]["text"];
    assert_eq!(
        trace.evidence,
        BTreeSet::from(["chart".into(), "gauge".into()])
    );
    assert_eq!(
        trace.caveats,
        BTreeSet::from(["old".into(), "noisy".into()])
    );
    assert!(!trace.evidence.contains("unread"));
}

#[test]
fn text_returning_functions_keep_eager_numeric_arguments() {
    let mut game = session(
        r#"
        claim route;
        evidence chart from "forecast";
        caveat old consequence high;
        chart supports route;
        old qualifies chart;
        fn constant(ignored) = "steady";
        state sample = qualified(0, chart);
        event step;
        bind label.text = constant(sample);
    "#,
    );
    let before = game.snapshot();
    assert_eq!(before.bindings["label"]["text"].as_str(), Some("steady"));
    assert!(before.binding_qualifications["label"]["text"]
        .evidence
        .contains("chart"));
    assert_eq!(
        game.dispatch_json("step", "{}").unwrap().values,
        before.values
    );
    assert!(ReactiveSession::from_source(
        "fn constant(ignored) = \"steady\"; event step; bind label.text = constant(1 / 0);"
    )
    .is_err());
}

#[test]
fn failed_clamp_requirement_rolls_back_prior_graph_effects() {
    let mut game = session(
        r#"
        claim route;
        evidence chart from "forecast";
        chart supports route;
        state value = 0;
        event step;
        on step commit proceed because enough using qualified(1, chart);
        on step set value = clamp(5, 10, 0);
    "#,
    );
    let before = game.snapshot();
    assert!(game.dispatch_json("step", "{}").is_err());
    assert_eq!(game.snapshot(), before);
}

#[test]
fn oversized_text_rolls_back_graph_values_cues_and_binding_metadata() {
    let chunk = "x".repeat(33_000);
    let source = format!(
        r#"
        claim route;
        evidence chart from "forecast";
        chart supports route;
        fn chunk(ignored) = "{chunk}";
        state phase = 0;
        cue signal toast "ready" 1;
        event step;
        on step commit proceed because enough using qualified(1, chart);
        on step emit signal;
        on step set phase = 1;
        bind label.text = if(phase == 0, "ready", chunk(phase) + chunk(phase));
    "#
    );
    let mut game = session(&source);
    let before = game.snapshot();
    let error = game.dispatch_json("step", "{}").unwrap_err();
    assert!(error.contains("text"), "{error}");
    assert_eq!(game.snapshot(), before);
}

#[test]
fn text_types_and_prelude_names_are_validated_before_execution() {
    for source in [
        "event step; bind label.text = 1 + \"one\";",
        "event step; bind label.text = if(true, \"one\", 1);",
        "fn caption(value) = \"one\"; state value = caption(1); event step;",
        "fn number(value) = value; event step; bind label.text = number(\"one\");",
        "fn clamp(value, lower, upper) = value; event step;",
        "fn time_text(value) = \"override\"; event step;",
        "fn bad(value) = if(value > 0, bad(value - 1), \"done\"); event step;",
        "fn captured(value) = if(observed(chart), \"yes\", \"no\"); claim route; evidence chart from \"forecast\"; event step;",
    ] {
        assert!(ReactiveSession::from_source(source).is_err(), "{source}");
    }
}
