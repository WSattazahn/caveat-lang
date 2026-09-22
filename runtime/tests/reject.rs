//! `reject "message"`: an event that is not allowed fails, and keeps nothing.
use caveat_runtime::reactive::ReactiveSession;

fn session(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

const PROGRAM: &str = r#"
claim edible;
evidence first_bite from "first bite";
state consumed = 0 min 0 max 1;
state bites = 0;
event absorb;
on absorb set bites = bites + 1;
on absorb reveal first_bite supports edible;
on absorb when consumed == 1 reject "already absorbed";
on absorb set consumed = 1;
"#;

#[test]
fn a_rejected_event_keeps_nothing_it_did_before_the_rejection() {
    let mut game = session(PROGRAM);
    game.dispatch_json("absorb", "{}").unwrap();
    let before = game.snapshot();
    let error = game.dispatch_json("absorb", "{}").unwrap_err();
    assert!(error.contains("rejected: already absorbed"), "{error}");
    let after = game.snapshot();
    assert_eq!(after.sequence, before.sequence);
    assert_eq!(after.values, before.values);
    assert_eq!(after.relations, before.relations);
    assert_eq!(after.value_grounds, before.value_grounds);
}

#[test]
fn a_skipped_rejection_is_nothing() {
    let mut game = session(PROGRAM);
    let shown = game.dispatch_json("absorb", "{}").unwrap();
    assert_eq!(shown.values["bites"], 1.0);
    assert_eq!(shown.values["consumed"], 1.0);
    // A reject names no target, so skipping it leaves no dependency behind.
    assert!(shown.qualified_values["consumed"].provenance.is_empty());
}

#[test]
fn a_procedure_can_reject_its_calling_event() {
    let mut game = session(
        r#"
        state held = 0;
        event lift weight min 0 max 100;
        proc carry(weight) {
            set held = weight;
            when weight > 20 reject "too heavy";
        };
        on lift call carry(weight);
        "#,
    );
    game.dispatch_json("lift", r#"{"weight":10}"#).unwrap();
    let error = game.dispatch_json("lift", r#"{"weight":40}"#).unwrap_err();
    assert!(error.contains("too heavy"), "{error}");
    assert_eq!(game.snapshot().values["held"], 10.0);
}

#[test]
fn reject_takes_exactly_one_quoted_message() {
    for body in [
        "event e; on e reject;",
        "event e; on e reject too heavy;",
        "event e; on e reject \"a\" \"b\";",
    ] {
        assert!(
            ReactiveSession::from_source(body).is_err(),
            "{body} should be rejected"
        );
    }
    let mut game = session("event e; on e reject \"semicolons; and when are text\";");
    let error = game.dispatch_json("e", "{}").unwrap_err();
    assert!(error.contains("semicolons; and when are text"), "{error}");
}
