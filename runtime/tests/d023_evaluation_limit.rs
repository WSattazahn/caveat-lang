//! D023: an expression whose evaluation visits more than 65,536 nodes ended
//! the session as fatal `unclassified` on 1.0.0 to 1.3.0, although the event
//! had been rolled back. A `fold_history` over a long stream with a heavy
//! step reaches it on an ordinary program. It is refused as
//! `limit/evaluation_limit`, with the same text, and the session goes on.
//! See spec/caveat-dispatch-0.1.md ("Changes") and `MAX_EVALUATED_NODES` in
//! `reactive_expr.rs`.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// Each reading costs the fold about 670 nodes, so the 99th read's fold
/// passes 65,536.
const FOLD: &str = r#"
claim c;
evidence probe from "a probe";
readings temps from probe limit 256;
fn f(r) = r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r + r;
fn g(r) = f(r) + f(r) + f(r) + f(r) + f(r) + f(r) + f(r) + f(r);
fn heavy(total, reading) = total + g(reading);
state s = 0 min -1000000000000 max 1000000000000;
state count = 0 min 0 max 1000;
event read v min 0 max 1;
event tally;
on read sample temps = v supports c;
on read set s = fold_history(temps, 0, heavy);
on tally set count = count + 1;
"#;

const MESSAGE: &str = "event read, rule 2: expression exceeds evaluation limit 65536 nodes";

fn outcome(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

fn checkpoint(session: &ReactiveSession) -> (Value, String) {
    (
        serde_json::to_value(session.snapshot()).unwrap(),
        session.save_json().unwrap(),
    )
}

#[test]
fn d023_an_evaluation_past_the_node_limit_is_refused_and_the_session_goes_on() {
    let mut session = ReactiveSession::from_source(FOLD).unwrap();
    for event in 1..=98 {
        let accepted = outcome(&mut session, "read", r#"{"v":1}"#);
        assert_eq!(accepted["outcome"], "accepted", "read {event}");
    }
    let before = checkpoint(&session);
    let refused = outcome(&mut session, "read", r#"{"v":1}"#);
    assert_eq!(refused["outcome"], "rejected");
    assert_eq!(refused["origin"], "limit");
    assert_eq!(refused["code"], "evaluation_limit");
    assert_eq!(refused["message"], MESSAGE);
    // Rolled back: the sample of rule 1 is gone with the set of rule 2.
    assert_eq!(
        checkpoint(&session),
        before,
        "the refused read changed nothing"
    );
    assert_eq!(
        before.0["reading_streams"]["temps"]["occurrences"]
            .as_array()
            .map_or_else(
                || before.0["histories"]["temps"].as_array().map(Vec::len),
                |records| Some(records.len())
            ),
        Some(98)
    );
    // The session is usable: another event is accepted, and the save restores.
    assert_eq!(outcome(&mut session, "tally", "{}")["outcome"], "accepted");
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(FOLD, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(checkpoint(&restored), checkpoint(&session));
    let again = outcome(&mut session, "read", r#"{"v":0}"#);
    assert_eq!(again["code"], "evaluation_limit");
    assert_eq!(again["message"], MESSAGE);
}

#[test]
fn d023_the_legacy_entry_point_keeps_the_text() {
    let mut session = ReactiveSession::from_source(FOLD).unwrap();
    for _ in 0..98 {
        session.dispatch_json("read", r#"{"v":1}"#).unwrap();
    }
    assert_eq!(
        session.dispatch_json("read", r#"{"v":1}"#).unwrap_err(),
        MESSAGE
    );
    session.dispatch_json("tally", "{}").unwrap();
}
