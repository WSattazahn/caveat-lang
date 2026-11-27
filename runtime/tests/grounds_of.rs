//! `grounds_of(SERIES)`: the current revision's value on its frozen grounds,
//! where `latest(SERIES)` gives it on its whole basis. See
//! spec/caveat-grounds-of-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// The pharma-safety shape: a signal decided on one case, reopened when the
/// case is withdrawn, and decided again on another case. The second
/// revision's lineage still holds the withdrawn case, through the reopening;
/// its grounds hold only the new case.
const SIGNAL: &str = r#"
claim liver_risk;
evidence case_a from "the first case report";
evidence case_b from "the second case report";
evidence recheck from "the review that withdrew the first case";
evidence audit from "an audit of the second case";
caveat preliminary consequence low;
preliminary qualifies case_b;
decisions liver_signal limit 4;
state on_basis = 0;
state on_grounds = 0;
state rests = 0;
event report_a; event report_b; event retract_a; event reconsider; event decide_a; event decide_b; event look; event audit_b;
on report_a reveal case_a supports liver_risk;
on report_b reveal case_b supports liver_risk;
on decide_a commit liver_signal because enough using qualified(1, case_a);
on decide_b commit liver_signal because enough using qualified(2, case_b);
on retract_a reveal recheck opposes liver_risk;
on retract_a withdraw case_a because recheck;
on reconsider reopen liver_signal because recheck;
on look set on_basis = latest(liver_signal);
on look set on_grounds = grounds_of(liver_signal);
on look set rests = if(rests_on_withdrawn(liver_signal), 1, 0);
on audit_b reveal audit supports liver_risk;
on audit_b withdraw case_b because audit;
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn snapshot(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()
}

fn outcome(session: &mut ReactiveSession, event: &str) -> Value {
    serde_json::to_value(
        session
            .dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message)),
    )
    .unwrap()
}

fn accept(session: &mut ReactiveSession, event: &str) {
    let outcome = outcome(session, event);
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect()
}

/// A state's value, and the evidence and caveats it carries.
fn state(session: &ReactiveSession, name: &str) -> (f64, Vec<String>, Vec<String>) {
    let shown = snapshot(session);
    let value = shown["values"][name].as_f64().unwrap();
    let grounds = &shown["value_grounds"][name];
    (
        value,
        strings(&grounds["evidence"]),
        strings(&grounds["caveats"]),
    )
}

fn refusal(session: &mut ReactiveSession, event: &str) -> (String, String) {
    let outcome = outcome(session, event);
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    (
        format!(
            "{}/{}",
            outcome["origin"].as_str().unwrap(),
            outcome["code"].as_str().unwrap()
        ),
        outcome["message"].as_str().unwrap().to_string(),
    )
}

fn restored(session: &ReactiveSession) -> ReactiveSession {
    let saved = session.save_json().unwrap();
    let twin = ReactiveSession::restore_json(SIGNAL, &saved).unwrap();
    assert_eq!(twin.save_json().unwrap(), saved);
    assert_eq!(snapshot(&twin), snapshot(session));
    twin
}

#[test]
fn the_value_is_latests_and_the_basis_is_the_revisions_grounds() {
    let mut game = session(SIGNAL);
    for event in ["report_a", "report_b", "decide_a", "look"] {
        accept(&mut game, event);
    }
    // One revision: its grounds are its basis.
    assert_eq!(
        state(&game, "on_basis"),
        (1.0, vec!["case_a".into()], vec![])
    );
    assert_eq!(
        state(&game, "on_grounds"),
        (1.0, vec!["case_a".into()], vec![])
    );
    for event in ["retract_a", "reconsider", "decide_b", "look"] {
        accept(&mut game, event);
    }
    // The second revision's basis holds the first case, withdrawn, through
    // the reopening; its grounds hold the second case alone.
    assert_eq!(
        state(&game, "on_basis"),
        (
            2.0,
            vec!["case_a".into(), "case_b".into(), "recheck".into()],
            vec!["preliminary".into(), "withdrawn".into()]
        )
    );
    assert_eq!(
        state(&game, "on_grounds"),
        (2.0, vec!["case_b".into()], vec!["preliminary".into()])
    );
    assert_eq!(
        state(&game, "rests").0,
        0.0,
        "rests_on_withdrawn reads the grounds too"
    );
    let shown = snapshot(&game);
    assert_eq!(
        shown["value_grounds"]["on_grounds"], shown["commitment_grounds"]["liver_signal@2"],
        "the basis is the revision's frozen grounds, byte for byte"
    );
    restored(&game);
}

