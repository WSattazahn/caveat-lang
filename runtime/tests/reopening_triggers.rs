//! Declared reopening triggers. See spec/caveat-reopening-triggers-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const COMMON: &str = r#"
claim ready; claim changed;
evidence ci from "the checks";
evidence push from "a new commit";
readings checks from ci limit 32;
readings pushes from push limit 32;
event pushed; event check ok min 0 max 1; event decide; event push_twice;
event push_decide_push; event proc_push; event push_then_refuse;
event check_other;
on decide commit merge because enough;
"#;

/// The declaration.
const DECLARED: &str = r#"
decisions merge limit 8 reopened by pushes, checks opposing ready;
proc record() { sample pushes = 3 supports changed; };
on pushed sample pushes = 1 supports changed;
on check when ok == 1 sample checks = 1 supports ready;
on check when ok == 0 sample checks = 0 opposes ready;
on check_other sample checks = 0 opposes changed;
on push_twice sample pushes = 1 supports changed;
on push_twice sample pushes = 2 supports changed;
on push_decide_push sample pushes = 1 supports changed;
on push_decide_push commit merge because enough;
on push_decide_push sample pushes = 2 supports changed;
on proc_push call record();
on push_then_refuse sample pushes = 1 supports changed;
on push_then_refuse reject "not now";
"#;

/// The same policy, hand-written: a guarded reopening right after each
/// matching sample.
const WRITTEN: &str = r#"
decisions merge limit 8;
proc record() {
    sample pushes = 3 supports changed;
    when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
};
on pushed sample pushes = 1 supports changed;
on pushed when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
on check when ok == 1 sample checks = 1 supports ready;
on check when ok == 0 sample checks = 0 opposes ready;
on check when ok == 0 and committed(merge) and not reopened(merge) reopen merge because latest(checks);
on check_other sample checks = 0 opposes changed;
on push_twice sample pushes = 1 supports changed;
on push_twice when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
on push_twice sample pushes = 2 supports changed;
on push_twice when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
on push_decide_push sample pushes = 1 supports changed;
on push_decide_push when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
on push_decide_push commit merge because enough;
on push_decide_push sample pushes = 2 supports changed;
on push_decide_push when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
on proc_push call record();
on push_then_refuse sample pushes = 1 supports changed;
on push_then_refuse when committed(merge) and not reopened(merge) reopen merge because latest(pushes);
on push_then_refuse reject "not now";
"#;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn send(game: &mut ReactiveSession, event: &str, payload: Value) -> Value {
    serde_json::to_value(
        game.dispatch_outcome_json(event, &payload.to_string())
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn snapshot(game: &ReactiveSession) -> Value {
    let mut snapshot = serde_json::to_value(game.snapshot()).unwrap();
    snapshot.as_object_mut().unwrap().remove("source_id");
    snapshot
}

fn pair() -> (ReactiveSession, ReactiveSession) {
    (
        session(&format!("{COMMON}{DECLARED}")),
        session(&format!("{COMMON}{WRITTEN}")),
    )
}

/// Sends the same events to both and requires identical snapshots after each.
fn same(events: &[(&str, Value)]) -> Value {
    let (mut declared, mut written) = pair();
    for (event, payload) in events {
        let one = send(&mut declared, event, payload.clone());
        let two = send(&mut written, event, payload.clone());
        assert_eq!(one["outcome"], two["outcome"], "{event}");
        assert_eq!(snapshot(&declared), snapshot(&written), "after {event}");
    }
    snapshot(&declared)
}

fn changes(shot: &Value) -> Vec<(String, String, Value)> {
    shot["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["commitment"].as_str().unwrap().to_string(),
                entry["change"].as_str().unwrap().to_string(),
                entry["because"].clone(),
            )
        })
        .collect()
}

fn e(event: &str) -> (&str, Value) {
    (event, json!({}))
}

#[test]
fn a_reading_before_the_first_commitment_does_not_reopen_it() {
    let shot = same(&[e("pushed"), e("decide")]);
    assert_eq!(
        changes(&shot),
        vec![("merge@1".into(), "committed".into(), json!([]))]
    );
}

