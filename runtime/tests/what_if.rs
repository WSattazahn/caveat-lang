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

/// With an agent, the answer is what `dispatch_view_outcome_as` would return
/// for that agent, and asking still changes nothing. See
/// spec/caveat-agents-0.1.md, "Hosts".
#[test]
fn what_if_answers_for_the_agent_the_host_names() {
    const HANDOFF: &str = r#"
claim plan_ok;
evidence request from "the user's request";
evidence report from "the sub-agent's report";
readings reports from report limit 8;
decisions plan limit 4;
agent main sends delegate;
agent sub sends sub_report;
event delegate;
event sub_report n min 0 max 10;
event note;
on delegate reveal request supports plan_ok;
on delegate commit plan because enough citing qualified(1, request);
on sub_report sample reports = n;
"#;
    let mut asked = WebReactiveSession::new(HANDOFF).unwrap();
    let mut plain = WebReactiveSession::new(HANDOFF).unwrap();
    let fields = r#"["decision_journal","commitments"]"#;
    for (event, payload, agent) in [
        ("delegate", "{}", Some("sub")),
        ("delegate", "{}", None),
        ("delegate", "{}", Some("main")),
        ("sub_report", r#"{"n":3}"#, Some("sub")),
        ("sub_report", r#"{"n":3}"#, Some("mallory")),
        ("note", "{}", Some("main")),
        ("note", "{}", None),
        ("sub_report", r#"{"n":4}"#, Some("sub")),
    ] {
        let before = checkpoint(&asked);
        let (answer, selected) = match agent {
            Some(agent) => (
                asked
                    .what_if_view_outcome_as(event, payload, agent)
                    .unwrap(),
                asked
                    .what_if_view_fields_outcome_as(event, payload, fields, agent)
                    .unwrap(),
            ),
            None => (
                asked.what_if_view_outcome(event, payload).unwrap(),
                asked
                    .what_if_view_fields_outcome(event, payload, fields)
                    .unwrap(),
            ),
        };
        assert_eq!(checkpoint(&asked), before, "{event} {agent:?}");
        let mut twin = WebReactiveSession::restore(HANDOFF, &plain.save().unwrap()).unwrap();
        let (theirs, their_fields) = match agent {
            Some(agent) => (
                plain
                    .dispatch_view_outcome_as(event, payload, agent)
                    .unwrap(),
                twin.dispatch_view_fields_outcome_as(event, payload, fields, agent)
                    .unwrap(),
            ),
            None => (
                plain.dispatch_view_outcome(event, payload).unwrap(),
                twin.dispatch_view_fields_outcome(event, payload, fields)
                    .unwrap(),
            ),
        };
        assert_eq!(answer, theirs, "{event} {agent:?}");
        assert_eq!(selected, their_fields, "{event} {agent:?}");
        let answered: Value = json(&answer);
        let expected = match (event, agent) {
            ("delegate", Some("main")) | ("sub_report", Some("sub")) | ("note", _) => "accepted",
            _ => "rejected",
        };
        assert_eq!(answered["outcome"], expected, "{event} {agent:?}: {answer}");
        if expected == "rejected" {
            assert_eq!(
                answered["code"], "wrong_agent",
                "{event} {agent:?}: {answer}"
            );
        }
        match agent {
            Some(agent) => asked
                .dispatch_view_outcome_as(event, payload, agent)
                .unwrap(),
            None => asked.dispatch_view_outcome(event, payload).unwrap(),
        };
        assert_eq!(checkpoint(&asked), checkpoint(&plain), "{event} {agent:?}");
    }
    assert_eq!(json(&asked.snapshot())["last_agent"], "sub");
}
