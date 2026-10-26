//! D025: a text value past 65,536 bytes ended the session as fatal
//! `unclassified` on 1.0.0 to 1.4.0, although the event had been rolled
//! back. It is refused as `limit/text_limit`, with the same text, and the
//! session goes on. See spec/caveat-dispatch-0.1.md ("Changes") and
//! `MAX_TEXT_BYTES` in `reactive_expr.rs`.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// A 65,520-byte literal plus a number's text: 7 bytes of `big` keep the
/// binding at 65,527; 17 bytes take it past 65,536. The expression's own
/// source stays within its limit.
fn program(rule: &str) -> String {
    let wide = "A".repeat(65_520);
    format!(
        "state output = 0; event run; event check; on run set output = 1;\n\
         state big = 0 min 0 max 2000000;\n\
         {rule} set big = 1234567.123456789;\n\
         on check set big = 1234567;\n\
         bind wide.text = \"{wide}\" + text(big);\n"
    )
}

const MESSAGE: &str = "expression text exceeds limit 65536 bytes";

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
fn d025_a_text_past_the_limit_is_refused_and_the_session_goes_on() {
    for rule in ["on run", "on run when output > 0"] {
        let source = program(rule);
        let mut session = ReactiveSession::from_source(&source).unwrap();
        // Within the limit: the binding holds 65,527 bytes.
        assert_eq!(outcome(&mut session, "check")["outcome"], "accepted");
        let before = checkpoint(&session);
        assert_eq!(
            before.0["bindings"]["wide"]["text"]
                .as_str()
                .unwrap()
                .len(),
            65_527
        );
        let refused = outcome(&mut session, "run");
        assert_eq!(refused["outcome"], "rejected", "{rule}");
        assert_eq!(refused["origin"], "limit");
        assert_eq!(refused["code"], "text_limit");
        assert!(
            refused["message"].as_str().unwrap().ends_with(MESSAGE),
            "{}",
            refused["message"]
        );
        // Rolled back: `output` and `big` keep their values, the binding its text.
        assert_eq!(checkpoint(&session), before, "the refused event changed nothing");
        // The session is usable: another event is accepted, and the save restores.
        assert_eq!(outcome(&mut session, "check")["outcome"], "accepted");
        let save = session.save_json().unwrap();
        let restored = ReactiveSession::restore_json(&source, &save).unwrap();
        assert_eq!(restored.save_json().unwrap(), save);
        assert_eq!(checkpoint(&restored), checkpoint(&session));
        // The refusal is the same every time the event is tried.
        let again = outcome(&mut session, "run");
        assert_eq!(again["code"], "text_limit");
        assert_eq!(again["message"], refused["message"]);
    }
}

#[test]
fn d025_the_view_path_refuses_the_same_way() {
    let source = program("on run");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let mut viewed = ReactiveSession::from_source(&source).unwrap();
    let refused = serde_json::to_value(session.dispatch_outcome_json("run", "{}").unwrap()).unwrap();
    let on_view = serde_json::to_value(viewed.dispatch_view_outcome_json("run", "{}").unwrap()).unwrap();
    assert_eq!(on_view, refused);
    assert_eq!(refused["code"], "text_limit");
}

#[test]
fn d025_a_text_literal_past_the_limit_still_fails_to_load() {
    // The source limit catches it first; either way the program does not load.
    let source = format!(
        "state n = 0; event run; on run set n = 1; bind wide.text = \"{}\";",
        "A".repeat(65_537)
    );
    let error = ReactiveSession::from_source(&source).unwrap_err();
    assert!(error.contains("exceeds source limit 65536 bytes"), "{error}");
}
