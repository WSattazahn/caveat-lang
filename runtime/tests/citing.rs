//! `commit D because R citing X, Y`: a decision with grounds and no value.
//! See spec/caveat-citing-0.1.md and issue #198.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;
use serde_json::{json, Value};

/// Issue #198's release gate, with the hold citing the readings it rests on.
const GATE: &str = r#"
claim candidate_passes;
evidence ci_run from "a CI run reported for a commit";
evidence build_check from "a build-only check";
evidence operator from "the operator";
caveat other_commit consequence material;
decisions hold limit 4;
decisions sum_hold limit 4;
state ci = 0;
state build = 0;
state value = 0;
event ci_reported passed min 0 max 1;
event build_reported ok min 0 max 1;
event hold_release;
event sum_release;
event retract_ci;
event read_hold;
event cite_unobserved;
event cite_bad;
on ci_reported reveal ci_run supports candidate_passes;
on ci_reported set ci = qualified(passed, ci_run);
on build_reported reveal build_check supports candidate_passes;
on build_reported set build = qualified(ok, build_check);
on hold_release when reopened(hold) or not committed(hold)
    commit hold because enough citing ci, build;
on sum_release commit sum_hold because enough using ci + build;
on retract_ci reveal operator;
on retract_ci withdraw ci_run because operator;
on retract_ci when committed(hold) and rests_on_withdrawn(hold)
    reopen hold because ci_run;
on read_hold set value = latest(hold);
on cite_unobserved commit hold because enough citing qualified(1, operator);
on cite_bad commit hold because enough citing 1 / 0;
"#;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn run(game: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        game.dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn reported() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(GATE).unwrap();
    run(
        &mut game,
        &[
            ("ci_reported", r#"{"passed": 0}"#),
            ("build_reported", r#"{"ok": 1}"#),
        ],
    );
    game
}

fn refused(setup: &[(&str, &str)], event: &str) -> Value {
    let mut session = WebReactiveSession::new(GATE).unwrap();
    for (event, payload) in setup {
        let accepted = json(&session.dispatch_outcome(event, payload).unwrap());
        assert_eq!(accepted["outcome"], "accepted", "{event}: {accepted}");
    }
    let before = json(&session.save().unwrap());
    let result = json(&session.dispatch_outcome(event, "{}").unwrap());
    assert_eq!(result["outcome"], "rejected", "{event}: {result}");
    assert_eq!(
        json(&session.save().unwrap()),
        before,
        "a refusal keeps nothing"
    );
    result
}

#[test]
fn issue_198_a_hold_keeps_its_grounds_without_a_value() {
    let mut game = reported();
    run(&mut game, &[("hold_release", "{}"), ("sum_release", "{}")]);
    let shown = snapshot(&game);
    // Same grounds as the `using` idiom, and no number to explain.
    assert_eq!(
        shown["commitment_grounds"]["hold@1"]["evidence"],
        json!(["build_check", "ci_run"])
    );
    assert_eq!(
        shown["commitment_grounds"]["hold@1"]["evidence"],
        shown["commitment_grounds"]["sum_hold@1"]["evidence"]
    );
    assert_eq!(shown["commitment_bases"]["hold@1"]["value"], Value::Null);
    assert_eq!(shown["commitment_bases"]["sum_hold@1"]["value"], json!(1.0));
    let relies = shown["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|relation| relation.to_string().contains("relies_on"))
        .count();
    assert!(relies >= 4, "both decisions rely on both readings: {shown}");
}

#[test]
fn withdrawing_cited_evidence_reaches_the_decision() {
    let mut game = reported();
    run(&mut game, &[("hold_release", "{}"), ("retract_ci", "{}")]);
    let journal = snapshot(&game)["decision_journal"].clone();
    let last = journal.as_array().unwrap().last().unwrap();
    assert_eq!(last["change"], "reopened", "{journal}");
    // The reopened series can commit again, citing what holds now.
    run(&mut game, &[("hold_release", "{}")]);
    assert_eq!(
        snapshot(&game)["commitment_grounds"]["hold@2"]["evidence"],
        json!(["build_check", "ci_run"])
    );
}

#[test]
fn a_citing_decision_has_no_value_to_read() {
    let result = refused(
        &[
            ("ci_reported", r#"{"passed": 0}"#),
            ("build_reported", r#"{"ok": 1}"#),
            ("hold_release", "{}"),
        ],
        "read_hold",
    );
    assert_eq!(result["code"], "expression", "{result}");
}

#[test]
fn a_citation_refuses_as_a_using_value_would() {
    let result = refused(&[], "cite_unobserved");
    assert_eq!(result["code"], "unobserved_evidence", "{result}");
    let result = refused(&[], "cite_bad");
    assert_eq!(result["code"], "expression", "{result}");
}

#[test]
fn a_citing_session_saves_and_restores() {
    let mut game = reported();
    run(&mut game, &[("hold_release", "{}"), ("retract_ci", "{}")]);
    let saved = game.save_json().unwrap();
    let restored = ReactiveSession::restore_json(GATE, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    assert_eq!(restored.save_json().unwrap(), saved);
}

#[test]
fn citations_may_be_any_type_and_carry_permission_and_retaining() {
    let source = r#"
claim ok;
evidence check from "a check";
evidence go from "a go-ahead";
caveat stale consequence low;
decisions gate limit 2;
state seen = 0;
event run;
on run reveal check supports ok;
on run reveal go;
on run set seen = qualified(1, check);
on run commit gate because enough citing seen > 0, "noted"
    permitted by go retaining stale;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(&mut game, &[("run", "{}")]);
    let shown = snapshot(&game);
    assert_eq!(
        shown["commitment_grounds"]["gate@1"],
        json!({"evidence": ["check"], "caveats": ["stale"]})
    );
    assert_eq!(shown["commitment_bases"]["gate@1"]["value"], Value::Null);
}

#[test]
fn load_rules() {
    for (line, expected) in [
        (
            "on hold_release commit hold because enough citing;",
            "commit expects",
        ),
        (
            "on hold_release commit hold because enough citing ci,;",
            "citing expects",
        ),
        (
            "on hold_release commit hold because enough citing ci,, build;",
            "citing expects",
        ),
        (
            "on hold_release commit hold because enough citing nothing;",
            "leave citing out",
        ),
        (
            "on hold_release commit hold because enough citing missing;",
            "missing",
        ),
        (
            "on hold_release commit hold because enough using ci citing build;",
            "",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{GATE}\n{line}"))
            .err()
            .unwrap_or_else(|| panic!("{line} should not load"));
        assert!(error.contains(expected), "{line}: {error}");
    }
}

#[test]
fn citing_is_not_reserved() {
    let source = r#"
decisions plan limit 2;
state citing = 2;
event go;
on go commit plan because enough using citing;
"#;
    let mut game = ReactiveSession::from_source(source).unwrap();
    run(&mut game, &[("go", "{}")]);
    assert_eq!(
        snapshot(&game)["commitment_bases"]["plan@1"]["value"],
        json!(2.0)
    );
}
