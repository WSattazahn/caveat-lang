//! D037: a `carries` or `carries_any` read whose caveat search would give a
//! lineage of more than 1,024 identifiers ended the session as fatal
//! `unclassified` on 1.4.0 and 1.4.1, although the event had been rolled
//! back. Every other read past the limit, `observed` among them, was refused
//! as `limit/provenance_limit` (D022). The search ran inside a predicate that
//! kept only the error's message. It is refused as `limit/provenance_limit`
//! now, with the same text, and the session goes on. See
//! spec/caveat-dispatch-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// One evidence that `caveats` caveats qualify, so a search for what
/// qualifies it finds the evidence and every caveat: 1,025 identifiers at
/// 1,024 caveats. `reads` adds the rules or bindings under test.
fn program(caveats: usize, reads: &str) -> String {
    let mut text = String::from(
        "claim c;\n\
         evidence e from \"a source\";\n\
         state x = 0;\n\
         state count = 0;\n\
         event see;\n\
         event doubt;\n\
         event ask;\n\
         event other;\n",
    );
    for k in 0..caveats {
        text.push_str(&format!("caveat k{k} consequence low;\n"));
    }
    text.push_str("on see reveal e supports c;\n");
    for k in 0..caveats {
        text.push_str(&format!("on doubt qualify e with k{k};\n"));
    }
    text.push_str(reads);
    text.push_str("on other set count = count + 1;\n");
    text
}

/// The text the fatal error carried, and the refusal carries now: the rule
/// after `see`'s one rule and `doubt`'s 1,024.
const MESSAGE: &str = "event ask, rule 1026: value provenance exceeds limit 1024 identifiers";

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

/// Loads `reads` with 1,024 caveats, sees and doubts the evidence, and
/// checks that `ask` is refused past the limit with `message`, changes
/// nothing, and is refused the same way again, while the session goes on.
fn refused_past_the_limit(reads: &str, message: &str) {
    let source = program(1024, reads);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(outcome(&mut session, "see")["outcome"], "accepted");
    assert_eq!(outcome(&mut session, "doubt")["outcome"], "accepted");
    let before = checkpoint(&session);
    let refused = outcome(&mut session, "ask");
    assert_eq!(refused["outcome"], "rejected", "{refused}");
    assert_eq!(refused["origin"], "limit");
    assert_eq!(refused["code"], "provenance_limit");
    assert_eq!(refused["message"], message);
    assert_eq!(
        checkpoint(&session),
        before,
        "the refused event changed nothing"
    );
    // The session is usable: another event is accepted, and the save restores.
    assert_eq!(outcome(&mut session, "other")["outcome"], "accepted");
    let save = session.save_json().unwrap();
    let restored = ReactiveSession::restore_json(&source, &save).unwrap();
    assert_eq!(restored.save_json().unwrap(), save);
    assert_eq!(checkpoint(&restored), checkpoint(&session));
    // The refusal is the same every time the event is tried.
    let again = outcome(&mut session, "ask");
    assert_eq!(again["code"], "provenance_limit");
    assert_eq!(again["message"], message);
}

#[test]
fn d037_a_carries_read_past_the_provenance_limit_is_refused_and_the_session_goes_on() {
    refused_past_the_limit("on ask set x = if(carries(e, k0), 1, 0);\n", MESSAGE);
}

#[test]
fn d037_a_carries_guard_past_the_provenance_limit_is_refused() {
    refused_past_the_limit("on ask when carries(e, k0) set x = 1;\n", MESSAGE);
}

#[test]
fn d037_carries_any_past_the_provenance_limit_is_refused() {
    refused_past_the_limit("on ask set x = if(carries_any(e), 1, 0);\n", MESSAGE);
    refused_past_the_limit("on ask set x = if(carries_any(e, low), 1, 0);\n", MESSAGE);
}

/// `observed` was already refused; the same read through `carries` now is
/// refused alike.
#[test]
fn d037_carries_is_refused_as_observed_is() {
    refused_past_the_limit("on ask set x = if(observed(e), 1, 0);\n", MESSAGE);
}

/// A binding that reads `carries` is evaluated after the event that
/// qualifies the evidence past the limit, so that event is refused.
#[test]
fn d037_a_binding_reading_carries_past_the_provenance_limit_refuses_the_event() {
    let source = program(1024, "bind view.carried = if(carries(e, k0), 1, 0);\n");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    assert_eq!(outcome(&mut session, "see")["outcome"], "accepted");
    let before = checkpoint(&session);
    let refused = outcome(&mut session, "doubt");
    assert_eq!(refused["outcome"], "rejected", "{refused}");
    assert_eq!(refused["origin"], "limit");
    assert_eq!(refused["code"], "provenance_limit");
    assert_eq!(
        refused["message"],
        "binding view.carried: value provenance exceeds limit 1024 identifiers"
    );
    assert_eq!(
        checkpoint(&session),
        before,
        "the refused event changed nothing"
    );
    assert_eq!(outcome(&mut session, "other")["outcome"], "accepted");
    let again = outcome(&mut session, "doubt");
    assert_eq!(again["code"], "provenance_limit");
    assert_eq!(again["message"], refused["message"]);
}

/// At the limit, 1,023 caveats and the evidence, the read is accepted and
/// its lineage holds all 1,024 identifiers.
#[test]
fn d037_a_carries_read_at_the_provenance_limit_is_accepted() {
    let source = program(1023, "on ask set x = if(carries(e, k0), 1, 0);\n");
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for event in ["see", "doubt", "ask"] {
        assert_eq!(
            outcome(&mut session, event)["outcome"],
            "accepted",
            "{event}"
        );
    }
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    let x = &snapshot["qualified_values"]["x"];
    assert_eq!(x["value"], 1.0);
    assert_eq!(x["provenance"]["evidence"].as_array().unwrap().len(), 1);
    assert_eq!(x["provenance"]["caveats"].as_array().unwrap().len(), 1023);
}
