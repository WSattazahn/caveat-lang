//! D036: a binding scoped to histories kept its old lineage when an event
//! revealed a `supports` or `opposes` relation from evidence the graph
//! already held. The relation brings the caveats of the claim it names to
//! that evidence, and a scoped read can list evidence that belongs to no
//! history: the reason a reading was withdrawn for, the cause a decision was
//! reopened by, or what an outcome was recorded because of. Change detection
//! placed the relation in no history, so the binding went on without the new
//! caveat until something else made it evaluate again, while a restored
//! session, which evaluates every binding, showed it. Debug builds caught it,
//! since they check each event's bindings against a full evaluation. Present
//! from 1.2.0, which first scoped bindings to histories (checked on 1.2.0 and
//! 1.4.1). Found by the compiler thread, 2026-11-01. See
//! spec/caveat-incremental-evaluation-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const SOURCE: &str = r#"
claim c1;
claim c2;
caveat k consequence material;
k qualifies c2;
evidence a from "what the decision used";
evidence b from "a second look";
a supports c1;
b supports c1;
evidence tool from "the tool";
readings s from tool limit 4;
decisions d limit 4;
event go;
event retract;
event reopen_it;
event judge;
event relate;
event oppose;
on go sample s = 1;
on go commit d because enough using qualified(1, a);
on retract withdraw latest(s) because b;
on reopen_it reopen d because b;
on judge resolve d = 1 because b;
on relate reveal b supports c2;
on oppose reveal b opposes c2;
bind x.gone = 1 when withdrawn(latest(s));
bind x.r = "open";
bind x.r = "reopened" when reopened(d);
bind x.o = outcome(d) when resolved(d);
"#;

fn session(events: &[&str]) -> ReactiveSession {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    for event in events {
        let outcome = session
            .dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
        let outcome: Value = serde_json::to_value(outcome).unwrap();
        assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    }
    session
}

fn lineage(session: &ReactiveSession, property: &str) -> Value {
    let snapshot = serde_json::to_value(session.snapshot()).unwrap();
    snapshot["binding_qualifications"]["x"][property].clone()
}

fn names<'a>(lineage: &'a Value, key: &str) -> Vec<&'a str> {
    lineage[key]
        .as_array()
        .map(|list| list.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// `events` end with the relation. `property` lists `b` and, once `b`
/// bears on `c2`, carries `k`: as it does when the relation came before the
/// read, and as a restored session shows it.
fn carries_the_new_caveat(events: &[&str], property: &str) {
    let before = session(&events[..events.len() - 1]);
    let shown = lineage(&before, property);
    assert!(names(&shown, "evidence").contains(&"b"), "{shown}");
    assert!(!names(&shown, "caveats").contains(&"k"), "{shown}");

    let live = session(events);
    let shown = lineage(&live, property);
    assert!(
        names(&shown, "evidence").contains(&"b"),
        "{events:?}: {shown}"
    );
    assert!(
        names(&shown, "caveats").contains(&"k"),
        "{events:?}: x.{property} lacks k: {shown}"
    );

    let mut first = events.to_vec();
    let relation = first.pop().unwrap();
    first.insert(1, relation);
    assert_eq!(lineage(&session(&first), property), shown, "{first:?}");

    let restored = ReactiveSession::restore_json(SOURCE, &live.save_json().unwrap()).unwrap();
    assert_eq!(lineage(&restored, property), shown, "restored");
}

#[test]
fn d036_a_withdrawal_reason_that_gains_a_caveat_reaches_the_binding() {
    carries_the_new_caveat(&["go", "retract", "relate"], "gone");
    carries_the_new_caveat(&["go", "retract", "oppose"], "gone");
}

#[test]
fn d036_a_reopening_cause_that_gains_a_caveat_reaches_the_binding() {
    carries_the_new_caveat(&["go", "reopen_it", "relate"], "r");
}

#[test]
fn d036_an_outcome_reason_that_gains_a_caveat_reaches_the_binding() {
    carries_the_new_caveat(&["go", "judge", "relate"], "o");
}
