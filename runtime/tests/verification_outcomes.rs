//! A `qualify` that takes a value's provenance past its capacity refuses the
//! event as `limit/provenance_limit` and the session goes on (D022; before
//! 1.4.0 it was a fatal `unclassified` fault). See
//! runtime/tests/d022_provenance_limit.rs for the same bound reached by a
//! `set`.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn accepted(session: &mut ReactiveSession, event: &str) -> Value {
    let result = session.dispatch_outcome_json(event, "{}").unwrap();
    let result = serde_json::to_value(result).unwrap();
    assert_eq!(result["outcome"], "accepted");
    result
}

fn refused_overflow(mut session: ReactiveSession, expected: &str) {
    let before = (
        serde_json::to_value(session.snapshot()).unwrap(),
        session.save_json().unwrap(),
    );
    let refused = session
        .dispatch_outcome_json("overflow", "{}")
        .unwrap_or_else(|fatal| panic!("overflow was fatal: {}", fatal.message));
    let refused = serde_json::to_value(refused).unwrap();
    assert_eq!(refused["outcome"], "rejected");
    assert_eq!(refused["origin"], "limit");
    assert_eq!(refused["code"], "provenance_limit");
    let message = refused["message"].as_str().unwrap();
    assert!(message.ends_with(expected), "{message}");
    // Nothing the event did is kept, and the session continues.
    let after = (
        serde_json::to_value(session.snapshot()).unwrap(),
        session.save_json().unwrap(),
    );
    assert_eq!(after, before);
    accepted(&mut session, "seed");
}

#[test]
fn provenance_identifier_boundary_accepts_then_excess_is_refused() {
    let mut source = String::from(
        "evidence e from sensor; caveat extra consequence low;\n\
         state value = 0; event seed; event overflow;\n",
    );
    for index in 0..1023 {
        source.push_str(&format!(
            "caveat c{index} consequence low; c{index} qualifies e;\n"
        ));
    }
    source.push_str(
        "on seed reveal e; on seed set value = qualified(1, e);\n\
         on overflow qualify e with extra;",
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let shown = accepted(&mut session, "seed");
    let provenance = &shown["snapshot"]["qualified_values"]["value"]["provenance"];
    assert_eq!(provenance["evidence"].as_array().unwrap().len(), 1);
    assert_eq!(provenance["caveats"].as_array().unwrap().len(), 1023);
    refused_overflow(session, "value provenance exceeds limit 1024 identifiers");
}

#[test]
fn provenance_byte_boundary_accepts_then_excess_is_refused() {
    let left = "a".repeat(32_768);
    let right = "b".repeat(32_768);
    let source = format!(
        "evidence {left} from left_sensor; evidence {right} from right_sensor;\n\
         evidence x from extra_sensor;\n\
         state a = 0; state b = 0; state total = 0;\n\
         event seed; event overflow;\n\
         on seed reveal {left}; on seed reveal {right}; on seed reveal x;\n\
         on seed set a = qualified(1, {left});\n\
         on seed set b = qualified(2, {right});\n\
         on seed set total = a + b;\n\
         on overflow set total = total + qualified(0, x);"
    );
    let mut session = ReactiveSession::from_source(&source).unwrap();
    let shown = accepted(&mut session, "seed");
    let evidence = shown["snapshot"]["qualified_values"]["total"]["provenance"]["evidence"]
        .as_array()
        .unwrap();
    assert_eq!(evidence.len(), 2);
    assert_eq!(
        evidence
            .iter()
            .map(|name| name.as_str().unwrap().len())
            .sum::<usize>(),
        65_536
    );
    refused_overflow(session, "value provenance exceeds limit 65536 name bytes");
}
