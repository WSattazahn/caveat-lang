//! A binding scoped to histories is evaluated again after a change to the
//! qualification state only if its last evaluation consulted what changed:
//! the relations into a node it searched for caveats, the evidence a record
//! it read lists, a withdrawal it looked up, or a predicate dependency. These
//! programs make each kind of change reach a binding, or pass it by, and
//! check after every event that the session shows what a restored session,
//! which evaluates every binding in full, shows. See
//! spec/caveat-incremental-evaluation-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn shown(session: &ReactiveSession) -> Value {
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    serde_json::json!({
        "bindings": snapshot["bindings"],
        "lineage": snapshot["binding_qualifications"],
        "explanations": snapshot["binding_explanations"],
    })
}

fn same_as_restored(source: &str, session: &ReactiveSession, event: &str) -> Value {
    let live = shown(session);
    let restored = ReactiveSession::restore_json(source, &session.save_json().unwrap())
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    assert_eq!(live, shown(&restored), "after {event}");
    live
}

/// Dispatches each event, checking after each that the session shows what a
/// full evaluation shows. Returns what it shows after each.
fn run(source: &str, events: &[&str]) -> Vec<Value> {
    let mut session = ReactiveSession::from_source(source).unwrap();
    let mut after = Vec::new();
    for event in events {
        let outcome = session
            .dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        let outcome: Value = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
        after.push(same_as_restored(source, &session, event));
    }
    after
}

fn caveats<'a>(shown: &'a Value, target: &str, property: &str) -> Vec<&'a str> {
    shown["lineage"][target][property]["caveats"]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// No `qualifies` relation until `doubt`: the search for the reason's
/// caveats found none to follow, and the first one reaches it.
const FIRST_RELATION: &str = r#"
claim c;
caveat k consequence material;
evidence b from "a second look";
b supports c;
evidence tool from "the tool";
readings s from tool limit 4;
event go;
event retract;
event doubt;
event undoubt;
on go sample s = 1;
on retract withdraw latest(s) because b;
on doubt qualify b with k;
on undoubt unqualify b with k;
bind x.gone = 1 when withdrawn(latest(s));
bind x.count = history_count(s);
"#;

#[test]
fn the_first_qualifies_relation_reaches_a_reason_read_before_it() {
    let after = run(
        FIRST_RELATION,
        &["go", "retract", "doubt", "undoubt", "doubt"],
    );
    assert!(!caveats(&after[1], "x", "gone").contains(&"k"));
    assert!(caveats(&after[2], "x", "gone").contains(&"k"));
    assert!(!caveats(&after[3], "x", "gone").contains(&"k"));
    assert!(caveats(&after[4], "x", "gone").contains(&"k"));
}

/// A doubt about one stream's latest reading, taken back, and a withdrawal.
const STREAMS: &str = r#"
claim ok;
caveat flaky consequence material;
evidence tool from "the tool";
evidence recheck from "a recheck";
recheck supports ok;
readings s from tool limit 4;
readings t from tool limit 4;
event report;
event report_t;
event flag;
event unflag;
event pull;
on report sample s = 1 supports ok;
on report_t sample t = 2 supports ok;
on flag qualify latest(s) with flaky;
on unflag unqualify latest(s) with flaky;
on pull withdraw latest(s) because recheck;
bind v.s = latest(s) when has_sample(s);
bind v.t = latest(t) when has_sample(t);
bind v.n = history_count(s);
"#;

#[test]
fn a_doubt_its_take_back_and_a_withdrawal_reach_the_reading_that_lists_it() {
    let after = run(
        STREAMS,
        &[
            "report", "report_t", "flag", "unflag", "flag", "pull", "report",
        ],
    );
    assert!(caveats(&after[2], "v", "s").contains(&"flaky"));
    assert!(!caveats(&after[2], "v", "t").contains(&"flaky"));
    assert!(!caveats(&after[3], "v", "s").contains(&"flaky"));
    assert!(caveats(&after[4], "v", "s").contains(&"flaky"));
    assert!(caveats(&after[5], "v", "s").contains(&"withdrawn"));
    assert!(!caveats(&after[5], "v", "t").contains(&"withdrawn"));
    // The next reading carries neither.
    assert!(!caveats(&after[6], "v", "s").contains(&"withdrawn"));
}

/// A reopening skipped by its guard is a dependency of `reopened(d)`, and
/// one that later runs changes it.
const SKIPPED_REOPENING: &str = r#"
claim ok;
evidence a from "what the decision used";
a supports ok;
evidence r from "a recheck";
r supports ok;
decisions d limit 4;
state armed = 0;
event decide;
event arm;
event check;
on decide commit d because enough using qualified(1, a);
on arm set armed = 1;
on check when armed == 1 reopen d because r;
bind x.r = "open";
bind x.r = "reopened" when reopened(d);
bind x.c = latest(d) when committed(d);
"#;

#[test]
fn a_reopening_skipped_by_its_guard_and_then_run() {
    run(
        SKIPPED_REOPENING,
        &["decide", "check", "arm", "check", "decide"],
    );
}

#[test]
fn a_refused_event_keeps_what_each_binding_consulted() {
    let source = format!("{STREAMS}on unflag when history_count(s) > 0 reject \"no\";\n");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for event in ["report", "flag"] {
        session.dispatch_outcome_json(event, "{}").unwrap();
    }
    let before = shown(&session);
    let refused =
        serde_json::to_value(session.dispatch_outcome_json("unflag", "{}").unwrap()).unwrap();
    assert_eq!(refused["outcome"], "rejected");
    assert_eq!(shown(&session), before);
    // Later events are judged against what the bindings consulted before
    // the refusal.
    for event in ["pull", "report", "flag"] {
        session.dispatch_outcome_json(event, "{}").unwrap();
        same_as_restored(&source, &session, event);
    }
}
