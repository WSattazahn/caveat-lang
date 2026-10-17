//! `occurrence(EVIDENCE, INDEX)`: naming an earlier occurrence of renewable
//! evidence in `withdraw`, `qualify`, `unqualify`, `reopen … because` and the
//! `observed`, `withdrawn` and `carries` predicates. See
//! spec/caveat-earlier-occurrence-0.1.md and issue #238.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};

/// The spec's example: a check reports twice, then an operator says which
/// report was wrong.
const LEDGER: &str = r#"
claim passing;
caveat flaky consequence material;
evidence tests from "test runner";
renewable tests limit 8;
evidence operator from "the operator's correction";
decisions ship limit 8;
state first = 0;
state second = 0;
state flagged = 0;
state rests = 0;
event first_report;
event report;
event keep_first;
event keep_second;
event decide;
event correct n min 0 max 9;
event flag n min 0 max 9;
event unflag n min 0 max 9;
event unflag_all;
event ask n min 0 max 9;
// Not `on report when observed(tests) renew tests`: that guard would put
// the first report into the second's lineage, and a caveat on the first into
// every value made from the second.
on first_report reveal tests supports passing;
on report renew tests;
on report reveal tests supports passing;
on keep_first set first = qualified(1, tests);
on keep_second set second = qualified(1, tests);
on decide commit ship because enough using qualified(1, tests);
on correct when not observed(occurrence(tests, n)) reject "no such report";
on correct when withdrawn(occurrence(tests, n)) reject "already withdrawn";
on correct reveal operator;
on correct withdraw occurrence(tests, n) because operator;
on correct when committed(ship) and not reopened(ship) and rests_on_withdrawn(ship)
    reopen ship because occurrence(tests, n);
on flag qualify occurrence(tests, n) with flaky;
on unflag unqualify occurrence(tests, n) with flaky;
on unflag_all unqualify tests with flaky;
on ask set flagged = if(carries(occurrence(tests, n), flaky), 1, 0);
on ask set rests = if(rests_on_withdrawn(ship), 1, 0);
"#;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn state(game: &ReactiveSession, name: &str) -> Value {
    snapshot(game)["qualified_values"][name].clone()
}

fn caveats(game: &ReactiveSession, name: &str) -> Value {
    state(game, name)["provenance"]["caveats"].clone()
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

/// Two reports, a decision on the first, and a state copied from each.
fn two_reports() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(LEDGER).unwrap();
    run(
        &mut game,
        &[
            ("first_report", "{}"),
            ("keep_first", "{}"),
            ("decide", "{}"),
            ("report", "{}"),
            ("keep_second", "{}"),
        ],
    );
    game
}

fn refused(source: &str, setup: &[(&str, &str)], event: &str, payload: &str) -> Value {
    let mut session = WebReactiveSession::new(source).unwrap();
    for (event, payload) in setup {
        let accepted = json(&session.dispatch_outcome(event, payload).unwrap());
        assert_eq!(accepted["outcome"], "accepted", "{event}: {accepted}");
    }
    let before = json(&session.save().unwrap());
    let result = json(&session.dispatch_outcome(event, payload).unwrap());
    assert_eq!(result["outcome"], "rejected", "{event} {payload}: {result}");
    assert_eq!(
        json(&session.save().unwrap()),
        before,
        "a refusal keeps nothing"
    );
    result
}

