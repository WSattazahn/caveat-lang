//! The replay save. See spec/caveat-replay-save-0.1.md and 0.2.
use caveat_runtime::reactive::{
    ReactiveSession, POSITIONAL_REPLAY_SAVE_SCHEMA, REPLAY_SAVE_MAX_BYTES, REPLAY_SAVE_SCHEMA,
};
use serde_json::{json, Value};

const LEDGER: &str = r#"
identifiers limit 8;
claim ready;
evidence ci from "the pull request's checks";
caveat flaky consequence material;
renewable ci limit 8;
state passed = 0;
state offset = 1;
decisions gate limit 8;
event checks commit id, result in failed passed;
event nudge by min -1000 max 1000;
event decide;
event flag;
event unflag;
on checks when observed(ci) renew ci;
on checks reveal ci;
on checks set passed = qualified(if(result == result.passed, 1, 0), ci);
on nudge set offset = by;
on decide when passed == 0 reject "Nothing has passed.";
on decide commit gate because enough using passed;
on flag qualify ci with flaky;
on unflag unqualify ci with flaky;
bind gate.verdict = if(committed(gate), latest(gate), -1);
"#;

const SHA: &str = "602bdbec0a047a5319f53e83f336b9f7aec0e5ed";

fn session() -> ReactiveSession {
    ReactiveSession::from_source(LEDGER).unwrap_or_else(|error| panic!("fixture failed: {error}"))
}

fn send(session: &mut ReactiveSession, event: &str, payload: Value) -> Value {
    serde_json::to_value(
        session
            .dispatch_outcome_json(event, &payload.to_string())
            .unwrap_or_else(|fatal| panic!("{event} {payload}: fatal {}", fatal.message)),
    )
    .unwrap()
}

fn played() -> ReactiveSession {
    let mut session = session();
    send(
        &mut session,
        "checks",
        json!({"commit": SHA, "result": "passed"}),
    );
    send(&mut session, "nudge", json!({"by": -0.0}));
    send(&mut session, "decide", json!({}));
    send(&mut session, "flag", json!({}));
    send(
        &mut session,
        "checks",
        json!({"commit": SHA, "result": "failed"}),
    );
    send(&mut session, "unflag", json!({}));
    session
}

fn snapshot(session: &ReactiveSession) -> String {
    serde_json::to_string(&session.snapshot()).unwrap()
}

#[test]
fn a_compact_save_restores_the_session_that_wrote_it() {
    let mut saved = played();
    let compact = saved.save_compact_json().unwrap();
    let state = saved.save_json().unwrap();
    let parsed: Value = serde_json::from_str(&compact).unwrap();
    assert_eq!(parsed["schema"], REPLAY_SAVE_SCHEMA);
    assert!(
        compact.len() < state.len() / 2,
        "{} vs {}",
        compact.len(),
        state.len()
    );
    // The identifier is written as its text, and the signed zero as written.
    assert_eq!(parsed["events"][0][1]["commit"], SHA);
    assert!(compact.contains("\"by\":-0.0"), "{compact}");

    let mut restored = ReactiveSession::restore_json(LEDGER, &compact).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&saved));
    assert_eq!(restored.save_json().unwrap(), state);
    assert_eq!(restored.save_compact_json().unwrap(), compact);
    for (event, payload) in [
        ("decide", json!({})),
        ("checks", json!({"commit": "another", "result": "passed"})),
        ("decide", json!({})),
    ] {
        assert_eq!(
            send(&mut restored, event, payload.clone()),
            send(&mut saved, event, payload)
        );
    }
    assert_eq!(
        restored.save_compact_json().unwrap(),
        saved.save_compact_json().unwrap()
    );
}

#[test]
fn a_refused_event_is_not_kept() {
    let mut session = session();
    let before = session.save_compact_json().unwrap();
    let outcome = send(&mut session, "decide", json!({}));
    assert_eq!(outcome["outcome"], "rejected");
    assert_eq!(session.save_compact_json().unwrap(), before);
}

#[test]
fn a_session_restored_from_a_state_save_writes_state_saves() {
    let saved = played();
    let restored = ReactiveSession::restore_json(LEDGER, &saved.save_json().unwrap()).unwrap();
    assert_eq!(
        restored.save_compact_json().unwrap(),
        restored.save_json().unwrap()
    );
}

