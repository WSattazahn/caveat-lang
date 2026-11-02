//! A value read through a record gains each late caveat that now qualifies
//! the evidence the record lists (spec/caveat-caveat-sources-0.1.md). A read
//! that would gain more caveats than a lineage holds is refused as
//! `limit/provenance_limit`, and the session goes on. Past 1,024 caveats the
//! runtime names them in name order, so which of the two limits such a
//! refusal reports is the same in every process. These names are short, so it
//! is the identifier limit in any order.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// One more caveat than a lineage holds.
const CAVEATS: usize = 1025;

/// `flag` qualifies `e` with every caveat while no state lists `e`, so only
/// `trust`'s frozen basis does, and `committed(trust)` reads it live.
fn program() -> String {
    let mut source = String::from(
        r#"
evidence e from "tool";
event report;
event decide;
event clear;
event flag;
event late;
on report reveal e;
state v = 0;
on report set v = qualified(1, e);
on decide commit trust because enough using v;
on clear set v = 0;
state after = 0;
"#,
    );
    for caveat in 1..=CAVEATS {
        source.push_str(&format!("caveat c{caveat} consequence low;\n"));
    }
    for caveat in 1..=CAVEATS {
        source.push_str(&format!("on flag qualify e with c{caveat};\n"));
    }
    source.push_str("on late when committed(trust) set after = 1;\n");
    source
}

fn outcome(session: &mut ReactiveSession, event: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, "{}")
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

#[test]
fn a_read_that_reaches_more_late_caveats_than_a_lineage_holds_is_refused() {
    let source = program();
    let mut session = ReactiveSession::from_source(&source).unwrap();
    for event in ["report", "decide", "clear", "flag"] {
        assert_eq!(
            outcome(&mut session, event)["outcome"],
            "accepted",
            "{event}"
        );
    }
    let before = session.save_json().unwrap();
    // Four rules before the flag rules, then the late rule.
    let rule = 4 + CAVEATS + 1;
    let message =
        format!("event late, rule {rule}: value provenance exceeds limit 1024 identifiers");
    for _ in 0..2 {
        let refused = outcome(&mut session, "late");
        assert_eq!(refused["outcome"], "rejected");
        assert_eq!(refused["origin"], "limit");
        assert_eq!(refused["code"], "provenance_limit");
        assert_eq!(refused["message"], message.as_str());
        assert_eq!(
            session.save_json().unwrap(),
            before,
            "the refusal changed nothing"
        );
    }
    // The save restores, and the restored session refuses the same way.
    let mut restored = ReactiveSession::restore_json(&source, &before).unwrap();
    assert_eq!(outcome(&mut restored, "late")["message"], message.as_str());
}