#[test]
fn a_withdrawn_ground_reaches_it_exactly_when_rests_on_withdrawn_is_true() {
    let mut game = session(SIGNAL);
    for event in [
        "report_a",
        "report_b",
        "decide_a",
        "retract_a",
        "reconsider",
        "decide_b",
        "audit_b",
        "look",
    ] {
        accept(&mut game, event);
    }
    assert_eq!(
        state(&game, "on_grounds"),
        (
            2.0,
            vec!["case_b".into()],
            vec!["preliminary".into(), "withdrawn".into()]
        )
    );
    assert_eq!(state(&game, "rests").0, 1.0);
    let mut twin = restored(&game);
    accept(&mut twin, "look");
    assert_eq!(state(&twin, "on_grounds"), state(&game, "on_grounds"));
}

#[test]
fn a_skipped_commit_s_selection_guard_is_inherited_as_latest_inherits_it() {
    let source = SIGNAL.replace(
        "on audit_b reveal audit supports liver_risk;",
        "event maybe; event gate_in; evidence gate from \"a gate\";\non gate_in reveal gate;\non maybe when qualified(0, gate) > 0 commit liver_signal because enough using 5;\non audit_b reveal audit supports liver_risk;",
    );
    let mut game = session(&source);
    for event in ["report_a", "decide_a", "gate_in", "maybe", "look"] {
        accept(&mut game, event);
    }
    let shown = snapshot(&game);
    assert_eq!(
        shown["decision_series"]["liver_signal"]["selection_qualifications"]["evidence"],
        serde_json::json!(["gate"])
    );
    assert_eq!(
        state(&game, "on_basis"),
        (1.0, vec!["case_a".into(), "gate".into()], vec![])
    );
    assert_eq!(
        state(&game, "on_grounds"),
        (1.0, vec!["case_a".into(), "gate".into()], vec![])
    );
}

#[test]
fn a_series_without_a_revision_or_a_numeric_value_refuses_as_latest_does() {
    let mut game = session(SIGNAL);
    assert_eq!(
        refusal(&mut game, "look"),
        (
            "evaluation/expression".into(),
            "event look, rule 8: decision series liver_signal has no commitment".into()
        )
    );
    let citing = SIGNAL.replace("using qualified(1, case_a)", "citing qualified(1, case_a)");
    let mut game = session(&citing);
    for event in ["report_a", "decide_a"] {
        accept(&mut game, event);
    }
    let before = snapshot(&game);
    assert_eq!(
        refusal(&mut game, "look"),
        (
            "evaluation/expression".into(),
            "event look, rule 8: current decision liver_signal@1 has no numeric using value".into()
        )
    );
    assert_eq!(snapshot(&game), before, "a refusal changes nothing");
}

#[test]
fn load_rules() {
    for (form, message) in [
        (
            "grounds_of(case_a)",
            "grounds_of names a decision series; case_a is not one",
        ),
        (
            "grounds_of(on_basis)",
            "grounds_of names a decision series; on_basis is not one",
        ),
        // Any other form is a call of the program's own `fn grounds_of`.
        (
            "grounds_of(latest(liver_signal))",
            "unknown source function grounds_of",
        ),
        ("grounds_of()", "unknown source function grounds_of"),
        (
            "grounds_of(liver_signal, 1)",
            "unknown source function grounds_of",
        ),
    ] {
        let source = SIGNAL.replace("grounds_of(liver_signal)", form);
        let error = ReactiveSession::from_source(&source)
            .err()
            .unwrap_or_default();
        assert!(error.starts_with(message), "{form}: {error}");
    }
    let stream = SIGNAL
        .replace(
            "decisions liver_signal limit 4;",
            "decisions liver_signal limit 4;\nreadings cases from case_a limit 4;",
        )
        .replace("grounds_of(liver_signal)", "grounds_of(cases)");
    assert_eq!(
        ReactiveSession::from_source(&stream).err().unwrap(),
        "grounds_of names a decision series; cases is not one"
    );
    // A pure source function cannot read a decision's record.
    let pure = SIGNAL.replace(
        "on look set on_grounds = grounds_of(liver_signal);",
        "fn f(x) = x + grounds_of(liver_signal);\non look set on_grounds = f(1);",
    );
    assert!(ReactiveSession::from_source(&pure)
        .err()
        .unwrap()
        .starts_with("source function f: pure source functions cannot"));
}

