//! D022: a value whose lineage would hold more than 1,024 identifiers ended
//! the session as fatal `unclassified` on 1.0.0 to 1.3.0, although the event
//! had been rolled back. It is refused as `limit/provenance_limit`, with the
//! same text, and the session goes on. See spec/caveat-dispatch-0.1.md
//! ("Changes") and `MAX_PROVENANCE_IDENTIFIERS` in `reactive_expr.rs`.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// Two renewables grow a state's lineage by two occurrences per step, so the
/// 513th step would give it 1,026 identifiers.
const GROW: &str = r#"
claim c;
evidence a from "a source";
evidence b from "b source";
renewable a limit 1024;
renewable b limit 1024;
state acc = 0 min 0 max 1000000;
event step;
event check;
on step renew a;
on step renew b;
on step reveal a supports c;
on step reveal b supports c;
on step set acc = acc + qualified(1, a) + qualified(1, b);
on check set acc = acc;
"#;

const MESSAGE: &str = "event step, rule 5: value provenance exceeds limit 1024 identifiers";

fn outcome(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
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
fn d022_a_lineage_past_the_provenance_limit_is_refused_and_the_session_goes_on() {
    let mut session = ReactiveSession::from_source(GROW).unwrap();
    for step in 1..=512 {
        let accepted = outcome(&mut session, "step");
        assert_eq!(accepted["outcome"], "accepted", "step {step}");
    }
    let before = checkpoint(&session);
    assert_eq!(
        before.0["qualified_values"]["acc"]["provenance"]["evidence"]
            .as_array()
            .unwrap()
            .len(),
        1024
    );
    let refused = outcome(&mut session, "step");
    assert_eq!(refused["outcome"], "rejected");
    assert_eq!(refused["origin"], "limit");
    assert_eq!(refused["code"], "provenance_limit");
    assert_eq!(refused["message"], MESSAGE);
    // Rolled back: no renewal, no observation, no new value.
    assert_eq!(
        checkpoint(&session),
        before,
        "the refused step changed nothing"
    );
    // The session is usable: another event is accepted, and the save restores.
    assert_eq!(outcome(&mut session, "check")["outcome"], "accepted");
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(GROW, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(checkpoint(&restored), checkpoint(&session));
    // The refusal is the same every time the step is tried.
    let again = outcome(&mut session, "step");
    assert_eq!(again["code"], "provenance_limit");
    assert_eq!(again["message"], MESSAGE);
}

#[test]
fn d022_the_legacy_entry_point_keeps_the_text() {
    let mut session = ReactiveSession::from_source(GROW).unwrap();
    for _ in 0..512 {
        session.dispatch_json("step", "{}").unwrap();
    }
    assert_eq!(session.dispatch_json("step", "{}").unwrap_err(), MESSAGE);
    session.dispatch_json("check", "{}").unwrap();
}
