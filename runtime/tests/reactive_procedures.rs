use caveat_runtime::reactive::{Provenance, ReactiveSession};
use std::collections::BTreeSet;

const GRAPH: &str = r#"
budget 4;
claim safe;
evidence chart from "archive";
evidence sensor from "instrument";
evidence unseen from "unread instrument";
caveat age consequence high;
caveat calibration consequence high;
chart supports safe;
age qualifies chart;
calibration qualifies sensor;
"#;

fn session(body: &str) -> ReactiveSession {
    ReactiveSession::from_source(&format!("{GRAPH}\n{body}"))
        .unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn trace(actual: &Provenance, evidence: &[&str], caveats: &[&str]) {
    let names = |values: &[&str]| -> BTreeSet<String> {
        values.iter().map(|name| (*name).into()).collect()
    };
    assert_eq!(actual.evidence, names(evidence));
    assert_eq!(actual.caveats, names(caveats));
}

#[test]
fn calls_freeze_outer_guard_and_arguments_but_body_guards_see_prior_writes() {
    let mut program = session(
        r#"
        state gate = qualified(1, chart);
        state input = 7;
        state frozen = 0;
        state chosen = 0;
        state skipped = 3;
        event run;
        proc update(captured) {
            set gate = 0;
            set input = 99;
            set frozen = captured;
            when input == 99 set chosen = 1;
            when input < 0 set skipped = 5;
        };
        on run when gate > 0 call update(input);
    "#,
    );
    let result = program.dispatch_json("run", "{}").unwrap();
    assert_eq!(result.sequence, 1);
    assert_eq!(result.values["frozen"], 7.0);
    assert_eq!(result.values["input"], 99.0);
    assert_eq!(result.values["chosen"], 1.0);
    assert_eq!(result.values["skipped"], 3.0);
    for name in ["gate", "input", "frozen", "chosen", "skipped"] {
        trace(
            &result.qualified_values[name].provenance,
            &["chart"],
            &["age"],
        );
    }
}

#[test]
fn ignored_arguments_and_nested_calls_cannot_launder_qualifications() {
    let mut program = session(
        r#"
        state output = 0;
        state retained = 3;
        event run;
        cue note toast "Selected provisionally" 1;
        proc inner() {
            set output = 4;
            when false set retained = 9;
            emit note;
            commit course because enough using output;
        };
        proc outer(ignored) { call inner(); };
        on run call outer(qualified(2, chart));
    "#,
    );
    let result = program.dispatch_json("run", "{}").unwrap();
    assert_eq!(result.values["output"], 4.0);
    assert_eq!(result.values["retained"], 3.0);
    assert_eq!(result.cues.len(), 1);
    for provenance in [
        &result.qualified_values["output"].provenance,
        &result.qualified_values["retained"].provenance,
        &result.cue_qualifications[0],
        &result.commitment_bases["course"].provenance,
    ] {
        trace(provenance, &["chart"], &["age"]);
    }
}

#[test]
fn unused_arguments_are_eager_and_failed_calls_are_atomic() {
    let mut program = session(
        r#"
        state output = 0;
        event bad;
        proc ignore(arg) { set output = 1; };
        on bad set output = 9;
        on bad call ignore(1 / 0);
    "#,
    );
    let before = program.snapshot();
    assert!(program.dispatch_json("bad", "{}").is_err());
    assert_eq!(program.snapshot(), before);
}

#[test]
fn skipped_calls_do_not_evaluate_arguments_or_body_guards_but_retain_dependencies() {
    let mut program = session(
        r#"
        readings readings_log from sensor limit 4;
        decisions choices limit 4;
        state output = 8;
        state absent = 0;
        state gate = qualified(0, chart);
        event run;
        proc leaf(arg) {
            when 1 / 0 > 0 set output = arg;
            sample readings_log = arg supports safe;
            reveal unseen supports safe;
            examine calibration cost 1;
            commit choices because enough using arg;
            reopen choices because chart;
        };
        proc outer(ignored) { call leaf(1 / 0); };
        on run when gate > 0 call outer(qualified(1, unseen));
        on run when not observed(unseen) and not examined(calibration)
            and not has_sample(readings_log) and not committed(choices) set absent = 1;
    "#,
    );
    let result = program.dispatch_json("run", "{}").unwrap();
    assert_eq!(result.values["output"], 8.0);
    assert_eq!(result.values["absent"], 1.0);
    assert!(result.reading_streams["readings_log"]
        .occurrences
        .is_empty());
    assert!(result.decision_series["choices"].revisions.is_empty());
    assert!(result.effects.is_empty());
    assert_eq!(result.budget.as_ref().unwrap().spent, 0);
    trace(
        &result.qualified_values["output"].provenance,
        &["chart"],
        &["age"],
    );
    assert!(result.qualified_values["absent"]
        .provenance
        .evidence
        .contains("chart"));
    for provenance in [
        &result.reading_streams["readings_log"].selection_qualifications,
        &result.decision_series["choices"].selection_qualifications,
    ] {
        trace(provenance, &["chart"], &["age"]);
    }
    assert!(!result.relations.iter().any(|edge| edge.from == "unseen"));
}

#[test]
fn failed_late_effect_rolls_back_graph_attention_cues_history_and_ids() {
    let mut program = session(
        r#"
        readings readings_log from sensor limit 4;
        decisions choices limit 4;
        state output = 0;
        event bad;
        event good;
        cue note toast "Observed" 1;
        proc observe() {
            sample readings_log = 4 supports safe;
            reveal unseen opposes safe;
            examine calibration cost 1;
            commit choices because enough using latest(readings_log);
            reopen choices because unseen;
            emit note;
        };
        proc fail_late() { call observe(); set output = 1 / 0; };
        on bad call fail_late();
        on good call observe();
    "#,
    );
    let before = program.snapshot();
    assert!(program.dispatch_json("bad", "{}").is_err());
    assert_eq!(program.snapshot(), before);
    let result = program.dispatch_json("good", "{}").unwrap();
    assert_eq!(result.sequence, 1);
    assert_eq!(
        result.reading_streams["readings_log"].current.as_deref(),
        Some("readings_log@1")
    );
    assert_eq!(
        result.decision_series["choices"].current.as_deref(),
        Some("choices@1")
    );
    assert_eq!(result.cues.len(), 1);
    assert!(result.budget.as_ref().unwrap().spent > 0);
    assert!(
        result
            .commitments
            .iter()
            .find(|c| c.action == "choices@1")
            .unwrap()
            .open
    );
}

#[test]
fn bindings_are_evaluated_after_the_whole_call_and_can_roll_it_back() {
    let mut program = session(
        r#"
        state divisor = 1;
        event good;
        event bad;
        bind output.value = 1 / divisor;
        proc temporarily_invalid() { set divisor = 0; set divisor = 2; };
        proc invalid_at_end() { set divisor = 0; };
        on good call temporarily_invalid();
        on bad call invalid_at_end();
    "#,
    );
    let good = program.dispatch_json("good", "{}").unwrap();
    assert_eq!(good.bindings["output"]["value"].as_number(), Some(0.5));
    assert!(program.dispatch_json("bad", "{}").is_err());
    assert_eq!(program.snapshot(), good);
}

#[test]
fn procedure_frames_do_not_capture_caller_parameters() {
    let mut program = session(
        r#"
        state output = 0;
        event run value min 0 max 10;
        proc inner(value) { set output = value; };
        proc outer(value) { call inner(value + 1); set output = output + value; };
        on run call outer(value);
    "#,
    );
    assert_eq!(
        program
            .dispatch_json("run", r#"{"value":3}"#)
            .unwrap()
            .values["output"],
        7.0
    );
    for source in [
        "state x = 0; event run value min 0 max 10; proc bad() { set x = value; }; on run call bad();",
        "state x = 0; event run; proc inner() { set x = arg; }; proc outer(arg) { call inner(); }; on run call outer(1);",
    ] {
        assert!(ReactiveSession::from_source(source).is_err(), "accepted caller capture: {source}");
    }
}

#[test]
fn invalid_unused_procedures_and_calls_are_rejected_at_load_time() {
    for body in [
        "proc bad() { set missing = 1; };",
        "proc bad() { call missing(); };",
        "proc bad() { call bad(); };",
        "proc a() { call b(); }; proc b() { call a(); };",
        "proc bad(x, x) { set output = x; };",
        "proc bad(output) { set output = 1; };",
        "proc bad() { when 3 set output = 1; };",
        "proc bad() { set output = true; };",
        "proc bad() { set output = 1; }; proc bad() { set output = 2; };",
        "proc bad(arg) { set output = arg; }; on run call bad();",
        "proc bad(arg) { set output = arg; }; on run call bad(1, 2);",
        "proc bad(arg) { set output = arg; }; on run when false call bad(\"text\");",
        "proc bad(arg) { set output = arg; }; on run when false call bad(true);",
        "proc bad() { commit chart because enough; };",
        "proc bad() { proc nested() { set output = 1; }; };",
    ] {
        let source = format!("{GRAPH} state output = 0; event run; {body}");
        assert!(
            ReactiveSession::from_source(&source).is_err(),
            "accepted invalid definition: {body}"
        );
    }
}

#[test]
fn procedure_limits_cover_declarations_parameters_depth_and_unused_fanout() {
    let mut declarations = String::from("state output = 0; event run;");
    for index in 0..129 {
        declarations.push_str(&format!("proc p{index}() {{ set output = 1; }};"));
    }
    assert!(ReactiveSession::from_source(&declarations)
        .unwrap_err()
        .contains("limit"));

    let params = (0..33)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    assert!(ReactiveSession::from_source(&format!(
        "state x = 0; event run; proc p({params}) {{ set x = 1; }};"
    ))
    .is_err());

    let mut deep = String::from("state output = 0; event run; proc p0() { set output = 1; };");
    for index in 1..65 {
        deep.push_str(&format!("proc p{index}() {{ call p{}(); }};", index - 1));
    }
    assert!(ReactiveSession::from_source(&deep)
        .unwrap_err()
        .contains("depth"));

    let mut fanout = String::from("state output = 0; event run; proc p0() { set output = 1; };");
    for index in 1..13 {
        fanout.push_str(&format!(
            "proc p{index}() {{ call p{}(); call p{}(); }};",
            index - 1,
            index - 1
        ));
    }
    assert!(ReactiveSession::from_source(&fanout)
        .unwrap_err()
        .contains("work limit"));

    let steps = format!(
        "state x = 0; event run; proc p() {{ {} }};",
        "set x = 1;".repeat(4097)
    );
    assert!(ReactiveSession::from_source(&steps)
        .unwrap_err()
        .contains("step limit"));
}

#[test]
fn event_work_budget_is_shared_by_calls_and_skipped_traversal() {
    let body = "set output = output + 1;".repeat(2048);
    for guard in ["true", "false"] {
        let mut program = session(&format!(
            "state output = 0; event run; proc many() {{ {body} }}; on run when {guard} call many(); on run when {guard} call many();"
        ));
        let before = program.snapshot();
        let error = program.dispatch_json("run", "{}").unwrap_err();
        assert!(error.contains("limit"), "{error}");
        assert_eq!(program.snapshot(), before);
    }
}

#[test]
fn parser_preserves_quoted_braces_semicolons_comments_and_unicode_locations() {
    let mut program = session(
        r#"
        state output = 0;
        cue note toast "广州; {home} // #" 1;
        proc run() {
            // an ignored }; delimiter
            # another ignored { brace
            set output = 2;
            emit note;
        };
        event go;
        on go call run();
    "#,
    );
    assert_eq!(
        program.dispatch_json("go", "{}").unwrap().values["output"],
        2.0
    );
    for source in [
        "proc missing() {",
        "}",
        "proc bad() { set x = 1; } garbage;",
        "proc bad() { set x = 1 set x = 2; };",
    ] {
        let source = format!("state x = 0; event run; {source}");
        assert!(
            ReactiveSession::from_source(&source).is_err(),
            "accepted malformed body: {source}"
        );
    }
    let error =
        ReactiveSession::from_source("display safe \"广州\";\nproc bad() {\n    nonsense;\n};")
            .unwrap_err();
    assert!(
        error.contains("line 3, column 5"),
        "expected body location: {error}"
    );
    let mut optional_semicolon =
        session("state x = 0; event run; proc assign() { set x = 1 }; on run call assign()");
    assert_eq!(
        optional_semicolon
            .dispatch_json("run", "{}")
            .unwrap()
            .values["x"],
        1.0
    );
}

#[test]
fn history_arguments_freeze_their_value_and_occurrence_before_body_sampling() {
    let mut program = session(
        r#"
        readings readings_log from sensor limit 4;
        state saved = 0;
        event run;
        proc revise(previous) {
            sample readings_log = 9 supports safe;
            set saved = previous;
            commit course because enough using previous;
        };
        on run sample readings_log = 4 supports safe;
        on run call revise(latest(readings_log));
    "#,
    );
    let result = program.dispatch_json("run", "{}").unwrap();
    assert_eq!(result.values["saved"], 4.0);
    assert_eq!(result.commitment_bases["course"].value, Some(4.0));
    assert_eq!(
        result.reading_streams["readings_log"].occurrences[1].value,
        9.0
    );
    trace(
        &result.qualified_values["saved"].provenance,
        &["readings_log@1"],
        &["calibration"],
    );
    trace(
        &result.commitment_bases["course"].provenance,
        &["readings_log@1"],
        &["calibration"],
    );
}