#[test]
fn a_program_that_defines_fn_grounds_of_keeps_its_function() {
    let user = SIGNAL.replace(
        "on look set on_grounds = grounds_of(liver_signal);",
        "fn grounds_of(x) = x * 2;\non look set on_grounds = grounds_of(on_basis) + grounds_of(3);",
    );
    let mut game = session(&user);
    for event in ["report_a", "decide_a", "look"] {
        accept(&mut game, event);
    }
    assert_eq!(state(&game, "on_grounds").0, 8.0);
    // The word stays a legal name.
    let named = SIGNAL.replace(
        "state rests = 0;",
        "state rests = 0;\nstate grounds_of = 7;",
    );
    session(&named);
}

/// The AI change review demo's friction item 1 (`findings/takeback-irreversible.cav`):
/// a change accepted unreviewed, reviewed (the caveat taken back and the
/// decision reopened), accepted again, then released irreversibly. The second
/// revision's lineage keeps `unreviewed` through its predecessor (Take-back
/// 0.1, "Kept caveats"), so a release reading `latest(ok)` is refused; one
/// reading `grounds_of(ok)` rests on the reviewed revision's grounds alone.
const CHANGE_REVIEW: &str = r#"
claim sound;
caveat unreviewed consequence material;
evidence account from "the AI's account of a change";
evidence review from "a human review";
decisions ok limit 4;
decisions shipped limit 1 irreversible;
event propose; event accept; event review_it; event release;
on propose reveal account supports sound;
on propose qualify account with unreviewed;
on accept commit ok because enough using qualified(1, account);
on review_it reveal review supports sound;
on review_it unqualify account with unreviewed;
on review_it reopen ok because review;
on release commit shipped because enough using READ;
"#;

#[test]
fn a_reviewed_revision_releases_on_its_grounds_not_its_lineage() {
    let steps = ["propose", "accept", "review_it", "accept"];

    let mut on_lineage = session(&CHANGE_REVIEW.replace("READ", "latest(ok)"));
    for step in steps {
        accept(&mut on_lineage, step);
    }
    let (code, message) = refusal(&mut on_lineage, "release");
    assert_eq!(code, "evaluation/caveated_irreversible");
    assert!(message.contains("unreviewed"), "{message}");
    let view = snapshot(&on_lineage);
    assert_eq!(
        view["commitment_grounds"]["ok@2"]["caveats"],
        serde_json::json!([])
    );
    assert_eq!(
        strings(&view["commitment_bases"]["ok@2"]["provenance"]["caveats"]),
        ["unreviewed"]
    );

    let mut on_grounds = session(&CHANGE_REVIEW.replace("READ", "grounds_of(ok)"));
    for step in steps {
        accept(&mut on_grounds, step);
    }
    accept(&mut on_grounds, "release");
    let view = snapshot(&on_grounds);
    assert_eq!(view["decision_series"]["shipped"]["current"], "shipped@1");
    assert_eq!(
        strings(&view["commitment_grounds"]["shipped@1"]["evidence"]),
        ["account"]
    );
    assert_eq!(
        view["commitment_grounds"]["shipped@1"]["caveats"],
        serde_json::json!([])
    );
    assert_eq!(
        view["commitment_bases"]["shipped@1"]["provenance"]["caveats"],
        serde_json::json!([])
    );
}