#[test]
fn issue_238_withdrawing_the_first_report_after_a_second_reopens_the_decision_on_it() {
    let mut game = two_reports();
    let shown = game.dispatch_json("correct", r#"{"n": 1}"#).unwrap();
    let shown = serde_json::to_value(&shown).unwrap();
    // The record names the first occurrence; the current one is untouched.
    assert_eq!(shown["withdrawals"][0]["evidence"], "tests");
    assert_eq!(shown["withdrawals"][0]["because"], "operator");
    assert_eq!(caveats(&game, "first"), json!(["withdrawn"]));
    assert_eq!(caveats(&game, "second"), json!([]));
    // The decision rested on the first report, so the rule reopened it,
    // citing that report.
    let journal = &snapshot(&game)["decision_journal"];
    let reopening = journal.as_array().unwrap().last().unwrap();
    assert_eq!(reopening["change"], "reopened", "{journal}");
    assert!(
        reopening.to_string().contains("\"tests\""),
        "the reopening cites the first report: {reopening}"
    );
    // A second withdrawal of it is the program's own refusal.
    let result = refused(
        LEDGER,
        &[
            ("first_report", "{}"),
            ("report", "{}"),
            ("correct", r#"{"n": 1}"#),
        ],
        "correct",
        r#"{"n": 1}"#,
    );
    assert_eq!(result["code"], "reject");
    assert!(result["message"]
        .as_str()
        .unwrap()
        .contains("already withdrawn"));
}

#[test]
fn withdrawing_the_current_report_by_its_index_matches_withdrawing_it_by_name() {
    let mut game = two_reports();
    run(&mut game, &[("correct", r#"{"n": 2}"#)]);
    let withdrawals = snapshot(&game)["withdrawals"].clone();
    assert_eq!(withdrawals[0]["evidence"], "tests@2");
    assert_eq!(caveats(&game, "first"), json!([]));
    assert_eq!(caveats(&game, "second"), json!(["withdrawn"]));
    // The decision did not rest on the second report.
    run(&mut game, &[("ask", r#"{"n": 2}"#)]);
    assert_eq!(state(&game, "rests")["value"], json!(0.0));
}

#[test]
fn qualifying_an_earlier_report_reaches_only_values_that_hold_it() {
    let mut game = two_reports();
    let frozen = snapshot(&game)["decision_journal"].clone();
    run(&mut game, &[("flag", r#"{"n": 1}"#)]);
    assert_eq!(caveats(&game, "first"), json!(["flaky"]));
    assert_eq!(caveats(&game, "second"), json!([]));
    // A decision keeps what it was made on.
    assert_eq!(snapshot(&game)["decision_journal"], frozen);
    run(&mut game, &[("ask", r#"{"n": 1}"#)]);
    assert_eq!(state(&game, "flagged")["value"], json!(1.0));
    run(&mut game, &[("ask", r#"{"n": 2}"#)]);
    assert_eq!(state(&game, "flagged")["value"], json!(0.0));
}

#[test]
fn unqualify_of_one_occurrence_marks_only_that_relation() {
    let mut game = two_reports();
    run(
        &mut game,
        &[
            ("flag", r#"{"n": 1}"#),
            ("flag", r#"{"n": 2}"#),
            ("unflag", r#"{"n": 1}"#),
        ],
    );
    let taken = snapshot(&game)["taken_back"].clone();
    assert_eq!(taken.as_array().unwrap().len(), 1, "{taken}");
    assert_eq!(taken[0]["evidence"], "tests");
    assert_eq!(caveats(&game, "first"), json!([]));
    assert_eq!(caveats(&game, "second"), json!(["flaky"]));
    // The plain form still reaches every held occurrence.
    run(&mut game, &[("flag", r#"{"n": 1}"#), ("unflag_all", "{}")]);
    let taken = snapshot(&game)["taken_back"].clone();
    assert_eq!(taken.as_array().unwrap().len(), 3, "{taken}");
    assert_eq!(caveats(&game, "second"), json!([]));
}

#[test]
fn an_unqualify_of_an_occurrence_with_nothing_to_take_back_records_nothing() {
    let mut game = two_reports();
    run(
        &mut game,
        &[("flag", r#"{"n": 2}"#), ("unflag", r#"{"n": 1}"#)],
    );
    assert_eq!(
        snapshot(&game)
            .get("taken_back")
            .cloned()
            .unwrap_or(json!([])),
        json!([])
    );
    assert_eq!(caveats(&game, "second"), json!(["flaky"]));
}

#[test]
fn an_index_that_names_no_held_occurrence_refuses_an_effect_and_falsifies_a_predicate() {
    for n in ["0", "3", "1.5"] {
        let payload = format!(r#"{{"n": {n}}}"#);
        let result = refused(
            LEDGER,
            &[("first_report", "{}"), ("report", "{}")],
            "flag",
            &payload,
        );
        assert_eq!(result["origin"], "evaluation", "{n}: {result}");
        assert_eq!(result["code"], "expression", "{n}: {result}");
        let result = refused(
            LEDGER,
            &[("first_report", "{}"), ("report", "{}")],
            "unflag",
            &payload,
        );
        assert_eq!(result["code"], "expression", "{n}: {result}");
        // The guard's `observed(occurrence(tests, n))` is false, so the
        // program's own refusal comes first.
        let result = refused(
            LEDGER,
            &[("first_report", "{}"), ("report", "{}")],
            "correct",
            &payload,
        );
        assert_eq!(result["code"], "reject", "{n}: {result}");
        assert!(result["message"]
            .as_str()
            .unwrap()
            .contains("no such report"));
    }
}

#[test]
fn an_earlier_occurrence_renewed_before_it_was_observed_refuses_as_unobserved() {
    let source = format!("{LEDGER}\nevent skip;\non skip renew tests;");
    // tests@2 was made by a renew and never revealed before tests@3.
    let setup = [("first_report", "{}"), ("skip", "{}"), ("report", "{}")];
    let result = refused(&source, &setup, "flag", r#"{"n": 2}"#);
    assert_eq!(result["code"], "unobserved_evidence", "{result}");
    // An unqualify needs nothing observed and records nothing.
    let mut session = WebReactiveSession::new(&source).unwrap();
    for (event, payload) in setup.iter().chain([("unflag", r#"{"n": 2}"#)].iter()) {
        let accepted = json(&session.dispatch_outcome(event, payload).unwrap());
        assert_eq!(accepted["outcome"], "accepted", "{event}: {accepted}");
    }
}

#[test]
fn a_retired_occurrence_is_not_held() {
    let source = LEDGER.replace("renewable tests limit 8;", "renewable tests window 2;");
    let setup = [("first_report", "{}"), ("report", "{}"), ("report", "{}")];
    let result = refused(&source, &setup, "flag", r#"{"n": 1}"#);
    assert_eq!(result["code"], "expression", "{result}");
    let mut game = ReactiveSession::from_source(&source).unwrap();
    run(&mut game, &setup);
    run(
        &mut game,
        &[("flag", r#"{"n": 2}"#), ("ask", r#"{"n": 1}"#)],
    );
    assert_eq!(state(&game, "flagged")["value"], json!(0.0));
}

#[test]
fn the_index_joins_the_lineage_of_what_reads_it() {
    let source = r#"
claim passing;
caveat flaky consequence material;
evidence tests from "test runner";
renewable tests limit 8;
evidence pick from "which report the operator meant";
state which = 1;
state seen = 0;
event report;
event choose;
event look;
on report when observed(tests) renew tests;
on report reveal tests supports passing;
on choose reveal pick;
on choose set which = qualified(1, pick);
on look set seen = if(observed(occurrence(tests, which)), 1, 0);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(
        &mut game,
        &[
            ("report", "{}"),
            ("report", "{}"),
            ("choose", "{}"),
            ("look", "{}"),
        ],
    );
    let seen = state(&game, "seen");
    assert_eq!(seen["value"], json!(1.0));
    let lineage = seen["provenance"]["evidence"].as_array().unwrap();
    assert!(lineage.contains(&json!("pick")), "{seen}");
    assert!(lineage.contains(&json!("tests")), "{seen}");
}

#[test]
fn a_session_that_named_earlier_occurrences_saves_and_restores() {
    let mut game = two_reports();
    run(
        &mut game,
        &[
            ("flag", r#"{"n": 1}"#),
            ("flag", r#"{"n": 2}"#),
            ("unflag", r#"{"n": 2}"#),
            ("correct", r#"{"n": 1}"#),
        ],
    );
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(LEDGER, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    assert_eq!(restored.save_json().unwrap(), saved);
}

#[test]
fn load_rules() {
    for (line, expected) in [
        (
            "evidence plain from \"x\";\nevent go;\non go withdraw occurrence(plain, 1) because operator;",
            "renewable",
        ),
        (
            "event go;\non go qualify occurrence(tests, \"one\") with flaky;",
            "numeric index",
        ),
        (
            "event go;\non go qualify occurrence(tests) with flaky;",
            "EVIDENCE, INDEX",
        ),
        (
            "state s = 0;\nevent go;\non go set s = if(observed(occurrence(operator, 1)), 1, 0);",
            "renewable",
        ),
        (
            "event go;\non go commit ship because enough permitted by occurrence(tests, 1);",
            "",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{LEDGER}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
}

#[test]
fn occurrence_is_not_reserved() {
    let source = r#"
caveat flaky consequence material;
evidence occurrence from "a source named occurrence";
evidence operator from "operator";
state s = 0;
fn occurrence2(a, b) = a + b;
event go;
on go reveal occurrence;
on go reveal operator;
on go qualify occurrence with flaky;
on go set s = if(observed(occurrence), 1, 0);
on go withdraw occurrence because operator;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(&mut game, &[("go", "{}")]);
    assert_eq!(state(&game, "s")["value"], json!(1.0));
    assert_eq!(snapshot(&game)["withdrawals"][0]["evidence"], "occurrence");
}

#[test]
fn a_procedure_names_an_occurrence_of_its_evidence_parameter() {
    let source = r#"
claim passing;
evidence tests from "test runner";
renewable tests limit 8;
evidence operator from "operator";
state first = 0;
event report;
event keep;
event correct n min 1 max 8;
proc retract(e evidence, k) {
    reveal operator;
    withdraw occurrence(e, k) because operator;
};
on report when observed(tests) renew tests;
on report reveal tests supports passing;
on keep set first = qualified(1, tests);
on correct call retract(tests, n);
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(
        &mut game,
        &[
            ("report", "{}"),
            ("keep", "{}"),
            ("report", "{}"),
            ("correct", r#"{"n": 1}"#),
        ],
    );
    assert_eq!(snapshot(&game)["withdrawals"][0]["evidence"], "tests");
    assert_eq!(caveats(&game, "first"), json!(["withdrawn"]));
}