#[test]
fn a_reading_after_a_commitment_reopens_it_because_of_that_reading() {
    let shot = same(&[e("decide"), e("pushed")]);
    assert_eq!(
        changes(&shot)[1],
        ("merge@1".into(), "reopened".into(), json!(["pushes@1"]))
    );
    assert!(shot["relations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["from"] == "pushes@1" && r["relation"] == "reopens" && r["to"] == "merge@1"));
}

#[test]
fn a_second_reading_finds_the_revision_already_open() {
    let shot = same(&[e("decide"), e("push_twice")]);
    assert_eq!(changes(&shot).len(), 2);
}

#[test]
fn each_reading_reopens_the_revision_in_force_when_it_was_taken() {
    let shot = same(&[e("push_decide_push"), e("push_decide_push")]);
    assert_eq!(
        changes(&shot)
            .iter()
            .map(|(id, change, _)| format!("{id} {change}"))
            .collect::<Vec<_>>(),
        [
            "merge@1 committed",
            "merge@1 reopened",
            "merge@2 committed",
            "merge@2 reopened"
        ]
    );
}

#[test]
fn a_filter_matches_only_its_relation_and_claim() {
    // `checks opposing ready`, three readings on the one watched stream.
    let decide = e("decide");
    let supports_ready = ("check", json!({ "ok": 1 }));
    let opposes_changed = e("check_other");
    let opposes_ready = ("check", json!({ "ok": 0 }));

    let shot = same(&[decide.clone(), supports_ready.clone()]);
    assert_eq!(changes(&shot).len(), 1, "the wrong relation reopened it");
    let shot = same(&[
        decide.clone(),
        supports_ready.clone(),
        opposes_changed.clone(),
    ]);
    assert_eq!(changes(&shot).len(), 1, "the wrong claim reopened it");
    let shot = same(&[decide, supports_ready, opposes_changed, opposes_ready]);
    assert_eq!(
        changes(&shot)[1],
        ("merge@1".into(), "reopened".into(), json!(["checks@3"]))
    );
}

#[test]
fn a_reading_taken_inside_a_procedure_triggers_too() {
    let shot = same(&[e("decide"), e("proc_push"), e("decide"), e("proc_push")]);
    assert_eq!(changes(&shot).len(), 4);
}

#[test]
fn a_refused_event_rolls_back_the_reading_and_the_reopening_it_triggered() {
    let (mut declared, _) = pair();
    send(&mut declared, "decide", json!({}));
    let before = (snapshot(&declared), declared.save_json().unwrap());
    let outcome = send(&mut declared, "push_then_refuse", json!({}));
    assert_eq!(outcome["outcome"], "rejected");
    assert_eq!((snapshot(&declared), declared.save_json().unwrap()), before);
}

/// An event that spends one step calling a procedure of `fill` steps and one
/// sampling a watched stream; the triggered step comes next. The work limit
/// is 4096 steps.
fn heavy(fill: usize) -> String {
    format!(
        "state n = 0; event heavy; proc fill() {{ {} }};\n\
         on heavy call fill();\n\
         on heavy sample pushes = 1 supports changed;\n",
        "set n = 1;".repeat(fill)
    )
}

const HEAVY_WRITTEN: &str =
    "on heavy when committed(merge) and not reopened(merge) reopen merge because latest(pushes);\n";

/// Sends `before`, then `heavy`, to the declared and hand-written programs.
/// Both must give the same outcome and snapshot, and a refused `heavy` must
/// leave each exactly as it was. Returns the outcome and the snapshot.
fn at_the_boundary(fill: usize, before: &[&str]) -> (Value, Value) {
    let mut declared = session(&format!("{COMMON}{DECLARED}{}", heavy(fill)));
    let mut written = session(&format!("{COMMON}{WRITTEN}{}{HEAVY_WRITTEN}", heavy(fill)));
    for event in before {
        send(&mut declared, event, json!({}));
        send(&mut written, event, json!({}));
    }
    let prior = [&declared, &written].map(|game| (snapshot(game), game.save_json().unwrap()));
    let one = send(&mut declared, "heavy", json!({}));
    let two = send(&mut written, "heavy", json!({}));
    for field in ["outcome", "origin", "code"] {
        assert_eq!(one[field], two[field], "{field} after {before:?}");
    }
    assert_eq!(snapshot(&declared), snapshot(&written), "after {before:?}");
    if one["outcome"] == "rejected" {
        let after = [&declared, &written].map(|game| (snapshot(game), game.save_json().unwrap()));
        assert_eq!(
            after, prior,
            "a refused event left changes after {before:?}"
        );
    }
    (one, snapshot(&declared))
}

#[test]
fn a_triggered_step_costs_the_event_budget_what_the_written_step_does() {
    // Not yet committed, committed, and already reopened: the guarded step
    // costs one step whether or not it reopens anything.
    for (before, journal) in [
        (&[][..], 0),
        (&["decide"][..], 2),
        (&["decide", "pushed"][..], 2),
    ] {
        let (outcome, _) = at_the_boundary(4094, before);
        assert_eq!(
            [&outcome["outcome"], &outcome["origin"], &outcome["code"]],
            [&json!("rejected"), &json!("limit"), &json!("work_limit")],
            "after {before:?}"
        );
        let (outcome, shot) = at_the_boundary(4093, before);
        assert_eq!(outcome["outcome"], "accepted", "after {before:?}");
        assert_eq!(changes(&shot).len(), journal, "after {before:?}");
    }
    let (_, shot) = at_the_boundary(4093, &["decide"]);
    assert_eq!(
        changes(&shot)[1],
        ("merge@1".into(), "reopened".into(), json!(["pushes@1"]))
    );

    // Control: without the trigger the same event fits, so the refusal above
    // is the triggered step's charge.
    let mut control = session(&format!(
        "{COMMON}decisions merge limit 8;\n{}",
        heavy(4094)
    ));
    send(&mut control, "decide", json!({}));
    assert_eq!(
        send(&mut control, "heavy", json!({}))["outcome"],
        "accepted"
    );
}

#[test]
fn several_series_watching_one_stream_reopen_in_name_order() {
    let source = format!(
        "{COMMON}decisions zeta limit 2 reopened by pushes;\ndecisions merge limit 2 reopened by pushes, pushes;\ndecisions alpha limit 2 reopened by pushes;\non decide commit zeta because enough;\non decide commit alpha because enough;\non pushed sample pushes = 1 supports changed;"
    );
    let mut game = session(&source);
    send(&mut game, "decide", json!({}));
    send(&mut game, "pushed", json!({}));
    let reopened = changes(&snapshot(&game))
        .into_iter()
        .filter(|(_, change, _)| change == "reopened")
        .map(|(id, _, _)| id)
        .collect::<Vec<_>>();
    // Name order; `merge`, listed twice, reopens once.
    assert_eq!(reopened, ["alpha@1", "merge@1", "zeta@1"]);
}

#[test]
fn a_triggered_reopening_saves_and_restores_and_an_impossible_one_is_refused() {
    let source = format!("{COMMON}{DECLARED}");
    let mut game = session(&source);
    for event in ["decide", "pushed", "decide", "proc_push"] {
        send(&mut game, event, json!({}));
    }
    let saved = game.save_json().unwrap();
    let mut restored = ReactiveSession::restore_json(&source, &saved).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&game));
    for session in [&mut game, &mut restored] {
        send(session, "decide", json!({}));
        send(session, "check", json!({ "ok": 0 }));
    }
    assert_eq!(snapshot(&restored), snapshot(&game));

    // A reopening recorded against an event that samples nothing it watches.
    let mut tampered: Value = serde_json::from_str(&saved).unwrap();
    let entry = tampered["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "reopened")
        .unwrap();
    assert_eq!(entry["event"], "pushed");
    entry["event"] = json!("decide");
    let error = ReactiveSession::restore_json(&source, &tampered.to_string())
        .err()
        .unwrap();
    assert!(
        error.contains("event cannot make this decision change"),
        "{error}"
    );
}

#[test]
fn declarations_are_checked_when_the_program_loads() {
    for (declaration, expected) in [
        (
            "decisions merge limit 2 reopened by nowhere;",
            "nowhere must name a declared reading stream",
        ),
        (
            "decisions merge limit 2 reopened by checks opposing nowhere;",
            "nowhere must name a declared claim",
        ),
        (
            "decisions merge limit 2 reopened by checks sideways ready;",
            "reopened by takes STREAM",
        ),
        (
            "decisions merge limit 2 reopened by;",
            "decisions expects NAME limit CAPACITY [reopened by",
        ),
        (
            "decisions merge limit 2 reopened checks;",
            "decisions expects NAME limit CAPACITY [reopened by",
        ),
    ] {
        let error = ReactiveSession::from_source(&format!("{COMMON}{declaration}"))
            .err()
            .unwrap();
        assert!(error.contains(expected), "{declaration}: {error}");
    }
}