#[test]
fn a_compact_save_is_never_larger_than_the_state_save_or_the_bound() {
    let mut session = played();
    let mut replays = 0;
    for step in 0..300 {
        send(&mut session, "nudge", json!({"by": step}));
        let compact = session.save_compact_json().unwrap();
        let state = session.save_json().unwrap();
        if compact.contains(REPLAY_SAVE_SCHEMA) {
            assert!(compact.len() < state.len() && compact.len() <= REPLAY_SAVE_MAX_BYTES);
            let restored = ReactiveSession::restore_json(LEDGER, &compact).unwrap();
            assert_eq!(restored.save_json().unwrap(), state);
            replays += 1;
        } else {
            assert_eq!(compact, state);
        }
    }
    assert!(replays > 0);
}

fn refused(save: &str) -> String {
    match ReactiveSession::restore_json(LEDGER, save) {
        Ok(_) => panic!("restored {save}"),
        Err(error) => error,
    }
}

#[test]
fn a_replay_save_it_cannot_replay_is_refused() {
    let compact: Value = serde_json::from_str(&played().save_compact_json().unwrap()).unwrap();
    let with = |field: &str, value: Value| {
        let mut save = compact.clone();
        save[field] = value;
        save.to_string()
    };
    assert!(refused(&with("source_id", json!("another"))).contains("different program"));
    assert!(refused(&with("source_sha256", json!("00"))).contains("source digest"));
    assert!(refused(&with("extra", json!(1))).contains("unknown field"));
    for events in [
        json!([["decide"]]),
        json!([["checks", {"commit": SHA}]]),
        json!([["undeclared"]]),
    ] {
        assert!(refused(&with("events", events)).contains("is not accepted"));
    }
    for events in [
        json!([[]]),
        json!([["flag", {}]]),
        json!([["flag", 1]]),
        json!([["flag", {"a": 1}, 3]]),
    ] {
        assert!(refused(&with("events", events)).contains("event entry"));
    }
    // A replay save is never larger than the bound.
    let large = format!(
        "{{\"schema\":\"{REPLAY_SAVE_SCHEMA}\",\"source_id\":{},\"source_sha256\":{},\"events\":{}}}",
        compact["source_id"],
        compact["source_sha256"],
        Value::Array(vec![json!(["flag"]); REPLAY_SAVE_MAX_BYTES / 8])
    );
    assert!(refused(&large).contains("at most"));
}

#[test]
fn the_archive_is_not_restored() {
    let restored =
        ReactiveSession::restore_json(LEDGER, &played().save_compact_json().unwrap()).unwrap();
    assert_eq!(restored.undrained(), 0);
}

#[test]
fn a_positional_save_restores_the_session_that_wrote_it() {
    let mut saved = played();
    let named = saved.save_compact_json().unwrap();
    let positional = saved.save_compact_positional_json().unwrap();
    let parsed: Value = serde_json::from_str(&positional).unwrap();
    assert_eq!(parsed["schema"], POSITIONAL_REPLAY_SAVE_SCHEMA);
    assert!(positional.len() < named.len(), "{positional} vs {named}");
    // Events in name order: checks 0, decide 1, flag 2, nudge 3, unflag 4,
    // each followed by its arguments in one list. `result in failed passed`
    // is written as its position, the identifier as its text and the signed
    // zero as written. The digest is the source's SHA-256 in base64url.
    assert_eq!(
        parsed["events"],
        json!([0, SHA, 2, 3, -0.0, 1, 2, 0, SHA, 1, 4])
    );
    assert!(positional.contains(",3,-0.0,1,"), "{positional}");
    assert_eq!(parsed["sha256"].as_str().map(str::len), Some(43));
    assert!(parsed.get("source_id").is_none(), "{positional}");

    let mut restored = ReactiveSession::restore_json(LEDGER, &positional).unwrap();
    assert_eq!(snapshot(&restored), snapshot(&saved));
    assert_eq!(restored.save_json().unwrap(), saved.save_json().unwrap());
    assert_eq!(restored.save_compact_json().unwrap(), named);
    assert_eq!(restored.save_compact_positional_json().unwrap(), positional);
    for (event, payload) in [
        ("decide", json!({})),
        ("checks", json!({"commit": "another", "result": "passed"})),
    ] {
        assert_eq!(
            send(&mut restored, event, payload.clone()),
            send(&mut saved, event, payload)
        );
    }
    assert_eq!(
        restored.save_compact_positional_json().unwrap(),
        saved.save_compact_positional_json().unwrap()
    );
}

