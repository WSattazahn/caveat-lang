use caveat_runtime::reactive::{Provenance, ReactiveSession};
use std::collections::BTreeSet;

const GRAPH: &str = r#"
claim warm;
evidence sensor from "thermometer";
evidence forecast from "forecast";
forecast supports warm;
caveat calibration consequence high;
caveat uncertain_forecast consequence material;
calibration qualifies sensor;
uncertain_forecast qualifies forecast;
readings temperatures from sensor limit 256;
decisions heating limit 8;
event read value min -100 max 100;
on read sample temperatures = value supports warm;
fn sum(total, reading) = total + reading;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{GRAPH}\n{source}"))
        .unwrap_or_else(|error| panic!("fixture: {error}"))
}

fn read(session: &mut ReactiveSession, value: i32) {
    session
        .dispatch_json("read", &format!(r#"{{"value":{value}}}"#))
        .unwrap();
}

fn evidence(provenance: &Provenance, names: &[&str]) {
    assert_eq!(
        provenance.evidence,
        names
            .iter()
            .map(|name| (*name).into())
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn source_reducers_compute_ordered_history_with_all_qualifications() {
    let mut program = session(
        r#"
        fn digits(total, reading) = total * 10 + reading;
        fn squared(total, reading) = total + reading * reading;
        state total = 0;
        state ordered = 0;
        state squares = 0;
        event calculate;
        on calculate set total = fold_history(temperatures, 0, sum);
        on calculate set ordered = fold_history(temperatures, 0, digits);
        on calculate set squares = fold_history(temperatures, 0, squared);
        on calculate commit heating because enough using total / history_count(temperatures);
        bind average.text = if(history_count(temperatures) > 0,
            number_text(fold_history(temperatures, 0, sum) / history_count(temperatures)), "Unread");
    "#,
    );
    assert_eq!(
        program.snapshot().bindings["average"]["text"].as_str(),
        Some("Unread")
    );
    for value in [2, 4, 8] {
        read(&mut program, value);
    }
    let records = program.snapshot().reading_streams["temperatures"]
        .occurrences
        .clone();
    let result = program.dispatch_json("calculate", "{}").unwrap();
    assert_eq!(result.values["total"], 14.0);
    assert_eq!(result.values["ordered"], 248.0);
    assert_eq!(result.values["squares"], 84.0);
    assert_eq!(result.commitment_bases["heating@1"].value, Some(14.0 / 3.0));
    evidence(
        &result.qualified_values["total"].provenance,
        &["temperatures@1", "temperatures@2", "temperatures@3"],
    );
    assert!(result.commitment_bases["heating@1"]
        .provenance
        .caveats
        .contains("calibration"));
    assert_eq!(result.reading_streams["temperatures"].occurrences, records);
}

#[test]
fn an_empty_fold_keeps_its_initial_basis_and_does_not_execute_the_reducer() {
    let program = session(
        r#"
        fn cannot_run(total, reading) = require(false, total + reading);
        state empty = fold_history(temperatures, qualified(7, forecast), cannot_run);
        state count = history_count(temperatures);
    "#,
    );
    let result = program.snapshot();
    assert_eq!(result.values["empty"], 7.0);
    assert_eq!(result.values["count"], 0.0);
    evidence(&result.qualified_values["empty"].provenance, &["forecast"]);
    assert!(result.reading_streams["temperatures"]
        .occurrences
        .is_empty());
}

#[test]
fn ignored_reducer_inputs_cannot_erase_their_observations() {
    let mut program = session(
        r#"
        fn ignore(total, reading) = 0;
        state output = 0;
        event calculate;
        on calculate set output = fold_history(temperatures, qualified(5, forecast), ignore);
    "#,
    );
    read(&mut program, 2);
    read(&mut program, 3);
    let result = program.dispatch_json("calculate", "{}").unwrap();
    assert_eq!(result.values["output"], 0.0);
    evidence(
        &result.qualified_values["output"].provenance,
        &["forecast", "temperatures@1", "temperatures@2"],
    );
    assert!(result.qualified_values["output"]
        .provenance
        .caveats
        .contains("calibration"));
}

#[test]
fn indexed_reads_keep_index_provenance_without_relabeling_older_records() {
    let mut program = session(
        r#"
        state selected = qualified(0, forecast);
        state saved = 0;
        event select;
        on select set saved = history_at(temperatures, selected);
        bind last.value = if(history_count(temperatures) > 0,
            history_at(temperatures, history_count(temperatures) - 1), 0);
    "#,
    );
    read(&mut program, 17);
    read(&mut program, 25);
    let records = program.snapshot().reading_streams["temperatures"]
        .occurrences
        .clone();
    let result = program.dispatch_json("select", "{}").unwrap();
    assert_eq!(result.values["saved"], 17.0);
    evidence(
        &result.qualified_values["saved"].provenance,
        &["forecast", "temperatures@1"],
    );
    assert_eq!(result.bindings["last"]["value"].as_number(), Some(25.0));
    assert_eq!(result.reading_streams["temperatures"].occurrences, records);
    read(&mut program, 30);
    assert_eq!(
        program.snapshot().qualified_values["saved"],
        result.qualified_values["saved"]
    );
}

#[test]
fn skipped_membership_guards_reach_counts_indexes_and_empty_folds() {
    let mut program = session(
        r#"
        state gate = qualified(0, forecast);
        state count = 0;
        state total = 0;
        state indexed = 0;
        event skip;
        event calculate;
        on skip when gate > 0 sample temperatures = 5 supports warm;
        on calculate set count = history_count(temperatures);
        on calculate set total = fold_history(temperatures, 0, sum);
        on calculate when history_count(temperatures) > 0 set indexed = history_at(temperatures, 0);
    "#,
    );
    program.dispatch_json("skip", "{}").unwrap();
    let empty = program.dispatch_json("calculate", "{}").unwrap();
    evidence(&empty.qualified_values["count"].provenance, &["forecast"]);
    evidence(&empty.qualified_values["total"].provenance, &["forecast"]);
    read(&mut program, 17);
    let record = program.snapshot().reading_streams["temperatures"].occurrences[0].clone();
    program.dispatch_json("skip", "{}").unwrap();
    let result = program.dispatch_json("calculate", "{}").unwrap();
    for state in ["count", "total", "indexed"] {
        evidence(
            &result.qualified_values[state].provenance,
            &["forecast", "temperatures@1"],
        );
    }
    assert_eq!(
        result.reading_streams["temperatures"].occurrences[0],
        record
    );
}

#[test]
fn invalid_indexes_rollback_the_entire_event_without_consuming_ids() {
    for index in ["-1", "0.5", "2", "256", "1000000000000"] {
        let mut program = session(&format!(
            r#"
            state result = 0;
            event bad;
            on bad sample temperatures = 99 supports warm;
            on bad set result = history_at(temperatures, {index});
        "#
        ));
        read(&mut program, 17);
        let before = program.snapshot();
        assert!(program
            .dispatch_json("bad", "{}")
            .unwrap_err()
            .contains("index"));
        assert_eq!(program.snapshot(), before);
        read(&mut program, 25);
        assert_eq!(
            program.snapshot().reading_streams["temperatures"].occurrences[1].id,
            "temperatures@2"
        );
    }
}

#[test]
fn reducer_failure_rolls_back_samples_decisions_and_cues() {
    let mut program = session(
        r#"
        fn positive(total, reading) = require(reading >= 0, total + reading);
        event bad;
        cue beep sound 440 0.1 0.1;
        on bad emit beep;
        on bad sample temperatures = -1 opposes warm;
        on bad commit heating because enough using fold_history(temperatures, 0, positive);
    "#,
    );
    read(&mut program, 17);
    let before = program.snapshot();
    assert!(program
        .dispatch_json("bad", "{}")
        .unwrap_err()
        .contains("requirement"));
    assert_eq!(program.snapshot(), before);
}

#[test]
fn decision_queries_use_frozen_bases_and_do_not_reopen_or_revise() {
    let mut program = session(
        r#"
        event decide;
        on decide when committed(heating) reopen heating because latest(temperatures);
        on decide commit heating because enough using latest(temperatures) * 2;
        bind first.value = if(history_count(heating) > 0, history_at(heating, 0), 0);
        bind total.value = fold_history(heating, 0, sum);
    "#,
    );
    read(&mut program, 17);
    let first = program.dispatch_json("decide", "{}").unwrap();
    read(&mut program, 25);
    let second = program.dispatch_json("decide", "{}").unwrap();
    assert_eq!(second.bindings["first"]["value"].as_number(), Some(34.0));
    assert_eq!(second.bindings["total"]["value"].as_number(), Some(84.0));
    assert_eq!(
        second.commitment_bases["heating@1"],
        first.commitment_bases["heating@1"]
    );
    assert_eq!(second.decision_series["heating"].revisions.len(), 2);
}

#[test]
fn nonnumeric_decisions_can_be_counted_but_never_silently_become_zero() {
    let mut program = session(
        r#"
        state output = 0;
        event decide;
        event calculate;
        on decide commit heating because enough;
        on calculate set output = fold_history(heating, 0, sum);
        bind count.value = history_count(heating);
    "#,
    );
    let before = program.dispatch_json("decide", "{}").unwrap();
    assert_eq!(before.bindings["count"]["value"].as_number(), Some(1.0));
    assert!(program
        .dispatch_json("calculate", "{}")
        .unwrap_err()
        .contains("no numeric"));
    assert_eq!(program.snapshot(), before);
}

#[test]
fn history_computation_is_typed_and_pure_reducers_cannot_capture_state() {
    for source in [
        "state x = history_count(sensor);",
        "state x = history_at(temperatures, true);",
        "state x = fold_history(temperatures, true, sum);",
        "state x = fold_history(temperatures, 0, missing);",
        "fn one(a) = a; state x = fold_history(temperatures, 0, one);",
        "fn words(a, b) = text(a); state x = fold_history(temperatures, 0, words);",
        "fn capture(a, b) = latest(temperatures);",
        "fn capture(a, b) = history_count(temperatures);",
        "fn capture(a, b) = history_at(temperatures, 0);",
        "fn capture(a, b) = fold_history(temperatures, 0, sum);",
        "state hidden = 3; fn capture(a, b) = hidden;",
        "fn history_at(a, b) = a;",
        "fn history_count(a) = a;",
        "fn fold_history(a, b) = a;",
        "state x = if(false, history_at(missing, 0), 0);",
    ] {
        assert!(
            ReactiveSession::from_source(&format!("{GRAPH}\n{source}")).is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn fold_work_is_bounded_and_budget_failure_is_atomic() {
    // A balanced source function remains within parse/expansion bounds, but
    // applying it to 256 records exceeds the shared evaluation work budget.
    let mut expression = "a + b".to_string();
    for _ in 0..6 {
        expression = format!("({expression}) + ({expression})");
    }
    let mut program = session(&format!(
        r#"
        fn expensive(a, b) = {expression};
        state output = 0;
        event calculate;
        on calculate set output = fold_history(temperatures, 0, expensive);
    "#
    ));
    for _ in 0..256 {
        read(&mut program, 0);
    }
    let before = program.snapshot();
    assert!(program
        .dispatch_json("calculate", "{}")
        .unwrap_err()
        .contains("evaluation limit"));
    assert_eq!(program.snapshot(), before);
}
