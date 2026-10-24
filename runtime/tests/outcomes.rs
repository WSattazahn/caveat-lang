//! Outcomes 0.1: `resolve D = VALUE because E [materialized C, …]`,
//! `resolved(…)` and `outcome(…)`.

use caveat_runtime::reactive::ReactiveSession;

/// A pilot whose premortem named thin demand: the outcome is recorded on the
/// revision, and says the named risk came true.
const PILOT: &str = "claim pilot_working;
evidence plan_review from \"planning review\";
evidence usage_feed from \"weekly active users\";
evidence correction from \"the analytics team's correction\";
caveat thin_demand consequence material;
caveat late_launch consequence low;
thin_demand qualifies plan_review;
readings usage from usage_feed limit 52;
decisions pilot limit 4;
event plan score min 0 max 10;
event weekly users min 0 max 100000;
event close users min 0 max 100000;
event late k min 1 max 4, users min 0 max 100000;
event miscounted;
event late_risk;
on plan reveal plan_review supports pilot_working;
on plan commit pilot because enough using qualified(score, plan_review);
on weekly sample usage = users;
on weekly when committed(pilot) and not reopened(pilot) and users < 20 reopen pilot because latest(usage);
on close sample usage = users;
on close when users < 20 resolve pilot = users because latest(usage) materialized thin_demand;
on close when users >= 20 resolve pilot = users because latest(usage);
on late sample usage = users;
on late resolve revision(pilot, k) = users because latest(usage);
on late_risk resolve pilot = 0 because plan_review materialized late_launch;
on miscounted reveal correction;
on miscounted withdraw latest(usage) because correction;
bind pilot.resolved = resolved(pilot);
bind pilot.outcome = outcome(pilot) when resolved(pilot);
bind first.resolved = resolved(revision(pilot, 1));
";

fn run(session: &mut ReactiveSession, events: &[(&str, &str)]) {
    for (event, payload) in events {
        session
            .dispatch_json(event, payload)
            .unwrap_or_else(|error| panic!("{event} {payload}: {error}"));
    }
}

fn shown(session: &ReactiveSession, target: &str, property: &str) -> String {
    let snapshot = session.snapshot();
    match snapshot
        .bindings
        .get(target)
        .and_then(|properties| properties.get(property))
    {
        Some(value) => serde_json::to_string(value).unwrap(),
        None => "null".into(),
    }
}

fn refusal(session: &mut ReactiveSession, event: &str, payload: &str) -> String {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .expect("not fatal");
    let outcome = serde_json::to_value(&outcome).unwrap();
    assert_eq!(outcome["outcome"], "rejected", "{outcome}");
    format!(
        "{}/{}: {}",
        outcome["origin"].as_str().unwrap(),
        outcome["code"].as_str().unwrap(),
        outcome["message"].as_str().unwrap()
    )
}

