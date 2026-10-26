//! The what-if path answers what `dispatch_view_outcome` would return, on a
//! copy of the session, and leaves the session exactly as it was.
use caveat_runtime::web::WebReactiveSession;
use serde_json::Value;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn checkpoint(session: &WebReactiveSession) -> (Value, Value, Value) {
    (
        json(&session.snapshot()),
        json(&session.view()),
        json(&session.save().unwrap()),
    )
}

const DECISION: &str = r#"
claim safe;
claim misreading;
evidence report from "the check report";
evidence recheck from "the agent re-read the log";
decisions ship limit 4;
state ready = 0;
event see;
event decide;
event misread;
event wait dt min 0 max 1;
clock wait every 0.5;
on see reveal report supports safe;
on see set ready = qualified(1, report);
on decide when ready > 0 commit ship because enough using ready;
on misread reveal recheck supports misreading;
on misread withdraw report because recheck;
on misread when committed(ship) and not reopened(ship) and rests_on_withdrawn(ship)
    reopen ship because recheck;
bind hud.ready = ready;
"#;

#[test]
fn what_if_answers_the_event_and_changes_nothing() {
    let mut asked = WebReactiveSession::new(DECISION).unwrap();
    let mut plain = WebReactiveSession::new(DECISION).unwrap();
    let fields = r#"["bindings","decision_journal"]"#;
    for (event, payload) in [
        ("see", "{}"),
        ("decide", "{}"),
        ("decide", "{}"),
        ("wait", r#"{"dt":1}"#),
        ("wait", r#"{"dt":2}"#),
        ("nothing", "{}"),
        ("misread", "{}"),
        ("decide", "{}"),
    ] {
        let before = checkpoint(&asked);
        let answer = asked.what_if_view_outcome(event, payload).unwrap();
        let selected = asked
            .what_if_view_fields_outcome(event, payload, fields)
            .unwrap();
        assert_eq!(checkpoint(&asked), before, "{event} {payload}");
        let mut twin = WebReactiveSession::restore(DECISION, &plain.save().unwrap()).unwrap();
        assert_eq!(
            selected,
            twin.dispatch_view_fields_outcome(event, payload, fields)
                .unwrap(),
            "{event} {payload}"
        );
        assert_eq!(
            answer,
            plain.dispatch_view_outcome(event, payload).unwrap(),
            "{event} {payload}"
        );
        assert_eq!(
            asked.dispatch_view_outcome(event, payload).unwrap(),
            answer,
            "{event} {payload}"
        );
        assert_eq!(checkpoint(&asked), checkpoint(&plain), "{event} {payload}");
    }
}

#[test]
fn what_if_rejects_unknown_fields_before_running() {
    let session = WebReactiveSession::new(DECISION).unwrap();
    let before = checkpoint(&session);
    assert!(session
        .what_if_view_fields_outcome("see", "{}", r#"["nope"]"#)
        .is_err());
    assert_eq!(checkpoint(&session), before);
}