#[test]
fn a_positional_save_of_small_whole_numbers_is_a_text_of_digits() {
    let mut saved = session();
    send(&mut saved, "nudge", json!({"by": 12}));
    send(&mut saved, "nudge", json!({"by": 35}));
    let digits = saved.save_compact_positional_json().unwrap();
    let parsed: Value = serde_json::from_str(&digits).unwrap();
    // nudge 3 by 12, then by 35.
    assert_eq!(parsed["events"], json!("3c3z"));
    let restored = ReactiveSession::restore_json(LEDGER, &digits).unwrap();
    assert_eq!(restored.save_json().unwrap(), saved.save_json().unwrap());
    assert_eq!(restored.save_compact_positional_json().unwrap(), digits);
    // A value past 35 writes the list.
    send(&mut saved, "nudge", json!({"by": 36}));
    let parsed: Value =
        serde_json::from_str(&saved.save_compact_positional_json().unwrap()).unwrap();
    assert_eq!(parsed["events"], json!([3, 12, 3, 35, 3, 36]));
    for (events, expected) in [
        (json!("3C"), "not a base-36 digit"),
        (json!("3-"), "not a base-36 digit"),
        (json!({}), "not a list or a text"),
        (json!("3"), "takes 1 arguments"),
    ] {
        let mut save = parsed.clone();
        save["events"] = events;
        let error = ReactiveSession::restore_json(LEDGER, &save.to_string()).unwrap_err();
        assert!(error.contains(expected), "{save}: {error}");
    }
}

#[test]
fn a_positional_save_outlasts_the_named_one_and_leaves_it_unchanged() {
    let mut with = played();
    let mut without = played();
    let (mut named, mut positional) = (0, 0);
    for step in 0..400 {
        send(&mut with, "nudge", json!({"by": step}));
        send(&mut without, "nudge", json!({"by": step}));
        // Asking for the positional save changes nothing the named one writes.
        let _ = with.save_compact_positional_json().unwrap();
        let compact = with.save_compact_json().unwrap();
        assert_eq!(compact, without.save_compact_json().unwrap());
        named += usize::from(compact.contains(REPLAY_SAVE_SCHEMA));
        let save = with.save_compact_positional_json().unwrap();
        if save.contains(POSITIONAL_REPLAY_SAVE_SCHEMA) {
            assert!(save.len() <= REPLAY_SAVE_MAX_BYTES);
            let restored = ReactiveSession::restore_json(LEDGER, &save).unwrap();
            assert_eq!(restored.save_json().unwrap(), with.save_json().unwrap());
            positional += 1;
        } else {
            assert_eq!(save, with.save_json().unwrap());
        }
    }
    assert!(positional > 2 * named, "{positional} vs {named}");
}

#[test]
fn a_positional_save_it_cannot_replay_is_refused() {
    let saved = played();
    let good: Value = serde_json::from_str(&saved.save_compact_positional_json().unwrap()).unwrap();
    let with_events = |events: Value| {
        let mut save = good.clone();
        save["events"] = events;
        save.to_string()
    };
    for (save, expected) in [
        (with_events(json!([9])), "position of an event"),
        (with_events(json!(["decide"])), "position of an event"),
        (with_events(json!([[1]])), "position of an event"),
        (with_events(json!([1, 5])), "position of an event"),
        (with_events(json!([0, SHA])), "takes 2 arguments"),
        (with_events(json!([0, SHA, null])), "not a number or a text"),
        (
            with_events(json!([0, SHA, 2, 0, SHA, 7])),
            "event 2 (checks) is not accepted",
        ),
    ] {
        let error = ReactiveSession::restore_json(LEDGER, &save)
            .err()
            .unwrap_or_else(|| panic!("{save} restored"));
        assert!(error.contains(expected), "{save}: {error}");
    }
    let mut moved = good.clone();
    moved["sha256"] = json!("A".repeat(43));
    let error = ReactiveSession::restore_json(LEDGER, &moved.to_string()).unwrap_err();
    assert!(error.contains("source digest"), "{error}");
    let mut named = good;
    named["source_id"] = json!("fnv1a64:0:0");
    let error = ReactiveSession::restore_json(LEDGER, &named.to_string()).unwrap_err();
    assert!(error.contains("unknown field"), "{error}");
}