#[test]
fn an_outcome_is_recorded_on_the_revision_and_changes_nothing_else() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    run(
        &mut session,
        &[("plan", r#"{"score":7}"#), ("weekly", r#"{"users":12}"#)],
    );
    let before = session.snapshot();
    run(&mut session, &[("close", r#"{"users":15}"#)]);
    let after = session.snapshot();
    assert_eq!(before.commitment_bases, after.commitment_bases);
    assert_eq!(before.commitment_grounds, after.commitment_grounds);
    assert_eq!(before.commitments, after.commitments);
    let revision = &after.decision_series["pilot"].revisions[0];
    let outcome = revision.outcome.as_ref().expect("an outcome");
    assert_eq!(outcome.value, 15.0);
    assert_eq!(outcome.because, ["usage@2"]);
    assert_eq!(outcome.materialized, ["thin_demand"]);
    let entry = after.decision_journal.last().unwrap();
    assert_eq!(entry.change, "resolved");
    assert_eq!(entry.commitment, "pilot@1");
    assert_eq!(entry.outcome, Some(15.0));
    assert_eq!(entry.value, Some(7.0));
    assert_eq!(shown(&session, "pilot", "resolved"), "true");
    assert_eq!(shown(&session, "pilot", "outcome"), "15.0");
    assert_eq!(shown(&session, "first", "resolved"), "true");
}

#[test]
fn a_second_resolve_is_refused_until_its_evidence_is_withdrawn() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    run(
        &mut session,
        &[("plan", r#"{"score":7}"#), ("close", r#"{"users":15}"#)],
    );
    assert_eq!(
        refusal(&mut session, "close", r#"{"users":30}"#),
        "evaluation/already_resolved: event close, rule 7: pilot@1 is already resolved at #2; withdraw its evidence to correct it"
    );
    run(&mut session, &[("miscounted", "{}")]);
    // The withdrawn outcome still stands, and carries the withdrawal.
    assert_eq!(shown(&session, "pilot", "resolved"), "true");
    run(&mut session, &[("close", r#"{"users":30}"#)]);
    let snapshot = session.snapshot();
    let outcome = snapshot.decision_series["pilot"].revisions[0]
        .outcome
        .as_ref()
        .unwrap();
    assert_eq!(outcome.value, 30.0);
    assert_eq!(outcome.because, ["usage@2"]);
    let changes = snapshot
        .decision_journal
        .iter()
        .filter(|entry| entry.change == "resolved")
        .count();
    assert_eq!(changes, 2);
}

#[test]
fn resolving_needs_a_revision_and_a_retained_caveat() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    assert_eq!(
        refusal(&mut session, "close", r#"{"users":3}"#),
        "evaluation/not_committed: event close, rule 6: cannot resolve pilot: it has no revision"
    );
    run(&mut session, &[("plan", r#"{"score":7}"#)]);
    assert_eq!(
        refusal(&mut session, "late_risk", "{}"),
        "evaluation/not_retained: event late_risk, rule 10: pilot@1 does not retain late_launch, so it cannot have materialized"
    );
    assert_eq!(
        refusal(&mut session, "late", r#"{"k":2,"users":3}"#),
        "evaluation/not_committed: event late, rule 9: cannot resolve pilot: it has no revision 2"
    );
}

#[test]
fn an_outcome_may_arrive_after_the_next_revision() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    run(
        &mut session,
        &[
            ("plan", r#"{"score":7}"#),
            ("weekly", r#"{"users":12}"#),
            ("plan", r#"{"score":4}"#),
            ("late", r#"{"k":1,"users":40}"#),
        ],
    );
    let snapshot = session.snapshot();
    let revisions = &snapshot.decision_series["pilot"].revisions;
    assert_eq!(revisions[0].outcome.as_ref().unwrap().value, 40.0);
    assert!(revisions[1].outcome.is_none());
    assert_eq!(shown(&session, "pilot", "resolved"), "false");
    assert_eq!(shown(&session, "first", "resolved"), "true");
}

#[test]
fn reading_an_outcome_that_is_not_there_refuses_the_event() {
    let source = "claim c; evidence e from \"e\"; decisions d limit 2; state x = 0;
event go; event read;
on go reveal e supports c;
on go commit d because enough using 1;
on read set x = outcome(d);";
    let mut session = ReactiveSession::from_source(source).expect("loads");
    run(&mut session, &[("go", "{}")]);
    assert_eq!(
        refusal(&mut session, "read", "{}"),
        "evaluation/expression: event read, rule 3: outcome(d): d@1 has no outcome"
    );
}

#[test]
fn the_new_words_do_not_load_on_what_is_not_a_series() {
    for (source, message) in [
        (
            "claim c; evidence e from \"e\"; event go; on go reveal e supports c; on go commit go_ahead because enough; on go resolve go_ahead = 1 because e;",
            "resolve names a decision series; go_ahead is not one",
        ),
        (
            "claim c; evidence e from \"e\"; state x = 0; event go; on go when resolved(x) set x = 1;",
            "resolved names a decision series; x is not one",
        ),
        (
            "claim c; evidence e from \"e\"; state x = 0; event go; on go set x = outcome(x);",
            "outcome names a decision series; x is not one",
        ),
        (
            "claim c; evidence e from \"e\"; decisions d limit 2; event go; on go resolve d = 1 because e materialized nothing;",
            "unknown caveat nothing",
        ),
        (
            "claim c; evidence e from \"e\"; caveat k consequence low; decisions d limit 2; event go; on go resolve d = 1 because e materialized k, k;",
            "duplicate materialized caveat k",
        ),
    ] {
        let error = ReactiveSession::from_source(source).expect_err(source);
        assert!(error.contains(message), "{source}: {error}");
    }
}

#[test]
fn a_program_keeps_its_own_resolve_outcome_and_resolved_names() {
    let source = "claim c; evidence e from \"e\";
state outcome = 0 min 0 max 9;
state resolve = 0 min 0 max 9;
fn resolved(x) = x + 1;
event go n min 0 max 5;
on go set outcome = resolved(n);
on go set resolve = outcome;
bind out.value = outcome;";
    let mut session = ReactiveSession::from_source(source).expect("loads");
    run(&mut session, &[("go", r#"{"n":2}"#)]);
    assert_eq!(shown(&session, "out", "value"), "3.0");
}

#[test]
fn a_save_with_an_outcome_restores_and_one_without_is_unchanged() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    run(&mut session, &[("plan", r#"{"score":7}"#)]);
    let plain = session.save_json().unwrap();
    assert!(!plain.contains("outcome"), "{plain}");
    run(&mut session, &[("close", r#"{"users":15}"#)]);
    let saved = session.save_json().unwrap();
    assert!(saved.contains("\"outcome\""), "{saved}");
    let restored = ReactiveSession::restore_json(PILOT, &saved).expect("restores");
    assert_eq!(restored.snapshot(), session.snapshot());
}

#[test]
fn restore_refuses_an_outcome_no_source_can_make() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    run(
        &mut session,
        &[("plan", r#"{"score":7}"#), ("close", r#"{"users":15}"#)],
    );
    let saved: serde_json::Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    let forge = |edit: &dyn Fn(&mut serde_json::Value)| {
        let mut forged = saved.clone();
        edit(&mut forged);
        ReactiveSession::restore_json(PILOT, &forged.to_string())
    };
    let outcome = |save: &mut serde_json::Value| -> serde_json::Value {
        save["decision_series"]["pilot"]["revisions"][0]["outcome"].take()
    };
    // A materialized caveat the revision does not retain.
    let error = forge(&|save| {
        let mut forged = outcome(save);
        forged["materialized"] = serde_json::json!(["late_launch"]);
        save["decision_series"]["pilot"]["revisions"][0]["outcome"] = forged;
    })
    .expect_err("late_launch is not retained");
    assert!(error.contains("materialized late_launch"), "{error}");
    // Evidence named twice.
    let error = forge(&|save| {
        let mut forged = outcome(save);
        forged["because"] = serde_json::json!(["usage@1", "usage@1"]);
        save["decision_series"]["pilot"]["revisions"][0]["outcome"] = forged;
    })
    .expect_err("cites usage@1 twice");
    assert!(error.contains("twice"), "{error}");
    // An outcome from an event that cannot resolve.
    let error = forge(&|save| {
        let mut forged = outcome(save);
        forged["event"] = serde_json::json!("plan");
        save["decision_series"]["pilot"]["revisions"][0]["outcome"] = forged;
    })
    .expect_err("plan cannot resolve");
    assert!(error.contains("no event of the source"), "{error}");
    // A program with no `resolve` cannot hold an outcome.
    let plain = PILOT
        .lines()
        .filter(|line| !line.contains("resolve"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut other = ReactiveSession::from_source(&plain).expect("loads");
    run(&mut other, &[("plan", r#"{"score":7}"#)]);
    let mut save: serde_json::Value = serde_json::from_str(&other.save_json().unwrap()).unwrap();
    save["decision_series"]["pilot"]["revisions"][0]["outcome"] =
        saved["decision_series"]["pilot"]["revisions"][0]["outcome"].clone();
    assert!(ReactiveSession::restore_json(&plain, &save.to_string()).is_err());
}

#[test]
fn a_withdrawn_outcome_is_read_with_the_withdrawal() {
    let mut session = ReactiveSession::from_source(PILOT).expect("loads");
    run(
        &mut session,
        &[
            ("plan", r#"{"score":7}"#),
            ("close", r#"{"users":15}"#),
            ("miscounted", "{}"),
        ],
    );
    let snapshot = session.snapshot();
    let lineage = &snapshot.binding_qualifications["pilot"]["outcome"];
    assert!(lineage.caveats.contains("withdrawn"), "{lineage:?}");
    assert!(lineage.evidence.contains("usage@1"), "{lineage:?}");
}

/// The spec's calibration example: each error rests on the forecast it
/// scores and the run that resolved it.
#[test]
fn a_forecast_pairs_with_its_outcome() {
    let source = "claim canary_passes;
evidence forecaster from \"release agent\";
evidence canary from \"canary monitor\";
evidence scorer from \"Brier score of a resolved forecast\";
readings canary_runs from canary limit 64;
readings errors from scorer limit 64;
decisions forecast limit 64;
fn add(total, record) = total + record;
event predict p min 0 max 1;
event report passed min 0 max 1;
on predict when committed(forecast) and not reopened(forecast) reopen forecast because forecaster;
on predict reveal forecaster supports canary_passes;
on predict commit forecast because enough using qualified(p, forecaster);
on report sample canary_runs = passed;
on report resolve forecast = passed because latest(canary_runs);
on report sample errors = (latest(forecast) - outcome(forecast)) * (latest(forecast) - outcome(forecast));
bind record.brier = fold_history(errors, 0, add) / history_count(errors) when has_sample(errors);
";
    let mut session = ReactiveSession::from_source(source).expect("loads");
    run(
        &mut session,
        &[
            ("predict", r#"{"p":0.9}"#),
            ("report", r#"{"passed":0}"#),
            ("predict", r#"{"p":0.8}"#),
            ("report", r#"{"passed":1}"#),
        ],
    );
    let snapshot = session.snapshot();
    let brier: f64 = serde_json::from_value(
        serde_json::to_value(&snapshot.bindings["record"]["brier"]).unwrap(),
    )
    .unwrap();
    assert!((brier - 0.425).abs() < 1e-12, "{brier}");
    let errors = &snapshot.reading_streams["errors"].occurrences;
    assert!(errors[1].provenance.evidence.contains("canary_runs@2"));
    let revisions = &snapshot.decision_series["forecast"].revisions;
    assert_eq!(revisions[0].outcome.as_ref().unwrap().value, 0.0);
    assert_eq!(revisions[1].outcome.as_ref().unwrap().value, 1.0);
}

#[test]
fn a_procedure_resolves_and_a_window_keeps_the_outcome_evidence() {
    let source = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
readings runs from feed window 2;
decisions d limit 4;
proc close(v) {
    sample runs = v;
    resolve d = v because latest(runs);
};
event go;
event done v min 0 max 9;
event reading v min 0 max 9;
on go reveal e supports c;
on go commit d because enough using 1;
on done call close(v);
on reading sample runs = v;
bind d.outcome = outcome(d) when resolved(d);
";
    let mut session = ReactiveSession::from_source(source).expect("loads");
    run(
        &mut session,
        &[
            ("go", "{}"),
            ("done", r#"{"v":4}"#),
            ("reading", r#"{"v":1}"#),
            ("reading", r#"{"v":2}"#),
            ("reading", r#"{"v":3}"#),
        ],
    );
    let snapshot = session.snapshot();
    assert_eq!(
        serde_json::to_string(&snapshot.bindings["d"]["outcome"]).unwrap(),
        "4.0"
    );
    let saved = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(source, &saved).expect("restores");
    assert_eq!(restored.snapshot(), snapshot);
}
